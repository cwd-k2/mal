//! A mutable walk over the closure program that names every identity a rewrite can touch.

use crate::anf::ast::ValueId;
use crate::closure::ast::{
    Atom, Block, Function, FunctionId, Operation, Pattern, Program, TopLevelBinding,
    TopLevelPattern,
};

/// Callbacks for the identities of one walk. Binders are reported where they bind, so a rewrite that renames
/// them sees each binder before the references that follow it in the same scope.
pub(crate) trait Visitor {
    fn binder(&mut self, _id: &mut ValueId) {}
    fn atom(&mut self, _atom: &mut Atom) {}
    /// The function a closure creation instantiates.
    fn created(&mut self, _function: &mut FunctionId) {}
    /// The identity a function definition declares.
    fn defined(&mut self, _function: &mut FunctionId) {}
}

pub(crate) fn program(program: &mut Program, visitor: &mut impl Visitor) {
    for binding in &mut program.bindings {
        top_level(binding, visitor);
    }
    for function in &mut program.functions {
        self::function(function, visitor);
    }
}

pub(crate) fn top_level(binding: &mut TopLevelBinding, visitor: &mut impl Visitor) {
    top_level_pattern(&mut binding.pattern, visitor);
    block(&mut binding.value, visitor);
}

pub(crate) fn function(function: &mut Function, visitor: &mut impl Visitor) {
    visitor.defined(&mut function.id);
    if let Some(binding) = &mut function.parameter.binding {
        visitor.binder(binding);
    }
    for join in &mut function.joins {
        pattern(&mut join.parameter, visitor);
        block(&mut join.body, visitor);
    }
    block(&mut function.body, visitor);
}

fn top_level_pattern(pattern: &mut TopLevelPattern, visitor: &mut impl Visitor) {
    match pattern {
        TopLevelPattern::Binding { id, .. } => visitor.binder(id),
        TopLevelPattern::Product { elements, .. } => {
            for element in elements {
                top_level_pattern(element, visitor);
            }
        }
        TopLevelPattern::Wildcard { .. } => {}
    }
}

fn pattern(pattern: &mut Pattern, visitor: &mut impl Visitor) {
    match pattern {
        Pattern::Binding { id, .. } => visitor.binder(id),
        Pattern::Product { elements, .. } => {
            for element in elements {
                self::pattern(element, visitor);
            }
        }
        Pattern::Wildcard { .. } => {}
    }
}

fn block(block: &mut Block, visitor: &mut impl Visitor) {
    for binding in &mut block.bindings {
        operation(&mut binding.operation, visitor);
        pattern(&mut binding.pattern, visitor);
    }
    visitor.atom(&mut block.result);
}

fn operation(operation: &mut Operation, visitor: &mut impl Visitor) {
    match operation {
        Operation::Atom(atom)
        | Operation::Goto { value: atom, .. }
        | Operation::ExternalCall { argument: atom, .. }
        | Operation::NumericConversion { operand: atom }
        | Operation::SumInjection { value: atom, .. }
        | Operation::PrimitiveUnary { operand: atom, .. } => visitor.atom(atom),
        Operation::MakeClosure { function, captures } => {
            visitor.created(function);
            for atom in captures {
                visitor.atom(atom);
            }
        }
        Operation::Call { callee, argument } => {
            visitor.atom(callee);
            visitor.atom(argument);
        }
        Operation::Memory { operands, .. }
        | Operation::Buffer { operands, .. }
        | Operation::Symbol { operands, .. } => {
            for atom in operands {
                visitor.atom(atom);
            }
        }
        Operation::Product(atoms) => {
            for atom in atoms {
                visitor.atom(atom);
            }
        }
        Operation::Case { scrutinee, arms } => {
            visitor.atom(scrutinee);
            for arm in arms {
                pattern(&mut arm.pattern, visitor);
                self::block(&mut arm.value, visitor);
            }
        }
        Operation::PrimitiveBranch {
            left,
            right,
            otherwise,
            then,
            ..
        } => {
            visitor.atom(left);
            visitor.atom(right);
            self::block(otherwise, visitor);
            self::block(then, visitor);
        }
        Operation::PrimitiveBinary { left, right, .. } => {
            visitor.atom(left);
            visitor.atom(right);
        }
    }
}
