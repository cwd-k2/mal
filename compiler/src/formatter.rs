use crate::ast::Program;
use crate::diagnostic::Diagnostic;
use crate::lexer::{Lexed, LexemeKind, TokenKind};
use crate::source::SourceFile;

mod layout;

use self::layout::{BlockLayout, top_level_breaks};

pub fn format(source: &SourceFile) -> Result<String, Diagnostic> {
    let lexed = crate::lexer::lex_lossless(source)?;
    let program = crate::parser::parse_tokens(source, &lexed.tokens)?;
    Ok(Formatter::new(source, &lexed, &program).finish())
}

#[derive(Clone, Copy)]
enum Previous {
    None,
    Word,
    Keyword,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,
    Operator,
    Unary,
    Backslash,
    CaptureClose,
}

enum IfStage {
    Condition,
    ThenKeyword,
    ThenBranch(usize),
    AwaitElse,
    ElseKeyword,
    ElseBranch(usize),
    Finished,
}

enum CaseStage {
    Keyword,
    Scrutinee(usize),
    Arms(usize),
    AwaitArmOrEnd(usize),
}

impl Previous {
    fn ends_expression(self) -> bool {
        matches!(
            self,
            Self::Word | Self::RightParen | Self::RightBracket | Self::RightBrace
        )
    }
}

