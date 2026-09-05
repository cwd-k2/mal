use crate::check::ast::Type;
use crate::core::ast::{BinaryPrimitive, UnaryPrimitive, ValueId as CoreValueId};
use crate::resolve::ast::{ExternalOperationId, LambdaId};
use crate::source::Span;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ValueId {
    Core(CoreValueId),
    Temporary(u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub external_types: Vec<ExternalType>,
    pub externals: Vec<ExternalOperation>,
    pub bindings: Vec<TopLevelBinding>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalType {
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalOperation {
    pub id: ExternalOperationId,
    pub name: String,
    pub parameter: Type,
    pub result: Type,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopLevelBinding {
    pub pattern: TopLevelPattern,
    pub value: Block,
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
pub struct Block {
    pub bindings: Vec<Binding>,
    pub result: Atom,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    pub pattern: Pattern,
    pub operation: Operation,
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
pub struct Atom {
    pub kind: AtomKind,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AtomKind {
    Reference(ValueId),
    Integer(i128),
    String(Vec<u8>),
    Unit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operation {
    Atom(Atom),
    Lambda(Lambda),
    Call {
        callee: Atom,
        argument: Atom,
    },
    StringLength {
        value: Atom,
    },
    StringAt {
        argument: Atom,
    },
    ExternalCall {
        id: ExternalOperationId,
        argument: Atom,
    },
    IntegerConversion {
        operand: Atom,
    },
    Product(Vec<Atom>),
    SumInjection {
        index: usize,
        value: Atom,
    },
    Case {
        scrutinee: Atom,
        arms: Vec<CaseArm>,
    },
    PrimitiveUnary {
        operator: UnaryPrimitive,
        operand: Atom,
    },
    PrimitiveBinary {
        operator: BinaryPrimitive,
        left: Atom,
        right: Atom,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lambda {
    pub id: LambdaId,
    pub self_binding: Option<ValueId>,
    pub captures: Vec<Capture>,
    pub parameter: Parameter,
    pub body: Block,
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
    pub value: Block,
    pub span: Span,
}
