use crate::control::ast::StateId;

/// One binding operation in the finalized control program.
///
/// Ownership facts use this identity instead of an unlabelled `(StateId, usize)` pair so a
/// binding position cannot be confused with another ordinal attached to the same state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct BindingSite {
    pub(super) state: StateId,
    pub(super) binding: usize,
}

impl BindingSite {
    pub(super) const fn new(state: StateId, binding: usize) -> Self {
        Self { state, binding }
    }
}

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
    SymbolOperand(usize),
    MemoryOperand(usize),
    BufferOperand(usize),
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