struct Formatter<'a> {
    source: &'a SourceFile,
    lexed: &'a Lexed,
    token_index: usize,
    output: String,
    indent: usize,
    line_start: bool,
    source_break: bool,
    pending_newline: bool,
    previous: Previous,
    in_capture: bool,
    ifs: Vec<IfStage>,
    paren_depth: usize,
    cases: Vec<CaseStage>,
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
            line_start: true,
            source_break: false,
            pending_newline: false,
            previous: Previous::None,
            in_capture: false,
            ifs: Vec::new(),
            paren_depth: 0,
            cases: Vec::new(),
            blocks: BlockLayout::new(lexed),
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
                    self.source_break |= text.bytes().any(|byte| matches!(byte, b'\r' | b'\n'));
                }
                LexemeKind::LineComment => self.write_comment(text),
                LexemeKind::Token => {
                    let token_index = self.token_index;
                    let kind = &self.lexed.tokens[self.token_index].kind;
                    self.token_index += 1;
                    self.write_token(token_index, kind, text);
                    self.source_break = false;
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
        if self.source_break {
            self.newline();
        } else if !self.line_start {
            self.space();
        }
        self.write(text);
        self.newline();
        self.source_break = false;
        self.pending_newline = false;
    }

    fn write_token(&mut self, token_index: usize, kind: &TokenKind, text: &str) {
        if self.blocks.omit[token_index] {
            return;
        }
        if let Some(CaseStage::AwaitArmOrEnd(arms_indent)) = self.cases.last() {
            if matches!(kind, TokenKind::LeftBracket) {
                let arms_indent = *arms_indent;
                *self.cases.last_mut().expect("matched case") = CaseStage::Arms(arms_indent);
            } else {
                self.cases.pop();
                self.indent = self.indent.saturating_sub(1);
            }
        }
        if matches!(self.ifs.last(), Some(IfStage::Finished)) {
            self.ifs.pop();
            self.indent = self.indent.saturating_sub(1);
        }
        if self.pending_newline {
            self.newline();
            self.pending_newline = false;
        }
        match kind {
            TokenKind::LeftBrace => {
                self.space();
                self.write(text);
                if self.blocks.compact[token_index] {
                    if let Some(stage) = self.ifs.last_mut() {
                        match stage {
                            IfStage::ThenKeyword => *stage = IfStage::ThenBranch(self.indent),
                            IfStage::ElseKeyword => *stage = IfStage::ElseBranch(self.indent),
                            _ => {}
                        }
                    }
                    self.space();
                    self.previous = Previous::LeftBrace;
                    return;
                }
                self.indent += 1;
                if let Some(stage) = self.ifs.last_mut() {
                    match stage {
                        IfStage::ThenKeyword => *stage = IfStage::ThenBranch(self.indent),
                        IfStage::ElseKeyword => *stage = IfStage::ElseBranch(self.indent),
                        _ => {}
                    }
                }
                self.newline();
                self.previous = Previous::LeftBrace;
            }
            TokenKind::RightBrace => {
                if self.blocks.compact[token_index] {
                    self.space();
                    self.write(text);
                    if let Some(stage) = self.ifs.last_mut() {
                        match stage {
                            IfStage::ThenBranch(branch_indent) if *branch_indent == self.indent => {
                                *stage = IfStage::AwaitElse;
                            }
                            IfStage::ElseBranch(branch_indent) if *branch_indent == self.indent => {
                                *stage = IfStage::Finished;
                            }
                            _ => {}
                        }
                    }
                    if let Some(CaseStage::Arms(arms_indent)) = self.cases.last()
                        && self.indent == *arms_indent
                    {
                        let arms_indent = *arms_indent;
                        *self.cases.last_mut().expect("matched case arms") =
                            CaseStage::AwaitArmOrEnd(arms_indent);
                    }
                    self.previous = Previous::RightBrace;
                    return;
                }
                if self.blocks.terminate[token_index] {
                    self.trim_space();
                    self.write(";");
                }
                let closing_indent = self.indent;
                self.indent = self.indent.saturating_sub(1);
                self.newline();
                self.write(text);
                if let Some(stage) = self.ifs.last_mut() {
                    match stage {
                        IfStage::ThenBranch(branch_indent) if *branch_indent == closing_indent => {
                            *stage = IfStage::AwaitElse;
                        }
                        IfStage::ElseBranch(branch_indent) if *branch_indent == closing_indent => {
                            *stage = IfStage::Finished;
                        }
                        _ => {}
                    }
                }
                if let Some(CaseStage::Arms(arms_indent)) = self.cases.last()
                    && self.indent == *arms_indent
                {
                    let arms_indent = *arms_indent;
                    *self.cases.last_mut().expect("matched case arms") =
                        CaseStage::AwaitArmOrEnd(arms_indent);
                }
                self.previous = Previous::RightBrace;
            }
            TokenKind::Semicolon => {
                self.trim_space();
                self.write(text);
                self.pending_newline = true;
                self.previous = Previous::Semicolon;
            }
            TokenKind::Comma => {
                self.trim_space();
                self.write(text);
                self.space();
                self.previous = Previous::Comma;
            }
            TokenKind::LeftParen => {
                if matches!(self.previous, Previous::Keyword) {
                    self.space();
                }
                self.write(text);
                self.paren_depth += 1;
                if matches!(self.cases.last(), Some(CaseStage::Keyword)) {
                    *self.cases.last_mut().expect("matched case keyword") =
                        CaseStage::Scrutinee(self.paren_depth);
                }
                self.previous = Previous::LeftParen;
            }
            TokenKind::RightParen => {
                self.trim_space();
                self.write(text);
                if matches!(
                    self.cases.last(),
                    Some(CaseStage::Scrutinee(depth)) if *depth == self.paren_depth
                ) {
                    self.indent += 1;
                    *self.cases.last_mut().expect("matched case scrutinee") =
                        CaseStage::Arms(self.indent);
                }
                self.paren_depth = self.paren_depth.saturating_sub(1);
                self.previous = Previous::RightParen;
            }
            TokenKind::LeftBracket => {
                if matches!(self.previous, Previous::RightParen | Previous::RightBrace) {
                    self.newline();
                }
                self.write(text);
                self.previous = Previous::LeftBracket;
            }
            TokenKind::RightBracket => {
                self.trim_space();
                self.write(text);
                self.previous = Previous::RightBracket;
            }
            TokenKind::Backslash => {
                if self.previous.ends_expression() {
                    self.space();
                }
                self.write(text);
                self.previous = Previous::Backslash;
            }
            TokenKind::Less if matches!(self.previous, Previous::Backslash) => {
                self.write(text);
                self.in_capture = true;
                self.previous = Previous::LeftBracket;
            }
            TokenKind::Greater if self.in_capture => {
                self.trim_space();
                self.write(text);
                self.in_capture = false;
                self.previous = Previous::CaptureClose;
            }
            TokenKind::Minus if !self.previous.ends_expression() => {
                if matches!(self.previous, Previous::Keyword) {
                    self.space();
                }
                self.write(text);
                self.previous = Previous::Unary;
            }
            TokenKind::Bang | TokenKind::Tilde => {
                if self.previous.ends_expression() {
                    self.space();
                }
                self.write(text);
                self.previous = Previous::Unary;
            }
            TokenKind::At => {
                self.write(text);
                self.previous = Previous::Unary;
            }
            TokenKind::DoubleColon
            | TokenKind::Bind
            | TokenKind::Arrow
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::BangEqual
            | TokenKind::EqualEqual
            | TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::Ampersand
            | TokenKind::AmpersandAmpersand
            | TokenKind::Pipe
            | TokenKind::PipePipe
            | TokenKind::Caret
            | TokenKind::ShiftLeft
            | TokenKind::ShiftRight => {
                self.space();
                self.write(text);
                self.space();
                self.previous = Previous::Operator;
            }
            TokenKind::Then if matches!(self.ifs.last(), Some(IfStage::Condition)) => {
                self.newline();
                self.indent += 1;
                self.write(text);
                *self.ifs.last_mut().expect("matched condition") = IfStage::ThenKeyword;
                self.previous = Previous::Keyword;
            }
            TokenKind::Else if matches!(self.ifs.last(), Some(IfStage::AwaitElse)) => {
                self.newline();
                self.write(text);
                *self.ifs.last_mut().expect("matched then branch") = IfStage::ElseKeyword;
                self.previous = Previous::Keyword;
            }
            TokenKind::If => {
                if matches!(
                    self.previous,
                    Previous::Word
                        | Previous::Keyword
                        | Previous::RightParen
                        | Previous::RightBracket
                        | Previous::RightBrace
                ) {
                    self.space();
                }
                self.write(text);
                self.ifs.push(IfStage::Condition);
                self.previous = Previous::Keyword;
            }
            TokenKind::Case => {
                if matches!(
                    self.previous,
                    Previous::Word
                        | Previous::Keyword
                        | Previous::RightParen
                        | Previous::RightBracket
                        | Previous::RightBrace
                ) {
                    self.space();
                }
                self.write(text);
                self.cases.push(CaseStage::Keyword);
                self.previous = Previous::Keyword;
            }
            TokenKind::Extern | TokenKind::Then | TokenKind::Else => {
                if matches!(
                    self.previous,
                    Previous::Word
                        | Previous::Keyword
                        | Previous::RightParen
                        | Previous::RightBracket
                        | Previous::RightBrace
                ) {
                    self.space();
                }
                self.write(text);
                self.previous = Previous::Keyword;
            }
            TokenKind::TypeIdentifier
            | TokenKind::ValueIdentifier
            | TokenKind::Integer(_)
            | TokenKind::Float(_)
            | TokenKind::Byte(_)
            | TokenKind::String(_)
            | TokenKind::Underscore => {
                if matches!(
                    self.previous,
                    Previous::Word
                        | Previous::Keyword
                        | Previous::RightParen
                        | Previous::RightBracket
                        | Previous::RightBrace
                ) {
                    self.space();
                }
                self.write(text);
                self.previous = Previous::Word;
            }
            TokenKind::Eof => unreachable!("EOF has no lossless lexeme"),
        }
    }

    fn write(&mut self, text: &str) {
        if self.line_start {
            self.output.push_str(&" ".repeat(self.indent * 4));
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
}
