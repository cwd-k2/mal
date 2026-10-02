//! Checked programs, top-level items, and the requirements generic bodies leave for specialization.

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
/// A checked program that may still contain generic bindings.
pub struct Program {
    /// Checked top-level items in dependency and source order.
    pub items: Vec<Node<TopItem>>,
    /// The root program span.
    pub span: Span,
    /// The admitted entry declaration, if the program declares one.
    pub entry: Option<EntryPoint>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// The entry binding and the process carrier selected from its checked type.
pub struct EntryPoint {
    /// The entry value identity.
    pub binding: ValueId,
    /// The host-supplied parameter form.
    pub parameter: EntryParameter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Parameter forms supported by the process entry shim.
pub enum EntryParameter {
    /// No process arguments; the entry receives `Unit`.
    Unit,
    /// The entry receives a `Buffer<Symbol>` containing `argv[1..]`.
    ProcessArguments,
}

impl EntryParameter {
    /// The `main` parameter type: a buffer holding one `Symbol` per process argument.
    pub fn process_arguments_type() -> Type {
        Type::Buffer(Arc::new(Type::Symbol))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A checked program whose reachable generic bindings have been specialized away.
pub struct MonomorphicProgram(Program);

impl MonomorphicProgram {
    pub(crate) fn new(program: Program) -> Self {
        Self(program)
    }

    /// Borrows the checked program after its lack of open generic bindings has been established.
    pub fn program(&self) -> &Program {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A checked top-level declaration or value binding.
pub enum TopItem {
    /// A canonicalized type alias retained for interface and editor information.
    TypeAlias {
        /// The alias declaration identity and spelling.
        binding: TypeBinding,
        /// The canonical aliased type.
        ty: Type,
        /// Source alias names corresponding to immediate structural elements.
        element_aliases: Vec<Option<String>>,
        /// Whether the alias was admitted for canonical host-memory access.
        host_memory_access: bool,
    },
    /// A source-defined opaque type retained until specialization erases its boundary.
    OpaqueType {
        /// The declaration identity and spelling.
        binding: TypeBinding,
    },
    /// An opaque host-defined type.
    ExternalType {
        /// The external type declaration.
        binding: TypeBinding,
    },
    /// A checked host operation declaration.
    ExternalOperation {
        /// The operation's stable host-interface index.
        id: ExternalOperationId,
        /// The source value declaration.
        binding: ValueBinding,
        /// The capture-free lambda identity used by lowering.
        lambda_id: LambdaId,
        /// The canonical parameter type.
        parameter: Type,
        /// The declared alias spelling of the whole parameter, when any.
        parameter_alias: Option<String>,
        /// Alias spellings of flattened parameter elements.
        parameter_aliases: Vec<Option<String>>,
        /// The canonical result type.
        result: Type,
        /// The declared alias spelling of the result, when any.
        result_alias: Option<String>,
    },
    /// A checked generic value definition awaiting specialization.
    GenericBinding(Box<GenericBinding>),
    /// A checked operation-family signature awaiting implementation selection.
    OperationFamily(Box<OperationFamily>),
    /// A checked exact or generic implementation awaiting reachability-driven selection.
    OperationImplementation(Box<OperationImplementation>),
    /// A checked monomorphic value definition.
    Binding(Box<Binding>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A checked generic value definition with its canonical scheme and body.
pub struct GenericBinding {
    /// The value declaration.
    pub binding: ValueBinding,
    /// Generic type parameters in declaration order.
    pub parameters: Vec<TypeBinding>,
    /// The checked function or value type containing those parameters.
    pub ty: Type,
    /// The checked initializer before substitution.
    pub value: Expression,
    /// Operation-family goals required by the generic body.
    pub operations: Vec<OperationRequirement>,
    /// Principal kinds of `parameters`, over which `kinds` is written.
    pub parameter_kinds: Vec<Kind>,
    /// Kind equations required by the generic body.
    pub kinds: Vec<KindRequirement>,
    /// The complete definition span.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A generic operation signature selected by canonical type arguments.
pub struct OperationFamily {
    /// The family declaration.
    pub binding: ValueBinding,
    /// Generic type parameters in declaration order.
    pub parameters: Vec<TypeBinding>,
    /// The canonical generic signature.
    pub ty: Type,
    /// The complete declaration span.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// One exact or generic implementation of an operation family.
pub struct OperationImplementation {
    /// The family identity and implementation-site spelling.
    pub family: ValueReference,
    /// Pattern parameters bound by a generic implementation key.
    pub parameters: Vec<TypeBinding>,
    /// Canonical type patterns forming the implementation key.
    pub arguments: Vec<Type>,
    /// The canonical instantiated family signature.
    pub ty: Type,
    /// The checked initializer.
    pub value: Expression,
    /// Operation goals required after matching this implementation.
    pub operations: Vec<OperationRequirement>,
    /// Principal kinds of `parameters`, over which `kinds` is written.
    pub parameter_kinds: Vec<Kind>,
    /// Kind equations required by the implementation body.
    pub kinds: Vec<KindRequirement>,
    /// `Storable` atoms of the instantiated signature. They may exceed the family's own requirements, so
    /// specialization checks them when it selects this implementation.
    pub requirements: Vec<Type>,
    /// The complete implementation span.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A kind equation that a generic body needs from the enclosing binding's kind-polymorphic parameters. Kinds are
/// inferred from signatures alone, so the body may use a parameter at a kind its signature leaves open; the
/// equation is checked when specialization makes the parameters concrete.
pub struct KindRequirement {
    /// A kind over the enclosing binding's kind variables.
    pub left: Kind,
    /// The kind it must equal.
    pub right: Kind,
    /// The type application in the body that needs the equation.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A typed operation-family goal retained by a generic body until specialization.
pub struct OperationRequirement {
    /// The required family.
    pub family: ValueReference,
    /// Canonical arguments, possibly containing the enclosing binding's rigid parameters.
    pub arguments: Vec<Type>,
}
