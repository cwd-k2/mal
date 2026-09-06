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

pub const UNIT_TYPE: TypeId = TypeId(0);
pub const INT8_TYPE: TypeId = TypeId(1);
pub const INT16_TYPE: TypeId = TypeId(2);
pub const INT32_TYPE: TypeId = TypeId(3);
pub const INT64_TYPE: TypeId = TypeId(4);
pub const UINT8_TYPE: TypeId = TypeId(5);
pub const UINT16_TYPE: TypeId = TypeId(6);
pub const UINT32_TYPE: TypeId = TypeId(7);
pub const UINT64_TYPE: TypeId = TypeId(8);
pub const BOOL_TYPE: TypeId = TypeId(9);
pub const SYMBOL_TYPE: TypeId = TypeId(10);
pub const FLOAT32_TYPE: TypeId = TypeId(11);
pub const FLOAT64_TYPE: TypeId = TypeId(12);
pub const PTR_TYPE: TypeId = TypeId(13);
pub const FALSE_VALUE: ValueId = ValueId(0);
pub const TRUE_VALUE: ValueId = ValueId(1);
pub const LOAD_INT64_VALUE: ValueId = ValueId(2);
pub const STORE_INT64_VALUE: ValueId = ValueId(3);
pub const LOAD_UINT8_VALUE: ValueId = ValueId(4);
pub const STORE_UINT8_VALUE: ValueId = ValueId(5);
pub const LOAD_INT8_VALUE: ValueId = ValueId(6);
pub const STORE_INT8_VALUE: ValueId = ValueId(7);
pub const LOAD_INT16_VALUE: ValueId = ValueId(8);
pub const STORE_INT16_VALUE: ValueId = ValueId(9);
pub const LOAD_INT32_VALUE: ValueId = ValueId(10);
pub const STORE_INT32_VALUE: ValueId = ValueId(11);
pub const LOAD_UINT16_VALUE: ValueId = ValueId(12);
pub const STORE_UINT16_VALUE: ValueId = ValueId(13);
pub const LOAD_UINT32_VALUE: ValueId = ValueId(14);
pub const STORE_UINT32_VALUE: ValueId = ValueId(15);
pub const LOAD_UINT64_VALUE: ValueId = ValueId(16);
pub const STORE_UINT64_VALUE: ValueId = ValueId(17);
pub const LOAD_FLOAT32_VALUE: ValueId = ValueId(18);
pub const STORE_FLOAT32_VALUE: ValueId = ValueId(19);
pub const LOAD_FLOAT64_VALUE: ValueId = ValueId(20);
pub const STORE_FLOAT64_VALUE: ValueId = ValueId(21);
pub const LOAD_PTR_VALUE: ValueId = ValueId(22);
pub const STORE_PTR_VALUE: ValueId = ValueId(23);
pub const LOAD_SYMBOL_VALUE: ValueId = ValueId(24);
pub const STORE_SYMBOL_VALUE: ValueId = ValueId(25);

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
