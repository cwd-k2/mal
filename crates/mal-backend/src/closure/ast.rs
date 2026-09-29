// Closure conversion keeps the patterns and parameters of ANF, which carry nothing that the conversion changes.
use crate::anf::ast::ValueId;
pub(crate) use crate::anf::ast::{Parameter, Pattern, TopLevelPattern};
use crate::core::ast::{
    BinaryPrimitive, BufferOperation, JoinId, ProgramInterface, UnaryPrimitive,
};
use mal_frontend::check::ast::{MemoryPrimitive, SymbolPrimitive, Type};
use mal_frontend::resolve::ast::{ExternalOperationId, LambdaId};
use mal_syntax::source::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Program {
    pub interface: ProgramInterface,
    pub bindings: Vec<TopLevelBinding>,
    pub functions: Vec<Function>,
    pub entry: Option<EntryPoint>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EntryPoint {
    pub function: FunctionId,
    pub parameter: mal_frontend::check::ast::EntryParameter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TopLevelBinding {
    pub pattern: TopLevelPattern,
    pub value: Block,
    pub span: Span,
}

impl TopLevelBinding {
    /// The capture-free function this binding names: its value creates one closure and returns it.
    pub(crate) fn known_function(&self) -> Option<(ValueId, FunctionId)> {
        let TopLevelPattern::Binding { id: creator, .. } = self.pattern else {
            return None;
        };
        let AtomKind::Reference(Reference::Binding(result)) = self.value.result.kind else {
            return None;
        };
        self.value.bindings.iter().find_map(|binding| {
            let Pattern::Binding { id, .. } = binding.pattern else {
                return None;
            };
            match &binding.operation {
                Operation::MakeClosure { function, captures }
                    if id == result && captures.is_empty() =>
                {
                    Some((creator, *function))
                }
                _ => None,
            }
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Function {
    pub id: FunctionId,
    pub captures: Vec<CaptureField>,
    pub parameter: Parameter,
    pub body: Block,
    pub joins: Vec<Join>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Join {
    pub parameter: Pattern,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum FunctionId {
    Lambda(LambdaId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CaptureField {
    pub ty: Type,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Block {
    pub bindings: Vec<Binding>,
    pub result: Atom,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Binding {
    pub pattern: Pattern,
    pub operation: Operation,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Atom {
    pub id: AtomId,
    pub kind: AtomKind,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct AtomId(pub usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AtomKind {
    Reference(Reference),
    Integer(i128),
    Float(u64),
    Symbol(Vec<u8>),
    Unit,
}

impl Atom {
    /// The local binding the atom reads, if it reads one.
    pub(crate) fn binding(&self) -> Option<ValueId> {
        match self.kind {
            AtomKind::Reference(Reference::Binding(id)) => Some(id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Reference {
    Binding(ValueId),
    Capture(usize),
    SelfClosure(FunctionId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Operation {
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
pub(crate) struct CaseArm {
    pub index: usize,
    pub pattern: Pattern,
    pub value: Block,
    pub span: Span,
}
