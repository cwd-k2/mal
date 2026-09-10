use crate::check::ast::Type;

use super::scalar::scalar_type;

#[derive(Clone)]
pub(super) struct ValueType {
    pub(super) llvm: String,
    pub(super) alignment: usize,
    pub(super) size: usize,
}

#[derive(Clone, Copy)]
pub(super) struct Types {
    pointer_size: usize,
}

impl Types {
    pub(super) fn new(pointer_size: usize) -> Option<Self> {
        pointer_size
            .is_power_of_two()
            .then_some(Self { pointer_size })
    }

    pub(super) fn value(self, ty: &Type) -> Option<ValueType> {
        if let Some(scalar) = scalar_type(ty) {
            return Some(ValueType {
                llvm: scalar.llvm.into(),
                alignment: scalar.alignment.into(),
                size: usize::from(scalar.bits) / 8,
            });
        }
        match ty {
            Type::Unit => Some(ValueType {
                llvm: "i8".into(),
                alignment: 1,
                size: 1,
            }),
            Type::Ptr => Some(ValueType {
                llvm: "ptr".into(),
                alignment: self.pointer_size,
                size: self.pointer_size,
            }),
            Type::Symbol => Some(ValueType {
                llvm: "ptr".into(),
                alignment: self.pointer_size,
                size: self.pointer_size,
            }),
            Type::Product(elements) => self.product(elements),
            Type::Sum(_) if is_bool(ty) => Some(ValueType {
                llvm: "i1".into(),
                alignment: 1,
                size: 1,
            }),
            Type::Sum(elements) if !crate::execution::ownership::is_managed(ty) => {
                self.sum(elements)
            }
            _ => None,
        }
    }

    fn product(self, elements: &[Type]) -> Option<ValueType> {
        aggregate_type(
            elements
                .iter()
                .map(|element| self.value(element))
                .collect::<Option<Vec<_>>>()?,
        )
    }

    fn sum(self, elements: &[Type]) -> Option<ValueType> {
        let mut fields = vec![ValueType {
            llvm: "i32".into(),
            alignment: 4,
            size: 4,
        }];
        fields.extend(
            elements
                .iter()
                .map(|element| self.value(element))
                .collect::<Option<Vec<_>>>()?,
        );
        aggregate_type(fields)
    }
}

fn aggregate_type(fields: Vec<ValueType>) -> Option<ValueType> {
    let alignment = fields
        .iter()
        .map(|field| field.alignment)
        .max()
        .unwrap_or(1);
    let mut size = 0usize;
    for field in &fields {
        size = align(size, field.alignment)?;
        size = size.checked_add(field.size)?;
    }
    Some(ValueType {
        llvm: format!(
            "{{ {} }}",
            fields
                .iter()
                .map(|field| field.llvm.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        alignment,
        size: align(size, alignment)?,
    })
}

pub(super) fn is_bool(ty: &Type) -> bool {
    matches!(ty, Type::Sum(elements) if elements == &[Type::Unit, Type::Unit])
}

pub(super) fn align(value: usize, alignment: usize) -> Option<usize> {
    value
        .checked_add(alignment.checked_sub(1)?)
        .map(|value| value & !(alignment - 1))
}
