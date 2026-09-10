use super::abi::Function as AbiFunction;
use super::artifact::LlvmArtifacts;

mod body;

pub(crate) struct Target<'a> {
    pub(crate) triple: &'a str,
    pub(crate) data_layout: &'a str,
}

pub(crate) fn supports(program: &crate::execution::Program) -> bool {
    body::generate(program).is_some()
}

pub(crate) fn generate(
    program: &crate::execution::Program,
    target: Target<'_>,
) -> Option<LlvmArtifacts> {
    let body = body::generate(program)?;
    let entry = AbiFunction::program_entry();
    let control_declarations = if body.uses_control {
        "declare ptr @mal_control_reserve_frame(ptr, i64, i64)\ndeclare ptr @mal_control_storage(ptr)\n\n"
    } else {
        ""
    };
    let module = format!(
        "target datalayout = \"{}\"\ntarget triple = \"{}\"\n\n{}{}define {} {{\nentry:\n  %mal_entry_result = call i32 @{}(ptr %mal_context)\n  store i32 %mal_entry_result, ptr %mal_result, align 4\n  ret void\n}}\n",
        target.data_layout,
        target.triple,
        control_declarations,
        body.definitions,
        entry.llvm_signature(),
        match body.main {
            crate::closure::ast::FunctionId::Lambda(id) => format!("mal_function_{}", id.0),
            crate::closure::ast::FunctionId::Memory(_) => return None,
        },
    );
    let shim = format!(
        "#include \"runtime.h\"\n\n#include <stdint.h>\n\n{}\n\nint main(void) {{\n    MalControlArena arena = {{0}};\n    int32_t result;\n    {}(&arena, NULL, &result);\n    mal_control_destroy(&arena);\n    return result;\n}}\n",
        entry.c_declaration(),
        entry.name(),
    );
    Some(LlvmArtifacts {
        module,
        shim,
        header: crate::backend::c::emit_header(&program.lowered.interface),
        runtime: crate::backend::runtime::control().into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{FileId, SourceFile};

    #[test]
    fn emits_targeted_llvm_and_a_c_shim_from_one_bridge_plan() {
        let source = SourceFile::new(
            FileId::new(75),
            "llvm-constant.mal",
            "main :: Unit -> Int32 := \\() { 7; };".into(),
        );
        let checked = crate::pipeline::check(&source).expect("check LLVM fixture");
        let core = crate::core::lower(&checked);
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution = crate::execution::lower(closure);
        let artifacts = generate(
            &execution,
            Target {
                triple: "x86_64-unknown-linux-gnu",
                data_layout: "e-p:64:64",
            },
        )
        .expect("constant main is supported");

        assert!(
            artifacts
                .module
                .contains("target triple = \"x86_64-unknown-linux-gnu\"")
        );
        assert!(artifacts.module.contains("ret i32 7"));
        assert!(artifacts.shim.contains(
            "void mal_program_entry(void *mal_context, const void *mal_argument, void *mal_result);"
        ));
    }
}
