use crate::c_emit::syntax::{Block, Expr, Initializer, Statement, TypeName};
use crate::check::ast::Type;
use crate::closure::ast::{Atom, AtomKind, FunctionId, Pattern, Reference};

use super::super::{
    BodyEmitter, ResultOwnership, environment_destroy_name, environment_name, function_name,
    value_name,
};

impl BodyEmitter<'_> {
    pub(super) fn emit_simple_result(
        &mut self,
        block: &mut Block,
        pattern: &Pattern,
        ty: &Type,
        expression: Expr,
        ownership: ResultOwnership,
    ) {
        self.emit_simple_result_with_transfers(block, pattern, ty, expression, ownership, &[]);
    }

    pub(super) fn emit_simple_result_with_transfers(
        &mut self,
        block: &mut Block,
        pattern: &Pattern,
        ty: &Type,
        expression: Expr,
        ownership: ResultOwnership,
        transfers: &[&Atom],
    ) {
        let expression = if ownership == ResultOwnership::Owned {
            expression
        } else {
            self.types.copy_value(ty, expression)
        };
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
                self.clear_transferred_atoms(block, transfers);
            }
            Pattern::Wildcard { .. } => {
                if self.types.contains_managed(ty) {
                    let target = self.result_target(pattern);
                    block.push(Statement::variable(
                        self.types.c_type(ty),
                        target.clone(),
                        Some(expression),
                    ));
                    self.types
                        .destroy_value(block, ty, Expr::identifier(target));
                } else {
                    block.push(Statement::expression(Expr::cast("void", expression)));
                }
                self.clear_transferred_atoms(block, transfers);
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
                self.clear_transferred_atoms(block, transfers);
                self.emit_owned_pattern_bindings(block, pattern, Expr::identifier(target));
            }
        }
    }

    pub(in crate::c_emit::body) fn materialize_atom<'a>(
        &self,
        atom: &'a Atom,
        transfers: &mut Vec<&'a Atom>,
    ) -> Expr {
        let expression = self.emit_atom(atom);
        if self.types.contains_managed(&atom.ty)
            && self.ownership.can_transfer(atom)
            && matches!(atom.kind, AtomKind::Reference(Reference::Binding(_)))
        {
            transfers.push(atom);
            expression
        } else {
            self.types.copy_value(&atom.ty, expression)
        }
    }

    pub(in crate::c_emit::body) fn clear_transferred_atoms(
        &self,
        block: &mut Block,
        transfers: &[&Atom],
    ) {
        for atom in transfers {
            let AtomKind::Reference(Reference::Binding(id)) = atom.kind else {
                unreachable!("only binding references can transfer ownership")
            };
            block.push(Statement::assignment(
                Expr::identifier(value_name(id)),
                Expr::compound_literal(
                    self.types.c_type(&atom.ty),
                    [Initializer::positional(Expr::number("0"))],
                ),
            ));
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
                Initializer::designated(
                    format!("field_{index}"),
                    self.types.copy_value(&atom.ty, self.emit_atom(atom)),
                )
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
                    Initializer::positional(if captures.is_empty() {
                        Expr::identifier("NULL")
                    } else {
                        Expr::identifier(environment_destroy_name(function))
                    }),
                ],
            )),
        ));
        block.push(Statement::expression(Expr::cast(
            "void",
            Expr::identifier(target.clone()),
        )));
        if matches!(pattern, Pattern::Wildcard { .. }) {
            self.types
                .destroy_value(block, ty, Expr::identifier(target));
        }
    }
}
