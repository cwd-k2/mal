use std::collections::HashMap;

use crate::check::ast::{SharedTypeId, Type};

use super::scalar::scalar_type;

#[derive(Clone)]
pub(in crate::backend::llvm) struct ValueType {
    pub(in crate::backend::llvm) llvm: String,
    pub(in crate::backend::llvm) alignment: usize,
    pub(in crate::backend::llvm) size: usize,
}

#[derive(Clone, Copy)]
pub(in crate::backend::llvm) struct Field {
    pub(in crate::backend::llvm) offset: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::backend::llvm) struct SourceLayout {
    pub(in crate::backend::llvm) alignment: usize,
    pub(in crate::backend::llvm) stride: usize,
}

#[derive(Clone, Copy)]
pub(in crate::backend::llvm) struct SourceSumLayout {
    pub(in crate::backend::llvm) tag_bits: usize,
    pub(in crate::backend::llvm) payload_offset: usize,
}

#[derive(Clone, Copy)]
pub(in crate::backend::llvm) struct Types {
    pointer_size: usize,
    index_size: usize,
}

impl Types {
    #[cfg(test)]
    pub(in crate::backend::llvm) fn new(pointer_size: usize) -> Option<Self> {
        Self::for_target(pointer_size, pointer_size)
    }

    pub(in crate::backend::llvm) fn for_target(
        pointer_size: usize,
        index_size: usize,
    ) -> Option<Self> {
        (pointer_size.is_power_of_two() && index_size.is_power_of_two()).then_some(Self {
            pointer_size,
            index_size,
        })
    }

    pub(in crate::backend::llvm) fn value(self, ty: &Type) -> Option<ValueType> {
        self.value_cached(ty, &mut HashMap::new())
    }

