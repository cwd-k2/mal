use crate::anf::ast::ValueId;
use crate::closure::ast::Atom;
use crate::control::ast::StateId;

use super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(super) fn emit_frame_call(&mut self, site: StateId, argument: &Atom) -> Option<()> {
        let frame = self.execution.control_frames.frame(site)?.clone();
        let frame_size = frame_size(frame.fields.len())?;
        let top = self.register();
        self.line(format!("  {top} = load i64, ptr %mal_control_top, align 8"));
        let storage = self.register();
        self.line(format!(
            "  {storage} = call ptr @mal_control_reserve_frame(ptr %mal_context, i64 {top}, i64 {frame_size})"
        ));
        let next_top = self.register();
        self.line(format!("  {next_top} = add i64 {top}, {frame_size}"));
        let frame_pointer = self.register();
        self.line(format!(
            "  {frame_pointer} = getelementptr i8, ptr {storage}, i64 {top}"
        ));
        self.line(format!(
            "  store i32 {}, ptr {frame_pointer}, align 4",
            site.0
        ));
        for (index, field) in frame.fields.iter().enumerate() {
            let value = self.load_binding(field.value.id)?;
            let pointer = self.register();
            self.line(format!(
                "  {pointer} = getelementptr i8, ptr {frame_pointer}, i64 {}",
                4 + index * 4
            ));
            self.line(format!("  store i32 {value}, ptr {pointer}, align 4"));
        }
        let footer = self.register();
        self.line(format!(
            "  {footer} = getelementptr i8, ptr {frame_pointer}, i64 {}",
            frame_size - 8
        ));
        self.line(format!("  store i64 {top}, ptr {footer}, align 8"));
        self.line(format!(
            "  store i64 {next_top}, ptr %mal_control_top, align 8"
        ));
        let argument = self.atom(argument)?;
        if argument.ty != crate::check::ast::Type::Int32 {
            return None;
        }
        let parameter = self.function.parameter.binding?;
        let slot = self.slots.get(&parameter)?.clone();
        self.line(format!(
            "  store i32 {}, ptr %mal_slot_{}, align 4",
            argument.representation, slot.index
        ));
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
        self.line(format!("  ret i32 {result}"));
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
        self.line(format!(
            "mal_frame_{}_from_{}:",
            frame_site.0, return_site.0
        ));
        for (index, field) in frame.fields.iter().enumerate() {
            let pointer = self.register();
            self.line(format!(
                "  {pointer} = getelementptr i8, ptr {frame_pointer}, i64 {}",
                4 + index * 4
            ));
            let value = self.register();
            self.line(format!("  {value} = load i32, ptr {pointer}, align 4"));
            let slot = self.slots.get(&field.value.id)?.clone();
            self.line(format!(
                "  store i32 {value}, ptr %mal_slot_{}, align 4",
                slot.index
            ));
        }
        let input = self.control.states[frame.resume.0].input.as_ref()?;
        self.store_pattern(
            input,
            Some(&EmittedValue {
                ty: crate::check::ast::Type::Int32,
                representation: result.into(),
            }),
        )?;
        self.line(format!("  br label %mal_state_{}", frame.resume.0));
        Some(())
    }

    fn load_binding(&mut self, id: ValueId) -> Option<String> {
        let slot = self.slots.get(&id)?.clone();
        if slot.ty != crate::check::ast::Type::Int32 {
            return None;
        }
        let register = self.register();
        self.line(format!(
            "  {register} = load i32, ptr %mal_slot_{}, align 4",
            slot.index
        ));
        Some(register)
    }
}

fn frame_size(field_count: usize) -> Option<usize> {
    let payload = 4usize.checked_add(field_count.checked_mul(4)?)?;
    let with_padding = payload.checked_add(7)? & !7;
    with_padding.checked_add(8)
}
