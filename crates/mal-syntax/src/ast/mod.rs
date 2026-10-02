//! Source-oriented syntax tree that preserves spans and literal structure for later admission stages.

use crate::lexer::{DecimalFloatLiteral, IntegerLiteral};
use crate::source::Span;

mod expression;

pub use expression::*;

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
