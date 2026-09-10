use crate::anf::ast::ValueId;
use crate::closure::ast::Atom;
use crate::control::ast::StateId;

use super::FunctionEmitter;

impl FunctionEmitter<'_> {
    pub(super) fn emit_frame_call(&mut self, site: StateId, argument: &Atom) -> Option<()> {
        let frame = self.execution.control_frames.frame(site)?.clone();
        let slot_size = frame.fields.len().checked_mul(4)?;
        let top = self.register();
        self.line(format!("  {top} = load i64, ptr %mal_control_top, align 8"));
        let next_top = self.register();
        self.line(format!("  {next_top} = add i64 {top}, 1"));
        let storage = self.register();
        self.line(format!(
            "  {storage} = call ptr @mal_control_reserve_slots(ptr %mal_context, i64 {next_top}, i64 {slot_size})"
        ));
        let offset = self.register();
        self.line(format!("  {offset} = mul i64 {top}, {slot_size}"));
        let frame_pointer = self.register();
        self.line(format!(
            "  {frame_pointer} = getelementptr i8, ptr {storage}, i64 {offset}"
        ));
        for (index, field) in frame.fields.iter().enumerate() {
            let value = self.load_binding(field.value.id)?;
            let pointer = self.register();
            self.line(format!(
                "  {pointer} = getelementptr i8, ptr {frame_pointer}, i64 {}",
                index * 4
            ));
            self.line(format!("  store i32 {value}, ptr {pointer}, align 4"));
        }
        self.line(format!(
            "  store i64 {next_top}, ptr %mal_control_top, align 8"
        ));
        let argument = self.atom(argument)?;
        let parameter = self.function.parameter.binding?;
        let slot = self.slots[&parameter];
        self.line(format!(
            "  store i32 {argument}, ptr %mal_slot_{slot}, align 4"
        ));
        self.line(format!("  br label %mal_state_{}", self.function.entry.0));
        Some(())
    }

    pub(super) fn emit_frame_return(&mut self, site: StateId, result: &str) -> Option<()> {
        let frame_site = self
            .states
            .iter()
            .find(|candidate| self.execution.control_frames.frame(**candidate).is_some())
            .copied()?;
        let frame = self.execution.control_frames.frame(frame_site)?.clone();
        let slot_size = frame.fields.len().checked_mul(4)?;
        let top = self.register();
        self.line(format!("  {top} = load i64, ptr %mal_control_top, align 8"));
        let finished = self.register();
        self.line(format!("  {finished} = icmp eq i64 {top}, 0"));
        self.line(format!(
            "  br i1 {finished}, label %mal_return_done_{}, label %mal_return_resume_{}",
            site.0, site.0
        ));
        self.line(format!("mal_return_done_{}:", site.0));
        self.line(format!("  ret i32 {result}"));
        self.line(format!("mal_return_resume_{}:", site.0));
        let previous_top = self.register();
        self.line(format!("  {previous_top} = sub i64 {top}, 1"));
        self.line(format!(
            "  store i64 {previous_top}, ptr %mal_control_top, align 8"
        ));
        let storage = self.register();
        self.line(format!(
            "  {storage} = call ptr @mal_control_storage(ptr %mal_context)"
        ));
        let offset = self.register();
        self.line(format!("  {offset} = mul i64 {previous_top}, {slot_size}"));
        let frame_pointer = self.register();
        self.line(format!(
            "  {frame_pointer} = getelementptr i8, ptr {storage}, i64 {offset}"
        ));
        for (index, field) in frame.fields.iter().enumerate() {
            let pointer = self.register();
            self.line(format!(
                "  {pointer} = getelementptr i8, ptr {frame_pointer}, i64 {}",
                index * 4
            ));
            let value = self.register();
            self.line(format!("  {value} = load i32, ptr {pointer}, align 4"));
            let slot = self.slots[&field.value.id];
            self.line(format!(
                "  store i32 {value}, ptr %mal_slot_{slot}, align 4"
            ));
        }
        let input = self.control.states[frame.resume.0].input.as_ref()?;
        self.store_pattern(input, Some(result))?;
        self.line(format!("  br label %mal_state_{}", frame.resume.0));
        Some(())
    }

    fn load_binding(&mut self, id: ValueId) -> Option<String> {
        let slot = self.slots.get(&id).copied()?;
        let register = self.register();
        self.line(format!(
            "  {register} = load i32, ptr %mal_slot_{slot}, align 4"
        ));
        Some(register)
    }
}
