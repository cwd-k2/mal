use crate::c_emit::syntax::{Block, Expr, Initializer, Statement, TypeName};
use crate::check::ast::Type;
use crate::closure::ast::{Atom, FunctionId, Pattern};

use super::super::{BodyEmitter, environment_name, function_name, value_name};

impl BodyEmitter<'_> {
    pub(super) fn emit_simple_result(
        &mut self,
        block: &mut Block,
        pattern: &Pattern,
        ty: &Type,
        expression: Expr,
    ) {
        match pattern {
            Pattern::Binding { id, .. } => {
                let name = value_name(*id);
                block.push(Statement::variable(
                    self.types.c_type(ty),
                    &name,
                    Some(expression),
                ));
                block.push(Statement::expression(Expr::cast(
                    "void",
                    Expr::identifier(name),
                )));
            }
            Pattern::Wildcard { .. } => {
                block.push(Statement::expression(Expr::cast("void", expression)))
            }
            Pattern::Product { .. } => {
                let target = self.result_target(pattern);
                block.push(Statement::variable(
                    self.types.c_type(ty),
                    target.clone(),
                    Some(expression),
                ));
                block.push(Statement::expression(Expr::cast(
                    "void",
                    Expr::identifier(target.clone()),
                )));
                self.emit_pattern_bindings(block, pattern, Expr::identifier(target));
            }
        }
    }

    pub(super) fn emit_unit_result(&self, block: &mut Block, pattern: &Pattern) {
        match pattern {
            Pattern::Binding { id, .. } => {
                let name = value_name(*id);
                block.push(Statement::variable(
                    "MalType_Unit",
                    &name,
                    Some(Expr::compound_literal(
                        "MalType_Unit",
                        [Initializer::positional(Expr::named_call(
                            "UINT8_C",
                            [Expr::number("0")],
                        ))],
                    )),
                ));
                block.push(Statement::expression(Expr::cast(
                    "void",
                    Expr::identifier(name),
                )));
            }
            Pattern::Wildcard { .. } => {}
            Pattern::Product { .. } => {
                unreachable!("a product pattern cannot match a Unit operation")
            }
        }
    }

    pub(super) fn emit_make_closure(
        &mut self,
        block: &mut Block,
        pattern: &Pattern,
        ty: &Type,
        function: FunctionId,
        captures: &[Atom],
    ) {
        let target = self.result_target(pattern);
        let environment = if captures.is_empty() {
            Expr::identifier("NULL")
        } else {
            let allocation = format!("mal_new_environment_{target}");
            let environment_type = environment_name(function);
            block.push(Statement::variable(
                TypeName::named(environment_type.clone()).pointer(),
                allocation.clone(),
                Some(Expr::cast(
                    TypeName::named(environment_type.clone()).pointer(),
                    Expr::named_call(
                        "mal_allocate",
                        [
                            Expr::identifier("mal_context"),
                            Expr::sizeof_type(environment_type.clone()),
                        ],
                    ),
                )),
            ));
            let fields = captures.iter().enumerate().map(|(index, atom)| {
                Initializer::designated(format!("field_{index}"), self.emit_atom(atom))
            });
            block.push(Statement::assignment(
                Expr::dereference(Expr::identifier(allocation.clone())),
                Expr::compound_literal(environment_type, fields),
            ));
            Expr::identifier(allocation)
        };
        block.push(Statement::variable(
            self.types.c_type(ty),
            &target,
            Some(Expr::compound_literal(
                self.types.c_type(ty),
                [
                    Initializer::positional(Expr::identifier(function_name(function))),
                    Initializer::positional(environment),
                ],
            )),
        ));
        block.push(Statement::expression(Expr::cast(
            "void",
            Expr::identifier(target),
        )));
    }
}
