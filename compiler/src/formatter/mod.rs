use crate::ast::Program;
use crate::diagnostic::Diagnostic;
use crate::lexer::{Lexed, LexemeKind, TokenKind};
use crate::source::SourceFile;

mod control;
mod layout;
mod token;

use self::control::ControlLayout;
use self::layout::{BlockLayout, top_level_breaks};
use self::token::{BracketLayout, IfStage, Previous};

pub fn format(source: &SourceFile) -> Result<String, Diagnostic> {
    let lexed = crate::lexer::lex_lossless(source)?;
    let program = crate::parser::parse_tokens(source, &lexed.tokens)?;
    Ok(Formatter::new(source, &lexed, &program).finish())
}

struct Formatter<'a> {
    source: &'a SourceFile,
    lexed: &'a Lexed,
    token_index: usize,
    output: String,
    indent: usize,
    line_indent: usize,
    line_start: bool,
    source_break: bool,
    source_blank_line: bool,
    pending_newline: bool,
    source_line_indent: Option<usize>,
    previous: Previous,
    ifs: Vec<IfStage>,
    parenthesis_indents: Vec<usize>,
    brackets: Vec<BracketLayout>,
    brace_depth: usize,
    binding_continuations: Vec<usize>,
    controls: ControlLayout,
    blocks: BlockLayout,
    top_level_breaks: Vec<usize>,
    next_top_level_break: usize,
}

impl<'a> Formatter<'a> {
    fn new(source: &'a SourceFile, lexed: &'a Lexed, program: &Program) -> Self {
        Self {
            source,
            lexed,
            token_index: 0,
            output: String::new(),
            indent: 0,
            line_indent: 0,
            line_start: true,
            source_break: false,
            source_blank_line: false,
            pending_newline: false,
            source_line_indent: None,
            previous: Previous::None,
            ifs: Vec::new(),
            parenthesis_indents: Vec::new(),
            brackets: Vec::new(),
            brace_depth: 0,
            binding_continuations: Vec::new(),
            controls: ControlLayout::new(lexed, program),
            blocks: BlockLayout::new(source, lexed),
            top_level_breaks: top_level_breaks(source, lexed, program),
            next_top_level_break: 0,
        }
    }

    fn finish(mut self) -> String {
        for lexeme in &self.lexed.lexemes {
            self.write_top_level_break(lexeme.span.start());
            let text = &self.source.text()[lexeme.span.start()..lexeme.span.end()];
            match lexeme.kind {
                LexemeKind::Whitespace => {
                    let line_breaks = line_break_count(text);
                    self.source_break |= line_breaks > 0;
                    self.source_blank_line |= line_breaks > 1;
                }
                LexemeKind::LineComment => self.write_comment(text),
                LexemeKind::Token => {
                    let token_index = self.token_index;
                    let kind = &self.lexed.tokens[self.token_index].kind;
                    self.token_index += 1;
                    self.write_token(token_index, kind, text);
                    self.source_break = false;
                    self.source_blank_line = false;
                }
            }
        }
        while self.output.ends_with(char::is_whitespace) {
            self.output.pop();
        }
        self.output.push('\n');
        self.output
    }

    fn write_top_level_break(&mut self, offset: usize) {
        if self
            .top_level_breaks
            .get(self.next_top_level_break)
            .is_some_and(|anchor| offset >= *anchor)
        {
            self.pending_newline = false;
            self.newline();
            if !self.output.is_empty() && !self.output.ends_with("\n\n") {
                self.output.push('\n');
            }
            self.next_top_level_break += 1;
        }
    }

    fn write_comment(&mut self, text: &str) {
        let continuation_indent = self
            .token_index
            .checked_sub(1)
            .and_then(|index| self.lexed.tokens.get(index))
            .is_some_and(|token| matches!(token.kind, TokenKind::Arrow | TokenKind::FatArrow))
            .then_some(self.indent + 1);
        if self
            .blocks
            .terminate
            .get(self.token_index)
            .is_some_and(|terminate| *terminate)
        {
            self.trim_space();
            self.write(";");
            self.blocks.terminate[self.token_index] = false;
        }
        if self.should_preserve_blank_line() {
            self.blank_line();
        } else if self.source_break {
            self.newline();
            self.source_line_indent = continuation_indent;
        } else if !self.line_start {
            self.space();
        }
        self.write(text);
        self.newline();
        self.source_line_indent = continuation_indent;
        self.source_break = false;
        self.source_blank_line = false;
        self.pending_newline = false;
    }

    fn write(&mut self, text: &str) {
        if self.line_start {
            let indent = self.source_line_indent.take().unwrap_or(self.indent);
            self.output.push_str(&" ".repeat(indent * 4));
            self.line_indent = indent;
            self.line_start = false;
        }
        self.output.push_str(text);
    }

    fn space(&mut self) {
        if !self.line_start && !self.output.ends_with(char::is_whitespace) {
            self.output.push(' ');
        }
    }

    fn trim_space(&mut self) {
        while self.output.ends_with(' ') {
            self.output.pop();
        }
    }

    fn newline(&mut self) {
        self.trim_space();
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.line_start = true;
    }

    fn blank_line(&mut self) {
        self.newline();
        if !self.output.is_empty() && !self.output.ends_with("\n\n") {
            self.output.push('\n');
        }
    }

    fn should_preserve_blank_line(&self) -> bool {
        self.source_blank_line
            && matches!(self.previous, Previous::Semicolon | Previous::RightBrace)
    }
}

fn line_break_count(text: &str) -> usize {
    let bytes = text.as_bytes();
    let mut count = 0;
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\r' => {
                count += 1;
                index += usize::from(bytes.get(index + 1) == Some(&b'\n'));
            }
            b'\n' => count += 1,
            _ => {}
        }
        index += 1;
    }
    count
}
