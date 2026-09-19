use crate::anf::ast::ValueId;
use crate::check::ast::{MemoryPrimitive, Type};
use crate::core::ast::{
    BinaryPrimitive, JoinId, PackedBuilderOperation, ProgramInterface, UnaryPrimitive,
};
use crate::resolve::ast::{ExternalOperationId, LambdaId};
use crate::source::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub interface: ProgramInterface,
    pub bindings: Vec<TopLevelBinding>,
    pub functions: Vec<Function>,
    pub entry: Option<EntryPoint>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntryPoint {
    pub function: FunctionId,
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
pub struct Function {
    pub id: FunctionId,
    pub kind: FunctionKind,
    pub parameter: Parameter,
    pub body: Block,
    pub joins: Vec<Join>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FunctionKind {
    Ordinary { captures: Vec<CaptureField> },
}

impl FunctionKind {
    pub fn captures(&self) -> Option<&[CaptureField]> {
        match self {
            Self::Ordinary { captures } => Some(captures),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Join {
    pub parameter: Pattern,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FunctionId {
    Lambda(LambdaId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureField {
    pub ty: Type,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    pub binding: Option<ValueId>,
    pub ty: Type,
    pub span: Span,
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
    pub id: AtomId,
    pub kind: AtomKind,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AtomId(pub usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AtomKind {
    Reference(Reference),
    Integer(i128),
    Float(u64),
    Symbol(Vec<u8>),
    StorageSize(Type),
    Unit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reference {
    Binding(ValueId),
    Capture(usize),
    SelfClosure(FunctionId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operation {
    Atom(Atom),
    Goto {
        target: JoinId,
        value: Atom,
    },
    MakeClosure {
        function: FunctionId,
        captures: Vec<Atom>,
    },
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
        operands: Vec<Atom>,
    },
    PackedBuilder {
        operation: PackedBuilderOperation,
        element: Type,
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
pub struct CaseArm {
    pub index: usize,
    pub pattern: Pattern,
    pub value: Block,
    pub span: Span,
}
