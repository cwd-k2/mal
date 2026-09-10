use super::abi::Function as AbiFunction;
use super::artifact::LlvmArtifacts;

mod body;

pub(crate) struct Target<'a> {
    pub(crate) triple: &'a str,
    pub(crate) data_layout: &'a str,
}

pub(crate) fn supports(program: &crate::execution::Program) -> bool {
    body::supports(program)
}

pub(crate) fn generate(
    program: &crate::execution::Program,
    target: Target<'_>,
) -> Option<LlvmArtifacts> {
    let body = body::generate(program, pointer_size(target.data_layout)?)?;
    let entry = AbiFunction::program_entry();
    let external_bridges = program
        .lowered
        .interface
        .externals
        .iter()
        .map(external_bridge)
        .collect::<Option<Vec<_>>>()?;
    let external_declarations = external_bridges
        .iter()
        .map(|bridge| bridge.0.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let external_definitions = external_bridges
        .iter()
        .map(|bridge| bridge.1.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    let control_declarations = if body.uses_control {
        "declare ptr @mal_control_reserve_frame(ptr, i64, i64)\ndeclare ptr @mal_control_storage(ptr)\n\n"
    } else {
        ""
    };
    let module = format!(
        "target datalayout = \"{}\"\ntarget triple = \"{}\"\n\n{}{}\n\n{}define {} {{\nentry:\n  %mal_entry_result = call i32 @{}(ptr %mal_context)\n  store i32 %mal_entry_result, ptr %mal_result, align 4\n  ret void\n}}\n",
        target.data_layout,
        target.triple,
        control_declarations,
        external_declarations,
        body.definitions,
        entry.llvm_signature(),
        match body.main {
            crate::closure::ast::FunctionId::Lambda(id) => format!("mal_function_{}", id.0),
            crate::closure::ast::FunctionId::Memory(_) => return None,
        },
    );
    let shim = format!(
        "#include \"program.mal.h\"\n#include \"runtime.h\"\n\n{}\n\n{}\n\nint main(void) {{\n    MalContext context = {{0}};\n    int32_t result;\n    {}(&context, NULL, &result);\n    mal_control_destroy(&context);\n    return result;\n}}\n",
        entry.c_declaration(),
        external_definitions,
        entry.name(),
    );
    Some(LlvmArtifacts {
        module,
        shim,
        header: crate::backend::c::emit_header(&program.lowered.interface),
        runtime: crate::backend::runtime::control().into(),
    })
}

fn pointer_size(data_layout: &str) -> Option<usize> {
    let bits: usize = data_layout
        .split('-')
        .find_map(|component| {
            component
                .strip_prefix("p:")
                .or_else(|| component.strip_prefix("p0:"))
        })
        .map_or(Some(64), |pointer| pointer.split(':').next()?.parse().ok())?;
    bits.is_multiple_of(8)
        .then_some(bits / 8)
        .filter(|bytes| (*bytes).is_power_of_two())
}

fn external_bridge(external: &crate::core::ast::ExternalOperation) -> Option<(String, String)> {
    let parameter = c_scalar_type(&external.parameter)?;
    let result = c_scalar_type(&external.result)?;
    let bridge = AbiFunction::external_bridge(external.id);
    let llvm = format!("declare {}", bridge.llvm_signature());
    let signature = bridge.c_declaration();
    let signature = signature.strip_suffix(';')?;
    let c = format!(
        "{signature} {{\n    *({result} *)mal_result = mal_ext_{}(\n        (MalContext *)mal_context,\n        *(const {parameter} *)mal_argument\n    );\n}}",
        external.name
    );
    Some((llvm, c))
}

fn c_scalar_type(ty: &crate::check::ast::Type) -> Option<&'static str> {
    use crate::check::ast::Type;

    match ty {
        Type::Int8 => Some("MalType_Int8"),
        Type::Int16 => Some("MalType_Int16"),
        Type::Int32 => Some("MalType_Int32"),
        Type::Int64 => Some("MalType_Int64"),
        Type::UInt8 => Some("MalType_UInt8"),
        Type::UInt16 => Some("MalType_UInt16"),
        Type::UInt32 => Some("MalType_UInt32"),
        Type::UInt64 => Some("MalType_UInt64"),
        Type::Float32 => Some("MalType_Float32"),
        Type::Float64 => Some("MalType_Float64"),
        Type::Ptr => Some("MalType_Ptr"),
        _ => None,
    }
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

    #[test]
    fn reads_the_default_pointer_layout_and_an_explicit_address_space_zero_layout() {
        assert_eq!(pointer_size("e-m:e-i64:64"), Some(8));
        assert_eq!(pointer_size("e-p:32:32-i64:64"), Some(4));
        assert_eq!(pointer_size("e-p0:128:128"), Some(16));
        assert_eq!(pointer_size("e-p:7:8"), None);
    }
}
