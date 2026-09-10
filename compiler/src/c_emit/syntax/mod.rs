// Header and host-stub generation share one validated C syntax layer. Some nodes are
// exercised only by its renderer tests so that future interface changes cannot bypass it.
#![allow(dead_code)]

mod declaration;
mod expression;
mod expression_render;
#[cfg(test)]
mod expression_tests;
mod literal;
mod name;
mod operator;
mod preprocessor;
mod statement;
mod translation_unit;
mod unit;

pub(super) use self::declaration::{
    FunctionSignature, FunctionSpecifier, Parameter, TypeName, VariableDeclaration,
};
pub(super) use self::expression::{Expr, Initializer};
pub(super) use self::literal::{NumericLiteral, StringLiteral};
pub(super) use self::name::Identifier;
pub(super) use self::operator::{BinaryOperator, UnaryOperator};
pub(super) use self::preprocessor::{Attribute, Directive, MacroInvocation, PreprocessorExpr};
pub(super) use self::statement::{Block, FunctionDefinition, Statement, SwitchCase};
pub(super) use self::translation_unit::TranslationUnit;
pub(super) use self::unit::{
    AggregateDefinition, AggregateField, AggregateKind, Comment, Declaration,
};
