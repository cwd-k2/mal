use crate::check::ast::Type;
use crate::core::ast::TypeAlias;

use super::{TypeRegistry, is_bool};

impl TypeRegistry {
    pub(in crate::c_emit) fn header_declarations(&self) -> String {
        let mut output = String::new();
        for name in &self.opaque_names {
            c_line!(
                &mut output,
                0,
                "typedef struct {{ uintptr_t bits; }} MalType_{name};"
            );
        }
        if !self.opaque_names.is_empty() {
            output.push('\n');
        }
        output.push_str(&self.declarations(true));
        output
    }

    pub(in crate::c_emit) fn header_alias_declarations(&self, aliases: &[TypeAlias]) -> String {
        let mut output = String::new();
        for alias in aliases {
            if self.is_host_type(&alias.ty) {
                c_line!(
                    &mut output,
                    0,
                    "typedef {} MalType_{};",
                    self.c_type(&alias.ty),
                    alias.name
                );
            }
        }
        if !output.is_empty() {
            output.push('\n');
        }
        output
    }

    pub(in crate::c_emit) fn header_opaque_helpers(&self) -> String {
        let mut output = String::new();
        for name in &self.opaque_names {
            c_line!(
                &mut output,
                0,
                "static inline MalType_{name} mal_{name}_from_bits(uintptr_t bits) {{"
            );
            c_line!(&mut output, 1, "return (MalType_{name}){{ .bits = bits }};");
            output.push_str("}\n\n");
            c_line!(
                &mut output,
                0,
                "static inline uintptr_t mal_{name}_bits(MalType_{name} value) {{"
            );
            c_line!(&mut output, 1, "return value.bits;");
            output.push_str("}\n\n");
        }
        output
    }

