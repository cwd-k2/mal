use super::abi::Function as AbiFunction;
use super::artifact::LlvmArtifacts;

mod body;

pub(crate) struct Target<'a> {
    pub(crate) triple: &'a str,
    pub(crate) data_layout: &'a str,
}

pub(crate) fn supports(program: &crate::execution::Program) -> bool {
    body::supports(program)
        && program.lowered.interface.externals.iter().all(|external| {
            bridge_type_supported(&external.parameter) && bridge_type_supported(&external.result)
        })
}

pub(crate) fn generate(
    program: &crate::execution::Program,
    target: Target<'_>,
) -> Option<LlvmArtifacts> {
    let pointer_size = pointer_size(target.data_layout)?;
    let body = body::generate(program, pointer_size)?;
    let entry = AbiFunction::program_entry();
    let raw_types = crate::c_emit::RawHostTypes::new(&program.lowered.interface);
    let external_bridges = program
        .lowered
        .interface
        .externals
        .iter()
        .map(|external| external_bridge(external, pointer_size, &raw_types))
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
    let symbol_declarations = if body.uses_symbols {
        "declare i64 @mal_runtime_symbol_length(ptr)\ndeclare i8 @mal_runtime_symbol_at(ptr, i64)\ndeclare ptr @mal_runtime_symbol_retain(ptr, ptr)\ndeclare void @mal_runtime_symbol_release(ptr)\ndeclare ptr @mal_runtime_symbol_concatenate(ptr, ptr, ptr)\ndeclare i8 @mal_runtime_symbol_equal(ptr, ptr)\ndeclare ptr @mal_runtime_symbol_read(ptr, ptr, i64)\ndeclare void @mal_runtime_symbol_write(ptr, ptr)\n\n"
    } else {
        ""
    };
    let module = format!(
        "target datalayout = \"{}\"\ntarget triple = \"{}\"\n\n{}{}{}\n{}\n{}define {} {{\nentry:\n  %mal_entry_result = call i32 @{}(ptr %mal_context)\n  store i32 %mal_entry_result, ptr %mal_result, align 4\n  ret void\n}}\n",
        target.data_layout,
        target.triple,
        control_declarations,
        symbol_declarations,
        external_declarations,
        body.globals,
        body.definitions,
        entry.llvm_signature(),
        match body.main {
            crate::closure::ast::FunctionId::Lambda(id) => format!("mal_function_{}", id.0),
            crate::closure::ast::FunctionId::Memory(_) => return None,
        },
    );
    let symbol_bridge_runtime = if body.uses_symbols {
        "MalType_Symbol mal_symbol_materialize(MalContext *context, MalType_Symbol value) {\n    (void)context;\n    return value;\n}\n\nMalType_Symbol mal_symbol_copy_from_bytes(MalContext *context, const uint8_t *data, uint64_t length) {\n    void *ownership = mal_runtime_symbol_read(context, data, length);\n    return (MalType_Symbol){\n        .data = mal_runtime_symbol_data(ownership),\n        .length = length,\n        .ownership = ownership,\n    };\n}\n\nMalType_Symbol mal_symbol_retain(MalContext *context, MalType_Symbol value) {\n    value.ownership = mal_runtime_symbol_retain(context, value.ownership);\n    return value;\n}\n"
    } else {
        ""
    };
    let shim = format!(
        "#include \"program.mal.h\"\n#include \"runtime.h\"\n\n{}\n\n{}\n\n{}\nint main(void) {{\n    MalContext context = {{0}};\n    int32_t result;\n    {}(&context, NULL, &result);\n    mal_control_destroy(&context);\n    return result;\n}}\n",
        entry.c_declaration(),
        external_definitions,
        symbol_bridge_runtime,
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

fn external_bridge(
    external: &crate::core::ast::ExternalOperation,
    pointer_size: usize,
    raw_types: &crate::c_emit::RawHostTypes,
) -> Option<(String, String)> {
    use crate::check::ast::Type;

    let types = body::types::Types::new(pointer_size)?;
    let mut marshalling = BridgeMarshalling::new(external.id.0, types, raw_types);
    let bridge = AbiFunction::external_bridge(external.id);
    let llvm = format!("declare {}", bridge.llvm_signature());
    let signature = bridge.c_declaration();
    let signature = signature.strip_suffix(';')?;
    let (parameter, arguments) = match &external.parameter {
        Type::Unit => ("    (void)mal_argument;\n".into(), Vec::new()),
        Type::Product(elements) => {
            let fields = types.product_fields(&external.parameter)?;
            let arguments = elements
                .iter()
                .zip(fields)
                .map(|(element, field)| {
                    marshalling.read(
                        element,
                        "mal_argument",
                        field.offset,
                        "(MalContext *)mal_context",
                    )
                })
                .collect::<Option<Vec<_>>>()?;
            (String::new(), arguments)
        }
        ty => (
            String::new(),
            vec![marshalling.read(ty, "mal_argument", 0, "(MalContext *)mal_context")?],
        ),
    };
    let argument = arguments
        .iter()
        .map(|argument| format!(", {argument}"))
        .collect::<String>();
    let call = format!(
        "mal_ext_{}((MalContext *)mal_context{argument})",
        external.name
    );
    let result = match &external.result {
        Type::Unit => {
            format!("    {call};\n    *(uint8_t *)mal_result = UINT8_C(0);")
        }
        Type::Symbol => format!(
            "    MalType_Symbol result = {call};\n    *(void **)mal_result = result.ownership;"
        ),
        Type::Product(_) | Type::Sum(_) | Type::External { .. }
            if c_scalar_type(&external.result).is_none() =>
        {
            let result_type = raw_types.c_type(&external.result);
            let writes =
                marshalling.write(&external.result, "result", 0, "(MalContext *)mal_context")?;
            format!("    {result_type} result = {call};\n{writes}")
        }
        ty => {
            let result = c_scalar_type(ty)?;
            format!("    *({result} *)mal_result = {call};")
        }
    };
    let c = format!(
        "{}{signature} {{\n{parameter}{result}\n}}",
        marshalling.helpers
    );
    Some((llvm, c))
}

struct BridgeMarshalling<'a> {
    external: u32,
    types: body::types::Types,
    raw_types: &'a crate::c_emit::RawHostTypes,
    next_helper: usize,
    helpers: String,
}

impl<'a> BridgeMarshalling<'a> {
    fn new(
        external: u32,
        types: body::types::Types,
        raw_types: &'a crate::c_emit::RawHostTypes,
    ) -> Self {
        Self {
            external,
            types,
            raw_types,
            next_helper: 0,
            helpers: String::new(),
        }
    }

    fn helper_name(&mut self, direction: &str) -> String {
        let helper = self.next_helper;
        self.next_helper += 1;
        format!(
            "mal_bridge_external_{}_{}_sum_{}",
            self.external, direction, helper
        )
    }

    fn read(
        &mut self,
        ty: &crate::check::ast::Type,
        base: &str,
        offset: usize,
        context: &str,
    ) -> Option<String> {
        use crate::check::ast::Type;

        let pointer = bridge_pointer(base, offset, true);
        match ty {
            Type::Unit => Some("(MalType_Unit){.unused = UINT8_C(0)}".into()),
            Type::Symbol => {
                let ownership = format!("*(void *const *){pointer}");
                Some(format!(
                    "(MalType_Symbol){{.data = mal_runtime_symbol_data({ownership}), .length = mal_runtime_symbol_length({ownership}), .ownership = {ownership}}}"
                ))
            }
            Type::External { .. } => Some(format!(
                "({}){{.bits = *(const uintptr_t *){pointer}}}",
                self.raw_types.c_type(ty)
            )),
            Type::Product(elements) => {
                let fields = self.types.product_fields(ty)?;
                let initializers = elements
                    .iter()
                    .zip(fields)
                    .enumerate()
                    .map(|(index, (element, field))| {
                        Some(format!(
                            ".field_{index} = {}",
                            self.read(element, base, offset.checked_add(field.offset)?, context)?
                        ))
                    })
                    .collect::<Option<Vec<_>>>()?
                    .join(", ");
                Some(format!("({}){{{initializers}}}", self.raw_types.c_type(ty)))
            }
            Type::Sum(elements) if !body::types::is_bool(ty) => {
                self.read_sum(ty, elements, &pointer, context)
            }
            _ => {
                let c_type = c_scalar_type(ty)?;
                Some(format!("*(const {c_type} *){pointer}"))
            }
        }
    }

    fn read_sum(
        &mut self,
        ty: &crate::check::ast::Type,
        elements: &[crate::check::ast::Type],
        pointer: &str,
        context: &str,
    ) -> Option<String> {
        let helper = self.helper_name("read");
        let c_type = self.raw_types.c_type(ty);
        let fields = self.types.sum_fields(ty)?;
        let tag_offset = fields.first()?.offset;
        let cases = elements
            .iter()
            .zip(fields.into_iter().skip(1))
            .enumerate()
            .map(|(index, (element, field))| {
                Some(format!(
                    "    case UINT32_C({index}):\n        return ({c_type}){{.tag = UINT32_C({index}), .payload.variant_{index} = {}}};",
                    self.read(element, "value", field.offset, "context")?
                ))
            })
            .collect::<Option<Vec<_>>>()?
            .join("\n");
        self.helpers.push_str(&format!(
            "static {c_type} {helper}(MalContext *context, const unsigned char *value) {{\n    uint32_t tag = *(const uint32_t *)(value + {tag_offset});\n    switch (tag) {{\n{cases}\n    default:\n        mal_trap(context, \"invalid sum tag at LLVM bridge\");\n    }}\n}}\n\n"
        ));
        Some(format!("{helper}({context}, {pointer})"))
    }

    fn write(
        &mut self,
        ty: &crate::check::ast::Type,
        value: &str,
        offset: usize,
        context: &str,
    ) -> Option<String> {
        self.write_at(ty, "mal_result", value, offset, context)
    }

    fn write_at(
        &mut self,
        ty: &crate::check::ast::Type,
        base: &str,
        value: &str,
        offset: usize,
        context: &str,
    ) -> Option<String> {
        use crate::check::ast::Type;

        let pointer = bridge_pointer(base, offset, false);
        match ty {
            Type::Unit => Some(format!("    *(uint8_t *){pointer} = UINT8_C(0);")),
            Type::Symbol => Some(format!("    *(void **){pointer} = {value}.ownership;")),
            Type::External { .. } => Some(format!("    *(uintptr_t *){pointer} = {value}.bits;")),
            Type::Product(elements) => {
                let fields = self.types.product_fields(ty)?;
                elements
                    .iter()
                    .zip(fields)
                    .enumerate()
                    .map(|(index, (element, field))| {
                        self.write_at(
                            element,
                            base,
                            &format!("{value}.field_{index}"),
                            offset.checked_add(field.offset)?,
                            context,
                        )
                    })
                    .collect::<Option<Vec<_>>>()
                    .map(|writes| writes.join("\n"))
            }
            Type::Sum(elements) if !body::types::is_bool(ty) => {
                self.write_sum(ty, elements, value, &pointer, context)
            }
            _ => {
                let c_type = c_scalar_type(ty)?;
                Some(format!("    *({c_type} *){pointer} = {value};"))
            }
        }
    }

    fn write_sum(
        &mut self,
        ty: &crate::check::ast::Type,
        elements: &[crate::check::ast::Type],
        value: &str,
        pointer: &str,
        context: &str,
    ) -> Option<String> {
        let helper = self.helper_name("write");
        let c_type = self.raw_types.c_type(ty);
        let fields = self.types.sum_fields(ty)?;
        let tag_offset = fields.first()?.offset;
        let cases = elements
            .iter()
            .zip(fields.into_iter().skip(1))
            .enumerate()
            .map(|(index, (element, field))| {
                let writes = self.write_at(
                    element,
                    "value",
                    &format!("input.payload.variant_{index}"),
                    field.offset,
                    "context",
                )?;
                Some(format!(
                    "    case UINT32_C({index}):\n{writes}\n        return;"
                ))
            })
            .collect::<Option<Vec<_>>>()?
            .join("\n");
        self.helpers.push_str(&format!(
            "static void {helper}(MalContext *context, unsigned char *value, {c_type} input) {{\n    *(uint32_t *)(value + {tag_offset}) = input.tag;\n    switch (input.tag) {{\n{cases}\n    default:\n        mal_trap(context, \"invalid sum tag at LLVM bridge\");\n    }}\n}}\n\n"
        ));
        Some(format!("    {helper}({context}, {pointer}, {value});"))
    }
}

fn bridge_pointer(base: &str, offset: usize, read_only: bool) -> String {
    let qualifier = if read_only { "const " } else { "" };
    if offset == 0 {
        return format!("(({qualifier}unsigned char *){base})");
    }
    format!("(({qualifier}unsigned char *){base} + {offset})")
}

fn bridge_type_supported(ty: &crate::check::ast::Type) -> bool {
    use crate::check::ast::Type;

    c_scalar_type(ty).is_some()
        || matches!(ty, Type::Unit | Type::Symbol)
        || matches!(ty, Type::External { .. })
        || matches!(ty, Type::Product(elements) if elements.iter().all(bridge_type_supported))
        || matches!(ty, Type::Sum(elements) if elements.iter().all(bridge_type_supported))
}

fn c_scalar_type(ty: &crate::check::ast::Type) -> Option<&'static str> {
    use crate::check::ast::Type;

    if body::types::is_bool(ty) {
        return Some("MalType_Bool");
    }
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

    #[test]
    fn admits_product_external_calls() {
        for (index, source) in [
            "extern inspect :: (UInt64, UInt64) -> UInt64; main :: Unit -> Int32 := \\() { Int32(inspect(1u64, 2u64)); };",
            "extern inspect :: Bool -> Bool; main :: Unit -> Int32 := \\() { if (inspect(true)) then { 0 } else { 1 }; };",
            "extern inspect :: (UInt64, Symbol) -> UInt64; main :: Unit -> Int32 := \\() { Int32(inspect(1u64, \"x\")); };",
            "extern inspect :: (UInt64, Symbol) -> (UInt64, Symbol); main :: Unit -> Int32 := \\() { (value, _) := inspect(1u64, \"x\"); Int32(value); };",
            "Packet :: (UInt64, Symbol); extern exchange :: Packet -> Packet; main :: Unit -> Int32 := \\() { (number, text) := exchange(41u64, \"a\" + \"b\"); Int32(number); };",
        ]
        .into_iter()
        .enumerate()
        {
            let source = SourceFile::new(FileId::new(76), "product-extern.mal", source.into());
            let checked = crate::pipeline::check(&source).expect("check product extern fixture");
            let core = crate::core::lower(&checked);
            let anf = crate::anf::lower(&core);
            let closure = crate::closure::convert(&anf);
            let execution = crate::execution::lower(closure);
            assert!(supports(&execution), "unsupported fixture {index}");
        }
    }

    #[test]
    fn admits_sum_external_calls_recursively() {
        for (index, source) in [
            "Choice :: [Symbol, Symbol]; extern inspect :: Choice -> Choice; main :: Unit -> Int32 := \\() { 0; };",
            "Choice :: [Unit, (UInt64, Symbol)]; Envelope :: (UInt8, Choice); extern inspect :: Envelope -> Envelope; main :: Unit -> Int32 := \\() { 0; };",
            "extern Handle; Choice :: [Unit, (UInt64, Handle)]; extern inspect :: Choice -> Choice; main :: Unit -> Int32 := \\() { 0; };",
        ]
        .into_iter()
        .enumerate()
        {
            let source = SourceFile::new(FileId::new(77), "sum-extern.mal", source.into());
            let checked = crate::pipeline::check(&source).expect("check sum extern fixture");
            let core = crate::core::lower(&checked);
            let anf = crate::anf::lower(&core);
            let closure = crate::closure::convert(&anf);
            let execution = crate::execution::lower(closure);
            assert!(supports(&execution), "unsupported fixture {index}");
        }
    }
}
