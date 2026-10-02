//! Checked bindings, patterns, expressions, lambdas, and blocks.

use super::*;

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
    /// `==` or `!=` on `Bool`, which lowering expands into continuation applications.
    BoolEquality {
        /// Whether the operator is `==` rather than `!=`.
        equal: bool,
        /// The checked left operand, evaluated first.
        left: Box<Expression>,
        /// The checked right operand.
        right: Box<Expression>,
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
    /// A unary operator selected by the checker.
    Unary {
        /// The operation and the span of its source operator.
        operator: Node<UnaryOperation>,
        /// The checked operand.
        operand: Box<Expression>,
    },
    /// A binary operator selected by the checker; Symbol and Bool equality operations have their own forms.
    Binary {
        /// The operation and the span of its source operator.
        operator: Node<BinaryOperation>,
        /// The checked left operand.
        left: Box<Expression>,
        /// The checked right operand.
        right: Box<Expression>,
    },
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
