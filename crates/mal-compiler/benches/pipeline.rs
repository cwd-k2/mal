use std::fmt::Write;
use std::hint::black_box;
use std::time::{Duration, Instant};

use mal_syntax::source::{FileId, SourceFile};

const SAMPLE_COUNT: usize = 7;
const MINIMUM_SAMPLE_TIME: Duration = Duration::from_millis(100);

fn main() {
    let source = SourceFile::new(FileId::new(900), "benchmark.mal", large_source());
    let tokens = mal_syntax::lexer::lex(&source).expect("benchmark source must lex");
    let parsed = mal_syntax::parser::parse(&source).expect("benchmark source must parse");
    let resolved = mal_frontend::resolve::resolve(&parsed).expect("benchmark source must resolve");
    let specialized = mal_frontend::check::specialize(
        mal_frontend::check::check(&resolved)
            .expect("benchmark source must check for specialization"),
    )
    .expect("benchmark source must specialize");
    let core = mal_backend::core::lower(&specialized);
    let anf = mal_backend::anf::lower(&core);
    mal_backend::closure::convert(&anf);
    mal_frontend::editor::analyze(&source).expect("benchmark source must support editor analysis");

    println!("source_bytes={}", source.text().len());
    measure("lex", || mal_syntax::lexer::lex(black_box(&source)));
    measure("parse_tokens", || {
        mal_syntax::parser::parse_tokens(black_box(&source), black_box(&tokens))
    });
    measure("parse", || mal_syntax::parser::parse(black_box(&source)));
    measure("resolve", || {
        mal_frontend::resolve::resolve(black_box(&parsed))
    });
    measure("check", || mal_frontend::check::check(black_box(&resolved)));
    measure("check_specialize", || {
        mal_frontend::check::check(black_box(&resolved)).and_then(mal_frontend::check::specialize)
    });
    measure("core", || mal_backend::core::lower(black_box(&specialized)));
    measure("anf", || mal_backend::anf::lower(black_box(&core)));
    measure("closure", || mal_backend::closure::convert(black_box(&anf)));
    measure("pipeline_check", || {
        mal_frontend::analysis::check(black_box(&source))
    });
    measure("editor_analyze", || {
        mal_frontend::editor::analyze(black_box(&source))
    });
}

fn measure<T>(name: &str, mut operation: impl FnMut() -> T) {
    drop(black_box(operation()));
    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let start = Instant::now();
        let mut iterations = 0_u64;
        while start.elapsed() < MINIMUM_SAMPLE_TIME {
            drop(black_box(operation()));
            iterations += 1;
        }
        samples.push(start.elapsed().as_nanos() / u128::from(iterations));
    }
    samples.sort_unstable();
    println!(
        "{name:>14}: {median:>10} ns/iter ({minimum}..{maximum})",
        median = samples[SAMPLE_COUNT / 2],
        minimum = samples[0],
        maximum = samples[SAMPLE_COUNT - 1],
    );
}

fn large_source() -> String {
    let mut source = String::new();
    writeln!(source, "Alias0 :: Int32;").unwrap();
    for index in 1..250 {
        writeln!(source, "Alias{index} :: Alias{};", index - 1).unwrap();
    }
    for index in 0..500 {
        writeln!(source, "constant{index} :: Alias249 := {index};").unwrap();
    }
    for index in 0..250 {
        writeln!(
            source,
            "function{index} :: Int32 -> Int32 := (value) -> {{ local := value + constant{index}; local; }};"
        )
        .unwrap();
    }
    writeln!(
        source,
        "main :: Unit -> Int32 := () -> {{ function249(constant499); }};"
    )
    .unwrap();
    source
}
