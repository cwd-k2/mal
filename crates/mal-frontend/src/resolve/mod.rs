//! Name identity, lexical scope, result authority, and capture inference.

use std::collections::{HashMap, HashSet};

use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

pub mod ast;
mod continuation;
mod expression;
mod files;
mod key;
mod predefined;
mod scope;
mod top;
mod types;

pub use self::predefined::{
    COPY_VALUE, FILL_VALUE, GET_VALUE, MAKE_VALUE, NEW_VALUE, PUT_VALUE, TYPES as PREDEFINED_TYPES,
    VALUES as PREDEFINED_VALUES,
};

use self::ast::{
    ExternalOperationId, LambdaId, Program, TypeBinding, ValueBinding, ValueId, ValueOwner,
};

/// Resolves one parsed file. `require` declarations are not followed; use `resolve_graph` for several files.
pub fn resolve(program: &mal_syntax::ast::Program) -> Result<Program, Diagnostic> {
    Resolver::new(program.span).resolve_program(program)
}

/// Resolves the files of `graph`, given in graph order, as one program. Public names flow only along direct requirements.
pub fn resolve_graph(
    graph: &mal_syntax::source::SourceGraph,
    programs: &[mal_syntax::ast::Program],
) -> Result<Program, Diagnostic> {
    files::resolve(graph, programs)
}

#[derive(Clone)]
struct ExternalBinding {
    id: ExternalOperationId,
    binding: ValueBinding,
    lambda_id: LambdaId,
}

struct LambdaFrame {
    id: LambdaId,
    /// The binding a recursive lambda refers to itself by. Its body, and lambdas nested in it, reach the closure
    /// through this binding instead of capturing the binding, which is not yet initialized when the closure is built.
    self_binding: Option<ValueId>,
    captures: Vec<ast::Capture>,
    captured_sources: HashMap<ValueId, ValueBinding>,
}

struct Resolver {
    types: HashMap<String, TypeBinding>,
    externals: HashMap<String, ExternalBinding>,
    operation_families: HashSet<ValueId>,
    value_scopes: Vec<HashMap<String, ValueBinding>>,
    current_lambda: Option<LambdaId>,
    lambda_frames: Vec<LambdaFrame>,
    next_type: u32,
    next_value: u32,
    next_external: u32,
    next_lambda: u32,
    synthetic_span: Span,
}

impl Resolver {
    fn new(program_span: Span) -> Self {
        let mut resolver = Self {
            types: HashMap::new(),
            externals: HashMap::new(),
            operation_families: HashSet::new(),
            value_scopes: vec![HashMap::new()],
            current_lambda: None,
            lambda_frames: Vec::new(),
            next_type: predefined::first_source_type_id(),
            next_value: predefined::first_source_value_id(),
            next_external: 0,
            next_lambda: 0,
            synthetic_span: program_span,
        };
        resolver.begin_file(program_span);
        resolver
    }

    fn resolve_program(
        mut self,
        program: &mal_syntax::ast::Program,
    ) -> Result<Program, Diagnostic> {
        self.predeclare_unit_names(program)?;
        let mut items = Vec::with_capacity(program.items.len());
        for item in &program.items {
            items.push(self.resolve_top_item(item)?);
        }
        Ok(Program {
            items,
            span: program.span,
        })
    }

    /// Clears the file-local scopes and brings the predefined names back, keeping identity counters program-wide.
    fn begin_file(&mut self, span: Span) {
        self.types.clear();
        self.externals.clear();
        self.operation_families.clear();
        self.value_scopes.clear();
        self.value_scopes.push(HashMap::new());
        self.current_lambda = None;
        self.lambda_frames.clear();
        self.synthetic_span = Span::new(span.file(), span.start(), span.start());
        for entry in PREDEFINED_TYPES {
            self.add_predefined_type(entry.name, entry.id);
        }
        for entry in PREDEFINED_VALUES {
            self.add_predefined_value(entry.name, entry.id);
        }
    }

    fn resolve_binding(
        &mut self,
        binding: &mal_syntax::ast::Binding,
        owner: ValueOwner,
    ) -> Result<ast::Binding, Diagnostic> {
        let annotation = binding
            .annotation
            .as_ref()
            .map(|ty| self.resolve_type(ty))
            .transpose()?;

        if annotation.is_some()
            && let mal_syntax::ast::Pattern::Name(name) = &binding.pattern.kind
            && matches!(&binding.value.kind, mal_syntax::ast::Expression::Lambda(_))
        {
            let declared = self.declare_value(name, owner)?;
            let value = self.resolve_initializer_with_self(&binding.value, declared.clone())?;
            let pattern =
                mal_syntax::ast::Node::new(ast::Pattern::Binding(declared), binding.pattern.span);
            return Ok(ast::Binding {
                pattern,
                annotation,
                value,
            });
        }

        let value = self.resolve_expression(&binding.value)?;
        let pattern = self.declare_pattern(&binding.pattern, owner)?;
        Ok(ast::Binding {
            pattern,
            annotation,
            value,
        })
    }

    fn allocate_lambda(&mut self) -> LambdaId {
        let id = LambdaId(self.next_lambda);
        self.next_lambda += 1;
        id
    }

    fn local_owner(&self, span: Span) -> Result<ValueOwner, Diagnostic> {
        self.current_lambda.map(ValueOwner::Lambda).ok_or_else(|| {
            Diagnostic::error("local binding outside a lambda is not supported")
                .with_primary(span, "this binding has no owning lambda")
        })
    }
}
