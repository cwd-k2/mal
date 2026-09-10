use crate::check::ast::Type;
use crate::closure::ast::{AtomKind, FunctionId, Operation, Pattern, Reference, TopLevelPattern};

use super::abi::Function as AbiFunction;
use super::artifact::LlvmArtifacts;

pub(crate) struct Target<'a> {
    pub(crate) triple: &'a str,
    pub(crate) data_layout: &'a str,
}

pub(crate) fn supports(program: &crate::execution::Program) -> bool {
    constant_main(program).is_some() && program.lowered.interface.externals.is_empty()
}

pub(crate) fn generate(
    program: &crate::execution::Program,
    target: Target<'_>,
) -> Option<LlvmArtifacts> {
    let value = constant_main(program)?;
    let entry = AbiFunction::program_entry();
    let module = format!(
        "target datalayout = \"{}\"\ntarget triple = \"{}\"\n\ndefine {} {{\nentry:\n  store i32 {}, ptr %mal_result, align 4\n  ret void\n}}\n",
        target.data_layout,
        target.triple,
        entry.llvm_signature(),
        value as i32,
    );
    let shim = format!(
        "#include <stddef.h>\n#include <stdint.h>\n\n{}\n\nint main(void) {{\n    int32_t result;\n    {}(NULL, NULL, &result);\n    return result;\n}}\n",
        entry.c_declaration(),
        entry.name(),
    );
    Some(LlvmArtifacts {
        module,
        shim,
        header: crate::backend::c::emit_header(&program.lowered.interface),
    })
}

fn constant_main(program: &crate::execution::Program) -> Option<i128> {
    let binding = program.lowered.bindings.iter().find(|binding| {
        matches!(&binding.pattern, TopLevelPattern::Binding { name, .. } if name == "main")
    })?;
    let TopLevelPattern::Binding { ty, .. } = &binding.pattern else {
        return None;
    };
    if *ty
        != (Type::Function {
            parameter: Box::new(Type::Unit),
            result: Box::new(Type::Int32),
        })
    {
        return None;
    }
    let AtomKind::Reference(Reference::Binding(result)) = binding.value.result.kind else {
        return None;
    };
    let function = binding.value.bindings.iter().find_map(|binding| {
        let Pattern::Binding { id, .. } = binding.pattern else {
            return None;
        };
        match &binding.operation {
            Operation::MakeClosure { function, captures }
                if id == result && captures.is_empty() =>
            {
                Some(*function)
            }
            _ => None,
        }
    })?;
    let FunctionId::Lambda(_) = function else {
        return None;
    };
    let function = program
        .lowered
        .functions
        .iter()
        .find(|candidate| candidate.id == function)?;
    if !function.environment.is_empty() || function.parameter.ty != Type::Unit {
        return None;
    }
    match function.body.result.kind {
        AtomKind::Integer(value) if function.body.bindings.is_empty() => Some(value),
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
        assert!(artifacts.module.contains("store i32 7, ptr %mal_result"));
        assert!(artifacts.shim.contains(
            "void mal_program_entry(void *mal_context, const void *mal_argument, void *mal_result);"
        ));
    }
}
