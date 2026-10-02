//! Source type expansion into canonical kinded type terms. Expansion is an explicit stack machine so that deep or
//! long alias chains never recurse on the host stack: `pending` holds the work still to do, and `values` the terms
//! already built, which each finishing step pops and combines.

use std::collections::HashMap;
use std::sync::Arc;

use crate::resolve::ast::{self as resolved, TypeId};
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::super::{Checker, ast::Kind, ast::KindRequirement, ast::Type};
use super::{ensure_representable, term};

mod declaration;

pub(super) use declaration::predefined_type;

type Substitutions = Arc<HashMap<TypeId, Type>>;

pub(in crate::check) fn require_type_argument_kinds(
    parameters: &[Kind],
    arguments: &mut [Type],
    span: Span,
) -> Result<Vec<KindRequirement>, Diagnostic> {
    term::normalize_argument_kinds(parameters, arguments, span)
}

pub(in crate::check) fn instantiate_signature_kinds(
    parameters: &[Kind],
    arguments: &[Type],
    ty: &Type,
    span: Span,
) -> Result<(Vec<Kind>, Type, Vec<KindRequirement>), Diagnostic> {
    term::instantiate_kinds(parameters, arguments, ty, span)
}

pub(super) fn ensure_value_type(ty: &Type, span: Span) -> Result<(), Diagnostic> {
    if ty.kind() == Kind::Type {
        return Ok(());
    }
    Err(
        Diagnostic::error("type constructor used as a value type").with_primary(
            span,
            format!(
                "expected `Type` but found `{}`",
                term::kind_name(&ty.kind())
            ),
        ),
    )
}

/// One step of the expansion machine.
enum Expansion {
    /// Expand a source expression under the given parameter substitutions.
    Expression(Node<resolved::TypeExpression>, Substitutions),
    /// Resolve a type name to the term it stands for.
    Reference(TypeId, Span, Substitutions),
    /// Apply the arguments from `index` on to the constructor on top of `values`. A parameter the alias never uses
    /// is applied without forming its argument.
    ApplyArguments {
        arguments: Vec<Node<resolved::TypeExpression>>,
        phantom: Vec<bool>,
        substitutions: Substitutions,
        index: usize,
    },
    /// Apply the argument on top of `values` to the constructor below it.
    ApplyOne(Span),
    /// Finish a plain alias whose expansion is on top of `values`.
    Alias(TypeId),
    /// Abstract a generic alias body over its parameters.
    AliasAbstraction {
        id: TypeId,
        parameter_kinds: Vec<Kind>,
    },
    /// Wrap an opaque representation in its declaration and abstract it over its parameters.
    OpaqueAbstraction {
        id: TypeId,
        arguments: Vec<Type>,
        parameter_kinds: Vec<Kind>,
    },
    Product(usize),
    Sum(usize),
    Function,
}

/// The stacks of one expansion and the normalizer budget it shares.
struct Machine {
    pending: Vec<Expansion>,
    values: Vec<Type>,
    normalizer: term::Normalizer,
}

impl Machine {
    fn new(initial: impl IntoIterator<Item = Expansion>) -> Self {
        let pending = initial.into_iter().collect::<Vec<_>>();
        let span = match pending.last().expect("expansion has a root") {
            Expansion::Expression(expression, _) => expression.span,
            Expansion::Reference(_, span, _) => *span,
            _ => unreachable!("only source expansions may be roots"),
        };
        Self {
            pending,
            values: Vec::new(),
            normalizer: term::Normalizer::new(span),
        }
    }

    fn pop_value(&mut self, what: &str) -> Type {
        self.values.pop().unwrap_or_else(|| panic!("{what}"))
    }

    fn take_last(&mut self, length: usize) -> Vec<Type> {
        let start = self
            .values
            .len()
            .checked_sub(length)
            .expect("composite expansion must have all children");
        self.values.split_off(start)
    }

