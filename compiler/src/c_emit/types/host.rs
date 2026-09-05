use crate::check::ast::Type;
use crate::closure::ast as closure;

use super::{TypeRegistry, is_bool};

impl TypeRegistry {
    pub(in crate::c_emit) fn header_declarations(&self) -> String {
        let mut output = String::new();
        for name in &self.opaque_names {
            c_line!(
                &mut output,
                0,
                "typedef struct {{ uintptr_t bits; }} MalOpaque_{name};"
            );
        }
        if !self.opaque_names.is_empty() {
            output.push('\n');
        }
        output.push_str(&self.declarations(true));
        output
    }

    pub(in crate::c_emit) fn header_alias_declarations(
        &self,
        aliases: &[closure::TypeAlias],
    ) -> String {
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
                "static inline MalOpaque_{name} mal_{name}_from_bits(uintptr_t bits) {{ return (MalOpaque_{name}){{ .bits = bits }}; }}"
            );
            c_line!(
                &mut output,
                0,
                "static inline uintptr_t mal_{name}_bits(MalOpaque_{name} value) {{ return value.bits; }}"
            );
        }
        if !output.is_empty() {
            output.push('\n');
        }
        output
    }

    pub(in crate::c_emit) fn header_alias_helpers(&self, aliases: &[closure::TypeAlias]) -> String {
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

    pub(in crate::c_emit) fn header_c_type(
        &self,
        ty: &Type,
        aliases: &[closure::TypeAlias],
    ) -> String {
        let mut matching = aliases
            .iter()
            .filter(|alias| alias.ty == *ty && self.is_host_type(&alias.ty));
        let Some(alias) = matching.next() else {
            return self.c_type(ty);
        };
        if matching.next().is_some() {
            return self.c_type(ty);
        }
        format!("MalType_{}", alias.name)
    }

    fn emit_product_constructor(
        &self,
        output: &mut String,
        alias: &closure::TypeAlias,
        elements: &[Type],
    ) {
        c_write!(
            output,
            "static inline MalType_{} mal_make_{}(",
            alias.name,
            alias.name
        );
        self.emit_parameters(output, elements);
        c_write!(output, ") {{ return (MalType_{}){{", alias.name);
        for index in 0..elements.len() {
            c_write!(output, " .field_{index} = value_{index},");
        }
        output.push_str(" }; }\n");
    }

    fn emit_product_accessors(
        &self,
        output: &mut String,
        alias: &closure::TypeAlias,
        elements: &[Type],
    ) {
        for (index, element) in elements.iter().enumerate() {
            c_line!(
                output,
                0,
                "static inline {} mal_get_{}_{index}(MalType_{} value) {{ return value.field_{index}; }}",
                self.c_type(element),
                alias.name,
                alias.name
            );
        }
    }

    fn emit_sum_helpers(&self, output: &mut String, alias: &closure::TypeAlias, members: &[Type]) {
        for index in 0..members.len() {
            c_line!(
                output,
                0,
                "#define MAL_TAG_{}_{index} UINT32_C({index})",
                alias.name
            );
        }
        c_line!(
            output,
            0,
            "static inline uint32_t mal_tag_{}(MalType_{} value) {{ return value.tag; }}",
            alias.name,
            alias.name
        );
        for (index, member) in members.iter().enumerate() {
            c_line!(
                output,
                0,
                "static inline uint8_t mal_is_{}_{index}(MalType_{} value) {{ return value.tag == MAL_TAG_{}_{index}; }}",
                alias.name,
                alias.name,
                alias.name
            );
            self.emit_sum_constructor(output, alias, index, member);
            self.emit_sum_accessors(output, alias, index, member);
        }
    }

    fn emit_sum_constructor(
        &self,
        output: &mut String,
        alias: &closure::TypeAlias,
        index: usize,
        member: &Type,
    ) {
        c_write!(
            output,
            "static inline MalType_{} mal_make_{}_{index}(",
            alias.name,
            alias.name
        );
        match member {
            Type::Unit => output.push_str("void"),
            Type::Product(elements) => self.emit_parameters(output, elements),
            _ => c_write!(output, "{} value", self.c_type(member)),
        }
        c_write!(
            output,
            ") {{ return (MalType_{}){{ .tag = MAL_TAG_{}_{index}, .payload.variant_{index} = ",
            alias.name,
            alias.name
        );
        match member {
            Type::Unit => output.push_str("{ .unused = UINT8_C(0) }"),
            Type::Product(elements) => {
                output.push('{');
                for element_index in 0..elements.len() {
                    c_write!(output, " .field_{element_index} = value_{element_index},");
                }
                output.push_str(" }");
            }
            _ => output.push_str("value"),
        }
        output.push_str(" }; }\n");
    }

    fn emit_sum_accessors(
        &self,
        output: &mut String,
        alias: &closure::TypeAlias,
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
                        "static inline {} mal_get_{}_{index}_{element_index}(MalContext *context, MalType_{} value) {{ if (!mal_is_{}_{index}(value)) mal_trap(context, \"expected {} variant {index}\"); return value.payload.variant_{index}.field_{element_index}; }}",
                        self.c_type(element),
                        alias.name,
                        alias.name,
                        alias.name,
                        alias.name
                    );
                }
            }
            _ => c_line!(
                output,
                0,
                "static inline {} mal_get_{}_{index}(MalContext *context, MalType_{} value) {{ if (!mal_is_{}_{index}(value)) mal_trap(context, \"expected {} variant {index}\"); return value.payload.variant_{index}; }}",
                self.c_type(member),
                alias.name,
                alias.name,
                alias.name,
                alias.name
            ),
        }
    }

    fn emit_parameters(&self, output: &mut String, elements: &[Type]) {
        for (index, element) in elements.iter().enumerate() {
            if index != 0 {
                output.push_str(", ");
            }
            c_write!(output, "{} value_{index}", self.c_type(element));
        }
    }
}
