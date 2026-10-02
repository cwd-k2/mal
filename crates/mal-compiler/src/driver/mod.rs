//! Filesystem and toolchain boundary for compiler use cases.

use std::path::Path;

use mal_syntax::graph;

mod build;
mod error;
mod files;
mod toolchain;

pub use build::{OptimizationMode, ToolchainOptions, build, emit_atcoder};
pub use error::Error;
pub use files::write_output;
use files::{TemporaryDirectory, create_parent};

/// Loads the program rooted at `source_path`, type-checks it, and, when the root declares `main`, specializes it so that
/// every source error a build would report is reported here. Nothing is written.
pub fn check(source_path: &Path) -> Result<(), Error> {
    let graph = graph::load(source_path)?;
    let checked = mal_frontend::analysis::check_graph(&graph)
        .map_err(|error| Error::diagnostic(error, &graph))?;
    match mal_frontend::analysis::specialization_error(&checked) {
        Some(error) => Err(Error::diagnostic(error, &graph)),
        None => Ok(()),
    }
}

/// Returns the C header for the program rooted at `source_path`, which needs no `main`.
pub fn emit_header(source_path: &Path) -> Result<String, Error> {
    let graph = graph::load(source_path)?;
    let target = toolchain::host_target()?;
    mal_backend::pipeline::emit_header(
        &graph,
        mal_backend::pipeline::Target {
            triple: &target.triple,
            data_layout: &target.data_layout,
        },
    )
    .map_err(|error| Error::backend(error, &graph))
}

/// Returns a host implementation template that includes `header_name`.
pub fn emit_host(source_path: &Path, header_name: &str) -> Result<String, Error> {
    let graph = graph::load(source_path)?;
    mal_backend::pipeline::emit_host(&graph, header_name)
        .map_err(|error| Error::diagnostic(error, &graph))
}