    fn value_cached(
        self,
        ty: &Type,
        cache: &mut HashMap<SharedTypeId, ValueType>,
    ) -> Option<ValueType> {
        if let Some(value) = ty.shared_id().and_then(|id| cache.get(&id)) {
            return Some(value.clone());
        }
        if let Some(scalar) = scalar_type(ty, self.index_size) {
            return Some(ValueType {
                llvm: scalar.llvm.into(),
                alignment: scalar.alignment.into(),
                size: usize::from(scalar.bits) / 8,
            });
        }
        let value = match ty {
            Type::Unit => Some(ValueType {
                llvm: "i8".into(),
                alignment: 1,
                size: 1,
            }),
            Type::Address | Type::Cursor(_) => Some(ValueType {
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
            Type::Function { .. } => aggregate_type(vec![
                ValueType {
                    llvm: "ptr".into(),
                    alignment: self.pointer_size,
                    size: self.pointer_size,
                },
                ValueType {
                    llvm: "ptr".into(),
                    alignment: self.pointer_size,
                    size: self.pointer_size,
                },
            ]),
            Type::Region(_) => aggregate_type(vec![
                ValueType {
                    llvm: "ptr".into(),
                    alignment: self.pointer_size,
                    size: self.pointer_size,
                },
                ValueType {
                    llvm: self.pointer_integer()?,
                    alignment: self.index_size,
                    size: self.index_size,
                },
            ]),
            Type::Packed(_) => aggregate_type(vec![
                ValueType {
                    llvm: "ptr".into(),
                    alignment: self.pointer_size,
                    size: self.pointer_size,
                },
                ValueType {
                    llvm: self.pointer_integer()?,
                    alignment: self.index_size,
                    size: self.index_size,
                },
                ValueType {
                    llvm: self.pointer_integer()?,
                    alignment: self.index_size,
                    size: self.index_size,
                },
            ]),
            Type::Product(elements) => self.product(elements, cache),
            Type::Sum(_) if is_bool(ty) => Some(ValueType {
                llvm: "i1".into(),
                alignment: 1,
                size: 1,
            }),
            Type::Sum(elements) => self.sum(elements, cache),
            _ => None,
        }?;
        if let Some(id) = ty.shared_id() {
            cache.insert(id, value.clone());
        }
        Some(value)
    }

    pub(in crate::backend::llvm) fn pointer_integer(self) -> Option<String> {
        Some(format!("i{}", self.index_size.checked_mul(8)?))
    }

    pub(in crate::backend::llvm) fn pointer_representation_integer(self) -> Option<String> {
        Some(format!("i{}", self.pointer_size.checked_mul(8)?))
    }

    pub(in crate::backend::llvm) fn pointer_size(self) -> usize {
        self.pointer_size
    }

    pub(in crate::backend::llvm) fn index_size(self) -> usize {
        self.index_size
    }

    pub(in crate::backend::llvm) fn source_layout(self, ty: &Type) -> Option<SourceLayout> {
        if let Some(scalar) = scalar_type(ty, self.index_size) {
            return Some(SourceLayout {
                alignment: scalar.alignment.into(),
                stride: usize::from(scalar.bits) / 8,
            });
        }
        match ty {
            Type::Unit => Some(SourceLayout {
                alignment: 1,
                stride: 0,
            }),
            Type::Address => Some(SourceLayout {
                alignment: self.pointer_size,
                stride: self.pointer_size,
            }),
            Type::Product(elements) => {
                let mut offset = 0usize;
                let mut alignment = 1usize;
                for element in elements.iter() {
                    let field = self.source_layout(element)?;
                    offset = align(offset, field.alignment)?;
                    offset = offset.checked_add(field.stride)?;
                    alignment = alignment.max(field.alignment);
                }
                Some(SourceLayout {
                    alignment,
                    stride: align(offset, alignment)?,
                })
            }
            Type::Sum(elements) if elements.len() >= 2 => {
                let tag_size = if elements.len() <= 1 << 8 {
                    1
                } else if elements.len() <= 1 << 16 {
                    2
                } else if u32::try_from(elements.len()).is_ok() {
                    4
                } else {
                    8
                };
                let mut payload_alignment = 1usize;
                let mut payload_extent = 0usize;
                for element in elements.iter() {
                    let variant = self.source_layout(element)?;
                    payload_alignment = payload_alignment.max(variant.alignment);
                    payload_extent = payload_extent.max(variant.stride);
                }
                let alignment = tag_size.max(payload_alignment);
                let payload_offset = align(tag_size, payload_alignment)?;
                Some(SourceLayout {
                    alignment,
                    stride: align(payload_offset.checked_add(payload_extent)?, alignment)?,
                })
            }
            _ => None,
        }
    }

    pub(in crate::backend::llvm) fn source_product_fields(self, ty: &Type) -> Option<Vec<Field>> {
        let Type::Product(elements) = ty else {
            return None;
        };
        let layouts = elements
            .iter()
            .map(|element| {
                let layout = self.source_layout(element)?;
                Some(ValueType {
                    llvm: String::new(),
                    alignment: layout.alignment,
                    size: layout.stride,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        field_layouts(&layouts)
    }

    pub(in crate::backend::llvm) fn source_sum_layout(self, ty: &Type) -> Option<SourceSumLayout> {
        let Type::Sum(elements) = ty else {
            return None;
        };
        if elements.len() < 2 {
            return None;
        }
        let tag_bits = if elements.len() <= 1 << 8 {
            8
        } else if elements.len() <= 1 << 16 {
            16
        } else if u32::try_from(elements.len()).is_ok() {
            32
        } else {
            64
        };
        let payload_alignment = elements
            .iter()
            .map(|element| self.source_layout(element).map(|layout| layout.alignment))
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .max()
            .unwrap_or(1);
        Some(SourceSumLayout {
            tag_bits,
            payload_offset: align(tag_bits / 8, payload_alignment)?,
        })
    }

    fn product(
        self,
        elements: &[Type],
        cache: &mut HashMap<SharedTypeId, ValueType>,
    ) -> Option<ValueType> {
        aggregate_type(self.fields(elements, cache)?)
    }

    fn sum(
        self,
        elements: &[Type],
        cache: &mut HashMap<SharedTypeId, ValueType>,
    ) -> Option<ValueType> {
        let mut fields = vec![ValueType {
            llvm: "i32".into(),
            alignment: 4,
            size: 4,
        }];
        if let Some(payload) = self.sum_payload(elements, cache)? {
            fields.push(payload);
        }
        aggregate_type(fields)
    }

    pub(in crate::backend::llvm) fn product_fields(self, ty: &Type) -> Option<Vec<Field>> {
        let Type::Product(elements) = ty else {
            return None;
        };
        field_layouts(&self.fields(elements, &mut HashMap::new())?)
    }

    pub(in crate::backend::llvm) fn sum_fields(self, ty: &Type) -> Option<Vec<Field>> {
        let Type::Sum(elements) = ty else {
            return None;
        };
        let tag = ValueType {
            llvm: "i32".into(),
            alignment: 4,
            size: 4,
        };
        let Some(payload) = self.sum_payload(elements, &mut HashMap::new())? else {
            return field_layouts(&[tag]);
        };
        let layouts = field_layouts(&[tag, payload])?;
        let payload_offset = layouts.get(1)?.offset;
        let mut variants = vec![layouts[0]];
        variants.extend((0..elements.len()).map(|_| Field {
            offset: payload_offset,
        }));
        Some(variants)
    }

    fn fields(
        self,
        elements: &[Type],
        cache: &mut HashMap<SharedTypeId, ValueType>,
    ) -> Option<Vec<ValueType>> {
        elements
            .iter()
            .map(|element| self.value_cached(element, cache))
            .collect()
    }

    fn sum_payload(
        self,
        elements: &[Type],
        cache: &mut HashMap<SharedTypeId, ValueType>,
    ) -> Option<Option<ValueType>> {
        let size = elements
            .iter()
            .map(|element| self.value_cached(element, cache).map(|value| value.size))
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .max();
        Some(size.map(|size| ValueType {
            llvm: format!("[{size} x i8]"),
            alignment: 1,
            size,
        }))
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
    matches!(ty, Type::Sum(elements) if elements.as_ref() == [Type::Unit, Type::Unit])
}

pub(super) fn align(value: usize, alignment: usize) -> Option<usize> {
    value
        .checked_add(alignment.checked_sub(1)?)
        .map(|value| value & !(alignment - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_use_one_maximum_sized_payload_region() {
        let types = Types::new(8).unwrap();
        let ty = Type::Sum(vec![Type::UInt8, Type::UInt64].into());

        let value = types.value(&ty).unwrap();
        let fields = types.sum_fields(&ty).unwrap();

        assert_eq!(value.llvm, "{ i32, [8 x i8] }");
        assert_eq!(value.size, 12);
        assert_eq!(
            fields.iter().map(|field| field.offset).collect::<Vec<_>>(),
            [0, 4, 4]
        );
    }

    #[test]
    fn lays_out_shared_sum_dags_once_per_node() {
        let mut ty = Type::Unit;
        for _ in 0..64 {
            ty = Type::Sum(vec![ty.clone(), ty].into());
        }

        let value = Types::new(8).unwrap().value(&ty).unwrap();

        assert_eq!(value.size, 256);
        assert!(value.llvm.len() < 1_500);
    }
}
