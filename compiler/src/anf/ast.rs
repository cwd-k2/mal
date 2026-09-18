use crate::check::ast::{MemoryPrimitive, Type};
use crate::core::ast::{
    BinaryPrimitive, JoinId, ProgramInterface, UnaryPrimitive, ValueId as CoreValueId,
};
use crate::resolve::ast::{ExternalOperationId, LambdaId};
use crate::source::Span;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ValueId {
    Core(CoreValueId),
    Temporary(u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub interface: ProgramInterface,
    pub bindings: Vec<TopLevelBinding>,
    pub entry: Option<EntryPoint>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntryPoint {
    pub binding: ValueId,
    pub parameter: crate::check::ast::EntryParameter,
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
    Float(u64),
    Symbol(Vec<u8>),
    StorageSize(Type),
    Unit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operation {
    Atom(Atom),
    Goto {
        target: JoinId,
        value: Atom,
    },
    Lambda(Lambda),
    Call {
        callee: Atom,
        argument: Atom,
    },
    SymbolLength {
        value: Atom,
    },
    SymbolAt {
        argument: Atom,
    },
    Memory {
        primitive: MemoryPrimitive,
        argument: Atom,
    },
    ExternalCall {
        id: ExternalOperationId,
        argument: Atom,
    },
    NumericConversion {
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
    PrimitiveBranch {
        operator: BinaryPrimitive,
        left: Atom,
        right: Atom,
        otherwise: Box<Block>,
        then: Box<Block>,
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
    pub joins: Vec<Join>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Join {
    pub parameter: Pattern,
    pub body: Block,
    pub span: Span,
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
