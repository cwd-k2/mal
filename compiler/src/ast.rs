use crate::lexer::IntegerLiteral;
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
    pub items: Vec<Node<TopItem>>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TopItem {
    TypeAlias {
        name: Name,
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeExpression {
    Named(Name),
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
    Integer(IntegerLiteral),
    Unit,
    Parenthesized(Box<Node<Expression>>),
    Product(Vec<Node<Expression>>),
    Lambda(Lambda),
    Call {
        callee: Box<Node<Expression>>,
        arguments: Vec<Node<Expression>>,
    },
    ExternalCall {
        name: Name,
        arguments: Vec<Node<Expression>>,
    },
    SumInjection {
        type_name: Name,
        index: Node<IntegerLiteral>,
        value: Box<Node<Expression>>,
    },
    If {
        condition: Box<Node<Expression>>,
        then_branch: ExpressionBlock,
        else_branch: ExpressionBlock,
    },
    Case {
        scrutinee: Box<Node<Expression>>,
        arms: Vec<CaseArm>,
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
    pub captures: Vec<Name>,
    pub parameters: Vec<Parameter>,
    pub body: LambdaBody,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    pub name: Name,
    pub ty: Node<TypeExpression>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LambdaBody {
    pub items: Vec<BodyItem>,
    pub result: Box<Node<Expression>>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpressionBlock {
    pub items: Vec<BodyItem>,
    pub result: Box<Node<Expression>>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BodyItem {
    Binding(Node<Binding>),
    Expression(Node<Expression>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseArm {
    pub index: Node<IntegerLiteral>,
    pub pattern: Node<Pattern>,
    pub value: Node<Expression>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOperator {
    Negate,
    LogicalNot,
    BitwiseNot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOperator {
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
