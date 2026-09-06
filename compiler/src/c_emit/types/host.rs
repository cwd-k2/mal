use crate::c_emit::syntax::{
    Block, Declaration, Directive, Expr, FunctionDefinition, Initializer, Statement,
};
use crate::check::ast::Type;
use crate::core::ast::TypeAlias;

use super::{HostTypes, TypeRegistry, is_bool};

impl TypeRegistry {
    pub(in crate::c_emit) fn header_declarations(&self, host: &HostTypes) -> String {
        let mut output = String::new();
        for name in &host.opaque_names {
            output.push_str(
                &Declaration::new(format!(
                    "typedef struct {{ uintptr_t bits; }} MalType_{name}"
                ))
                .render(),
            );
        }
        if !host.opaque_names.is_empty() {
            output.push('\n');
        }
        output.push_str(&self.declarations(host, true));
        output
    }

    pub(in crate::c_emit) fn header_alias_declarations(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
    ) -> String {
        let mut output = String::new();
        for alias in aliases {
            if host.contains(&alias.ty) {
                output.push_str(
                    &Declaration::new(format!(
                        "typedef {} MalType_{}",
                        self.c_type(&alias.ty),
                        alias.name
                    ))
                    .render(),
                );
            }
        }
        if !output.is_empty() {
            output.push('\n');
        }
        output
    }

    pub(in crate::c_emit) fn header_opaque_helpers(&self, host: &HostTypes) -> String {
        let mut output = String::new();
        for name in &host.opaque_names {
            append_function(
                &mut output,
                format!("static inline MalType_{name} mal_{name}_from_bits(uintptr_t bits)"),
                Block::new([Statement::return_value(Expr::compound_literal(
                    format!("MalType_{name}"),
                    [Initializer::designated("bits", Expr::identifier("bits"))],
                ))]),
            );
            append_function(
                &mut output,
                format!("static inline uintptr_t mal_{name}_bits(MalType_{name} value)"),
                Block::new([Statement::return_value(
                    Expr::identifier("value").field("bits"),
                )]),
            );
        }
        output
    }

