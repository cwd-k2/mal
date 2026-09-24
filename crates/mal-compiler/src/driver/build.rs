use std::ffi::{OsStr, OsString};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use mal_syntax::source::SourceGraph;

use mal_syntax::graph;

use super::{Error, TemporaryDirectory, create_parent, toolchain};

const C_COMPILER_REQUIRED_OPTIONS: &[&str] = &[
    "-std=c11",
    "-Wall",
    "-Wextra",
    "-Werror",
    "-pedantic",
    "-fno-fast-math",
    "-ffp-contract=off",
    "-frounding-math",
    "-fexcess-precision=standard",
];

pub struct BuildOptions<'a> {
    pub output_path: &'a Path,
    pub artifact_directory: Option<&'a Path>,
    pub clang_arguments: &'a [OsString],
    pub optimization: OptimizationMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OptimizationMode {
    Baseline,
    Production,
}

pub fn build(source_path: &Path, options: BuildOptions<'_>) -> Result<(), Error> {
    let generated = generate_build_inputs(
        source_path,
        options.artifact_directory,
        options.optimization,
    )?;
    create_parent(options.output_path)?;
    run_compiler(
        &generated.build_directory,
        &generated.header_path,
        generated.generated_inputs.iter(),
        generated.graph.c_sources(),
        options.clang_arguments,
        toolchain_optimization(options.optimization),
        options.output_path,
    )
}

pub fn emit_atcoder(source_path: &Path, options: BuildOptions<'_>) -> Result<(), Error> {
    let generated = generate_build_inputs(
        source_path,
        options.artifact_directory,
        options.optimization,
    )?;
    create_parent(options.output_path)?;
    let assembly = run_atcoder_assembly_emitter(
        &generated.build_directory,
        &generated.header_path,
        generated.generated_inputs.iter(),
        generated.graph.c_sources(),
        options.clang_arguments,
        toolchain_optimization(options.optimization),
    )?;
    let submission = render_atcoder_submission(&generated.graph, &assembly);
    fs::write(options.output_path, submission)
        .map_err(|error| Error::io("write AtCoder submission", options.output_path, error))
}

struct GeneratedBuild {
    graph: SourceGraph,
    build_directory: PathBuf,
    header_path: PathBuf,
    generated_inputs: Vec<PathBuf>,
    _temporary: Option<TemporaryDirectory>,
}

fn generate_build_inputs(
    source_path: &Path,
    artifact_directory: Option<&Path>,
    optimization: OptimizationMode,
) -> Result<GeneratedBuild, Error> {
    let graph = graph::load(source_path)?;
    let execution_optimizations = match optimization {
        OptimizationMode::Baseline => crate::execution::OptimizationSet::none(),
        OptimizationMode::Production => crate::execution::OptimizationSet::production(),
    };
    let llvm_optimizations = match optimization {
        OptimizationMode::Baseline => crate::backend::llvm::OptimizationSet::none(),
        OptimizationMode::Production => crate::backend::llvm::OptimizationSet::production(),
    };
    let execution = crate::pipeline::lower_graph_execution(&graph, execution_optimizations)
        .map_err(|error| Error::diagnostic(error, &graph))?;
    let temporary = artifact_directory
        .is_none()
        .then(TemporaryDirectory::new)
        .transpose()?;
    let build_directory = match artifact_directory {
        Some(path) => {
            fs::create_dir_all(path)
                .map_err(|error| Error::io("create artifact directory", path, error))?;
            path.to_owned()
        }
        None => temporary
            .as_ref()
            .expect("temporary directory exists when artifacts are not retained")
            .path()
            .to_owned(),
    };
    let target = toolchain::host_target()?;
    let generated = crate::backend::llvm::generate(
        &execution,
        crate::backend::llvm::Target {
            triple: &target.triple,
            data_layout: &target.data_layout,
        },
        llvm_optimizations,
    )
    .map_err(|error| Error::backend(error, &graph))?;
    let module_path = build_directory.join("program.ll");
    let shim_path = build_directory.join("program-shim.c");
    let header_path = build_directory.join(crate::backend::c::GENERATED_HEADER_NAME);
    fs::write(&module_path, generated.module)
        .map_err(|error| Error::io("write generated LLVM module", &module_path, error))?;
    fs::write(&shim_path, generated.shim)
        .map_err(|error| Error::io("write generated C shim", &shim_path, error))?;
    fs::write(&header_path, generated.header)
        .map_err(|error| Error::io("write generated header", &header_path, error))?;
    let mut generated_inputs = vec![module_path, shim_path];
    for runtime in generated.runtime {
        let path = build_directory.join(runtime.name);
        fs::write(&path, runtime.contents)
            .map_err(|error| Error::io("write runtime input", &path, error))?;
        if path.extension() == Some(OsStr::new("c")) {
            generated_inputs.push(path);
        }
    }
    Ok(GeneratedBuild {
        graph,
        build_directory,
        header_path,
        generated_inputs,
        _temporary: temporary,
    })
}

