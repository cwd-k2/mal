use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, FunctionId, Parameter, Pattern, TopLevelPattern};
use crate::core::ast::{BinaryPrimitive, BufferOperation, UnaryPrimitive};
use mal_frontend::check::ast::{MemoryPrimitive, SymbolPrimitive, Type};
use mal_frontend::resolve::ast::ExternalOperationId;
use mal_syntax::source::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Program {
    pub bindings: Vec<TopLevelBinding>,
    pub functions: Vec<Function>,
    /// The function the process entry point calls, which no application site targets by its own convention.
    pub entry: Option<FunctionId>,
    pub states: Vec<State>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TopLevelBinding {
    pub pattern: TopLevelPattern,
    pub entry: StateId,
    /// The states of the initializer: those reachable from `entry`, in discovery order with `entry` first.
    pub states: Vec<StateId>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Function {
    pub id: FunctionId,
    pub parameter: Parameter,
    pub entry: StateId,
    /// The states of the body: those reachable from `entry`, in discovery order with `entry` first. A call rewritten
    /// into a tail call leaves its former resume state unreachable, so it belongs to no function.
    pub states: Vec<StateId>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct StateId(pub usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct State {
    pub input: Option<Pattern>,
    pub live: Vec<LiveValue>,
    pub needs_environment: bool,
    pub bindings: Vec<Binding>,
    pub terminator: Terminator,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Binding {
    pub pattern: Pattern,
    pub operation: Operation,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Operation {
    Atom(Atom),
    MakeClosure {
        function: FunctionId,
        captures: Vec<Atom>,
    },
    Symbol {
        primitive: SymbolPrimitive,
        operands: Vec<Atom>,
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
pub(crate) enum Terminator {
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
pub(crate) struct LiveValue {
    pub id: ValueId,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CaseArm {
    pub index: usize,
    pub target: StateId,
    pub span: Span,
}

impl Operation {
    /// Visits every atom the operation reads, in operand order.
    pub(crate) fn for_each_atom(&self, mut visit: impl FnMut(&Atom)) {
        match self {
            Self::Atom(atom)
            | Self::ExternalCall { argument: atom, .. }
            | Self::NumericConversion { operand: atom }
            | Self::SumInjection { value: atom, .. }
            | Self::PrimitiveUnary { operand: atom, .. } => visit(atom),
            Self::MakeClosure {
                captures: atoms, ..
            }
            | Self::Product(atoms)
            | Self::Memory {
                operands: atoms, ..
            }
            | Self::Symbol {
                operands: atoms, ..
            }
            | Self::Buffer {
                operands: atoms, ..
            } => atoms.iter().for_each(visit),
            Self::PrimitiveBinary { left, right, .. } => {
                visit(left);
                visit(right);
            }
        }
    }
}

impl Terminator {
    /// Visits every atom the terminator reads, in operand order.
    pub(crate) fn for_each_atom(&self, mut visit: impl FnMut(&Atom)) {
        match self {
            Self::Return(atom)
            | Self::Jump { value: atom, .. }
            | Self::Case {
                scrutinee: atom, ..
            } => visit(atom),
            Self::Call {
                callee, argument, ..
            }
            | Self::TailCall { callee, argument } => {
                visit(callee);
                visit(argument);
            }
            Self::PrimitiveBranch { left, right, .. } => {
                visit(left);
                visit(right);
            }
            Self::Goto(_) => {}
        }
    }

    /// The states control may continue at within the same function. A call continues at its resume state;
    /// a return and a tail call leave the function.
    pub(crate) fn successors(&self) -> impl Iterator<Item = StateId> + '_ {
        let (fixed, arms): ([Option<StateId>; 2], &[CaseArm]) = match self {
            Self::Goto(target) | Self::Jump { target, .. } => ([Some(*target), None], &[]),
            Self::Call { resume, .. } => ([Some(*resume), None], &[]),
            Self::PrimitiveBranch {
                otherwise, then, ..
            } => ([Some(*otherwise), Some(*then)], &[]),
            Self::Case { arms, .. } => ([None, None], arms),
            Self::Return(_) | Self::TailCall { .. } => ([None, None], &[]),
        };
        fixed
            .into_iter()
            .flatten()
            .chain(arms.iter().map(|arm| arm.target))
    }
}
