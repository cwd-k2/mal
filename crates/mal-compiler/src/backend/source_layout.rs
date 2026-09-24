use mal_frontend::check::ast::Type;

use crate::backend::llvm::TargetLayout;

#[derive(Clone, Copy)]
pub(crate) struct Field {
    pub(crate) offset: usize,
}

fn align(offset: usize, alignment: usize) -> Option<usize> {
    offset
        .checked_add(alignment.checked_sub(1)?)
        .map(|value| value & !(alignment - 1))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Layout {
    pub(crate) alignment: usize,
    pub(crate) stride: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct SumLayout {
    pub(crate) tag_bits: usize,
    pub(crate) payload_offset: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct SourceLayouts {
    target: TargetLayout,
}

impl SourceLayouts {
    pub(crate) fn new(target: TargetLayout) -> Self {
        Self { target }
    }

    pub(crate) fn layout(self, ty: &Type) -> Option<Layout> {
        self.layout_cached(ty, &mut std::collections::HashMap::new())
    }

    fn layout_cached(
        self,
        ty: &Type,
        cache: &mut std::collections::HashMap<mal_frontend::check::ast::SharedTypeId, Layout>,
    ) -> Option<Layout> {
        if let Some(layout) = ty.shared_id().and_then(|id| cache.get(&id)) {
            return Some(*layout);
        }
        if let Some((bits, floating)) = scalar_layout(ty, self.target.index_size) {
            return Some(Layout {
                alignment: self.target.scalar_alignment(bits, floating)?,
                stride: usize::from(bits) / 8,
            });
        }
        let layout = match ty {
            Type::Unit => Some(Layout {
                alignment: 1,
                stride: 0,
            }),
            Type::Address => Some(Layout {
                alignment: self.target.pointer_alignment,
                stride: self.target.pointer_size,
            }),
            Type::Product(elements) => {
                let mut offset = 0usize;
                let mut alignment = 1usize;
                for element in elements.iter() {
                    let field = self.layout_cached(element, cache)?;
                    offset = align(offset, field.alignment)?;
                    offset = offset.checked_add(field.stride)?;
                    alignment = alignment.max(field.alignment);
                }
                Some(Layout {
                    alignment,
                    stride: align(offset, alignment)?,
                })
            }
            Type::Sum(elements) if elements.len() >= 2 => {
                let tag_size = tag_bits(elements.len())? / 8;
                let tag_alignment = self
                    .target
                    .scalar_alignment(u8::try_from(tag_size.checked_mul(8)?).ok()?, false)?;
                let mut payload_alignment = 1usize;
                let mut payload_extent = 0usize;
                for element in elements.iter() {
                    let variant = self.layout_cached(element, cache)?;
                    payload_alignment = payload_alignment.max(variant.alignment);
                    payload_extent = payload_extent.max(variant.stride);
                }
                let alignment = tag_alignment.max(payload_alignment);
                let payload_offset = align(tag_size, payload_alignment)?;
                Some(Layout {
                    alignment,
                    stride: align(payload_offset.checked_add(payload_extent)?, alignment)?,
                })
            }
            _ => None,
        };
        if let (Some(id), Some(layout)) = (ty.shared_id(), layout) {
            cache.insert(id, layout);
        }
        layout
    }

    pub(crate) fn product_fields(self, ty: &Type) -> Option<Vec<Field>> {
        let Type::Product(elements) = ty else {
            return None;
        };
        let mut offset = 0usize;
        elements
            .iter()
            .map(|element| {
                let layout = self.layout(element)?;
                offset = align(offset, layout.alignment)?;
                let field = Field { offset };
                offset = offset.checked_add(layout.stride)?;
                Some(field)
            })
            .collect()
    }

    pub(crate) fn sum(self, ty: &Type) -> Option<SumLayout> {
        let Type::Sum(elements) = ty else {
            return None;
        };
        let tag_bits = tag_bits(elements.len())?;
        let payload_alignment = elements
            .iter()
            .map(|element| self.layout(element).map(|layout| layout.alignment))
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .max()
            .unwrap_or(1);
        Some(SumLayout {
            tag_bits,
            payload_offset: align(tag_bits / 8, payload_alignment)?,
        })
    }
}

fn scalar_layout(ty: &Type, index_size: usize) -> Option<(u8, bool)> {
    Some(match ty {
        Type::Int8 | Type::UInt8 => (8, false),
        Type::Int16 | Type::UInt16 => (16, false),
        Type::Int32 | Type::UInt32 => (32, false),
        Type::Int64 | Type::UInt64 => (64, false),
        Type::Float32 => (32, true),
        Type::Float64 => (64, true),
        Type::ByteSize | Type::USize => (u8::try_from(index_size.checked_mul(8)?).ok()?, false),
        _ => return None,
    })
}

fn tag_bits(variants: usize) -> Option<usize> {
    if variants < 2 {
        return None;
    }
    Some(if variants <= 1 << 8 {
        8
    } else if variants <= 1 << 16 {
        16
    } else if u32::try_from(variants).is_ok() {
        32
    } else {
        64
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_target_abi_alignment_instead_of_scalar_size() {
        let target =
            crate::backend::llvm::target_layout("e-p:64:32:64:32-i16:32-i64:32-f32:64-f64:128")
                .expect("synthetic target layout");
        let layouts = SourceLayouts::new(target);
        let ty =
            Type::Product(vec![Type::UInt8, Type::UInt16, Type::Float32, Type::Address].into());

        assert_eq!(
            layouts.layout(&ty),
            Some(Layout {
                alignment: 8,
                stride: 24,
            })
        );
        assert_eq!(
            layouts
                .product_fields(&ty)
                .unwrap()
                .into_iter()
                .map(|field| field.offset)
                .collect::<Vec<_>>(),
            [0, 4, 8, 12]
        );
    }
}
