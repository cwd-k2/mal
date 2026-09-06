use crate::ast::{BinaryOperator, Name, Node, UnaryOperator};
use crate::lexer::{DecimalFloatLiteral, IntegerLiteral};
use crate::source::Span;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TypeId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ValueId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ExternalOperationId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LambdaId(pub u32);

pub use super::predefined::{
    BOOL_TYPE, FALSE_VALUE, FLOAT32_TYPE, FLOAT64_TYPE, INT8_TYPE, INT16_TYPE, INT32_TYPE,
    INT64_TYPE, LOAD_FLOAT32_VALUE, LOAD_FLOAT64_VALUE, LOAD_INT8_VALUE, LOAD_INT16_VALUE,
    LOAD_INT32_VALUE, LOAD_INT64_VALUE, LOAD_PTR_VALUE, LOAD_SYMBOL_VALUE, LOAD_UINT8_VALUE,
    LOAD_UINT16_VALUE, LOAD_UINT32_VALUE, LOAD_UINT64_VALUE, PTR_TYPE, STORE_FLOAT32_VALUE,
    STORE_FLOAT64_VALUE, STORE_INT8_VALUE, STORE_INT16_VALUE, STORE_INT32_VALUE, STORE_INT64_VALUE,
    STORE_PTR_VALUE, STORE_SYMBOL_VALUE, STORE_UINT8_VALUE, STORE_UINT16_VALUE, STORE_UINT32_VALUE,
    STORE_UINT64_VALUE, SYMBOL_TYPE, TRUE_VALUE, UINT8_TYPE, UINT16_TYPE, UINT32_TYPE, UINT64_TYPE,
    UNIT_TYPE,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueOwner {
    Predefined,
    TopLevel,
    Lambda(LambdaId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeBinding {
    pub id: TypeId,
    pub name: Name,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueBinding {
    pub id: ValueId,
    pub name: Name,
    pub owner: ValueOwner,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeReference {
    pub id: TypeId,
    pub name: Name,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueReference {
    pub id: ValueId,
    pub name: Name,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalOperationReference {
    pub id: ExternalOperationId,
    pub name: Name,
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
        value: Node<TypeExpression>,
    },
    ExternalType {
        binding: TypeBinding,
    },
    ExternalOperation {
        id: ExternalOperationId,
        name: Name,
        ty: Node<TypeExpression>,
    },
    Binding(Binding),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeExpression {
    Named(TypeReference),
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
    Binding(ValueBinding),
    Wildcard,
    Product(Vec<Node<Pattern>>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    Reference(ValueReference),
    Integer(IntegerLiteral),
    Float(DecimalFloatLiteral),
    Byte(u8),
    Symbol(Vec<u8>),
    StorageSize(Node<TypeExpression>),
    Unit,
    Parenthesized(Box<Node<Expression>>),
    Product(Vec<Node<Expression>>),
    Lambda(Lambda),
    Call {
        callee: Box<Node<Expression>>,
        arguments: Vec<Node<Expression>>,
    },
    ExternalCall {
        operation: ExternalOperationReference,
        arguments: Vec<Node<Expression>>,
    },
    Conversion {
        type_ref: TypeReference,
        value: Box<Node<Expression>>,
    },
    SumInjection {
        type_ref: TypeReference,
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
    pub id: LambdaId,
    pub self_binding: Option<ValueId>,
    pub captures: Vec<Capture>,
    pub parameters: Vec<Parameter>,
    pub body: LambdaBody,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capture {
    pub source: ValueReference,
    pub binding: ValueBinding,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    pub binding: ValueBinding,
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
    pub body: ExpressionBlock,
    pub span: Span,
}
