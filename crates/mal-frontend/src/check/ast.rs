//! Typed program representation emitted only after language-rule admission.

use crate::resolve::ast::{
    ExternalOperationId, LambdaId, TypeBinding, TypeId, ValueBinding, ValueId, ValueReference,
};
use mal_syntax::ast::{BinaryOperator, Node, UnaryOperator};
use mal_syntax::source::FileId;
use mal_syntax::source::Span;
use std::{collections::HashSet, sync::Arc};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
/// The compiler-inferred shape of a type-level term.
pub enum Kind {
    /// A complete type inhabited by runtime values.
    Type,
    /// A generalized variable in a principal kind scheme.
    Variable(u32),
    /// A type-level function from one kind to another.
    Function {
        /// The accepted argument kind.
        parameter: Arc<Kind>,
        /// The kind produced by application.
        result: Arc<Kind>,
    },
}

impl Kind {
    /// Constructs a right-associated type-level function kind.
    pub fn function(parameter: Kind, result: Kind) -> Self {
        Self::Function {
            parameter: Arc::new(parameter),
            result: Arc::new(result),
        }
    }
}

#[derive(Clone, Debug)]
/// A canonical checked type-level term; value expressions always carry a term of kind `Type`.
pub enum Type {
    /// The single-value `Unit` type.
    Unit,
    /// Signed 8-bit integer.
    Int8,
    /// Signed 16-bit integer.
    Int16,
    /// Signed 32-bit integer.
    Int32,
    /// Signed 64-bit integer.
    Int64,
    /// Unsigned 8-bit integer.
    UInt8,
    /// Unsigned 16-bit integer.
    UInt16,
    /// Unsigned 32-bit integer.
    UInt32,
    /// Unsigned 64-bit integer.
    UInt64,
    /// IEEE 754 binary32.
    Float32,
    /// IEEE 754 binary64.
    Float64,
    /// An immutable owned byte string.
    Symbol,
    /// An opaque capability for host-managed storage.
    Address,
    /// A target-width unsigned byte quantity.
    ByteSize,
    /// A target-width unsigned element count or index.
    USize,
    /// A generic parameter before monomorphization.
    Parameter {
        /// The resolved parameter identity.
        id: TypeId,
        /// Its source name for diagnostics.
        name: String,
        /// The inferred kind of this use.
        kind: Kind,
    },
    /// A bound variable in a canonical type-level abstraction, counted from the nearest binder.
    Bound {
        /// The de Bruijn index of the binder.
        index: usize,
        /// The kind assigned to the binder.
        kind: Kind,
    },
    /// A type-level application that cannot yet beta-reduce because its callee is open.
    Application {
        /// The constructor term.
        constructor: Arc<Type>,
        /// The applied argument term.
        argument: Arc<Type>,
        /// The result kind established by kind checking.
        kind: Kind,
        /// The application site retained for delayed formation diagnostics.
        span: Span,
    },
    /// A canonical type-level abstraction used for partial constructor application.
    Abstraction {
        /// The accepted argument kind.
        parameter_kind: Kind,
        /// The body, whose nearest bound variable has index zero.
        body: Arc<Type>,
    },
    /// A shared mutable sequence of elements.
    Buffer(Arc<Type>),
    /// An opaque host-defined type.
    External {
        /// The resolved external type identity.
        id: TypeId,
        /// Its source name for diagnostics and ABI names.
        name: String,
    },
    /// A source-defined abstract type with a hidden zero-cost representation.
    Opaque {
        /// The declaration identity used for canonical equality.
        id: TypeId,
        /// The source name used for diagnostics.
        name: Arc<str>,
        /// Canonical type arguments retained as part of the identity.
        arguments: Arc<[Type]>,
        /// The representation used after frontend abstraction checks.
        representation: Arc<Type>,
        /// The only file allowed to view the representation.
        declaration_file: FileId,
    },
    /// An ordered product of field types.
    Product(Arc<[Type]>),
    /// An ordered sum of variant payload types.
    Sum(Arc<[Type]>),
    /// A function from one carrier type to one result type.
    Function {
        /// The parameter carrier type.
        parameter: Arc<Type>,
        /// The result type.
        result: Arc<Type>,
    },
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        let mut pending = vec![(self, other)];
        let mut compared = HashSet::new();