    fn apply_arguments(
        &mut self,
        arguments: Vec<Node<resolved::TypeExpression>>,
        phantom: Vec<bool>,
        substitutions: Substitutions,
        index: usize,
    ) -> Result<(), Diagnostic> {
        if index == arguments.len() {
            return Ok(());
        }
        let span = arguments[index].span;
        let argument = arguments[index].clone();
        let unused = phantom.get(index).is_some_and(|used| !used);
        self.pending.push(Expansion::ApplyArguments {
            arguments,
            phantom,
            substitutions: substitutions.clone(),
            index: index + 1,
        });
        if unused {
            let constructor = self.pop_value("type application has a constructor term");
            let applied = self.normalizer.apply_unused(constructor, span)?;
            self.values.push(applied);
        } else {
            self.pending.push(Expansion::ApplyOne(span));
            self.pending
                .push(Expansion::Expression(argument, substitutions));
        }
        Ok(())
    }

    fn apply_one(&mut self, span: Span) -> Result<(), Diagnostic> {
        let argument = self.pop_value("type application has one argument");
        let constructor = self.pop_value("type application has a constructor term");
        let applied = self.normalizer.apply(constructor, argument, span)?;
        self.values.push(applied);
        Ok(())
    }

    fn abstract_over(
        &mut self,
        parameter_kinds: Vec<Kind>,
        mut body: Type,
    ) -> Result<Type, Diagnostic> {
        for kind in parameter_kinds.into_iter().rev() {
            body = self.normalizer.abstraction(kind, body)?;
        }
        Ok(body)
    }
}

impl Checker {
    pub(in crate::check) fn expand_type(
        &mut self,
        ty: &Node<resolved::TypeExpression>,
    ) -> Result<Type, Diagnostic> {
        let expanded = self.expand_type_term(ty)?;
        ensure_value_type(&expanded, ty.span)?;
        ensure_representable(&expanded, ty.span)?;
        Ok(expanded)
    }

    pub(in crate::check) fn expand_type_term(
        &mut self,
        ty: &Node<resolved::TypeExpression>,
    ) -> Result<Type, Diagnostic> {
        self.expand([Expansion::Expression(
            ty.clone(),
            self.type_substitutions.clone(),
        )])
    }

    pub(in crate::check) fn expand_type_id(
        &mut self,
        id: TypeId,
        use_span: Span,
    ) -> Result<Type, Diagnostic> {
        let expanded = self.expand_term_id(id, use_span)?;
        ensure_value_type(&expanded, use_span)?;
        ensure_representable(&expanded, use_span)?;
        Ok(expanded)
    }

    pub(in crate::check) fn expand_term_id(
        &mut self,
        id: TypeId,
        use_span: Span,
    ) -> Result<Type, Diagnostic> {
        self.expand([Expansion::Reference(
            id,
            use_span,
            self.type_substitutions.clone(),
        )])
    }

    pub(super) fn expand_expression(
        &mut self,
        expression: Node<resolved::TypeExpression>,
        substitutions: Substitutions,
    ) -> Result<Type, Diagnostic> {
        self.expand([Expansion::Expression(expression, substitutions)])
    }

