//! Resolved bindings, patterns, expressions, lambdas, and blocks.

use super::*;

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