fn toolchain_optimization(mode: OptimizationMode) -> toolchain::OptimizationMode {
    match mode {
        OptimizationMode::Baseline => toolchain::OptimizationMode::Baseline,
        OptimizationMode::Production => toolchain::OptimizationMode::Production,
    }
}

fn run_compiler<'a>(
    include_directory: &Path,
    generated_header: &Path,
    generated_inputs: impl IntoIterator<Item = &'a PathBuf>,
    required_inputs: impl IntoIterator<Item = &'a PathBuf>,
    additional_arguments: &[OsString],
    optimization: toolchain::OptimizationMode,
    output_path: &Path,
) -> Result<(), Error> {
    let compiler = OsStr::new(toolchain::CLANG);
    let result = compiler_command(
        include_directory,
        generated_header,
        generated_inputs,
        required_inputs,
        additional_arguments,
        optimization,
    )
    .arg("-o")
    .arg(output_path)
    .output()
    .map_err(|error| Error::tool_start(compiler, error))?;
    if !result.status.success() {
        return Err(Error::tool_failure(
            compiler,
            result.status.code(),
            &result.stderr,
        ));
    }
    Ok(())
}

fn run_atcoder_assembly_emitter<'a>(
    include_directory: &Path,
    generated_header: &Path,
    generated_inputs: impl IntoIterator<Item = &'a PathBuf>,
    required_inputs: impl IntoIterator<Item = &'a PathBuf>,
    additional_arguments: &[OsString],
    optimization: toolchain::OptimizationMode,
) -> Result<String, Error> {
    let compiler = OsStr::new(toolchain::CLANG);
    let output_base = include_directory.join("program-atcoder");
    let result = compiler_command(
        include_directory,
        generated_header,
        generated_inputs,
        required_inputs,
        additional_arguments,
        optimization,
    )
    .arg("-flto")
    .args(["-fuse-ld=lld", "-Wl,--lto-emit-asm"])
    .arg("-o")
    .arg(&output_base)
    .output()
    .map_err(|error| Error::tool_start(compiler, error))?;
    if !result.status.success() {
        return Err(Error::tool_failure(
            compiler,
            result.status.code(),
            &result.stderr,
        ));
    }
    let assembly_path = output_base.with_extension("lto.s");
    fs::read_to_string(&assembly_path)
        .map_err(|error| Error::io("read generated AtCoder assembly", &assembly_path, error))
}

fn compiler_command<'a>(
    include_directory: &Path,
    generated_header: &Path,
    generated_inputs: impl IntoIterator<Item = &'a PathBuf>,
    required_inputs: impl IntoIterator<Item = &'a PathBuf>,
    additional_arguments: &[OsString],
    optimization: toolchain::OptimizationMode,
) -> Command {
    let mut command = Command::new(toolchain::CLANG);
    command
        .args(C_COMPILER_REQUIRED_OPTIONS)
        .args(optimization.arguments())
        .arg("-I")
        .arg(include_directory)
        .arg("-include")
        .arg(generated_header)
        .args(generated_inputs)
        .args(required_inputs)
        .args(additional_arguments);
    command
}

fn render_atcoder_submission(graph: &SourceGraph, assembly: &str) -> String {
    let mut submission = String::from(
        "// Generated by malc emit-atcoder. The commented mal modules are the source.\n",
    );
    let root_directory = graph.root_source().path().parent().unwrap_or(Path::new(""));
    for source in graph.files() {
        let display_path = source
            .path()
            .strip_prefix(root_directory)
            .ok()
            .filter(|path| !path.as_os_str().is_empty())
            .or_else(|| source.path().file_name().map(Path::new))
            .unwrap_or(source.path());
        let _ = writeln!(
            submission,
            "// ---- mal module: {} ----",
            display_path.display()
        );
        for line in source.text().split('\n') {
            submission.push_str("// ");
            submission.push_str(line.strip_suffix('\r').unwrap_or(line));
            submission.push('\n');
        }
    }
    submission.push_str("// ---- generated x86_64 assembly ----\n");
    let assembly = assembly
        .lines()
        .filter(|line| !line.trim_start().starts_with(".addrsig"))
        .collect::<Vec<_>>()
        .join("\n");
    let delimiter = (0_u32..)
        .map(|index| format!("mal{index}"))
        .find(|delimiter| !assembly.contains(&format!("){delimiter}\"")))
        .expect("an assembly file cannot contain every raw string delimiter");
    let _ = write!(
        submission,
        "asm(R\"{delimiter}(\n{assembly}\n){delimiter}\");\n"
    );
    submission
}

#[cfg(test)]
mod tests {
    use super::C_COMPILER_REQUIRED_OPTIONS;
    #[test]
    fn compiler_required_options_only_own_admission_and_semantics() {
        assert_eq!(
            C_COMPILER_REQUIRED_OPTIONS,
            [
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-pedantic",
                "-fno-fast-math",
                "-ffp-contract=off",
                "-frounding-math",
                "-fexcess-precision=standard"
            ]
        );
    }
}
