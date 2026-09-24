use crate::anf::ast::ValueId;
use crate::check::ast::{MemoryPrimitive, Type};
use crate::closure::ast::{Atom, FunctionId, Parameter, Pattern, TopLevelPattern};
use crate::core::ast::{BinaryPrimitive, BufferOperation, UnaryPrimitive};
use crate::resolve::ast::ExternalOperationId;
use mal_syntax::source::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub bindings: Vec<TopLevelBinding>,
    pub functions: Vec<Function>,
    pub states: Vec<State>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopLevelBinding {
    pub pattern: TopLevelPattern,
    pub entry: StateId,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Function {
    pub id: FunctionId,
    pub parameter: Parameter,
    pub entry: StateId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StateId(pub usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
    pub input: Option<Pattern>,
    pub live: Vec<LiveValue>,
    pub needs_environment: bool,
    pub bindings: Vec<Binding>,
    pub terminator: Terminator,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    pub pattern: Pattern,
    pub operation: Operation,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operation {
    Atom(Atom),
    MakeClosure {
        function: FunctionId,
        captures: Vec<Atom>,
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
    Buffer {
        operation: BufferOperation,
        element: Type,
        operands: Vec<Atom>,
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
pub enum Terminator {
    Return(Atom),
    Goto(StateId),
    Jump {
        target: StateId,
        value: Atom,
    },
    Call {
        callee: Atom,
        argument: Atom,
        resume: StateId,
    },
    TailCall {
        callee: Atom,
        argument: Atom,
    },
    Case {
        scrutinee: Atom,
        arms: Vec<CaseArm>,
    },
    PrimitiveBranch {
        operator: BinaryPrimitive,
        left: Atom,
        right: Atom,
        otherwise: StateId,
        then: StateId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveValue {
    pub id: ValueId,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseArm {
    pub index: usize,
    pub target: StateId,
    pub span: Span,
}