        while let Some((left, right)) = pending.pop() {
            if std::ptr::eq(left, right) {
                continue;
            }
            if let (Some(left), Some(right)) = (shared_id(left), shared_id(right))
                && !compared.insert((left, right))
            {
                continue;
            }

            match (left, right) {
                (Self::Unit, Self::Unit)
                | (Self::Int8, Self::Int8)
                | (Self::Int16, Self::Int16)
                | (Self::Int32, Self::Int32)
                | (Self::Int64, Self::Int64)
                | (Self::UInt8, Self::UInt8)
                | (Self::UInt16, Self::UInt16)
                | (Self::UInt32, Self::UInt32)
                | (Self::UInt64, Self::UInt64)
                | (Self::Float32, Self::Float32)
                | (Self::Float64, Self::Float64)
                | (Self::Symbol, Self::Symbol)
                | (Self::Address, Self::Address)
                | (Self::ByteSize, Self::ByteSize)
                | (Self::USize, Self::USize) => {}
                (
                    Self::Parameter {
                        id: left_id,
                        name: left_name,
                        kind: left_kind,
                        ..
                    },
                    Self::Parameter {
                        id: right_id,
                        name: right_name,
                        kind: right_kind,
                        ..
                    },
                ) if left_id == right_id && left_name == right_name && left_kind == right_kind => {}
                (
                    Self::Bound {
                        index: left_index,
                        kind: left_kind,
                    },
                    Self::Bound {
                        index: right_index,
                        kind: right_kind,
                    },
                ) if left_index == right_index && left_kind == right_kind => {}
                (
                    Self::Application {
                        constructor: left_constructor,
                        argument: left_argument,
                        kind: left_kind,
                        ..
                    },
                    Self::Application {
                        constructor: right_constructor,
                        argument: right_argument,
                        kind: right_kind,
                        ..
                    },
                ) if left_kind == right_kind => {
                    pending.push((left_constructor, right_constructor));
                    pending.push((left_argument, right_argument));
                }
                (
                    Self::Abstraction {
                        parameter_kind: left_kind,
                        body: left_body,
                    },
                    Self::Abstraction {
                        parameter_kind: right_kind,
                        body: right_body,
                    },
                ) if left_kind == right_kind => pending.push((left_body, right_body)),
                (Self::Buffer(left), Self::Buffer(right)) => pending.push((left, right)),
                (
                    Self::Opaque {
                        id: left_id,
                        arguments: left_arguments,
                        ..
                    },
                    Self::Opaque {
                        id: right_id,
                        arguments: right_arguments,
                        ..
                    },
                ) if left_id == right_id && left_arguments.len() == right_arguments.len() => {
                    pending.extend(left_arguments.iter().zip(right_arguments.iter()));
                }
                (
                    Self::External {
                        id: left_id,
                        name: left_name,
                    },
                    Self::External {
                        id: right_id,
                        name: right_name,
                    },
                ) if left_id == right_id && left_name == right_name => {}
                (Self::Product(left), Self::Product(right))
                | (Self::Sum(left), Self::Sum(right))
                    if left.len() == right.len() =>
                {
                    if !Arc::ptr_eq(left, right) {
                        pending.extend(left.iter().zip(right.iter()));
                    }
                }
                (
                    Self::Function {
                        parameter: left_parameter,
                        result: left_result,
                    },
                    Self::Function {
                        parameter: right_parameter,
                        result: right_result,
                    },
                ) => {
                    if !Arc::ptr_eq(left_parameter, right_parameter) {
                        pending.push((left_parameter, right_parameter));
                    }
                    if !Arc::ptr_eq(left_result, right_result) {
                        pending.push((left_result, right_result));
                    }
                }
                _ => return false,
            }
        }

        true
    }
}

impl Eq for Type {}

impl Type {
    /// Returns the kind established for this canonical term.
    pub fn kind(&self) -> Kind {
        match self {
            Self::Parameter { kind, .. }
            | Self::Bound { kind, .. }
            | Self::Application { kind, .. } => kind.clone(),
            Self::Abstraction {
                parameter_kind,
                body,
            } => Kind::function(parameter_kind.clone(), body.kind()),
            _ => Kind::Type,
        }
    }

