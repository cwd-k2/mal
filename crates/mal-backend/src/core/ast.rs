use mal_frontend::check::ast::{MemoryPrimitive, Type};
use mal_frontend::resolve::ast::{ExternalOperationId, LambdaId, ValueId as SourceValueId};
use mal_syntax::source::Span;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ValueId {
    Source(SourceValueId),
    Temporary(u32),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct JoinId(pub usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Program {
    pub interface: ProgramInterface,
    pub bindings: Vec<TopLevelBinding>,
    pub entry: Option<EntryPoint>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EntryPoint {
    pub binding: ValueId,
    pub parameter: mal_frontend::check::ast::EntryParameter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProgramInterface {
    pub type_aliases: Vec<TypeAlias>,
    pub external_types: Vec<ExternalType>,
    pub externals: Vec<ExternalOperation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TypeAlias {
    pub name: String,
    pub ty: Type,
    pub element_aliases: Vec<Option<String>>,
    pub host_memory_access: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExternalType {
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TopLevelBinding {
    pub pattern: TopLevelPattern,
    pub value: Expression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TopLevelPattern {
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
pub(crate) struct ExternalOperation {
    pub id: ExternalOperationId,
    pub name: String,
    pub parameter: Type,
    pub parameter_alias: Option<String>,
    pub parameter_aliases: Vec<Option<String>>,
    pub result: Type,
    pub result_alias: Option<String>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Binding {
    pub pattern: Pattern,
    pub value: Expression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Pattern {
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
pub(crate) struct Expression {
    pub kind: ExpressionKind,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExpressionKind {
    Reference(ValueId),
    Integer(i128),
    Float(u64),
    Symbol(Vec<u8>),
    Unit,
    Product(Vec<Expression>),
    Let {
        binding: Box<Binding>,
        body: Box<Expression>,
    },
    Goto {
        target: JoinId,
        value: Box<Expression>,
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
    Memory {
        primitive: MemoryPrimitive,
        operands: Vec<Expression>,
    },
    Buffer {
        operation: BufferOperation,
        element: Type,
        operands: Vec<Expression>,
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
pub(crate) enum BufferOperation {
    Make,
    New,
    Get,
    Put,
    Fill,
    Copy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnaryPrimitive {
    Negate,
    BitwiseNot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BinaryPrimitive {
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
pub(crate) struct Lambda {
    pub id: LambdaId,
    pub self_binding: Option<ValueId>,
    pub kind: LambdaKind,
    pub captures: Vec<Capture>,
    pub parameter: Parameter,
    pub body: Box<Expression>,
    pub joins: Vec<Join>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LambdaKind {
    Ordinary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Join {
    pub parameter: Pattern,
    pub body: Expression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Capture {
    pub source: ValueId,
    pub binding: ValueId,
    pub ty: Type,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Parameter {
    pub binding: Option<ValueId>,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CaseArm {
    pub index: usize,
    pub pattern: Pattern,
    pub value: Expression,
    pub span: Span,
}
