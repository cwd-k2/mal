use crate::ast::{BinaryOperator, Node, UnaryOperator};
use crate::resolve::ast::{
    ExternalOperationId, LambdaId, TypeBinding, TypeId, ValueBinding, ValueId, ValueReference,
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
    Float32,
    Float64,
    Symbol,
    Ptr,
    External {
        id: TypeId,
        name: String,
    },
    Product(Vec<Type>),
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
        target_alias: Option<String>,
        element_aliases: Vec<Option<String>>,
    },
    ExternalType {
        binding: TypeBinding,
    },
    ExternalOperation {
        id: ExternalOperationId,
        binding: ValueBinding,
        lambda_id: LambdaId,
        parameter: Type,
        parameter_alias: Option<String>,
        parameter_aliases: Vec<Option<String>>,
        result: Type,
        result_alias: Option<String>,
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
    Binding {
        binding: ValueBinding,
        ty: Type,
    },
    Wildcard {
        ty: Type,
        span: Span,
    },
    Product {
        elements: Vec<Pattern>,
        ty: Type,
        span: Span,
    },
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
    Float(u64),
    Symbol(Vec<u8>),
    StorageSize(Type),
    Unit,
    Product(Vec<Expression>),
    Parenthesized(Box<Expression>),
    Lambda(Lambda),
    Call {
        callee: Box<Expression>,
        argument: Box<Expression>,
    },
    SymbolLength {
        value: Box<Expression>,
    },
    SymbolAt {
        argument: Box<Expression>,
    },
    MemoryFunction {
        primitive: MemoryPrimitive,
    },
    Memory {
        primitive: MemoryPrimitive,
        argument: Box<Expression>,
    },
    NumericConversion {
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MemoryPrimitive {
    OffsetForward,
    OffsetBackward,
    Load(MemoryScalar),
    Store(MemoryScalar),
    LoadPtr,
    StorePtr,
    LoadSymbol,
    StoreSymbol,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MemoryScalar {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
}

impl MemoryPrimitive {
    pub(crate) fn signature(self) -> (Type, Type) {
        match self {
            Self::OffsetForward | Self::OffsetBackward => {
                (Type::Product(vec![Type::Ptr, Type::UInt64]), Type::Ptr)
            }
            Self::Load(scalar) => (Type::Ptr, scalar.ty()),
            Self::Store(scalar) => (Type::Product(vec![Type::Ptr, scalar.ty()]), Type::Unit),
            Self::LoadPtr => (Type::Ptr, Type::Ptr),
            Self::StorePtr => (Type::Product(vec![Type::Ptr, Type::Ptr]), Type::Unit),
            Self::LoadSymbol => (Type::Product(vec![Type::Ptr, Type::UInt64]), Type::Symbol),
            Self::StoreSymbol => (Type::Product(vec![Type::Ptr, Type::Symbol]), Type::Unit),
        }
    }
}

impl MemoryScalar {
    fn ty(self) -> Type {
        match self {
            Self::Int8 => Type::Int8,
            Self::Int16 => Type::Int16,
            Self::Int32 => Type::Int32,
            Self::Int64 => Type::Int64,
            Self::UInt8 => Type::UInt8,
            Self::UInt16 => Type::UInt16,
            Self::UInt32 => Type::UInt32,
            Self::UInt64 => Type::UInt64,
            Self::Float32 => Type::Float32,
            Self::Float64 => Type::Float64,
        }
    }
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
    pub body: ExpressionBlock,
    pub span: Span,
}
