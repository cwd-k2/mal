use crate::check::ast::Type;

use super::scalar::scalar_type;

#[derive(Clone)]
pub(in crate::backend::llvm) struct ValueType {
    pub(in crate::backend::llvm) llvm: String,
    pub(in crate::backend::llvm) alignment: usize,
    pub(in crate::backend::llvm) size: usize,
}

pub(in crate::backend::llvm) struct Field {
    pub(in crate::backend::llvm) offset: usize,
}

#[derive(Clone, Copy)]
pub(in crate::backend::llvm) struct Types {
    pointer_size: usize,
}

impl Types {
    pub(in crate::backend::llvm) fn new(pointer_size: usize) -> Option<Self> {
        pointer_size
            .is_power_of_two()
            .then_some(Self { pointer_size })
    }

    pub(in crate::backend::llvm) fn value(self, ty: &Type) -> Option<ValueType> {
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
            Type::External { .. } => Some(ValueType {
                llvm: format!("i{}", self.pointer_size.checked_mul(8)?),
                alignment: self.pointer_size,
                size: self.pointer_size,
            }),
            Type::Product(elements) => self.product(elements),
            Type::Sum(_) if is_bool(ty) => Some(ValueType {
                llvm: "i1".into(),
                alignment: 1,
                size: 1,
            }),
            Type::Sum(elements) => self.sum(elements),
            _ => None,
        }
    }

    fn product(self, elements: &[Type]) -> Option<ValueType> {
        aggregate_type(self.fields(elements)?)
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

    pub(in crate::backend::llvm) fn product_fields(self, ty: &Type) -> Option<Vec<Field>> {
        let Type::Product(elements) = ty else {
            return None;
        };
        field_layouts(&self.fields(elements)?)
    }

    pub(in crate::backend::llvm) fn sum_fields(self, ty: &Type) -> Option<Vec<Field>> {
        let Type::Sum(elements) = ty else {
            return None;
        };
        let mut fields = vec![ValueType {
            llvm: "i32".into(),
            alignment: 4,
            size: 4,
        }];
        fields.extend(self.fields(elements)?);
        field_layouts(&fields)
    }

    fn fields(self, elements: &[Type]) -> Option<Vec<ValueType>> {
        elements.iter().map(|element| self.value(element)).collect()
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

fn field_layouts(fields: &[ValueType]) -> Option<Vec<Field>> {
    let mut offset = 0usize;
    fields
        .iter()
        .map(|value_type| {
            offset = align(offset, value_type.alignment)?;
            let field = Field { offset };
            offset = offset.checked_add(value_type.size)?;
            Some(field)
        })
        .collect()
}

pub(in crate::backend::llvm) fn is_bool(ty: &Type) -> bool {
    matches!(ty, Type::Sum(elements) if elements == &[Type::Unit, Type::Unit])
}

pub(super) fn align(value: usize, alignment: usize) -> Option<usize> {
    value
        .checked_add(alignment.checked_sub(1)?)
        .map(|value| value & !(alignment - 1))
}
