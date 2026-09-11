use super::body;
use crate::backend::abi::Function as AbiFunction;

pub(super) fn generate(
    external: &crate::core::ast::ExternalOperation,
    pointer_size: usize,
    raw_types: &crate::backend::c::RawHostTypes,
) -> Option<(String, String)> {
    use crate::check::ast::Type;

    let types = body::types::Types::new(pointer_size)?;
    let mut marshalling = Marshalling::new(external.id.0, types, raw_types);
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
        Type::Unit => format!("    {call};\n    *(uint8_t *)mal_result = UINT8_C(0);"),
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

struct Marshalling<'a> {
    external: u32,
    types: body::types::Types,
    raw_types: &'a crate::backend::c::RawHostTypes,
    next_helper: usize,
    helpers: String,
}

impl<'a> Marshalling<'a> {
    fn new(
        external: u32,
        types: body::types::Types,
        raw_types: &'a crate::backend::c::RawHostTypes,
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
                    "mal_symbol_materialize({context}, (MalType_Symbol){{.data = NULL, .length = mal_runtime_symbol_length({ownership}), .ownership = {ownership}}})"
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

pub(super) fn type_supported(ty: &crate::check::ast::Type) -> bool {
    use crate::check::ast::Type;

    c_scalar_type(ty).is_some()
        || matches!(ty, Type::Unit | Type::Symbol)
        || matches!(ty, Type::External { .. })
        || matches!(ty, Type::Product(elements) if elements.iter().all(type_supported))
        || matches!(ty, Type::Sum(elements) if elements.iter().all(type_supported))
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
