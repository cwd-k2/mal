//! Parsed structure annotated with stable name identities, captures, and result authority.

use mal_syntax::ast::{BinaryOperator, Name, Node, UnaryOperator};
use mal_syntax::lexer::{DecimalFloatLiteral, IntegerLiteral};
use mal_syntax::source::Span;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Program-wide identity of a resolved type binding.
pub struct TypeId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Program-wide identity of a resolved value binding.
pub struct ValueId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Stable index of an external operation in the resolved program.
pub struct ExternalOperationId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Program-wide identity of a lambda invocation boundary.
pub struct LambdaId(pub u32);

pub use super::predefined::{
    ADDRESS_TYPE, BOOL_TYPE, BUFFER_TYPE, BYTE_SIZE_TYPE, FALSE_VALUE, FLOAT32_TYPE, FLOAT64_TYPE,
    INT8_TYPE, INT16_TYPE, INT32_TYPE, INT64_TYPE, SYMBOL_TYPE, TRUE_VALUE, U_SIZE_TYPE,
    UINT8_TYPE, UINT16_TYPE, UINT32_TYPE, UINT64_TYPE, UNIT_TYPE,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// The scope boundary that owns a resolved value identity.
pub enum ValueOwner {
    /// A compiler-defined value with no source binder.
    Predefined,
    /// A binding visible at file scope.
    TopLevel,
    /// A local binding or capture owned by a lambda.
    Lambda(LambdaId),
    /// A result binder whose authority is confined to a lambda invocation.
    Result(LambdaId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A type declaration paired with its stable identity.
pub struct TypeBinding {
    /// The identity used by all references to this declaration.
    pub id: TypeId,
    /// The source name and declaration span.
    pub name: Name,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A value declaration paired with its stable identity and scope owner.
pub struct ValueBinding {
    /// The identity used by all references to this declaration.
    pub id: ValueId,
    /// The source name and declaration span.
    pub name: Name,
    /// The boundary controlling visibility and capture behavior.
    pub owner: ValueOwner,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A type use resolved to its declaration while preserving use-site spelling.
pub struct TypeReference {
    /// The referenced declaration identity.
    pub id: TypeId,
    /// The source name and use-site span.
    pub name: Name,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A value use resolved to its declaration while preserving use-site spelling.
pub struct ValueReference {
    /// The referenced declaration identity.
    pub id: ValueId,
    /// The source name and use-site span.
    pub name: Name,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// One resolved source graph flattened into dependency and root items.
pub struct Program {
    /// Resolved top-level items in graph order.
    pub items: Vec<Node<TopItem>>,
    /// The root program span used for program-level diagnostics.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A top-level item after declaration identities have been assigned.
pub enum TopItem {
    /// A monomorphic type alias.
    TypeAlias {
        /// The alias declaration.
        binding: TypeBinding,
        /// Its resolved definition.
        value: Node<TypeExpression>,
    },
    /// A generic type alias with scoped parameter identities.
    GenericTypeAlias {
        /// The alias declaration.
        binding: TypeBinding,
        /// Generic parameter bindings in declaration order.
        parameters: Vec<TypeBinding>,
        /// Its resolved definition.
        value: Node<TypeExpression>,
    },
    /// A source-defined abstract type with file-local representation authority.
    OpaqueType {
        /// The opaque declaration identity.
        binding: TypeBinding,
        /// Generic parameter bindings in declaration order.
        parameters: Vec<TypeBinding>,
        /// The resolved hidden representation.
        representation: Node<TypeExpression>,
    },
    /// An opaque host type declaration.
    ExternalType {
        /// The external type declaration.
        binding: TypeBinding,
    },
    /// A host operation represented as a capture-free function boundary.
    ExternalOperation {
        /// The operation's stable host-interface index.
        id: ExternalOperationId,
        /// The value declaration referenced from expressions.
        binding: ValueBinding,
        /// The synthetic lambda identity used by later function stages.
        lambda_id: LambdaId,
        /// The resolved source function type.
        ty: Node<TypeExpression>,
    },
    /// A monomorphic value binding.
    Binding(Binding),
    /// A generic value binding with scoped type parameters.
    GenericBinding {
        /// The value declaration.
        binding: ValueBinding,
        /// Generic parameter bindings in declaration order.
        parameters: Vec<TypeBinding>,
        /// The required resolved annotation.
        annotation: Node<TypeExpression>,
        /// The resolved initializer.
        value: Node<Expression>,
    },
    /// A generic operation signature whose implementations are keyed by exact type arguments.
    OperationFamily {
        /// The family declaration shared by references and implementations.
        binding: ValueBinding,
        /// Generic parameter bindings in declaration order.
        parameters: Vec<TypeBinding>,
        /// The resolved family signature.
        annotation: Node<TypeExpression>,
    },
    /// One exact or generic implementation of an operation family.
    OperationImplementation {
        /// A reference to the previously declared family.
        family: ValueReference,
        /// Pattern variables the key binds, empty for an exact key.
        parameters: Vec<TypeBinding>,
        /// Canonical type expressions forming the implementation key.
        arguments: Vec<Node<TypeExpression>>,
        /// The implementation annotation.
        annotation: Node<TypeExpression>,
        /// The resolved implementation initializer.
        value: Node<Expression>,
        /// Each binder spelled one edit away from a visible type, with that type, for diagnostics.
        similar_types: Vec<(String, String)>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A type expression whose names point to stable declarations.
pub enum TypeExpression {
    /// A resolved named type or type parameter.
    Named(TypeReference),
    /// Application of a resolved generic constructor.
    Application {
        /// The constructor declaration.
        constructor: TypeReference,
        /// Resolved type arguments in source order.
        arguments: Vec<Node<TypeExpression>>,
    },
    /// The `Unit` type.
    Unit,
    /// An explicitly parenthesized type retained for editor ranges.
    Parenthesized(Box<Node<TypeExpression>>),
    /// A product type in field order.
    Product(Vec<Node<TypeExpression>>),
    /// A sum type in variant order.
    Sum(Vec<Node<TypeExpression>>),
    /// A function type.
    Function {
        /// The parameter carrier type.
        parameter: Box<Node<TypeExpression>>,
        /// The result carrier type.
        result: Box<Node<TypeExpression>>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A resolved binding whose introduced names carry stable identities.
pub struct Binding {
    /// The resolved binding pattern.
    pub pattern: Node<Pattern>,
    /// The optional resolved type annotation.
    pub annotation: Option<Node<TypeExpression>>,
    /// The resolved initializer.
    pub value: Node<Expression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A resolved binding pattern.
pub enum Pattern {
    /// One newly introduced value identity.
    Binding(ValueBinding),
    /// A discarded value.
    Wildcard,
    /// A product pattern in field order.
    Product(Vec<Node<Pattern>>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// An expression annotated with resolved identities but not yet types.
pub enum Expression {
    /// A use of a resolved value declaration.
    Reference(ValueReference),
    /// A resolved generic value use with explicit type arguments.
    GenericReference {
        /// The generic declaration.
        reference: ValueReference,
        /// Resolved type arguments in source order.
        arguments: Vec<Node<TypeExpression>>,
    },
    /// An integer literal awaiting expected-type checking.
    Integer(IntegerLiteral),
    /// A decimal float awaiting expected-type rounding.
    Float(DecimalFloatLiteral),
    /// A decoded byte literal.
    Byte(u8),
    /// A decoded Symbol literal.
    Symbol(Vec<u8>),
    /// The unit value.
    Unit,
    /// Parentheses retained for editor source structure.
    Parenthesized(Box<Node<Expression>>),
    /// A product whose elements retain source evaluation order.
    Product(Vec<Node<Expression>>),
    /// A direct expression block.
    Block(ExpressionBlock),
    /// A block with invocation-local result authorities.
    ResultBlock {
        /// Result binders in variant order.
        result_binders: Vec<ValueBinding>,
        /// The body governed by those result binders.
        body: ExpressionBlock,
    },
    /// A lambda with inferred capture bindings.
    Lambda(Lambda),
    /// An ordinary application.
    Call {
        /// The callee expression.
        callee: Box<Node<Expression>>,
        /// Arguments in source evaluation order.
        arguments: Vec<Node<Expression>>,
    },
    /// Elimination or forwarding through continuations.
    ContinuationApplication {
        /// The value supplied to the continuation set.
        value: Box<Node<Expression>>,
        /// Resolved continuation roles in source order.
        continuations: Vec<Continuation>,
    },
    /// A postfix numeric conversion.
    Conversion {
        /// The resolved destination type.
        type_ref: TypeReference,
        /// The converted value.
        value: Box<Node<Expression>>,
    },
    /// A value-producing conditional.
    If {
        /// The condition expression.
        condition: Box<Node<Expression>>,
        /// The true branch.
        then_branch: ExpressionBlock,
        /// The false branch.
        else_branch: ExpressionBlock,
    },
    /// A conditional block with an implicit `Unit` alternative.
    When {
        /// The condition expression.
        condition: Box<Node<Expression>>,
        /// The true branch.
        body: ExpressionBlock,
    },
    /// A prefix unary operation.
    Unary {
        /// The source operator and its span.
        operator: Node<UnaryOperator>,
        /// The operand.
        operand: Box<Node<Expression>>,
    },
    /// A precedence-resolved binary operation.
    Binary {
        /// The source operator and its span.
        operator: Node<BinaryOperator>,
        /// The left operand.
        left: Box<Node<Expression>>,
        /// The right operand.
        right: Box<Node<Expression>>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A resolved lambda boundary and the captures forwarded into it.
pub struct Lambda {
    /// The lambda's program-wide identity.
    pub id: LambdaId,
    /// The binding used for admitted direct self-reference, when present.
    pub self_binding: Option<ValueId>,
    /// Captures in first lexical-reference order.
    pub captures: Vec<Capture>,
    /// The optional parameter pattern; absence denotes `Unit`.
    pub parameter: Option<Box<Node<Pattern>>>,
    /// The resolved lambda body.
    pub body: LambdaBody,
}

/// One continuation of a continuation application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Continuation {
    /// A function value applied to the payload, or a result binder name.
    Function(Node<Expression>),
    /// A lambda literal placed as a sum elimination continuation. It never becomes a function value:
    /// its body belongs to the enclosing lambda invocation.
    Branch(Branch),
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// An in-place sum continuation owned by the enclosing lambda invocation.
pub struct Branch {
    /// The optional payload pattern; absence denotes `Unit`.
    pub parameter: Option<Box<Node<Pattern>>>,
    /// The continuation body.
    pub body: ExpressionBlock,
    /// The complete continuation source range.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// One value forwarded across a lambda boundary as a capture.
pub struct Capture {
    /// The reference visible in the enclosing boundary.
    pub source: ValueReference,
    /// The fresh binding visible inside the capturing lambda.
    pub binding: ValueBinding,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A resolved lambda body with invocation-local result scope.
pub struct LambdaBody {
    /// Body items in source evaluation order.
    pub items: Vec<BodyItem>,
    /// The final result expression.
    pub result: Box<Node<Expression>>,
    /// The complete body range.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A resolved direct block nested within an expression.
pub struct ExpressionBlock {
    /// Body items in source evaluation order.
    pub items: Vec<BodyItem>,
    /// The final result expression.
    pub result: Box<Node<Expression>>,
    /// The complete block range.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// One non-result item in a resolved body.
pub enum BodyItem {
    /// A binding whose identities enter scope after its initializer.
    Binding(Box<Node<Binding>>),
    /// An expression evaluated for effects or abrupt completion.
    Expression(Node<Expression>),
}
