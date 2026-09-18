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
    Diagnostic(crate::diagnostic::Diagnostic),
    InvalidTargetDataLayout,
    InconsistentExecutionPlan(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Diagnostic(diagnostic) => formatter.write_str(&diagnostic.message),
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
}

pub(crate) fn generate(
    program: &crate::execution::Program,
    target: Target<'_>,
    optimizations: OptimizationSet,
) -> Result<LlvmArtifacts, Error> {
    let layout = target_layout(target.data_layout).ok_or(Error::InvalidTargetDataLayout)?;
    body::admit_target(program, layout).map_err(Error::Diagnostic)?;
    let body = body::generate(program, layout, optimizations)
        .ok_or(Error::InconsistentExecutionPlan("LLVM body emission"))?;
    let types = body::types::Types::for_target(layout)
        .ok_or(Error::InconsistentExecutionPlan("target type construction"))?;
    let entry = AbiFunction::program_entry();
    let raw_types = crate::backend::c::RawHostTypes::new(&program.lowered.interface);
    let external_bridges = program
        .lowered
        .interface
        .externals
        .iter()
        .map(|external| {
            host_bridge::generate(external, layout, &raw_types)
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
    let byte_declarations = if body.uses_byte_runtime {
        let index = types
            .pointer_integer()
            .ok_or(Error::InconsistentExecutionPlan(
                "byte runtime ABI construction",
            ))?;
        format!(
            "declare ptr @mal_runtime_bytes_data(ptr)\n\
             declare ptr @mal_runtime_bytes_read(ptr, ptr, {index})\n\
             declare ptr @mal_runtime_bytes_retain(ptr, ptr)\n\
             declare void @mal_runtime_bytes_release(ptr)\n\
             declare void @mal_runtime_bytes_write(ptr, ptr, {index}, {index})\n\
             declare i8 @mal_runtime_symbol_at(ptr, {index}, {index})\n\
             declare void @mal_runtime_symbol_concatenate(ptr, ptr, ptr, {index}, {index}, ptr, {index}, {index})\n\
             declare void @mal_runtime_symbol_concatenate_consuming_left(ptr, ptr, ptr, {index}, {index}, ptr, {index}, {index})\n\
             declare void @mal_runtime_symbol_concatenate_consuming_right(ptr, ptr, ptr, {index}, {index}, ptr, {index}, {index})\n\
             declare i8 @mal_runtime_symbol_equal(ptr, {index}, {index}, ptr, {index}, {index})\n\n"
        )
    } else {
        String::new()
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
                types.index_alignment()
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
        byte_declarations,
        external_declarations,
        body.globals,
        body.definitions,
        entry.llvm_signature(),
        control_entry,
        entry_argument,
        entry_call,
    );
    let main = shim::entry_main(&body.main_parameter, types, entry.name())
        .ok_or(Error::InconsistentExecutionPlan("process entry emission"))?
        .render();
    let entry_declaration =
        crate::backend::c::syntax::Declaration::function(entry.c_signature()).render();
    let shim = format!(
        "#include \"program.mal.h\"\n#include \"runtime.h\"\n\n#include <string.h>\n\n{}\n\n{}\n\n{}",
        entry_declaration.trim_end(),
        external_definitions,
        main,
    );
    Ok(LlvmArtifacts {
        module,
        shim,
        header: crate::backend::c::emit_header_for_target(&program.lowered.interface, layout),
        runtime: crate::backend::runtime::control().into(),
    })
}

fn function_name(id: crate::closure::ast::FunctionId) -> Option<String> {
    let crate::closure::ast::FunctionId::Lambda(id) = id;
    Some(format!("mal_function_{}", id.0))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TargetLayout {
    pub(crate) pointer_size: usize,
    pub(crate) pointer_alignment: usize,
    pub(crate) index_size: usize,
    pub(crate) integer_alignments: [usize; 4],
    pub(crate) float_alignments: [usize; 2],
    pub(crate) supports_pointer_alignment: bool,
}

impl TargetLayout {
    pub(crate) fn natural(pointer_size: usize, index_size: usize) -> Option<Self> {
        (pointer_size.is_power_of_two() && index_size.is_power_of_two()).then_some(Self {
            pointer_size,
            pointer_alignment: pointer_size,
            index_size,
            integer_alignments: [1, 2, 4, 8],
            float_alignments: [4, 8],
            supports_pointer_alignment: true,
        })
    }

    pub(crate) fn scalar_alignment(self, bits: u8, floating: bool) -> Option<usize> {
        if floating {
            return match bits {
                32 => Some(self.float_alignments[0]),
                64 => Some(self.float_alignments[1]),
                _ => None,
            };
        }
        match bits {
            8 => Some(self.integer_alignments[0]),
            16 => Some(self.integer_alignments[1]),
            32 => Some(self.integer_alignments[2]),
            64 => Some(self.integer_alignments[3]),
            _ => None,
        }
    }
}

pub(crate) fn target_layout(data_layout: &str) -> Option<TargetLayout> {
    let pointer = data_layout.split('-').find_map(|component| {
        component
            .strip_prefix("p:")
            .or_else(|| component.strip_prefix("p0:"))
    });
    let (pointer_bits, pointer_alignment_bits, index_bits) = if let Some(pointer) = pointer {
        let fields = pointer.split(':').collect::<Vec<_>>();
        let pointer_bits = fields.first()?.parse::<usize>().ok()?;
        let pointer_alignment_bits = fields.get(1)?.parse::<usize>().ok()?;
        let index_bits = fields
            .get(3)
            .map_or(Some(pointer_bits), |bits| bits.parse().ok())?;
        (pointer_bits, pointer_alignment_bits, index_bits)
    } else {
        (64, 64, 64)
    };
    let byte_alignment = |bits: usize| {
        bits.is_multiple_of(8)
            .then_some(bits / 8)
            .filter(|bytes| bytes.is_power_of_two())
    };
    let supported_size =
        |bits: usize| byte_alignment(bits).filter(|bytes| matches!(bytes, 1 | 2 | 4 | 8));
    let mut layout = TargetLayout {
        pointer_size: supported_size(pointer_bits)?,
        pointer_alignment: byte_alignment(pointer_alignment_bits)?,
        index_size: supported_size(index_bits)?,
        integer_alignments: [1, 2, 4, 8],
        float_alignments: [4, 8],
        supports_pointer_alignment: !data_layout.split('-').any(|component| {
            component
                .strip_prefix("ni:")
                .is_some_and(|spaces| spaces.split(':').any(|space| space == "0"))
        }),
    };
    for component in data_layout.split('-') {
        let (floating, fields) = if let Some(fields) = component.strip_prefix('i') {
            (false, fields)
        } else if let Some(fields) = component.strip_prefix('f') {
            (true, fields)
        } else {
            continue;
        };
        let mut fields = fields.split(':');
        let bits = fields.next()?.parse::<u8>().ok()?;
        let alignment = byte_alignment(fields.next()?.parse::<usize>().ok()?)?;
        match (floating, bits) {
            (false, 8) => layout.integer_alignments[0] = alignment,
            (false, 16) => layout.integer_alignments[1] = alignment,
            (false, 32) => layout.integer_alignments[2] = alignment,
            (false, 64) => layout.integer_alignments[3] = alignment,
            (true, 32) => layout.float_alignments[0] = alignment,
            (true, 64) => layout.float_alignments[1] = alignment,
            _ => {}
        }
    }
    Some(layout)
}

#[cfg(test)]
mod tests;
