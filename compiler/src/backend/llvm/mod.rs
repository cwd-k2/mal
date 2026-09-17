use super::abi::Function as AbiFunction;
use super::artifact::LlvmArtifacts;
use std::fmt;

mod body;
mod host_bridge;
mod optimization;
mod shim;

pub(crate) use optimization::OptimizationSet;

pub(crate) struct Target<'a> {
    pub(crate) triple: &'a str,
    pub(crate) data_layout: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Error {
    InvalidTargetDataLayout,
    InconsistentExecutionPlan(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTargetDataLayout => {
                formatter.write_str("target data layout does not define a supported pointer size")
            }
            Self::InconsistentExecutionPlan(phase) => {
                write!(
                    formatter,
                    "admitted execution plan is inconsistent during {phase}"
                )
            }
        }
    }
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
) -> Result<LlvmArtifacts, Error> {
    let layout = target_layout(target.data_layout).ok_or(Error::InvalidTargetDataLayout)?;
    let pointer_size = layout.pointer_size;
    let body = body::generate(program, pointer_size, layout.index_size, optimizations)
        .ok_or(Error::InconsistentExecutionPlan("LLVM body emission"))?;
    let types = body::types::Types::for_target(pointer_size, layout.index_size)
        .ok_or(Error::InconsistentExecutionPlan("target type construction"))?;
    let entry = AbiFunction::program_entry();
    let raw_types = crate::backend::c::RawHostTypes::new(&program.lowered.interface);
    let external_bridges = program
        .lowered
        .interface
        .externals
        .iter()
        .map(|external| {
            host_bridge::generate(external, pointer_size, layout.index_size, &raw_types)
                .ok_or(Error::InconsistentExecutionPlan("extern bridge emission"))
        })
        .collect::<Result<Vec<_>, _>>()?;
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
            types
                .pointer_integer()
                .ok_or(Error::InconsistentExecutionPlan("control ABI construction"))?
        )
    } else {
        String::new()
    };
    let symbol_declarations = if body.uses_symbol_runtime {
        "declare i64 @mal_runtime_symbol_length(ptr)\ndeclare i8 @mal_runtime_symbol_at(ptr, i64)\ndeclare ptr @mal_runtime_symbol_data(ptr, ptr)\ndeclare ptr @mal_runtime_symbol_retain(ptr, ptr)\ndeclare void @mal_runtime_symbol_release(ptr)\ndeclare ptr @mal_runtime_symbol_concatenate(ptr, ptr, ptr)\ndeclare ptr @mal_runtime_symbol_concatenate_consuming_left(ptr, ptr, ptr)\ndeclare ptr @mal_runtime_symbol_concatenate_consuming_right(ptr, ptr, ptr)\ndeclare i8 @mal_runtime_symbol_equal(ptr, ptr)\ndeclare ptr @mal_runtime_symbol_read(ptr, ptr, i64)\ndeclare void @mal_runtime_symbol_write(ptr, ptr)\n\n"
    } else {
        ""
    };
    let (control_entry, control_top) = if body.uses_control {
        (
            format!(
                "  %mal_control_top = alloca {0}, align {1}\n  store {0} 0, ptr %mal_control_top, align {1}\n",
                types
                    .pointer_integer()
                    .ok_or(Error::InconsistentExecutionPlan(
                        "control entry construction"
                    ))?,
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
                function_name(body.main)
                    .ok_or(Error::InconsistentExecutionPlan("entry function selection"))?,
            ),
        ),
        ty => {
            let value = types
                .value(ty)
                .ok_or(Error::InconsistentExecutionPlan("entry argument layout"))?;
            (
                format!(
                    "  %mal_entry_argument = load {}, ptr %mal_argument, align {}\n",
                    value.llvm, value.alignment
                ),
                format!(
                    "call i32 @{}(ptr %mal_context, ptr {control_top}, ptr null, {} %mal_entry_argument)",
                    function_name(body.main)
                        .ok_or(Error::InconsistentExecutionPlan("entry function selection"))?,
                    value.llvm
                ),
            )
        }
    };
    let module = format!(
        "target datalayout = \"{}\"\ntarget triple = \"{}\"\n\ndeclare ptr @mal_runtime_environment_allocate(ptr, {}, ptr)\ndeclare ptr @mal_runtime_environment_retain(ptr, ptr)\ndeclare void @mal_runtime_environment_release(ptr)\ndeclare ptr @llvm.ptrmask.p0.i{}(ptr, {})\ndeclare void @llvm.memcpy.p0.p0.i{}(ptr, ptr, {}, i1 immarg)\n{}{}{}\n{}\n{}define {} {{\nentry:\n{}{}  %mal_entry_result = {}\n  store i32 %mal_entry_result, ptr %mal_result, align 4\n  ret void\n}}\n",
        target.data_layout,
        target.triple,
        types
            .pointer_integer()
            .ok_or(Error::InconsistentExecutionPlan("runtime ABI construction"))?,
        types.pointer_size() * 8,
        types
            .pointer_representation_integer()
            .ok_or(Error::InconsistentExecutionPlan("ptrmask ABI construction"))?,
        layout.index_size * 8,
        types
            .pointer_integer()
            .ok_or(Error::InconsistentExecutionPlan("memcpy ABI construction"))?,
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
    let main = shim::entry_main(&body.main_parameter, types, entry.name())
        .ok_or(Error::InconsistentExecutionPlan("process entry emission"))?
        .render();
    let entry_declaration =
        crate::backend::c::syntax::Declaration::function(entry.c_signature()).render();
    let shim = format!(
        "#include \"program.mal.h\"\n#include \"runtime.h\"\n\n#include <string.h>\n\n{}\n\n{}\n\n{}\n{}",
        entry_declaration.trim_end(),
        external_definitions,
        symbol_bridge_runtime,
        main,
    );
    Ok(LlvmArtifacts {
        module,
        shim,
        header: crate::backend::c::emit_header_for_target(
            &program.lowered.interface,
            layout.index_size * 8,
        ),
        runtime: crate::backend::runtime::control().into(),
    })
}

