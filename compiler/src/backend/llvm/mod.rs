use super::abi::Function as AbiFunction;
use super::artifact::LlvmArtifacts;

mod body;
mod host_bridge;
mod optimization;
mod shim;

pub(crate) use optimization::OptimizationSet;

pub(crate) struct Target<'a> {
    pub(crate) triple: &'a str,
    pub(crate) data_layout: &'a str,
}

#[cfg(test)]
pub(crate) fn supports(program: &crate::execution::Program) -> bool {
    body::supports(program)
        && program.lowered.interface.externals.iter().all(|external| {
            host_bridge::type_supported(&external.parameter)
                && host_bridge::type_supported(&external.result)
        })
}

pub(crate) fn generate(
    program: &crate::execution::Program,
    target: Target<'_>,
    optimizations: OptimizationSet,
) -> Option<LlvmArtifacts> {
    let pointer_size = pointer_size(target.data_layout)?;
    let body = body::generate(program, pointer_size, optimizations)?;
    let types = body::types::Types::new(pointer_size)?;
    let entry = AbiFunction::program_entry();
    let raw_types = crate::backend::c::RawHostTypes::new(&program.lowered.interface);
    let external_bridges = program
        .lowered
        .interface
        .externals
        .iter()
        .map(|external| host_bridge::generate(external, pointer_size, &raw_types))
        .collect::<Option<Vec<_>>>()?;
    let external_declarations = external_bridges
        .iter()
        .map(|bridge| bridge.llvm_declaration.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let external_definitions = external_bridges
        .iter()
        .map(|bridge| bridge.c_definitions.render())
        .collect::<Vec<_>>()
        .join("\n\n");
    let control_declarations = if body.uses_control {
        format!(
            "declare ptr @mal_control_reserve_frame(ptr, {0}, {0})\ndeclare ptr @mal_control_storage(ptr)\n\n",
            types.pointer_integer()?
        )
    } else {
        String::new()
    };
    let symbol_declarations = if body.uses_symbol_runtime {
        "declare i64 @mal_runtime_symbol_length(ptr)\ndeclare i8 @mal_runtime_symbol_at(ptr, i64)\ndeclare ptr @mal_runtime_symbol_retain(ptr, ptr)\ndeclare void @mal_runtime_symbol_release(ptr)\ndeclare ptr @mal_runtime_symbol_concatenate(ptr, ptr, ptr)\ndeclare ptr @mal_runtime_symbol_concatenate_consuming_left(ptr, ptr, ptr)\ndeclare ptr @mal_runtime_symbol_concatenate_consuming_right(ptr, ptr, ptr)\ndeclare i8 @mal_runtime_symbol_equal(ptr, ptr)\ndeclare ptr @mal_runtime_symbol_read(ptr, ptr, i64)\ndeclare void @mal_runtime_symbol_write(ptr, ptr)\n\n"
    } else {
        ""
    };
    let (control_entry, control_top) = if body.uses_control {
        (
            format!(
                "  %mal_control_top = alloca {0}, align {1}\n  store {0} 0, ptr %mal_control_top, align {1}\n",
                types.pointer_integer()?,
                types.pointer_size()
            ),
            "%mal_control_top",
        )
    } else {
        (String::new(), "null")
    };
    let (entry_argument, entry_call) = match &body.main_parameter {
        crate::check::ast::Type::Unit => (
            String::new(),
            format!(
                "call i32 @{}(ptr %mal_context, ptr {control_top}, ptr null)",
                function_name(body.main)?,
            ),
        ),
        ty => {
            let value = types.value(ty)?;
            (
                format!(
                    "  %mal_entry_argument = load {}, ptr %mal_argument, align {}\n",
                    value.llvm, value.alignment
                ),
                format!(
                    "call i32 @{}(ptr %mal_context, ptr {control_top}, ptr null, {} %mal_entry_argument)",
                    function_name(body.main)?,
                    value.llvm
                ),
            )
        }
    };
    let module = format!(
        "target datalayout = \"{}\"\ntarget triple = \"{}\"\n\ndeclare ptr @mal_runtime_environment_allocate(ptr, {}, ptr)\ndeclare ptr @mal_runtime_environment_retain(ptr, ptr)\ndeclare void @mal_runtime_environment_release(ptr)\n{}{}{}\n{}\n{}define {} {{\nentry:\n{}{}  %mal_entry_result = {}\n  store i32 %mal_entry_result, ptr %mal_result, align 4\n  ret void\n}}\n",
        target.data_layout,
        target.triple,
        types.pointer_integer()?,
        control_declarations,
        symbol_declarations,
        external_declarations,
        body.globals,
        body.definitions,
        entry.llvm_signature(),
        control_entry,
        entry_argument,
        entry_call,
    );
    let symbol_bridge_runtime = if body.uses_symbol_runtime {
        shim::symbol_bridge_runtime()
    } else {
        String::new()
    };
    let main = shim::entry_main(&body.main_parameter, types, entry.name())?.render();
    let entry_declaration =
        crate::backend::c::syntax::Declaration::function(entry.c_signature()).render();
    let shim = format!(
        "#include \"program.mal.h\"\n#include \"runtime.h\"\n\n#include <string.h>\n\n{}\n\n{}\n\n{}\n{}",
        entry_declaration.trim_end(),
        external_definitions,
        symbol_bridge_runtime,
        main,
    );
    Some(LlvmArtifacts {
        module,
        shim,
        header: crate::backend::c::emit_header(&program.lowered.interface),
        runtime: crate::backend::runtime::control().into(),
    })
}

fn function_name(id: crate::closure::ast::FunctionId) -> Option<String> {
    match id {
        crate::closure::ast::FunctionId::Lambda(id) => Some(format!("mal_function_{}", id.0)),
        crate::closure::ast::FunctionId::Memory(_) => None,
    }
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

fn bridge_type_supported(ty: &crate::check::ast::Type) -> bool {
    host_bridge::type_supported(ty)
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
            "main :: Unit -> Int32 := () { 7; };".into(),
        );
        let checked = crate::pipeline::check(&source).expect("check LLVM fixture");
        let core = crate::core::lower(&checked);
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution =
            crate::execution::lower(closure, crate::execution::OptimizationSet::production());
        let artifacts = generate(
            &execution,
            Target {
                triple: "x86_64-unknown-linux-gnu",
                data_layout: "e-p:64:64",
            },
            OptimizationSet::production(),
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
    fn admits_direct_self_handoffs_to_wildcard_parameters() {
        for (index, source) in [
            "extern again :: Unit -> Bool; walk :: Int32 -> Int32 := (_) { if (again()) then { child := walk(1i32); child + 1i32; } else { 0i32 }; }; main :: Unit -> Int32 := () { walk(0i32); };",
            "extern again :: Unit -> Bool; make :: Int32 -> (Unit -> Int32) := (value) { () { value }; }; walk :: (Unit -> Int32) -> Int32 := (_) { if (again()) then { child := walk(make(1i32)); child + 1i32; } else { 0i32 }; }; main :: Unit -> Int32 := () { walk(make(0i32)); };",
            "extern again :: Unit -> Bool; walk :: Int32 -> Int32 := (_) { if (again()) then { walk(1i32) } else { 0i32 }; }; main :: Unit -> Int32 := () { walk(0i32); };",
            "extern again :: Unit -> Bool; make :: Int32 -> (Unit -> Int32) := (value) { () { value }; }; walk :: (Unit -> Int32) -> Int32 := (_) { if (again()) then { walk(make(1i32)) } else { 0i32 }; }; main :: Unit -> Int32 := () { walk(make(0i32)); };",
        ]
        .into_iter()
        .enumerate()
        {
            let source = SourceFile::new(
                FileId::new(80),
                "direct-self-wildcard.mal",
                source.into(),
            );
            let checked = crate::pipeline::check(&source).expect("check wildcard fixture");
            let core = crate::core::lower(&checked);
            let anf = crate::anf::lower(&core);
            let closure = crate::closure::convert(&anf);
            let execution = crate::execution::lower(
                closure,
                crate::execution::OptimizationSet::production(),
            );

            assert!(supports(&execution), "unsupported wildcard fixture {index}");
        }
    }

    #[test]
    fn admits_recursive_control_without_optional_execution_techniques() {
        let source = SourceFile::new(
            FileId::new(86),
            "baseline-recursion.mal",
            "apply :: ((Int32 -> Int32), Int32) -> Int32 := (function, value) { function(value); };\n\
             main :: Unit -> Int32 := () {\n\
               walk :: Int32 -> Int32 := (value) {\n\
                 if (value == 0i32) then { 0i32 } else { apply(walk, value - 1i32) };\n\
               };\n\
               walk(4i32);\n\
             };"
                .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check baseline fixture");
        let core = crate::core::lower(&checked);
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution = crate::execution::lower(closure, crate::execution::OptimizationSet::none());

        assert!(supports(&execution));
    }

    #[test]
    fn selects_symbol_storage_reuse_only_when_enabled() {
        let source = SourceFile::new(
            FileId::new(87),
            "symbol-concat-optimization.mal",
            "main :: Unit -> Int32 := () { prefix := \"a\" + \"b\"; text := prefix + \"c\"; Int32(#text); };"
                .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check Symbol concat fixture");
        let core = crate::core::lower(&checked);
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution =
            crate::execution::lower(closure, crate::execution::OptimizationSet::production());
        let target = || Target {
            triple: "x86_64-unknown-linux-gnu",
            data_layout: "e-p:64:64",
        };
        let baseline = generate(&execution, target(), OptimizationSet::none())
            .expect("baseline Symbol concat is supported");
        let optimized = generate(&execution, target(), OptimizationSet::production())
            .expect("optimized Symbol concat is supported");

        assert!(
            !baseline
                .module
                .contains("call ptr @mal_runtime_symbol_concatenate_consuming_left")
        );
        assert!(
            baseline
                .module
                .contains("call ptr @mal_runtime_symbol_concatenate(")
        );
        assert!(
            optimized
                .module
                .contains("call ptr @mal_runtime_symbol_concatenate_consuming_left")
        );
    }

    #[test]
    fn reads_the_default_pointer_layout_and_an_explicit_address_space_zero_layout() {
        assert_eq!(pointer_size("e-m:e-i64:64"), Some(8));
        assert_eq!(pointer_size("e-p:32:32-i64:64"), Some(4));
        assert_eq!(pointer_size("e-p0:128:128"), Some(16));
        assert_eq!(pointer_size("e-p:7:8"), None);
    }

    #[test]
    fn uses_the_target_size_type_for_control_storage_offsets() {
        let source = SourceFile::new(
            FileId::new(79),
            "llvm-32-bit-control.mal",
            "apply :: ((Int32 -> Int32), Int32) -> Int32 := (operation, value) { operation(value); };\n\
             sum :: Int32 -> Int32 := (value) {\n\
               if (value == 0i32)\n\
               then { 0i32 }\n\
               else {\n\
                 rest := apply(sum, value - 1i32);\n\
                 value + rest;\n\
               };\n\
             };\n\
             main :: Unit -> Int32 := () { sum(4i32); };"
                .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check 32-bit control fixture");
        let core = crate::core::lower(&checked);
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution =
            crate::execution::lower(closure, crate::execution::OptimizationSet::production());
        let artifacts = generate(
            &execution,
            Target {
                triple: "i386-unknown-linux-gnu",
                data_layout: "e-p:32:32-i64:64",
            },
            OptimizationSet::production(),
        )
        .expect("32-bit control fixture is supported");

        assert!(
            artifacts
                .module
                .contains("declare ptr @mal_control_reserve_frame(ptr, i32, i32)")
        );
        assert!(
            artifacts
                .module
                .contains("%mal_control_top = alloca i32, align 4")
        );
        assert!(
            artifacts
                .module
                .contains("%mal_active_environment = alloca ptr, align 4")
        );
        assert!(
            !artifacts
                .module
                .contains("ptr %mal_active_environment, align 8")
        );
    }

    #[test]
    fn admits_product_external_calls() {
        for (index, source) in [
            "extern inspect :: (UInt64, UInt64) -> UInt64; main :: Unit -> Int32 := () { Int32(inspect(1u64, 2u64)); };",
            "extern inspect :: Bool -> Bool; main :: Unit -> Int32 := () { if (inspect(true)) then { 0 } else { 1 }; };",
            "extern inspect :: (UInt64, Symbol) -> UInt64; main :: Unit -> Int32 := () { Int32(inspect(1u64, \"x\")); };",
            "extern inspect :: (UInt64, Symbol) -> (UInt64, Symbol); main :: Unit -> Int32 := () { (value, _) := inspect(1u64, \"x\"); Int32(value); };",
            "Packet :: (UInt64, Symbol); extern exchange :: Packet -> Packet; main :: Unit -> Int32 := () { (number, text) := exchange(41u64, \"a\" + \"b\"); Int32(number); };",
        ]
        .into_iter()
        .enumerate()
        {
            let source = SourceFile::new(FileId::new(76), "product-extern.mal", source.into());
            let checked = crate::pipeline::check(&source).expect("check product extern fixture");
            let core = crate::core::lower(&checked);
            let anf = crate::anf::lower(&core);
            let closure = crate::closure::convert(&anf);
            let execution = crate::execution::lower(
                closure,
                crate::execution::OptimizationSet::production(),
            );
            assert!(supports(&execution), "unsupported fixture {index}");
        }
    }

    #[test]
    fn admits_sum_external_calls_recursively() {
        for (index, source) in [
            "Choice :: [Symbol, Symbol]; extern inspect :: Choice -> Choice; main :: Unit -> Int32 := () { 0; };",
            "Choice :: [Unit, (UInt64, Symbol)]; Envelope :: (UInt8, Choice); extern inspect :: Envelope -> Envelope; main :: Unit -> Int32 := () { 0; };",
            "extern Handle; Choice :: [Unit, (UInt64, Handle)]; extern inspect :: Choice -> Choice; main :: Unit -> Int32 := () { 0; };",
        ]
        .into_iter()
        .enumerate()
        {
            let source = SourceFile::new(FileId::new(77), "sum-extern.mal", source.into());
            let checked = crate::pipeline::check(&source).expect("check sum extern fixture");
            let core = crate::core::lower(&checked);
            let anf = crate::anf::lower(&core);
            let closure = crate::closure::convert(&anf);
            let execution = crate::execution::lower(
                closure,
                crate::execution::OptimizationSet::production(),
            );
            assert!(supports(&execution), "unsupported fixture {index}");
        }
    }
}
