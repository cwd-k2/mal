use crate::check::ast::{MemoryPrimitive, Type};
use crate::resolve::ast::{ExternalOperationId, LambdaId, ValueId as SourceValueId};
use crate::source::Span;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ValueId {
    Source(SourceValueId),
    Temporary(u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub interface: ProgramInterface,
    pub bindings: Vec<TopLevelBinding>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramInterface {
    pub type_aliases: Vec<TypeAlias>,
    pub external_types: Vec<ExternalType>,
    pub externals: Vec<ExternalOperation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeAlias {
    pub name: String,
    pub ty: Type,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalType {
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopLevelBinding {
    pub pattern: TopLevelPattern,
    pub value: Expression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TopLevelPattern {
    Binding {
        id: ValueId,
        name: String,
        ty: Type,
    },
    Wildcard {
        ty: Type,
        span: Span,
    },
    Product {
        elements: Vec<TopLevelPattern>,
        ty: Type,
        span: Span,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalOperation {
    pub id: ExternalOperationId,
    pub name: String,
    pub parameter: Type,
    pub parameter_aliases: Vec<Option<String>>,
    pub result: Type,
    pub result_alias: Option<String>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    pub pattern: Pattern,
    pub value: Expression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pattern {
    Binding {
        id: ValueId,
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
    Reference(ValueId),
    Integer(i128),
    Float(u64),
    Symbol(Vec<u8>),
    StorageSize(Type),
    Unit,
    Product(Vec<Expression>),
    Let {
        binding: Box<Binding>,
        body: Box<Expression>,
    },
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
    ExternalCall {
        id: ExternalOperationId,
        argument: Box<Expression>,
    },
    NumericConversion {
        value: Box<Expression>,
    },
    SumInjection {
        index: usize,
        value: Box<Expression>,
    },
    Case {
        scrutinee: Box<Expression>,
        arms: Vec<CaseArm>,
    },
    PrimitiveBranch {
        operator: BinaryPrimitive,
        left: Box<Expression>,
        right: Box<Expression>,
        otherwise: Box<Expression>,
        then: Box<Expression>,
    },
    PrimitiveUnary {
        operator: UnaryPrimitive,
        operand: Box<Expression>,
    },
    PrimitiveBinary {
        operator: BinaryPrimitive,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryPrimitive {
    Negate,
    BitwiseNot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryPrimitive {
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lambda {
    pub id: LambdaId,
    pub self_binding: Option<ValueId>,
    pub captures: Vec<Capture>,
    pub parameter: Parameter,
    pub body: Box<Expression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capture {
    pub source: ValueId,
    pub binding: ValueId,
    pub ty: Type,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    pub binding: Option<ValueId>,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseArm {
    pub index: usize,
    pub pattern: Pattern,
    pub value: Expression,
    pub span: Span,
}