    pub(in crate::c_emit) fn header_alias_helpers(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
    ) -> String {
        let mut output = String::new();
        for alias in aliases {
            if !host.contains(&alias.ty) {
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
        }
        output
    }

    pub(in crate::c_emit) fn header_c_type(&self, ty: &Type, alias: Option<&str>) -> String {
        alias.map_or_else(|| self.c_type(ty), |alias| format!("MalType_{alias}"))
    }

    fn emit_product_constructor(&self, output: &mut String, alias: &TypeAlias, elements: &[Type]) {
        let parameters = self.parameters(elements);
        let fields = elements.iter().enumerate().map(|(index, _)| {
            Initializer::designated(
                format!("field_{index}"),
                Expr::identifier(format!("value_{index}")),
            )
        });
        append_function(
            output,
            format!(
                "static inline MalType_{} mal_{}_make({parameters})",
                alias.name, alias.name
            ),
            Block::new([Statement::return_value(Expr::compound_literal(
                format!("MalType_{}", alias.name),
                fields,
            ))]),
        );
    }

    fn emit_product_accessors(&self, output: &mut String, alias: &TypeAlias, elements: &[Type]) {
        for (index, element) in elements.iter().enumerate() {
            append_function(
                output,
                format!(
                    "static inline {} mal_{}_get_{index}(MalType_{} value)",
                    self.c_type(element),
                    alias.name,
                    alias.name
                ),
                Block::new([Statement::return_value(
                    Expr::identifier("value").field(format!("field_{index}")),
                )]),
            );
        }
    }

    fn emit_sum_helpers(&self, output: &mut String, alias: &TypeAlias, members: &[Type]) {
        for index in 0..members.len() {
            output.push_str(
                &Directive::define(
                    format!("MAL_{}_TAG_{index}", alias.name),
                    format!("UINT32_C({index})"),
                )
                .render(),
            );
        }
        append_function(
            output,
            format!(
                "static inline uint32_t mal_{}_tag(MalType_{} value)",
                alias.name, alias.name
            ),
            Block::new([Statement::return_value(
                Expr::identifier("value").field("tag"),
            )]),
        );
        for (index, member) in members.iter().enumerate() {
            append_function(
                output,
                format!(
                    "static inline MalType_Bool mal_{}_is_{index}(MalType_{} value)",
                    alias.name, alias.name
                ),
                Block::new([Statement::return_value(Expr::binary(
                    "==",
                    Expr::identifier("value").field("tag"),
                    Expr::identifier(format!("MAL_{}_TAG_{index}", alias.name)),
                ))]),
            );
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
        let (parameters, payload) = match member {
            Type::Unit => ("void".into(), unit()),
            Type::Product(elements) => {
                let fields = elements.iter().enumerate().map(|(element_index, _)| {
                    Initializer::designated(
                        format!("field_{element_index}"),
                        Expr::identifier(format!("value_{element_index}")),
                    )
                });
                (
                    self.parameters(elements),
                    Expr::compound_literal(self.c_type(member), fields),
                )
            }
            _ => (
                format!("{} value", self.c_type(member)),
                Expr::identifier("value"),
            ),
        };
        append_function(
            output,
            format!(
                "static inline MalType_{} mal_{}_make_{index}({parameters})",
                alias.name, alias.name
            ),
            Block::new([Statement::return_value(Expr::compound_literal(
                format!("MalType_{}", alias.name),
                [
                    Initializer::designated(
                        "tag",
                        Expr::identifier(format!("MAL_{}_TAG_{index}", alias.name)),
                    ),
                    Initializer::designated(format!("payload.variant_{index}"), payload),
                ],
            ))]),
        );
    }

    fn emit_sum_accessors(
        &self,
        output: &mut String,
        alias: &TypeAlias,
        index: usize,
        member: &Type,
    ) {
        let elements: Vec<(Option<usize>, &Type)> = match member {
            Type::Unit => return,
            Type::Product(elements) => elements
                .iter()
                .enumerate()
                .map(|(i, ty)| (Some(i), ty))
                .collect(),
            ty => vec![(None, ty)],
        };
        for (element_index, element) in elements {
            let suffix = element_index.map_or_else(String::new, |i| format!("_{i}"));
            let result = element_index.map_or_else(
                || {
                    Expr::identifier("value")
                        .field("payload")
                        .field(format!("variant_{index}"))
                },
                |i| {
                    Expr::identifier("value")
                        .field("payload")
                        .field(format!("variant_{index}"))
                        .field(format!("field_{i}"))
                },
            );
            append_function(
                output,
                format!(
                    "static inline {} mal_{}_expect_{index}{suffix}(MalContext *context, MalType_{} value)",
                    self.c_type(element),
                    alias.name,
                    alias.name
                ),
                Block::new([
                    Statement::if_then(
                        Expr::unary(
                            "!",
                            Expr::named_call(
                                format!("mal_{}_is_{index}", alias.name),
                                [Expr::identifier("value")],
                            ),
                        ),
                        Block::new([Statement::expression(Expr::named_call(
                            "mal_trap",
                            [
                                Expr::identifier("context"),
                                Expr::literal(format!(
                                    "\"expected {} variant {index}\"",
                                    alias.name
                                )),
                            ],
                        ))]),
                    ),
                    Statement::return_value(result),
                ]),
            );
        }
    }

    fn parameters(&self, elements: &[Type]) -> String {
        elements
            .iter()
            .enumerate()
            .map(|(index, element)| format!("{} value_{index}", self.c_type(element)))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn unit() -> Expr {
    Expr::compound_literal(
        "MalType_Unit",
        [Initializer::designated(
            "unused",
            Expr::named_call("UINT8_C", [Expr::literal("0")]),
        )],
    )
}

fn append_function(output: &mut String, signature: impl Into<String>, body: Block) {
    output.push_str(&FunctionDefinition::new(signature, body).render());
    output.push('\n');
}