fn function_name(id: crate::closure::ast::FunctionId) -> Option<String> {
    let crate::closure::ast::FunctionId::Lambda(id) = id;
    Some(format!("mal_function_{}", id.0))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TargetLayout {
    pointer_size: usize,
    index_size: usize,
}

fn target_layout(data_layout: &str) -> Option<TargetLayout> {
    let pointer = data_layout.split('-').find_map(|component| {
        component
            .strip_prefix("p:")
            .or_else(|| component.strip_prefix("p0:"))
    });
    let (pointer_bits, index_bits) = if let Some(pointer) = pointer {
        let fields = pointer.split(':').collect::<Vec<_>>();
        let pointer_bits = fields.first()?.parse::<usize>().ok()?;
        let index_bits = fields
            .get(3)
            .map_or(Some(pointer_bits), |bits| bits.parse().ok())?;
        (pointer_bits, index_bits)
    } else {
        (64, 64)
    };
    let valid = |bits: usize| {
        bits.is_multiple_of(8)
            .then_some(bits / 8)
            .filter(|bytes| bytes.is_power_of_two())
    };
    Some(TargetLayout {
        pointer_size: valid(pointer_bits)?,
        index_size: valid(index_bits)?,
    })
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
            "main :: Unit -> Int32 := () -> { 7; };".into(),
        );
        let checked = crate::pipeline::check(&source).expect("check LLVM fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize checked program"),
        );
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

        assert!(matches!(
            generate(
                &execution,
                Target {
                    triple: "x86_64-unknown-linux-gnu",
                    data_layout: "e-p:7:8",
                },
                OptimizationSet::production(),
            ),
            Err(Error::InvalidTargetDataLayout)
        ));

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
    fn emits_typed_scalar_cursor_access_with_exact_alignment() {
        let source = SourceFile::new(
            FileId::new(89),
            "llvm-cursor.mal",
            "extern memory :: Unit -> Address;\n\
             main :: Unit -> Int32 := () -> {\n\
               cursor := memory()@u64;\n\
               _ := cursor <- 41u64;\n\
               (value, _) := <-cursor;\n\
               value.i32;\n\
             };"
            .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check cursor fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize checked program"),
        );
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
        .expect("typed scalar cursor fixture is supported");

        assert!(artifacts.module.contains("store i64 %mal_value"));
        assert!(artifacts.module.contains("load i64, ptr"));
        assert!(artifacts.module.contains("align 1"));
        assert!(artifacts.header.contains("mal_Address_return"));
    }

    #[test]
    fn emits_packed_views_indexing_and_symbol_conversion() {
        let source = SourceFile::new(
            FileId::new(91),
            "llvm-packed.mal",
            "main :: Unit -> Int32 := () -> {\n\
               packed := *\"abc\";\n\
               prefix := packed / 2usize;\n\
               byte := prefix # 1usize;\n\
               text := *prefix;\n\
               byte.i32 + (#text).i32;\n\
             };"
            .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check Packed fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize checked program"),
        );
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
        .expect("Packed fixture is supported");

        assert!(artifacts.module.contains("@mal_runtime_symbol_data"));
        assert!(artifacts.module.contains("@mal_runtime_symbol_read"));
        assert!(
            artifacts
                .module
                .contains("call void @mal_runtime_symbol_release")
        );
    }

    #[test]
    fn emits_long_left_associative_expressions_without_host_recursion() {
        let expression = std::iter::repeat_n("0i32", 4096)
            .collect::<Vec<_>>()
            .join(" + ");
        let source = SourceFile::new(
            FileId::new(76),
            "llvm-long-expression.mal",
            format!("main :: Unit -> Int32 := () -> {{ {expression}; }};"),
        );
        let checked = crate::pipeline::check(&source).expect("check long expression");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize checked program"),
        );
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
        .expect("long expression is supported");

        assert!(artifacts.module.contains("mal_function_"));
    }

    #[test]
    fn emits_long_completion_control_sequences_without_ast_duplication() {
        let source = SourceFile::new(
            FileId::new(78),
            "llvm-long-completion.mal",
            format!(
                "main :: Unit -> Int32 := () -> [return] => {{ {}return(0i32) }};",
                "when (false) { return(1i32) };".repeat(1_024)
            ),
        );
        let checked = crate::pipeline::check(&source).expect("check long completion sequence");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize checked program"),
        );
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
        .expect("long completion sequence is supported");

        assert!(artifacts.module.contains("mal_function_"));
    }

    #[test]
    fn emits_shared_extern_sum_helpers_once_per_type() {
        let mut declarations = String::from("Choice0 :: [UInt8, UInt8];\n");
        for depth in 1..16 {
            declarations.push_str(&format!(
                "Choice{depth} :: [Choice{}, Choice{}];\n",
                depth - 1,
                depth - 1
            ));
        }
        declarations.push_str(
            "extern exchange :: Choice15 -> Choice15;\n\
             main :: Unit -> Int32 := () -> { 0i32; };",
        );
        let source = SourceFile::new(FileId::new(77), "llvm-shared-extern-sum.mal", declarations);
        let checked = crate::pipeline::check(&source).expect("check shared extern sum");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize checked program"),
        );
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
        .expect("shared extern sum is supported");

        assert!(artifacts.shim.len() < 250_000);
        assert_eq!(
            artifacts
                .shim
                .matches("static void mal_bridge_external_0_write_sum_")
                .count(),
            16
        );
    }

    #[test]
    fn admits_direct_self_handoffs_to_wildcard_parameters() {
        for (index, source) in [
            "extern again :: Unit -> Bool; walk :: Int32 -> Int32 := (_) -> { if (again()) then { child := walk(1i32); child + 1i32; } else { 0i32 }; }; main :: Unit -> Int32 := () -> { walk(0i32); };",
            "extern again :: Unit -> Bool; make :: Int32 -> (Unit -> Int32) := (value) -> { () -> { value }; }; walk :: (Unit -> Int32) -> Int32 := (_) -> { if (again()) then { child := walk(make(1i32)); child + 1i32; } else { 0i32 }; }; main :: Unit -> Int32 := () -> { walk(make(0i32)); };",
            "extern again :: Unit -> Bool; walk :: Int32 -> Int32 := (_) -> { if (again()) then { walk(1i32) } else { 0i32 }; }; main :: Unit -> Int32 := () -> { walk(0i32); };",
            "extern again :: Unit -> Bool; make :: Int32 -> (Unit -> Int32) := (value) -> { () -> { value }; }; walk :: (Unit -> Int32) -> Int32 := (_) -> { if (again()) then { walk(make(1i32)) } else { 0i32 }; }; main :: Unit -> Int32 := () -> { walk(make(0i32)); };",
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
            let core = crate::core::lower(&crate::check::specialize(checked).expect("specialize checked program"));
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
            "apply :: ((Int32 -> Int32), Int32) -> Int32 := (function, value) -> { function(value); };\n\
             main :: Unit -> Int32 := () -> {\n\
               walk :: Int32 -> Int32 := (value) -> {\n\
                 if (value == 0i32) then { 0i32 } else { apply(walk, value - 1i32) };\n\
               };\n\
               walk(4i32);\n\
             };"
                .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check baseline fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize checked program"),
        );
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
            "main :: Unit -> Int32 := () -> { prefix := \"a\" + \"b\"; text := prefix + \"c\"; (#text).i32; };"
                .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check Symbol concat fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize checked program"),
        );
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
    fn reads_supported_pointer_widths_from_target_data_layouts() {
        assert_eq!(
            target_layout("e-m:e-i64:64"),
            Some(TargetLayout {
                pointer_size: 8,
                index_size: 8
            })
        );
        for bits in 0_usize..=256 {
            let bytes = bits / 8;
            let expected =
                (bits.is_multiple_of(8) && bytes.is_power_of_two()).then_some(TargetLayout {
                    pointer_size: bytes,
                    index_size: bytes,
                });
            assert_eq!(target_layout(&format!("e-p:{bits}:{bits}")), expected);
            assert_eq!(target_layout(&format!("e-p0:{bits}:{bits}")), expected);
        }
        assert_eq!(
            target_layout("e-p1:32:32-p0:64:64:64:32"),
            Some(TargetLayout {
                pointer_size: 8,
                index_size: 4
            })
        );
        assert_eq!(target_layout("e-p:invalid:64"), None);
    }

    #[test]
    fn uses_the_target_size_type_for_control_storage_offsets() {
        let source = SourceFile::new(
            FileId::new(79),
            "llvm-32-bit-control.mal",
            "apply :: ((Int32 -> Int32), Int32) -> Int32 := (operation, value) -> { operation(value); };\n\
             sum :: Int32 -> Int32 := (value) -> {\n\
               if (value == 0i32)\n\
               then { 0i32 }\n\
               else {\n\
                 rest := apply(sum, value - 1i32);\n\
                 value + rest;\n\
               };\n\
             };\n\
             main :: Unit -> Int32 := () -> { sum(4i32); };"
                .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check 32-bit control fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize checked program"),
        );
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
    fn separates_pointer_representation_and_index_widths() {
        let source = SourceFile::new(
            FileId::new(90),
            "llvm-index-width.mal",
            "scale :: (USize, ByteSize) -> ByteSize := (count, size) -> count * size;\n\
             main :: Unit -> Int32 := () -> scale(3usize, 8bytes).usize.i32;"
                .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check index-width fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize checked program"),
        );
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution =
            crate::execution::lower(closure, crate::execution::OptimizationSet::production());
        let artifacts = generate(
            &execution,
            Target {
                triple: "synthetic-unknown-none",
                data_layout: "e-p:64:64:64:32",
            },
            OptimizationSet::production(),
        )
        .expect("split pointer/index layout is supported");

        assert!(artifacts.module.contains("mul i32"));
        assert!(
            artifacts
                .module
                .contains("declare ptr @llvm.ptrmask.p0.i64(ptr, i64)")
        );
    }

    #[test]
    fn admits_product_external_calls() {
        for (index, source) in [
            "extern inspect :: (UInt64, UInt64) -> UInt64; main :: Unit -> Int32 := () -> { inspect(1u64, 2u64).i32; };",
            "extern inspect :: Bool -> Bool; main :: Unit -> Int32 := () -> { if (inspect(true)) then { 0 } else { 1 }; };",
            "extern inspect :: (UInt64, Symbol) -> UInt64; main :: Unit -> Int32 := () -> { inspect(1u64, \"x\").i32; };",
            "extern inspect :: (UInt64, Symbol) -> (UInt64, Symbol); main :: Unit -> Int32 := () -> { (value, _) := inspect(1u64, \"x\"); value.i32; };",
            "Packet :: (UInt64, Symbol); extern exchange :: Packet -> Packet; main :: Unit -> Int32 := () -> { (number, text) := exchange(41u64, \"a\" + \"b\"); number.i32; };",
        ]
        .into_iter()
        .enumerate()
        {
            let source = SourceFile::new(FileId::new(76), "product-extern.mal", source.into());
            let checked = crate::pipeline::check(&source).expect("check product extern fixture");
            let core = crate::core::lower(&crate::check::specialize(checked).expect("specialize checked program"));
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
            "Choice :: [Symbol, Symbol]; extern inspect :: Choice -> Choice; main :: Unit -> Int32 := () -> { 0; };",
            "Choice :: [Unit, (UInt64, Symbol)]; Envelope :: (UInt8, Choice); extern inspect :: Envelope -> Envelope; main :: Unit -> Int32 := () -> { 0; };",
            "extern Handle; Choice :: [Unit, (UInt64, Handle)]; extern inspect :: Choice -> Choice; main :: Unit -> Int32 := () -> { 0; };",
        ]
        .into_iter()
        .enumerate()
        {
            let source = SourceFile::new(FileId::new(77), "sum-extern.mal", source.into());
            let checked = crate::pipeline::check(&source).expect("check sum extern fixture");
            let core = crate::core::lower(&crate::check::specialize(checked).expect("specialize checked program"));
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
