//! Type admission and checked expression construction.

use std::collections::{HashMap, HashSet};

use crate::resolve::ast::{self as resolved, FALSE_VALUE, TRUE_VALUE, TypeId, ValueId};
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;

pub mod ast;
mod binding;
mod control;
mod entry;
mod expression;
mod float;
mod generic;
mod inference;
mod initializer;
mod integer;
mod interface;
mod lambda;
mod memory;
mod operation;
mod operator;
mod product;
mod program;
mod specialization_identity;
mod specialize;
pub mod type_fingerprint;
mod types;

/// Renders a bounded diagnostic name for an admitted type without exposing its internal sharing.
pub fn type_name(ty: &ast::Type) -> String {
    types::type_name(ty)
}

use self::ast::{AbruptExpression, Completion, Kind, Program, TopItem, Type};
use self::interface::ExternalSignature;
use self::operation::suggest_types;
use self::types::{GenericAliasDefinition, Kinds};

/// Applies every type and completion rule to a resolved program while retaining generic declarations.
pub fn check(program: &resolved::Program) -> Result<Program, Diagnostic> {
    Checker::new().check_program(program).map_err(diagnostic)
}

pub(crate) fn check_for_editor(
    program: &resolved::Program,
) -> Result<(Program, Option<Diagnostic>), Diagnostic> {
    Checker::new().check_program_for_editor(program)
}

/// Instantiates the generic bindings reachable from `main` once per concrete type argument list.
/// Fails when the program has no `main`.
pub fn specialize(program: Program) -> Result<ast::MonomorphicProgram, Diagnostic> {
    specialize::specialize(program)
}

/// Admits an already nongeneric checked program to the same downstream boundary as specialization.
pub fn admit_monomorphic(program: Program) -> Result<ast::MonomorphicProgram, Diagnostic> {
    if let Some(item) = program.items.iter().find(|item| {
        matches!(
            item.kind,
            TopItem::OpaqueType { .. }
                | TopItem::GenericBinding(_)
                | TopItem::OperationFamily(_)
                | TopItem::OperationImplementation(_)
        )
    }) {
        return Err(
            Diagnostic::error("program requires specialization").with_primary(
                item.span,
                "this opaque, generic, or operation item has not been specialized",
            ),
        );
    }
    Ok(ast::MonomorphicProgram::new(program))
}

/// Returns the first lambda identity unused by the checked program for downstream identity allocation.
pub fn next_lambda_identity(program: &Program) -> u32 {
    specialization_identity::next_identities(program)
        .expect("an admitted monomorphic program has remaining identity space")
        .lambda
}

#[derive(Clone)]
enum CheckFailure {
    Diagnostic(Diagnostic),
    Abrupt(Box<AbruptExpression>),
}

impl From<Diagnostic> for CheckFailure {
    fn from(value: Diagnostic) -> Self {
        Self::Diagnostic(value)
    }
}

type CheckResult<T> = Result<T, CheckFailure>;

#[derive(Clone)]
struct Checker {
    kinds: Kinds,
    next_kind_variable: u32,
    aliases: HashMap<TypeId, Node<resolved::TypeExpression>>,
    generic_aliases: HashMap<TypeId, GenericAliasDefinition>,
    opaque_types: HashMap<TypeId, types::OpaqueDefinition>,
    type_substitutions: std::sync::Arc<HashMap<TypeId, Type>>,
    active_requirements: Vec<Type>,
    active_generic: Option<(ValueId, Vec<TypeId>)>,
    external_types: HashMap<TypeId, resolved::TypeBinding>,
    expanded_aliases: HashMap<TypeId, Type>,
    expanded_generic_aliases: HashMap<TypeId, Type>,
    aggregate_alias_sources: HashMap<TypeId, Option<Vec<Node<resolved::TypeExpression>>>>,
    expanding: HashSet<TypeId>,
    values: HashMap<ValueId, Type>,
    generic_signatures: HashMap<ValueId, GenericSignature>,
    operation_families: HashSet<ValueId>,
    operation_keys: Vec<(ValueId, Vec<Type>)>,
    active_operations: Vec<ast::OperationRequirement>,
    /// Kind equations the active generic body needs from its parameters.
    active_kinds: Vec<ast::KindRequirement>,
    external_values: HashSet<ValueId>,
    externals: HashMap<resolved::ExternalOperationId, ExternalSignature>,
    result_targets: HashMap<ValueId, ResultTarget>,
    used_result_targets: HashSet<ValueId>,
    /// One memo per generic call being inferred, innermost last.
    argument_memos: Vec<inference::ArgumentMemo>,
}

fn diagnostic(error: CheckFailure) -> Diagnostic {
    match error {
        CheckFailure::Diagnostic(diagnostic) => diagnostic,
        CheckFailure::Abrupt(abrupt) => Diagnostic::error("abrupt completion outside a lambda")
            .with_primary(
                abrupt.span,
                "this control expression has no return boundary",
            ),
    }
}

#[derive(Clone)]
struct GenericSignature {
    parameters: Vec<resolved::TypeBinding>,
    parameter_kinds: Vec<Kind>,
    ty: Type,
    requirements: Vec<Type>,
    operations: Vec<ast::OperationRequirement>,
}

#[derive(Clone)]
struct ResultTarget {
    parameter: Type,
    result: Type,
    variant: Option<usize>,
    boundary: ValueId,
}

impl Checker {
    fn new() -> Self {
        let bool_type = Type::Sum(vec![Type::Unit, Type::Unit].into());
        Self {
            kinds: Kinds::default(),
            next_kind_variable: 0,
            aliases: HashMap::new(),
            generic_aliases: HashMap::new(),
            opaque_types: HashMap::new(),
            type_substitutions: Default::default(),
            active_requirements: Vec::new(),
            active_generic: None,
            external_types: HashMap::new(),
            expanded_aliases: HashMap::new(),
            expanded_generic_aliases: HashMap::new(),
            aggregate_alias_sources: HashMap::new(),
            expanding: HashSet::new(),
            values: HashMap::from([(FALSE_VALUE, bool_type.clone()), (TRUE_VALUE, bool_type)]),
            generic_signatures: HashMap::new(),
            operation_families: HashSet::new(),
            operation_keys: Vec::new(),
            active_operations: Vec::new(),
            active_kinds: Vec::new(),
            external_values: HashSet::new(),
            externals: HashMap::new(),
            result_targets: HashMap::new(),
            used_result_targets: HashSet::new(),
            argument_memos: Vec::new(),
        }
    }

    fn value_type(&self, reference: &resolved::ValueReference) -> Result<Type, Diagnostic> {
        if self.result_targets.contains_key(&reference.id) {
            return Err(Diagnostic::error("result binder is not a value")
                .with_primary(reference.name.span, "call this binder in callee position"));
        }
        self.values.get(&reference.id).cloned().ok_or_else(|| {
            Diagnostic::error(format!(
                "value `{}` has no inferred type",
                reference.name.text
            ))
            .with_primary(reference.name.span, "its binding is not available here")
        })
    }

    fn check_completion(
        &mut self,
        expression: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> Result<Completion, Diagnostic> {
        match self.check_expression(expression, expected) {
            Ok(value) => Ok(Completion::Value(value)),
            Err(CheckFailure::Abrupt(abrupt)) => Ok(Completion::Abrupt(*abrupt)),
            Err(CheckFailure::Diagnostic(diagnostic)) => Err(diagnostic),
        }
    }
}
