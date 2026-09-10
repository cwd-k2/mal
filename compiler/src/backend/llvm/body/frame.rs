use crate::anf::ast::ValueId;
use crate::closure::ast::Atom;
use crate::control::ast::StateId;

use super::types::{Types, ValueType, align};
use super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(super) fn emit_frame_call(&mut self, site: StateId, argument: &Atom) -> Option<()> {
        let frame = self.execution.control_frames.frame(site)?.clone();
        let layout = FrameLayout::new(&frame, self.types)?;
        let top = self.register();
        self.line(format!("  {top} = load i64, ptr %mal_control_top, align 8"));
        let storage = self.register();
        self.line(format!(
            "  {storage} = call ptr @mal_control_reserve_frame(ptr %mal_context, i64 {top}, i64 {})",
            layout.size
        ));
        let next_top = self.register();
        self.line(format!("  {next_top} = add i64 {top}, {}", layout.size));
        let frame_pointer = self.register();
        self.line(format!(
            "  {frame_pointer} = getelementptr i8, ptr {storage}, i64 {top}"
        ));
        self.line(format!(
            "  store i32 {}, ptr {frame_pointer}, align 4",
            site.0
        ));
        for (field, layout) in frame.fields.iter().zip(&layout.fields) {
            let value = self.load_binding(field.value.id)?;
            let pointer = self.register();
            self.line(format!(
                "  {pointer} = getelementptr i8, ptr {frame_pointer}, i64 {}",
                layout.offset
            ));
            self.line(format!(
                "  store {} {}, ptr {pointer}, align {}",
                layout.value_type.llvm, value.representation, layout.value_type.alignment
            ));
        }
        let footer = self.register();
        self.line(format!(
            "  {footer} = getelementptr i8, ptr {frame_pointer}, i64 {}",
            layout.footer
        ));
        self.line(format!("  store i64 {top}, ptr {footer}, align 8"));
        self.line(format!(
            "  store i64 {next_top}, ptr %mal_control_top, align 8"
        ));
        let argument = self.atom(argument)?;
        if argument.ty != self.function.parameter.ty {
            return None;
        }
        if let Some(parameter) = self.function.parameter.binding {
            let slot = self.slots.get(&parameter)?.clone();
            let value_type = self.types.value(&slot.ty)?;
            self.line(format!(
                "  store {} {}, ptr %mal_slot_{}, align {}",
                value_type.llvm, argument.representation, slot.index, value_type.alignment
            ));
        } else if self.function.parameter.ty != crate::check::ast::Type::Unit {
            return None;
        }
        self.line(format!("  br label %mal_state_{}", self.function.entry.0));
        Some(())
    }

    pub(super) fn emit_frame_return(&mut self, site: StateId, result: &str) -> Option<()> {
        let frame_sites = self
            .states
            .iter()
            .filter(|candidate| self.execution.control_frames.frame(**candidate).is_some())
            .copied()
            .collect::<Vec<_>>();
        let top = self.register();
        self.line(format!("  {top} = load i64, ptr %mal_control_top, align 8"));
        let finished = self.register();
        self.line(format!("  {finished} = icmp eq i64 {top}, 0"));
        self.line(format!(
            "  br i1 {finished}, label %mal_return_done_{}, label %mal_return_pop_{}",
            site.0, site.0
        ));
        self.line(format!("mal_return_done_{}:", site.0));
        let result_type = self.types.value(&self.result_type)?;
        self.line(format!("  ret {} {result}", result_type.llvm));
        self.line(format!("mal_return_pop_{}:", site.0));
        let storage = self.register();
        self.line(format!(
            "  {storage} = call ptr @mal_control_storage(ptr %mal_context)"
        ));
        let footer_offset = self.register();
        self.line(format!("  {footer_offset} = sub i64 {top}, 8"));
        let footer = self.register();
        self.line(format!(
            "  {footer} = getelementptr i8, ptr {storage}, i64 {footer_offset}"
        ));
        let previous_top = self.register();
        self.line(format!(
            "  {previous_top} = load i64, ptr {footer}, align 8"
        ));
        self.line(format!(
            "  store i64 {previous_top}, ptr %mal_control_top, align 8"
        ));
        let frame_pointer = self.register();
        self.line(format!(
            "  {frame_pointer} = getelementptr i8, ptr {storage}, i64 {previous_top}"
        ));
        let tag = self.register();
        self.line(format!("  {tag} = load i32, ptr {frame_pointer}, align 4"));
        let cases = frame_sites
            .iter()
            .map(|frame_site| {
                format!(
                    "    i32 {}, label %mal_frame_{}_from_{}",
                    frame_site.0, frame_site.0, site.0
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        self.line(format!(
            "  switch i32 {tag}, label %mal_invalid_frame_{0} [\n{cases}\n  ]",
            site.0
        ));
        self.line(format!("mal_invalid_frame_{}:", site.0));
        self.line("  unreachable");
        for frame_site in frame_sites {
            self.emit_frame_resume(site, frame_site, result, &frame_pointer)?;
        }
        Some(())
    }

    fn emit_frame_resume(
        &mut self,
        return_site: StateId,
        frame_site: StateId,
        result: &str,
        frame_pointer: &str,
    ) -> Option<()> {
        let frame = self.execution.control_frames.frame(frame_site)?.clone();
        let layout = FrameLayout::new(&frame, self.types)?;
        self.line(format!(
            "mal_frame_{}_from_{}:",
            frame_site.0, return_site.0
        ));
        for (field, layout) in frame.fields.iter().zip(&layout.fields) {
            let pointer = self.register();
            self.line(format!(
                "  {pointer} = getelementptr i8, ptr {frame_pointer}, i64 {}",
                layout.offset
            ));
            let value = self.register();
            self.line(format!(
                "  {value} = load {}, ptr {pointer}, align {}",
                layout.value_type.llvm, layout.value_type.alignment
            ));
            let slot = self.slots.get(&field.value.id)?.clone();
            self.line(format!(
                "  store {} {value}, ptr %mal_slot_{}, align {}",
                layout.value_type.llvm, slot.index, layout.value_type.alignment
            ));
        }
        let input = self.control.states[frame.resume.0].input.as_ref()?;
        self.store_pattern(
            input,
            Some(&EmittedValue {
                ty: self.result_type.clone(),
                representation: result.into(),
                owned: false,
            }),
        )?;
        self.line(format!("  br label %mal_state_{}", frame.resume.0));
        Some(())
    }

    fn load_binding(&mut self, id: ValueId) -> Option<EmittedValue> {
        let slot = self.slots.get(&id)?.clone();
        let value_type = self.types.value(&slot.ty)?;
        let register = self.register();
        self.line(format!(
            "  {register} = load {}, ptr %mal_slot_{}, align {}",
            value_type.llvm, slot.index, value_type.alignment
        ));
        Some(EmittedValue {
            ty: slot.ty,
            representation: register,
            owned: false,
        })
    }
}

struct FrameLayout {
    fields: Vec<FieldLayout>,
    footer: usize,
    size: usize,
}

struct FieldLayout {
    offset: usize,
    value_type: ValueType,
}

impl FrameLayout {
    fn new(frame: &crate::execution::ControlFrame, types: Types) -> Option<Self> {
        let mut offset = 4usize;
        let mut fields = Vec::with_capacity(frame.fields.len());
        for field in &frame.fields {
            let value_type = types.value(&field.value.ty)?;
            offset = align(offset, value_type.alignment)?;
            let size = value_type.size;
            fields.push(FieldLayout { offset, value_type });
            offset = offset.checked_add(size)?;
        }
        let footer = align(offset, 8)?;
        let size = footer.checked_add(8)?;
        Some(Self {
            fields,
            footer,
            size,
        })
    }
}