    pub(in crate::c_emit) fn header_alias_helpers(&self, aliases: &[TypeAlias]) -> String {
        let mut output = String::new();
        for alias in aliases {
            if !self.is_host_type(&alias.ty) {
                continue;
            }
            match &alias.ty {
                Type::Product(elements) => {
                    self.emit_product_constructor(&mut output, alias, elements);
                    self.emit_product_accessors(&mut output, alias, elements);
                }
                Type::Sum(members) if !is_bool(&alias.ty) => {
                    self.emit_sum_helpers(&mut output, alias, members);
                }
                _ => {}
            }
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push('\n');
            }
        }
        output
    }

    pub(in crate::c_emit) fn header_c_type(&self, ty: &Type, alias: Option<&str>) -> String {
        if let Some(alias) = alias {
            format!("MalType_{alias}")
        } else {
            self.c_type(ty)
        }
    }

    fn emit_product_constructor(&self, output: &mut String, alias: &TypeAlias, elements: &[Type]) {
        c_line!(
            output,
            0,
            "static inline MalType_{} mal_{}_make(",
            alias.name,
            alias.name
        );
        self.emit_parameter_lines(output, elements);
        output.push_str(") {\n");
        c_line!(output, 1, "return (MalType_{}){{", alias.name);
        for index in 0..elements.len() {
            c_line!(output, 2, ".field_{index} = value_{index},");
        }
        output.push_str("    };\n}\n\n");
    }

    fn emit_product_accessors(&self, output: &mut String, alias: &TypeAlias, elements: &[Type]) {
        for (index, element) in elements.iter().enumerate() {
            c_line!(
                output,
                0,
                "static inline {} mal_{}_get_{index}(MalType_{} value) {{",
                self.c_type(element),
                alias.name,
                alias.name
            );
            c_line!(output, 1, "return value.field_{index};");
            output.push_str("}\n\n");
        }
    }

    fn emit_sum_helpers(&self, output: &mut String, alias: &TypeAlias, members: &[Type]) {
        for index in 0..members.len() {
            c_line!(
                output,
                0,
                "#define MAL_{}_TAG_{index} UINT32_C({index})",
                alias.name
            );
        }
        c_line!(
            output,
            0,
            "static inline uint32_t mal_{}_tag(MalType_{} value) {{",
            alias.name,
            alias.name
        );
        c_line!(output, 1, "return value.tag;");
        output.push_str("}\n\n");
        for (index, member) in members.iter().enumerate() {
            c_line!(
                output,
                0,
                "static inline MalType_Bool mal_{}_is_{index}(MalType_{} value) {{",
                alias.name,
                alias.name
            );
            c_line!(
                output,
                1,
                "return value.tag == MAL_{}_TAG_{index};",
                alias.name
            );
            output.push_str("}\n\n");
            self.emit_sum_constructor(output, alias, index, member);
            self.emit_sum_accessors(output, alias, index, member);
        }
    }

    fn emit_sum_constructor(
        &self,
        output: &mut String,
        alias: &TypeAlias,
        index: usize,
        member: &Type,
    ) {
        c_line!(
            output,
            0,
            "static inline MalType_{} mal_{}_make_{index}(",
            alias.name,
            alias.name
        );
        match member {
            Type::Unit => c_line!(output, 1, "void"),
            Type::Product(elements) => self.emit_parameter_lines(output, elements),
            _ => c_line!(output, 1, "{} value", self.c_type(member)),
        }
        output.push_str(") {\n");
        c_line!(output, 1, "return (MalType_{}){{", alias.name);
        c_line!(output, 2, ".tag = MAL_{}_TAG_{index},", alias.name);
        c_write!(output, "        .payload.variant_{index} = ",);
        match member {
            Type::Unit => output.push_str("{ .unused = UINT8_C(0) },\n"),
            Type::Product(elements) => {
                output.push_str("{\n");
                for element_index in 0..elements.len() {
                    c_line!(output, 3, ".field_{element_index} = value_{element_index},");
                }
                output.push_str("        },\n");
            }
            _ => output.push_str("value,\n"),
        }
        output.push_str("    };\n}\n\n");
    }

    fn emit_sum_accessors(
        &self,
        output: &mut String,
        alias: &TypeAlias,
        index: usize,
        member: &Type,
    ) {
        match member {
            Type::Unit => {}
            Type::Product(elements) => {
                for (element_index, element) in elements.iter().enumerate() {
                    c_line!(
                        output,
                        0,
                        "static inline {} mal_{}_expect_{index}_{element_index}(",
                        self.c_type(element),
                        alias.name
                    );
                    output.push_str("    MalContext *context,\n");
                    c_line!(output, 1, "MalType_{} value", alias.name);
                    output.push_str(") {\n");
                    c_line!(output, 1, "if (!mal_{}_is_{index}(value))", alias.name);
                    c_line!(
                        output,
                        2,
                        "mal_trap(context, \"expected {} variant {index}\");",
                        alias.name
                    );
                    c_line!(
                        output,
                        1,
                        "return value.payload.variant_{index}.field_{element_index};"
                    );
                    output.push_str("}\n\n");
                }
            }
            _ => {
                c_line!(
                    output,
                    0,
                    "static inline {} mal_{}_expect_{index}(",
                    self.c_type(member),
                    alias.name
                );
                output.push_str("    MalContext *context,\n");
                c_line!(output, 1, "MalType_{} value", alias.name);
                output.push_str(") {\n");
                c_line!(output, 1, "if (!mal_{}_is_{index}(value))", alias.name);
                c_line!(
                    output,
                    2,
                    "mal_trap(context, \"expected {} variant {index}\");",
                    alias.name
                );
                c_line!(output, 1, "return value.payload.variant_{index};");
                output.push_str("}\n\n");
            }
        }
    }

    fn emit_parameter_lines(&self, output: &mut String, elements: &[Type]) {
        for (index, element) in elements.iter().enumerate() {
            let comma = if index + 1 == elements.len() { "" } else { "," };
            c_line!(output, 1, "{} value_{index}{comma}", self.c_type(element));
        }
    }
}
