//! Source-oriented syntax tree that preserves spans and literal structure for later admission stages.

use crate::lexer::{DecimalFloatLiteral, IntegerLiteral};
use crate::source::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
/// A syntax value paired with the complete source range that produced it.
pub struct Node<T> {
    /// The parsed syntax value.
    pub kind: T,
    /// The value's complete source range, including delimiters when the grammar owns them.
    pub span: Span,
}

impl<T> Node<T> {
    /// Pairs `kind` with its source range.
    pub const fn new(kind: T, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// An identifier spelling and its exact source range.
pub struct Name {
    /// The identifier text without surrounding syntax.
    pub text: String,
    /// The identifier token's source range.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// One parsed source file before name resolution.
pub struct Program {
    /// Leading file requirements in source order.
    pub requirements: Vec<Node<Requirement>>,
    /// Top-level declarations and bindings in source order.
    pub items: Vec<Node<TopItem>>,
    /// The complete source-file range.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A source-file requirement with its decoded path bytes.
pub struct Requirement {
    /// The decoded relative path.
    pub path: Vec<u8>,
    /// The range of the path literal, used for diagnostics and editor links.
    pub path_span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A declaration or binding admitted at file scope.
pub enum TopItem {
    /// A monomorphic type alias.
    TypeAlias {
        /// The declared alias name.
        name: Name,
        /// The aliased type expression.
        value: Node<TypeExpression>,
    },
    /// A type alias parameterized by one or more type names.
    GenericTypeAlias {
        /// The declared alias name.
        name: Name,
        /// Type parameters in declaration order.
        parameters: Vec<Name>,
        /// The aliased type expression.
        value: Node<TypeExpression>,
    },
    /// A source-defined abstract type whose representation is visible only in its declaring file.
    OpaqueType {
        /// The declared opaque type name.
        name: Name,
        /// Type parameters in declaration order.
        parameters: Vec<Name>,
        /// The hidden representation type.
        representation: Node<TypeExpression>,
    },
    /// An opaque type supplied by the C host.
    ExternalType {
        /// The declared external type name.
        name: Name,
    },
    /// A host operation declaration.
    ExternalOperation {
        /// The operation name exported through the host ABI.
        name: Name,
        /// Its source-level function type.
        ty: Node<TypeExpression>,
    },
    /// A monomorphic value binding.
    Binding(Binding),
    /// A generic value header, classified during name resolution as a binding, family, or implementation.
    GenericBinding {
        /// The declared value name.
        name: Name,
        /// Header type entries in source order.
        arguments: Vec<Node<TypeExpression>>,
        /// The required type annotation.
        annotation: Node<TypeExpression>,
        /// The initializer, absent for an operation-family declaration.
        value: Option<Node<Expression>>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A source-oriented type expression; names remain unresolved.
pub enum TypeExpression {
    /// A named type or type parameter.
    Named(Name),
    /// Application of a generic type constructor.
    Application {
        /// The unresolved constructor name.
        constructor: Name,
        /// Type arguments in source order.
        arguments: Vec<Node<TypeExpression>>,
    },
    /// The `Unit` type spelling.
    Unit,
    /// An explicitly parenthesized type.
    Parenthesized(Box<Node<TypeExpression>>),
    /// A product type, including its source field order.
    Product(Vec<Node<TypeExpression>>),
    /// A sum type, including its source variant order.
    Sum(Vec<Node<TypeExpression>>),
    /// A function from one parameter carrier to one result carrier.
    Function {
        /// The parameter carrier type.
        parameter: Box<Node<TypeExpression>>,
        /// The result carrier type.
        result: Box<Node<TypeExpression>>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A value binding before names and types are admitted.
pub struct Binding {
    /// The pattern introducing names.
    pub pattern: Node<Pattern>,
    /// An optional source type annotation.
    pub annotation: Option<Node<TypeExpression>>,
    /// The initializer expression.
    pub value: Node<Expression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A source binding pattern.
pub enum Pattern {
    /// A pattern introducing one name.
    Name(Name),
    /// A pattern that discards its value.
    Wildcard,
    /// A product pattern whose elements follow source field order.
    Product(Vec<Node<Pattern>>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A parsed expression that preserves source forms for resolution and diagnostics.
pub enum Expression {
    /// An unresolved value name.
    Name(Name),
    /// An unresolved value name with explicit type arguments.
    GenericName {
        /// The referenced value name.
        name: Name,
        /// Type arguments in source order.
        arguments: Vec<Node<TypeExpression>>,
    },
    /// An integer literal with its radix and optional suffix preserved.
    Integer(IntegerLiteral),
    /// A decimal floating-point literal before exact binary rounding.
    Float(DecimalFloatLiteral),
    /// A decoded byte literal.
    Byte(u8),
    /// A decoded Symbol byte sequence.
    Symbol(Vec<u8>),
    /// The unit value `()`.
    Unit,
    /// An explicitly parenthesized expression.
    Parenthesized(Box<Node<Expression>>),
    /// A product value in source evaluation order.
    Product(Vec<Node<Expression>>),
    /// A direct expression block.
    Block(ExpressionBlock),
    /// A block with named result boundaries.
    ResultBlock {
        /// Result binder names in variant order.
        result_binders: Vec<Name>,
        /// The block evaluated within those boundaries.
        body: ExpressionBlock,
    },
    /// A lambda literal.
    Lambda(Lambda),
    /// An ordinary or receiver-normalized application.
    Call {
        /// The callee expression.
        callee: Box<Node<Expression>>,
        /// Arguments in source evaluation order.
        arguments: Vec<Node<Expression>>,
    },
    /// Application of zero or more continuations to a value.
    ContinuationApplication {
        /// The value being eliminated or forwarded.
        value: Box<Node<Expression>>,
        /// Continuations in source order.
        continuations: Vec<Node<Expression>>,
    },
    /// A postfix numeric conversion.
    Conversion {
        /// The destination type name.
        type_name: Name,
        /// The converted value.
        value: Box<Node<Expression>>,
    },
    /// A value-producing conditional.
    If {
        /// The Boolean condition.
        condition: Box<Node<Expression>>,
        /// The block evaluated when true.
        then_branch: ExpressionBlock,
        /// The block evaluated when false.
        else_branch: ExpressionBlock,
    },
    /// A conditional block whose absent branch produces `Unit`.
    When {
        /// The Boolean condition.
        condition: Box<Node<Expression>>,
        /// The block evaluated when true.
        body: ExpressionBlock,
    },
    /// A prefix unary operation.
    Unary {
        /// The operator token and its range.
        operator: Node<UnaryOperator>,
        /// The operand expression.
        operand: Box<Node<Expression>>,
    },
    /// A binary operation after precedence parsing.
    Binary {
        /// The operator token and its range.
        operator: Node<BinaryOperator>,
        /// The left operand.
        left: Box<Node<Expression>>,
        /// The right operand.
        right: Box<Node<Expression>>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A lambda literal before capture inference and type checking.
pub struct Lambda {
    /// The optional source parameter pattern; absence denotes `Unit`.
    pub parameter: Option<Box<Node<Pattern>>>,
    /// The lambda body.
    pub body: LambdaBody,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A sequence of evaluated body items followed by one result expression.
pub struct ExpressionBlock {
    /// Bindings and discarded expressions in source order.
    pub items: Vec<BodyItem>,
    /// The block's final expression.
    pub result: Box<Node<Expression>>,
    /// The complete delimited block range.
    pub span: Span,
}

/// The source representation shared by lambda bodies and direct blocks.
pub type LambdaBody = ExpressionBlock;

#[derive(Clone, Debug, Eq, PartialEq)]
/// One non-result item in a body.
pub enum BodyItem {
    /// A value binding whose name enters scope after its initializer.
    Binding(Node<Binding>),
    /// An expression evaluated only for effects or abrupt completion.
    Expression(Node<Expression>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A prefix operator recognized by the parser.
pub enum UnaryOperator {
    /// Numeric negation (`-`).
    Negate,
    /// Boolean negation (`!`).
    LogicalNot,
    /// Integer bitwise complement (`~`).
    BitwiseNot,
    /// Symbol byte length (`#`).
    SymbolLength,
    /// Symbol/Buffer snapshot conversion (`*`).
    Star,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A binary operator after the parser has applied precedence and associativity.
pub enum BinaryOperator {
    /// Symbol byte access (`#`).
    SymbolAt,
    /// Multiplication (`*`).
    Multiply,
    /// Division (`/`).
    Divide,
    /// Remainder (`%`).
    Remainder,
    /// Addition or Symbol concatenation (`+`).
    Add,
    /// Subtraction (`-`).
    Subtract,
    /// Left shift (`<<`).
    ShiftLeft,
    /// Right shift (`>>`).
    ShiftRight,
    /// Less-than comparison (`<`).
    Less,
    /// Less-than-or-equal comparison (`<=`).
    LessEqual,
    /// Greater-than comparison (`>`).
    Greater,
    /// Greater-than-or-equal comparison (`>=`).
    GreaterEqual,
    /// Equality comparison (`==`).
    Equal,
    /// Inequality comparison (`!=`).
    NotEqual,
    /// Integer bitwise conjunction (`&`).
    BitwiseAnd,
    /// Integer bitwise exclusive-or (`^`).
    BitwiseXor,
    /// Integer bitwise disjunction (`|`).
    BitwiseOr,
    /// Short-circuit Boolean conjunction (`&&`).
    LogicalAnd,
    /// Short-circuit Boolean disjunction (`||`).
    LogicalOr,
}
