use crate::anf::ast::ValueId;
use crate::check::ast::{MemoryPrimitive, Type};
use crate::core::ast::{BinaryPrimitive, ProgramInterface, UnaryPrimitive};
use crate::resolve::ast::{ExternalOperationId, LambdaId};
use crate::source::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub interface: ProgramInterface,
    pub bindings: Vec<TopLevelBinding>,
    pub functions: Vec<Function>,
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
pub struct Function {
    pub id: FunctionId,
    pub environment: Vec<EnvironmentField>,
    pub parameter: Parameter,
    pub body: Block,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FunctionId {
    Lambda(LambdaId),
    Memory(MemoryPrimitive),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnvironmentField {
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
    EnvironmentField(usize),
    SelfClosure(FunctionId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operation {
    Atom(Atom),
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
