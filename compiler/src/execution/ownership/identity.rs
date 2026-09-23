use crate::control::ast::StateId;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct EdgeId {
    pub(crate) state: StateId,
    pub(crate) path: ControlPath,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ControlPath {
    Single,
    BranchOtherwise,
    BranchThen,
    CaseArm(usize),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct UseId {
    pub(crate) state: StateId,
    pub(crate) location: UseLocation,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum UseLocation {
    Binding {
        binding: usize,
        operand: BindingOperand,
    },
    Terminator(TerminatorOperand),
    FrameField(usize),
    CasePayload(usize),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum BindingOperand {
    Atom,
    Capture(usize),
    ProductElement(usize),
    SumValue,
    SymbolLength,
    SymbolAt,
    MemoryOperand(usize),
    BufferArgument,
    ExternalArgument,
    NumericOperand,
    UnaryOperand,
    BinaryLeft,
    BinaryRight,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum TerminatorOperand {
    Return,
    JumpValue,
    CallCallee,
    CallArgument,
    TailCallee,
    TailArgument,
    CaseScrutinee,
    BranchLeft,
    BranchRight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UseEffect {
    Borrow,
    Share,
    Consume,
}
