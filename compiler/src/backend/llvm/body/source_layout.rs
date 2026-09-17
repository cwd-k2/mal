use crate::check::ast::Type;

use super::types::{Field, align};
use crate::backend::llvm::TargetLayout;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Layout {
    pub(super) alignment: usize,
    pub(super) stride: usize,
}

#[derive(Clone, Copy)]
pub(super) struct SumLayout {
    pub(super) tag_bits: usize,
    pub(super) payload_offset: usize,
}

#[derive(Clone, Copy)]
pub(super) struct SourceLayouts {
    target: TargetLayout,
}

impl SourceLayouts {
    pub(super) fn new(target: TargetLayout) -> Self {
        Self { target }
    }

    pub(super) fn supports_alignment(self) -> bool {
        self.target.supports_pointer_alignment
    }

    pub(super) fn layout(self, ty: &Type) -> Option<Layout> {
        if let Some(scalar) = super::scalar::scalar_type(ty, self.target.index_size) {
            return Some(Layout {
                alignment: self.target.scalar_alignment(scalar.bits, scalar.floating)?,
                stride: usize::from(scalar.bits) / 8,
            });
        }
        match ty {
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
                    let field = self.layout(element)?;
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
                    let variant = self.layout(element)?;
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
        }
    }

    pub(super) fn product_fields(self, ty: &Type) -> Option<Vec<Field>> {
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

    pub(super) fn sum(self, ty: &Type) -> Option<SumLayout> {
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
