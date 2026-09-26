mod declaration;
mod expression;
mod literal;
mod macros;
mod name;
mod operator;
mod preprocessor;
mod render;
mod statement;
mod translation_unit;
mod unit;

pub(in crate::backend) use macros::{
    c_aggregate, c_aggregate_field, c_aggregate_fields, c_aggregate_fields_item, c_block,
    c_block_item, c_comment, c_declaration, c_directive, c_expr, c_expr_child, c_exprs,
    c_exprs_item, c_function, c_function_build, c_initializer, c_initializers, c_initializers_item,
    c_macro_invocation, c_parameter, c_parameter_attributes, c_parameters, c_parameters_items,
    c_preprocessor_expr, c_scalar, c_signature, c_signature_from_parts, c_statement, c_switch_case,
    c_switch_cases, c_switch_cases_item, c_type, c_variable,
};

pub(in crate::backend) use self::declaration::{
    FunctionSignature, FunctionSpecifier, Parameter, TypeName, VariableDeclaration,
};
pub(in crate::backend) use self::expression::{Expr, Initializer};
pub(in crate::backend) use self::literal::{NumericLiteral, StringLiteral};
pub(in crate::backend) use self::name::Identifier;
pub(in crate::backend) use self::operator::{BinaryOperator, UnaryOperator};
pub(in crate::backend) use self::preprocessor::{
    Attribute, Directive, MacroInvocation, PreprocessorExpr,
};
pub(in crate::backend) use self::statement::{Block, FunctionDefinition, Statement, SwitchCase};
pub(in crate::backend) use self::translation_unit::TranslationUnit;
pub(in crate::backend) use self::unit::{
    AggregateDefinition, AggregateField, AggregateKind, Comment, Declaration,
};