    fn expand(&mut self, initial: impl IntoIterator<Item = Expansion>) -> Result<Type, Diagnostic> {
        let mut machine = Machine::new(initial);
        while let Some(expansion) = machine.pending.pop() {
            match expansion {
                Expansion::Expression(expression, substitutions) => {
                    self.schedule_expression(&mut machine, expression, substitutions);
                }
                Expansion::Reference(id, use_span, substitutions) => {
                    self.expand_reference(&mut machine, id, use_span, substitutions)?;
                }
                Expansion::ApplyArguments {
                    arguments,
                    phantom,
                    substitutions,
                    index,
                } => machine.apply_arguments(arguments, phantom, substitutions, index)?,
                Expansion::ApplyOne(span) => machine.apply_one(span)?,
                Expansion::Alias(id) => self.finish_alias(&machine, id),
                Expansion::AliasAbstraction {
                    id,
                    parameter_kinds,
                } => self.finish_generic_alias(&mut machine, id, parameter_kinds)?,
                Expansion::OpaqueAbstraction {
                    id,
                    arguments,
                    parameter_kinds,
                } => self.finish_opaque(&mut machine, id, arguments, parameter_kinds)?,
                Expansion::Product(length) => {
                    let elements = machine.take_last(length);
                    machine.values.push(Type::Product(elements.into()));
                }
                Expansion::Sum(length) => {
                    let members = machine.take_last(length);
                    machine.values.push(Type::Sum(members.into()));
                }
                Expansion::Function => {
                    let [parameter, result] = machine.take_last(2).try_into().unwrap();
                    machine.values.push(Type::Function {
                        parameter: parameter.into(),
                        result: result.into(),
                    });
                }
            }
        }
        let [value] = machine
            .values
            .try_into()
            .expect("one expansion must produce one type");
        Ok(value)
    }

    /// Schedules the expansion of one source expression: its children first, then the step that combines them.
    fn schedule_expression(
        &self,
        machine: &mut Machine,
        expression: Node<resolved::TypeExpression>,
        substitutions: Substitutions,
    ) {
        match expression.kind {
            resolved::TypeExpression::Named(reference) => {
                machine.pending.push(Expansion::Reference(
                    reference.id,
                    reference.name.span,
                    substitutions,
                ));
            }
            resolved::TypeExpression::Application {
                constructor,
                arguments,
            } => {
                let phantom = self
                    .generic_aliases
                    .get(&constructor.id)
                    .map(|definition| definition.used_parameters.clone())
                    .unwrap_or_default();
                machine.pending.push(Expansion::ApplyArguments {
                    arguments,
                    phantom,
                    substitutions: substitutions.clone(),
                    index: 0,
                });
                machine.pending.push(Expansion::Reference(
                    constructor.id,
                    constructor.name.span,
                    substitutions,
                ));
            }
            resolved::TypeExpression::Unit => machine.values.push(Type::Unit),
            resolved::TypeExpression::Parenthesized(inner) => {
                machine
                    .pending
                    .push(Expansion::Expression(*inner, substitutions));
            }
            resolved::TypeExpression::Product(elements) => {
                machine.pending.push(Expansion::Product(elements.len()));
                machine.pending.extend(
                    elements
                        .into_iter()
                        .rev()
                        .map(|element| Expansion::Expression(element, substitutions.clone())),
                );
            }
            resolved::TypeExpression::Sum(members) => {
                machine.pending.push(Expansion::Sum(members.len()));
                machine.pending.extend(
                    members
                        .into_iter()
                        .rev()
                        .map(|member| Expansion::Expression(member, substitutions.clone())),
                );
            }
            resolved::TypeExpression::Function { parameter, result } => {
                machine.pending.push(Expansion::Function);
                machine
                    .pending
                    .push(Expansion::Expression(*result, substitutions.clone()));
                machine
                    .pending
                    .push(Expansion::Expression(*parameter, substitutions));
            }
        }
    }

    pub(super) fn declaration_parameter_kinds(
        &mut self,
        id: TypeId,
        count: usize,
        span: Span,
    ) -> Result<Vec<Kind>, Diagnostic> {
        let mut kind = self
            .kinds
            .declaration(id, &mut self.next_kind_variable)
            .expect("source type declaration has an inferred kind");
        let mut parameters = Vec::with_capacity(count);
        for _ in 0..count {
            let Kind::Function { parameter, result } = kind else {
                return Err(Diagnostic::error("type kind mismatch")
                    .with_primary(span, "declaration has fewer constructor parameters"));
            };
            parameters.push(parameter.as_ref().clone());
            kind = result.as_ref().clone();
        }
        Ok(parameters)
    }
}
