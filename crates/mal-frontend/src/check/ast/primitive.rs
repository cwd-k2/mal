//! Checked operators and the closed primitive families they select.

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// A checked unary operator.
pub enum UnaryOperation {
    /// A scalar primitive.
    Primitive(UnaryPrimitive),
    /// `!`, which lowering expands into a continuation application.
    LogicalNot,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// A unary operator on a numeric scalar.
pub enum UnaryPrimitive {
    /// Integer or float negation (`-`).
    Negate,
    /// Integer complement (`~`).
    BitwiseNot,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// A checked binary operator.
pub enum BinaryOperation {
    /// A scalar primitive evaluating both operands.
    Primitive(BinaryPrimitive),
    /// `&&` or `||`, which evaluates its right operand only when needed.
    ShortCircuit(ShortCircuit),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// A binary operator on numeric scalars; comparisons produce `Bool`.
pub enum BinaryPrimitive {
    /// `*`.
    Multiply,
    /// `/`.
    Divide,
    /// `%`.
    Remainder,
    /// `+`.
    Add,
    /// `-`.
    Subtract,
    /// `<<`.
    ShiftLeft,
    /// `>>`.
    ShiftRight,
    /// `<`.
    Less,
    /// `<=`.
    LessEqual,
    /// `>`.
    Greater,
    /// `>=`.
    GreaterEqual,
    /// `==`.
    Equal,
    /// `!=`.
    NotEqual,
    /// `&`.
    BitwiseAnd,
    /// `^`.
    BitwiseXor,
    /// `|`.
    BitwiseOr,
}

impl BinaryPrimitive {
    /// Whether the operator compares its operands and produces `Bool`.
    pub const fn is_comparison(self) -> bool {
        matches!(
            self,
            Self::Less
                | Self::LessEqual
                | Self::Greater
                | Self::GreaterEqual
                | Self::Equal
                | Self::NotEqual
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// A short-circuit Boolean operator.
pub enum ShortCircuit {
    /// `&&`.
    And,
    /// `||`.
    Or,
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
