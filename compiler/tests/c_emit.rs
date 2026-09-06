use malc::anf;
use malc::c_emit;
use malc::check;
use malc::closure;
use malc::core;
use malc::parser;
use malc::resolve;
use malc::source::{FileId, SourceFile};

mod support;

use support::NativeFixture;

fn emit(text: &str) -> Result<c_emit::Output, malc::diagnostic::Diagnostic> {
    let source = SourceFile::new(FileId::new(79), "c-emit-test.mal", text.into());
    let parsed = parser::parse(&source).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let resolved =
        resolve::resolve(&parsed).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let checked =
        check::check(&resolved).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let core = core::lower(&checked);
    let anf = anf::lower(&core);
    let closure = closure::convert(&anf);
    c_emit::emit(&closure)
}

fn contains_ignoring_whitespace(haystack: &str, needle: &str) -> bool {
    let compact = |text: &str| {
        text.chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>()
    };
    compact(haystack).contains(&compact(needle))
}

fn compile_and_run(source: &str, host: &str) -> std::process::Output {
    let generated = emit(source).expect("emit C");
    let fixture = NativeFixture::new("c-emit");
    let executable = fixture.compile_generated(generated, host);
    fixture.run(executable)
}

const PRINT_HOST: &str = r#"#include "program.mal.h"
#include <stdio.h>

#if MAL_HAS_EXTERN_printInt32 != 1
#error "missing printInt32 capability"
#endif

MAL_DEFINE_printInt32(context, value) {
    printf("%d\n", value);
}
"#;

#[path = "c_emit/calls.rs"]
mod calls;
#[path = "c_emit/host_interface.rs"]
mod host_interface;
#[path = "c_emit/memory.rs"]
mod memory;
#[path = "c_emit/numeric.rs"]
mod numeric;
#[path = "c_emit/semantics.rs"]
mod semantics;
#[path = "c_emit/symbol.rs"]
mod symbol;
