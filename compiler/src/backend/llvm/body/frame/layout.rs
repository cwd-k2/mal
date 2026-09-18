use super::super::types::{Types, ValueType, align};

pub(super) struct FrameLayout {
    pub(super) fields: Vec<FieldLayout>,
    pub(super) environment: Option<usize>,
    pub(super) footer: Option<usize>,
    pub(super) size: usize,
}

pub(super) struct FieldLayout {
    pub(super) offset: usize,
    pub(super) value_type: ValueType,
}

impl FrameLayout {
    pub(super) fn new(
        frame: &crate::execution::ControlFrame,
        types: Types,
        tagged: bool,
    ) -> Option<Self> {
        let mut offset = usize::from(tagged) * 4;
        let mut frame_alignment = 1;
        let mut fields = Vec::with_capacity(frame.fields.len());
        for field in &frame.fields {
            let value_type = types.value(&field.ty)?;
            frame_alignment = frame_alignment.max(value_type.alignment);
            offset = align(offset, value_type.alignment)?;
            let size = value_type.size;
            fields.push(FieldLayout { offset, value_type });
            offset = offset.checked_add(size)?;
        }
        let environment = if frame.carries_environment {
            frame_alignment = frame_alignment.max(types.pointer_alignment());
            offset = align(offset, types.pointer_alignment())?;
            let field = offset;
            offset = offset.checked_add(types.pointer_size())?;
            Some(field)
        } else {
            None
        };
        let (footer, unpadded_size) = if tagged {
            let footer = align(offset, types.index_alignment())?;
            (Some(footer), footer.checked_add(types.index_size())?)
        } else {
            (None, align(offset.max(1), frame_alignment)?)
        };
        let universal_alignment = types.maximum_value_alignment().max(4);
        let size = align(unpadded_size, universal_alignment)?;
        Some(Self {
            fields,
            environment,
            footer,
            size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anf::ast::ValueId;
    use crate::backend::llvm::TargetLayout;
    use crate::check::ast::Type;
    use crate::control::ast::LiveValue;
    use crate::execution::ControlFrame;
    use crate::source::{FileId, Span};

    fn frame(types: Vec<Type>) -> ControlFrame {
        ControlFrame {
            resume: crate::control::ast::StateId(0),
            fields: types
                .into_iter()
                .enumerate()
                .map(|(index, ty)| LiveValue {
                    id: ValueId::Temporary(u32::try_from(index).unwrap()),
                    ty,
                    span: Span::new(FileId::new(0), 0, 0),
                })
                .collect(),
            carries_environment: false,
        }
    }

    #[test]
    fn pads_every_frame_to_the_target_wide_control_alignment() {
        let types = Types::for_target(TargetLayout {
            pointer_size: 4,
            pointer_alignment: 4,
            index_size: 4,
            integer_alignments: [1, 2, 4, 8],
            float_alignments: [4, 16],
            supports_pointer_alignment: true,
        })
        .unwrap();

        let narrow = FrameLayout::new(&frame(vec![Type::UInt8]), types.clone(), false).unwrap();
        let wide = FrameLayout::new(&frame(vec![Type::Float64]), types.clone(), true).unwrap();

        assert_eq!(types.maximum_value_alignment(), 16);
        assert_eq!(narrow.size, 16);
        assert_eq!(wide.fields[0].offset, 16);
        assert_eq!(wide.size, 32);
        assert_eq!(narrow.size % 16, 0);
        assert_eq!(wide.size % 16, 0);
    }

    #[test]
    fn keeps_consecutive_tagged_frame_starts_aligned_for_metadata() {
        let types = Types::for_target(TargetLayout {
            pointer_size: 2,
            pointer_alignment: 1,
            index_size: 2,
            integer_alignments: [1, 1, 2, 2],
            float_alignments: [1, 2],
            supports_pointer_alignment: true,
        })
        .unwrap();

        let first = FrameLayout::new(&frame(vec![Type::UInt8]), types.clone(), true).unwrap();
        let second = FrameLayout::new(&frame(vec![Type::UInt16]), types.clone(), true).unwrap();

        assert_eq!(types.maximum_value_alignment(), 2);
        assert_eq!(first.size % 4, 0);
        assert_eq!(second.size % 4, 0);
        assert_eq!((first.size + second.size) % 4, 0);
    }
}
