use crate::ast::{BinaryOperator, Name, Node, UnaryOperator};
use crate::resolve::ast::{
    ExternalOperationId, LambdaId, TypeBinding, ValueBinding, ValueReference,
};
use crate::source::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Unit,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Sum(Vec<Type>),
    Function {
        parameter: Box<Type>,
        result: Box<Type>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub items: Vec<Node<TopItem>>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TopItem {
    TypeAlias {
        binding: TypeBinding,
        ty: Type,
    },
    ExternalOperation {
        id: ExternalOperationId,
        name: Name,
        parameter: Type,
        result: Type,
    },
    Binding(Box<Binding>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    pub pattern: Pattern,
    pub annotation: Option<Type>,
    pub value: Expression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pattern {
    Binding { binding: ValueBinding, ty: Type },
    Wildcard { ty: Type, span: Span },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpressionKind {
    Reference(ValueReference),
    Integer(i128),
    Unit,
    Parenthesized(Box<Expression>),
    Lambda(Lambda),
    Call {
        callee: Box<Expression>,
        argument: Box<Expression>,
    },
    ExternalCall {
        id: ExternalOperationId,
        name: Name,
        argument: Box<Expression>,
    },
    IntegerConversion {
        value: Box<Expression>,
    },
    SumInjection {
        index: usize,
        value: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        then_branch: ExpressionBlock,
        else_branch: ExpressionBlock,
    },
    Case {
        scrutinee: Box<Expression>,
        arms: Vec<CaseArm>,
    },
    Unary {
        operator: Node<UnaryOperator>,
        operand: Box<Expression>,
    },
    Binary {
        operator: Node<BinaryOperator>,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lambda {
    pub id: LambdaId,
    pub captures: Vec<Capture>,
    pub parameters: Vec<Parameter>,
    pub body: LambdaBody,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capture {
    pub source: ValueReference,
    pub binding: ValueBinding,
    pub ty: Type,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    pub binding: ValueBinding,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LambdaBody {
    pub items: Vec<BodyItem>,
    pub result: Box<Expression>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpressionBlock {
    pub items: Vec<BodyItem>,
    pub result: Box<Expression>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BodyItem {
    Binding(Binding),
    Expression(Expression),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseArm {
    pub index: usize,
    pub pattern: Pattern,
    pub value: Expression,
    pub span: Span,
}
