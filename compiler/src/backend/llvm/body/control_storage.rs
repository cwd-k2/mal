use super::FunctionEmitter;

pub(super) struct ControlReservation {
    pub(super) top: String,
    pub(super) next_top: String,
    pub(super) storage: String,
}

impl FunctionEmitter<'_> {
    pub(super) fn refresh_control_storage(&mut self) -> Option<()> {
        if !self.local_control_storage {
            return Some(());
        }
        let storage = self.register();
        self.line(format!(
            "  {storage} = call ptr @mal_control_storage(ptr %mal_context)"
        ));
        self.line(format!(
            "  store ptr {storage}, ptr %mal_local_control_storage, align {}",
            self.types.pointer_alignment()
        ));
        let index_type = self.types.pointer_integer()?;
        let capacity = self.register();
        self.line(format!(
            "  {capacity} = call {index_type} @mal_control_capacity(ptr %mal_context)"
        ));
        self.line(format!(
            "  store {index_type} {capacity}, ptr %mal_local_control_capacity, align {}",
            self.types.index_alignment()
        ));
        Some(())
    }

    pub(super) fn current_control_storage(&mut self) -> Option<String> {
        let storage = self.register();
        if self.local_control_storage {
            self.line(format!(
                "  {storage} = load ptr, ptr %mal_local_control_storage, align {}",
                self.types.pointer_alignment()
            ));
        } else {
            self.line(format!(
                "  {storage} = call ptr @mal_control_storage(ptr %mal_context)"
            ));
        }
        Some(storage)
    }

    pub(super) fn reserve_control_frame(
        &mut self,
        frame_size: usize,
        replacement: bool,
    ) -> Option<ControlReservation> {
        let index_type = self.types.pointer_integer()?;
        let top = self.register();
        self.line(format!(
            "  {top} = load {index_type}, ptr {}, align {}",
            self.control_top_pointer(),
            self.types.index_alignment()
        ));
        let next_top = self.register();
        self.line(format!(
            "  {next_top} = add {index_type} {top}, {frame_size}"
        ));
        if replacement {
            let storage = self.current_control_storage()?;
            return Some(ControlReservation {
                top,
                next_top,
                storage,
            });
        }
        if !self.local_control_storage {
            let storage = self.register();
            self.line(format!(
                "  {storage} = call ptr @mal_control_reserve_frame(ptr %mal_context, {index_type} {top}, {index_type} {frame_size})"
            ));
            return Some(ControlReservation {
                top,
                next_top,
                storage,
            });
        }

        let cached_storage = self.current_control_storage()?;
        let capacity = self.register();
        self.line(format!(
            "  {capacity} = load {index_type}, ptr %mal_local_control_capacity, align {}",
            self.types.index_alignment()
        ));
        let no_overflow = self.register();
        let frame_size = u64::try_from(frame_size).ok()?;
        let maximum_top = match self.types.index_size() {
            4 => u64::from(u32::MAX).checked_sub(frame_size)?,
            8 => u64::MAX.checked_sub(frame_size)?,
            _ => return None,
        };
        self.line(format!(
            "  {no_overflow} = icmp ule {index_type} {top}, {maximum_top}"
        ));
        let within_capacity = self.register();
        self.line(format!(
            "  {within_capacity} = icmp ule {index_type} {next_top}, {capacity}"
        ));
        let fast = self.register();
        self.line(format!(
            "  {fast} = and i1 {no_overflow}, {within_capacity}"
        ));
        let label = self.label_id();
        self.line(format!(
            "  br i1 {fast}, label %mal_control_fast_{label}, label %mal_control_slow_{label}"
        ));
        self.line(format!("mal_control_fast_{label}:"));
        self.line(format!("  br label %mal_control_ready_{label}"));
        self.line(format!("mal_control_slow_{label}:"));
        let grown = self.register();
        self.line(format!(
            "  {grown} = call ptr @mal_control_reserve_frame(ptr %mal_context, {index_type} {top}, {index_type} {frame_size})"
        ));
        self.line(format!(
            "  store ptr {grown}, ptr %mal_local_control_storage, align {}",
            self.types.pointer_alignment()
        ));
        let grown_capacity = self.register();
        self.line(format!(
            "  {grown_capacity} = call {index_type} @mal_control_capacity(ptr %mal_context)"
        ));
        self.line(format!(
            "  store {index_type} {grown_capacity}, ptr %mal_local_control_capacity, align {}",
            self.types.index_alignment()
        ));
        self.line(format!("  br label %mal_control_ready_{label}"));
        self.line(format!("mal_control_ready_{label}:"));
        let storage = self.register();
        self.line(format!(
            "  {storage} = phi ptr [{cached_storage}, %mal_control_fast_{label}], [{grown}, %mal_control_slow_{label}]"
        ));
        Some(ControlReservation {
            top,
            next_top,
            storage,
        })
    }
}
