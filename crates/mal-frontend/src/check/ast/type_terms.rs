//! Kinds and canonical type terms.

use super::*;

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