    /// Walks data-bearing subtypes in preorder, visiting shared aggregate nodes only once.
    ///
    /// Function parameter and result types are intentionally not data subtypes of the function value.
    pub fn data_subtypes(&self) -> DataSubtypes<'_> {
        DataSubtypes {
            pending: vec![self],
            visited: HashSet::new(),
        }
    }

    /// Returns the allocation identity used to memoize structurally shared aggregate and function nodes.
    pub fn shared_id(&self) -> Option<SharedTypeId> {
        shared_id(self)
    }
}

/// Preorder traversal of storage-bearing subtypes with shared nodes deduplicated.
pub struct DataSubtypes<'a> {
    pending: Vec<&'a Type>,
    visited: HashSet<SharedTypeId>,
}

impl<'a> Iterator for DataSubtypes<'a> {
    type Item = &'a Type;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(ty) = self.pending.pop() {
            if let Some(identity) = shared_id(ty)
                && !self.visited.insert(identity)
            {
                continue;
            }
            if let Type::Product(elements) | Type::Sum(elements) = ty {
                self.pending.extend(elements.iter().rev());
            }
            return Some(ty);
        }
        None
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
/// Allocation identity of a shared canonical type node.
pub enum SharedTypeId {
    /// One shared product field slice.
    Product(*const Type),
    /// One shared sum variant slice.
    Sum(*const Type),
    /// One shared function parameter/result pair.
    Function(*const Type, *const Type),
}

fn shared_id(ty: &Type) -> Option<SharedTypeId> {
    match ty {
        Type::Product(elements) => Some(SharedTypeId::Product(elements.as_ptr())),
        Type::Sum(elements) => Some(SharedTypeId::Sum(elements.as_ptr())),
        Type::Function { parameter, result } => Some(SharedTypeId::Function(
            std::ptr::from_ref(parameter.as_ref()),
            std::ptr::from_ref(result.as_ref()),
        )),
        _ => None,
    }
}

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
    /// The complete implementation span.
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

