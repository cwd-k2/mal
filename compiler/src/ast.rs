use crate::lexer::{DecimalFloatLiteral, IntegerLiteral};
use crate::source::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node<T> {
    pub kind: T,
    pub span: Span,
}

impl<T> Node<T> {
    pub const fn new(kind: T, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Name {
    pub text: String,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub requirements: Vec<Node<Requirement>>,
    pub items: Vec<Node<TopItem>>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Requirement {
    pub path: Vec<u8>,
    pub path_span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TopItem {
    TypeAlias {
        name: Name,
        value: Node<TypeExpression>,
    },
    GenericTypeAlias {
        name: Name,
        parameters: Vec<Name>,
        value: Node<TypeExpression>,
    },
    ExternalType {
        name: Name,
    },
    ExternalOperation {
        name: Name,
        ty: Node<TypeExpression>,
    },
    Binding(Binding),
    GenericBinding {
        name: Name,
        parameters: Vec<Name>,
        annotation: Node<TypeExpression>,
        value: Node<Expression>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeExpression {
    Named(Name),
    Application {
        constructor: Name,
        arguments: Vec<Node<TypeExpression>>,
    },
    Unit,
    Parenthesized(Box<Node<TypeExpression>>),
    Product(Vec<Node<TypeExpression>>),
    Sum(Vec<Node<TypeExpression>>),
    Function {
        parameter: Box<Node<TypeExpression>>,
        result: Box<Node<TypeExpression>>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    pub pattern: Node<Pattern>,
    pub annotation: Option<Node<TypeExpression>>,
    pub value: Node<Expression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pattern {
    Name(Name),
    Wildcard,
    Product(Vec<Node<Pattern>>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    Name(Name),
    GenericName {
        name: Name,
        arguments: Vec<Node<TypeExpression>>,
    },
    Integer(IntegerLiteral),
    Float(DecimalFloatLiteral),
    Byte(u8),
    Symbol(Vec<u8>),
    Unit,
    Parenthesized(Box<Node<Expression>>),
    Product(Vec<Node<Expression>>),
    Block(ExpressionBlock),
    ResultBlock {
        result_binders: Vec<Name>,
        body: ExpressionBlock,
    },
    Lambda(Lambda),
    Call {
        callee: Box<Node<Expression>>,
        arguments: Vec<Node<Expression>>,
    },
    ContinuationApplication {
        value: Box<Node<Expression>>,
        continuations: Vec<Node<Expression>>,
    },
    Conversion {
        type_name: Name,
        value: Box<Node<Expression>>,
    },
    If {
        condition: Box<Node<Expression>>,
        then_branch: ExpressionBlock,
        else_branch: ExpressionBlock,
    },
    When {
        condition: Box<Node<Expression>>,
        body: ExpressionBlock,
    },
    Unary {
        operator: Node<UnaryOperator>,
        operand: Box<Node<Expression>>,
    },
    Binary {
        operator: Node<BinaryOperator>,
        left: Box<Node<Expression>>,
        right: Box<Node<Expression>>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lambda {
    pub parameter: Option<Box<Node<Pattern>>>,
    pub body: LambdaBody,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpressionBlock {
    pub items: Vec<BodyItem>,
    pub result: Box<Node<Expression>>,
    pub span: Span,
}

pub type LambdaBody = ExpressionBlock;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BodyItem {
    Binding(Node<Binding>),
    Expression(Node<Expression>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOperator {
    Negate,
    LogicalNot,
    BitwiseNot,
    SymbolLength,
    Star,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOperator {
    SymbolAt,
    Multiply,
    Divide,
    Remainder,
    Add,
    Subtract,
    ShiftLeft,
    ShiftRight,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    LogicalAnd,
    LogicalOr,
}
