use std::fmt::{self, Write};

pub(super) trait RenderWrite: Write {
    fn push_str(&mut self, value: &str) {
        self.write_str(value)
            .expect("writing generated C cannot fail");
    }

    fn push(&mut self, value: char) {
        self.write_char(value)
            .expect("writing generated C cannot fail");
    }
}

impl<T: Write + ?Sized> RenderWrite for T {}

pub(super) struct MacroReplacementWriter {
    output: String,
    pending_newline: bool,
}

impl MacroReplacementWriter {
    pub(super) fn new(prefix: String) -> Self {
        Self {
            output: prefix,
            pending_newline: false,
        }
    }

    pub(super) fn finish(mut self) -> String {
        if self.pending_newline {
            self.output.push('\n');
        }
        self.output
    }

    fn prepare_line(&mut self) {
        if self.pending_newline {
            self.output.push_str(" \\\n");
            self.pending_newline = false;
        }
    }
}

impl fmt::Write for MacroReplacementWriter {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        for fragment in value.split_inclusive('\n') {
            self.prepare_line();
            if let Some(content) = fragment.strip_suffix('\n') {
                self.output.push_str(content);
                self.pending_newline = true;
            } else {
                self.output.push_str(fragment);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use super::MacroReplacementWriter;

    #[test]
    fn continues_only_newlines_followed_by_more_replacement_text() {
        let mut writer = MacroReplacementWriter::new("#define VALUE \\\n".into());
        write!(writer, "first\nsecond\n").unwrap();

        assert_eq!(writer.finish(), "#define VALUE \\\nfirst \\\nsecond\n");
    }
}