#[derive(Clone, Debug, Eq, PartialEq)]
/// A checked monomorphic value binding.
pub struct Binding {
    /// The typed binding pattern.
    pub pattern: Pattern,
    /// The canonical source annotation, when explicitly written.
    pub annotation: Option<Type>,
    /// The checked initializer.
    pub value: Expression,
    /// The complete binding span.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A binding pattern annotated with the type of every subpattern.
pub enum Pattern {
    /// A named binding.
    Binding {
        /// The resolved binding identity.
        binding: ValueBinding,
        /// The bound value type.
        ty: Type,
    },
    /// A discarded value.
    Wildcard {
        /// The discarded value type.
        ty: Type,
        /// The wildcard source span.
        span: Span,
    },
    /// A destructured product.
    Product {
        /// Element patterns in field order.
        elements: Vec<Pattern>,
        /// The complete product type.
        ty: Type,
        /// The complete pattern span.
        span: Span,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A value-producing checked expression.
pub struct Expression {
    /// The admitted expression form.
    pub kind: ExpressionKind,
    /// The expression's canonical result type.
    pub ty: Type,
    /// The source range used for later diagnostics.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Whether checking a subtree yields a value locally or transfers control away.
pub enum Completion {
    /// Local evaluation produces a value.
    Value(Expression),
    /// Evaluation transfers control before producing a local value.
    Abrupt(AbruptExpression),
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// An abrupt completion plus values that must run before its terminal transfer.
pub struct AbruptExpression {
    /// Expressions evaluated in order before the terminal form.
    pub preceding: Vec<Expression>,
    /// The control-transferring terminal form.
    pub kind: AbruptExpressionKind,
    /// The complete abrupt expression span.
    pub span: Span,
}

impl AbruptExpression {
    /// Prepends values that must be evaluated before this abrupt completion transfers control.
    pub fn preceded_by(mut self, mut values: Vec<Expression>) -> Self {
        values.append(&mut self.preceding);
        self.preceding = values;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A terminal form that cannot return a value to its immediate context.
pub enum AbruptExpressionKind {
    /// Application of a result binder transfers a value to its result block.
    ResultTransfer {
        /// The enclosing result-block identity.
        target: ValueId,
        /// The binder of the group that was applied, when the target is one binder of a group.
        variant: Option<usize>,
        /// The transferred value, already injected when a binder group requires it.
        value: Box<Expression>,
    },
    /// Elimination of the uninhabited empty sum.
    EmptyElimination {
        /// The checked empty-sum value.
        scrutinee: Box<Expression>,
    },
    /// A conditional whose branches are both abrupt.
    If {
        /// The Boolean condition.
        condition: Box<Expression>,
        /// The abrupt true branch.
        then_branch: ExpressionBlock,
        /// The abrupt false branch.
        else_branch: ExpressionBlock,
    },
    /// A sum elimination whose every continuation is `Abrupt`.
    SumElimination {
        /// The checked sum value.
        scrutinee: Box<Expression>,
        /// One abrupt continuation per variant.
        continuations: Vec<SumContinuation>,
    },
    /// A direct block whose final completion is abrupt.
    Block(ExpressionBlock),
}

/// One continuation of a sum elimination.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SumContinuation {
    /// A function value applied to the payload.
    Function(Expression),
    /// A lambda literal that belongs to the enclosing invocation. It binds the payload and runs in place.
    Branch(SumBranch),
    /// A result binder name that receives the payload.
    Transfer(SumTransfer),
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// An in-place continuation branch checked against one sum payload type.
pub struct SumBranch {
    /// The optional payload pattern; absence binds a `Unit` payload.
    pub parameter: Option<Box<Pattern>>,
    /// The canonical payload type even when no pattern is present.
    pub parameter_type: Type,
    /// The checked branch body.
    pub body: ExpressionBlock,
    /// The complete continuation span.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A sum continuation that transfers its payload to a result binder.
pub struct SumTransfer {
    /// The enclosing result-block identity.
    pub target: ValueId,
    /// The sum variant the payload selects when the target is one binder of a group.
    pub variant: Option<usize>,
    /// The payload type accepted by this continuation.
    pub payload_type: Type,
    /// The result type assembled by the complete binder group.
    pub result_type: Type,
    /// The result-binder use span.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// The admitted form of a value-producing checked expression.
pub enum ExpressionKind {
    /// A monomorphic resolved value reference.
    Reference(ValueReference),
    /// A generic reference awaiting reachability-driven specialization.
    GenericReference {
        /// The referenced generic definition.
        reference: ValueReference,
        /// Canonical type arguments in declaration order.
        arguments: Vec<Type>,
    },
    /// A family reference awaiting implementation selection during specialization.
    OperationReference {
        /// The referenced family declaration.
        family: ValueReference,
        /// Canonical family arguments in declaration order.
        arguments: Vec<Type>,
    },
    /// An integer value admitted into its checked scalar type.
    Integer(i128),
    /// IEEE bits admitted into the expression's `Float32` or `Float64` type.
    Float(u64),
    /// A decoded immutable byte sequence.
    Symbol(Vec<u8>),
    /// The unit value.
    Unit,
    /// A product in source evaluation order.
    Product(Vec<Expression>),
    /// Source parentheses retained for editor structure.
    Parenthesized(Box<Expression>),
    /// A direct value-producing block.
    Block(ExpressionBlock),
    /// A block defining one invocation-local result boundary.
    ResultBlock {
        /// The identity shared by the block's result binders.
        target: ValueId,
        /// Result binders in source/variant order.
        result_binders: Vec<ResultBinder>,
        /// The checked body governed by the boundary.
        body: ExpressionBlock,
    },
    /// A checked lambda value.
    Lambda(Lambda),
    /// A single-carrier function application.
    Call {
        /// The checked function value.
        callee: Box<Expression>,
        /// The checked argument carrier.
        argument: Box<Expression>,
    },
    /// A Symbol operation selected by the checker from its operator and operand types.
    SymbolOperation {
        /// The selected operation.
        primitive: SymbolPrimitive,
        /// Logical operands in source evaluation order.
        operands: Vec<Expression>,
    },
    /// A checked host-memory, Buffer, or snapshot primitive.
    Memory {
        /// The selected primitive identity.
        primitive: MemoryPrimitive,
        /// Logical operands in source evaluation order.
        operands: Vec<Expression>,
    },
    /// A numeric conversion whose source and destination types are on the enclosing expressions.
    NumericConversion {
        /// The converted scalar value.
        value: Box<Expression>,
    },
    /// Elimination of a sum into one continuation per variant.
    SumElimination {
        /// The checked sum value.
        scrutinee: Box<Expression>,
        /// Continuations in variant order.
        continuations: Vec<SumContinuation>,
    },
    /// Injection of one payload into a sum.
    SumInjection {
        /// The selected zero-based variant.
        index: usize,
        /// The checked payload value.
        value: Box<Expression>,
    },
    /// A value-producing Boolean conditional.
    If {
        /// The checked Boolean condition.
        condition: Box<Expression>,
        /// The true branch.
        then_branch: ExpressionBlock,
        /// The false branch.
        else_branch: ExpressionBlock,
    },
    /// A primitive unary operation not lowered into a dedicated form.
    Unary {
        /// The source operator and its span.
        operator: Node<UnaryOperator>,
        /// The checked operand.
        operand: Box<Expression>,
    },
    /// A primitive binary operation not lowered into dedicated control.
    Binary {
        /// The source operator and its span.
        operator: Node<BinaryOperator>,
        /// The checked left operand.
        left: Box<Expression>,
        /// The checked right operand.
        right: Box<Expression>,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// An operation on immutable Symbol bytes, with its operands listed in the documented order.
pub enum SymbolPrimitive {
    /// `#symbol`: the byte count as `USize`.
    Length,
    /// `symbol # index`: one byte as `UInt8`.
    ByteAt,
    /// `left + right`: a new Symbol holding both byte sequences.
    Concatenate,
    /// `symbol / index`: the first `index` bytes.
    Prefix,
    /// `symbol % index`: the bytes from `index` on.
    Suffix,
    /// `left == right`: byte-wise equality as `Bool`.
    Equal,
    /// `left != right`: byte-wise inequality as `Bool`.
    NotEqual,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// A predefined memory operation selected by resolved identity during checking.
pub enum MemoryPrimitive {
    /// Allocate an empty `Buffer<T>` with an initial capacity.
    BufferMake,
    /// Copy canonical elements from host storage into a new Buffer.
    BufferFromAddress,
    /// Copy canonical Buffer elements into host storage.
    BufferIntoAddress,
    /// Observe the element count of a Buffer view.
    ViewLength,
    /// Snapshot a `Buffer<UInt8>` as an immutable Symbol.
    BufferToSymbol,
    /// Copy a Symbol into a mutable `Buffer<UInt8>` snapshot.
    SymbolToBuffer,
    /// Append one element and return its stable index.
    BufferNew,
    /// Read one Buffer element.
    BufferGet,
    /// Replace one Buffer element.
    BufferPut,
    /// Assign one value across a Buffer range.
    BufferFill,
    /// Copy a range between Buffers of the same element type.
    BufferCopy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A checked lambda with explicit types and capture bindings.
pub struct Lambda {
    /// The lambda's program-wide identity.
    pub id: LambdaId,
    /// The admitted direct self-reference binding, when present.
    pub self_binding: Option<ValueId>,
    /// Captures in first lexical-reference order.
    pub captures: Vec<Capture>,
    /// The optional typed parameter pattern.
    pub parameter: Option<Box<Pattern>>,
    /// The canonical parameter carrier type.
    pub parameter_type: Type,
    /// The canonical result type.
    pub result_type: Type,
    /// The checked body and completion.
    pub body: LambdaBody,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// One named exit of a direct result block.
pub struct ResultBinder {
    /// The resolved result-binder identity.
    pub binding: ValueBinding,
    /// The payload type accepted at this exit.
    pub parameter_type: Type,
    /// The variant injected for a binder group, or `None` for a single binder.
    pub variant: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// One checked value forwarded into a lambda environment.
pub struct Capture {
    /// The reference in the enclosing lambda.
    pub source: ValueReference,
    /// The fresh binding inside the capturing lambda.
    pub binding: ValueBinding,
    /// The canonical captured value type.
    pub ty: Type,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A checked lambda body whose final subtree may complete abruptly.
pub struct LambdaBody {
    /// Body items in source evaluation order.
    pub items: Vec<BodyItem>,
    /// The final local or abrupt completion.
    pub result: Box<Completion>,
    /// The complete body range.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A checked direct block nested inside an expression.
pub struct ExpressionBlock {
    /// Body items in source evaluation order.
    pub items: Vec<BodyItem>,
    /// The final local or abrupt completion.
    pub result: Box<Completion>,
    /// The complete block range.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// One checked non-result item in a body.
pub enum BodyItem {
    /// A value binding.
    Binding(Box<Binding>),
    /// An expression evaluated for effects or abrupt completion.
    Expression(Box<Expression>),
}
