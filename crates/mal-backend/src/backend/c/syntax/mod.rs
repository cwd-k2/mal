mod declaration;
mod expression;
mod literal;
mod name;
mod operator;
mod preprocessor;
mod render;
mod statement;
mod translation_unit;
mod unit;

macro_rules! c_expr {
    (id $name:expr) => {
        $crate::backend::c::syntax::Expr::identifier($name)
    };
    (number $value:expr) => {
        $crate::backend::c::syntax::Expr::number($value.to_string())
    };
    (string $value:expr) => {
        $crate::backend::c::syntax::Expr::string($value)
    };
    (address $value:tt) => {
        $crate::backend::c::syntax::Expr::address_of($crate::backend::c::syntax::c_expr! $value)
    };
    (sizeof $value:tt) => {
        $crate::backend::c::syntax::Expr::sizeof_value($crate::backend::c::syntax::c_expr! $value)
    };
    (cast $ty:expr; $value:tt) => {
        $crate::backend::c::syntax::Expr::cast($ty, $crate::backend::c::syntax::c_expr! $value)
    };
    (call $name:expr; $($argument:tt),* $(,)?) => {
        $crate::backend::c::syntax::Expr::named_call(
            $name,
            [$($crate::backend::c::syntax::c_expr! $argument),*],
        )
    };
    (add $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::add(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (subtract $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::subtract(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (greater $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::greater(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (conditional $condition:tt; $then:tt; $otherwise:tt) => {
        $crate::backend::c::syntax::Expr::conditional(
            $crate::backend::c::syntax::c_expr! $condition,
            $crate::backend::c::syntax::c_expr! $then,
            $crate::backend::c::syntax::c_expr! $otherwise,
        )
    };
    (initializer $($element:tt),* $(,)?) => {
        $crate::backend::c::syntax::Expr::initializer_list([
            $($crate::backend::c::syntax::c_expr! $element),*
        ])
    };
}

macro_rules! c_statement {
    (call $name:expr; $($argument:tt),* $(,)?) => {
        $crate::backend::c::syntax::Statement::expression($crate::backend::c::syntax::c_expr!(
            call $name; $($argument),*
        ))
    };
    (return $value:tt) => {
        $crate::backend::c::syntax::Statement::return_value(
            $crate::backend::c::syntax::c_expr! $value
        )
    };
}

pub(in crate::backend) use c_expr;
pub(in crate::backend) use c_statement;

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

#[cfg(test)]
mod macro_tests {
    #[test]
    fn expression_macro_recursively_builds_typed_nodes() {
        let expression = super::c_expr!(conditional
            (greater (id "count"); (number 0));
            (call "read"; (address (id "value")));
            (cast "size_t"; (number 0))
        );

        assert_eq!(
            expression.to_string(),
            "(count > 0) ? read(&value) : (size_t)0"
        );
    }
}
