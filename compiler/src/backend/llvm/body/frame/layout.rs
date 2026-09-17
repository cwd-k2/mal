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
        let (footer, size) = if tagged {
            let footer = align(offset, types.index_alignment())?;
            (Some(footer), footer.checked_add(types.index_size())?)
        } else {
            (None, align(offset.max(1), frame_alignment)?)
        };
        Some(Self {
            fields,
            environment,
            footer,
            size,
        })
    }
}
