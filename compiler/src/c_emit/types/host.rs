use crate::c_emit::syntax::{
    AggregateDefinition, AggregateField, Block, Declaration, Directive, Expr, FunctionDefinition,
    FunctionSignature, Initializer, Parameter, Statement, TranslationUnit, TypeName,
};
use crate::check::ast::Type;
use crate::core::ast::TypeAlias;

use super::{HostTypes, TypeRegistry, is_bool};

impl TypeRegistry {
    pub(in crate::c_emit) fn header_declarations(&self, host: &HostTypes) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for name in &host.opaque_names {
            output.push(AggregateDefinition::typedef_structure(
                None,
                [AggregateField::variable("uintptr_t", "bits")],
                format!("MalType_{name}"),
            ));
        }
        if !host.opaque_names.is_empty() {
            output.blank_line();
        }
        output.extend(self.declarations(host, true));
        output
    }

    pub(in crate::c_emit) fn header_alias_declarations(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for alias in aliases {
            if host.contains(&alias.ty) {
                output.push(Declaration::type_alias(
                    self.c_type(&alias.ty),
                    format!("MalType_{}", alias.name),
                ));
            }
        }
        if !output.is_empty() {
            output.blank_line();
        }
        output
    }

    pub(in crate::c_emit) fn header_opaque_helpers(&self, host: &HostTypes) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for name in &host.opaque_names {
            append_function(
                &mut output,
                FunctionSignature::static_inline(
                    format!("MalType_{name}"),
                    format!("mal_{name}_from_bits"),
                    [Parameter::named("uintptr_t", "bits")],
                ),
                Block::new([Statement::return_value(Expr::compound_literal(
                    format!("MalType_{name}"),
                    [Initializer::designated("bits", Expr::identifier("bits"))],
                ))]),
            );
            append_function(
                &mut output,
                FunctionSignature::static_inline(
                    "uintptr_t",
                    format!("mal_{name}_bits"),
                    [Parameter::named(format!("MalType_{name}"), "value")],
                ),
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
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
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

    pub(in crate::c_emit) fn header_c_type(&self, ty: &Type, alias: Option<&str>) -> TypeName {
        alias.map_or_else(
            || self.c_type(ty),
            |alias| TypeName::named(format!("MalType_{alias}")),
        )
    }

    fn emit_product_constructor(
        &self,
        output: &mut TranslationUnit,
        alias: &TypeAlias,
        elements: &[Type],
    ) {
        let parameters = self.parameters(elements);
        let fields = elements.iter().enumerate().map(|(index, _)| {
            Initializer::designated(
                format!("field_{index}"),
                Expr::identifier(format!("value_{index}")),
            )
        });
        append_function(
            output,
            FunctionSignature::static_inline(
                format!("MalType_{}", alias.name),
                format!("mal_{}_make", alias.name),
                parameters,
            ),
            Block::new([Statement::return_value(Expr::compound_literal(
                format!("MalType_{}", alias.name),
                fields,
            ))]),
        );
    }

    fn emit_product_accessors(
        &self,
        output: &mut TranslationUnit,
        alias: &TypeAlias,
        elements: &[Type],
    ) {
        for (index, element) in elements.iter().enumerate() {
            append_function(
                output,
                FunctionSignature::static_inline(
                    self.c_type(element),
                    format!("mal_{}_get_{index}", alias.name),
                    [Parameter::named(format!("MalType_{}", alias.name), "value")],
                ),
                Block::new([Statement::return_value(
                    Expr::identifier("value").field(format!("field_{index}")),
                )]),
            );
        }
    }

    fn emit_sum_helpers(&self, output: &mut TranslationUnit, alias: &TypeAlias, members: &[Type]) {
        for index in 0..members.len() {
            output.push(Directive::define_expr(
                format!("MAL_{}_TAG_{index}", alias.name),
                Expr::named_call("UINT32_C", [Expr::number(index.to_string())]),
            ));
        }
        append_function(
            output,
            FunctionSignature::static_inline(
                "uint32_t",
                format!("mal_{}_tag", alias.name),
                [Parameter::named(format!("MalType_{}", alias.name), "value")],
            ),
            Block::new([Statement::return_value(
                Expr::identifier("value").field("tag"),
            )]),
        );
        for (index, member) in members.iter().enumerate() {
            append_function(
                output,
                FunctionSignature::static_inline(
                    "MalType_Bool",
                    format!("mal_{}_is_{index}", alias.name),
                    [Parameter::named(format!("MalType_{}", alias.name), "value")],
                ),
                Block::new([Statement::return_value(Expr::equal(
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
        output: &mut TranslationUnit,
        alias: &TypeAlias,
        index: usize,
        member: &Type,
    ) {
        let (parameters, payload) = match member {
            Type::Unit => (Vec::new(), unit()),
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
                vec![Parameter::named(self.c_type(member), "value")],
                Expr::identifier("value"),
            ),
        };
        append_function(
            output,
            FunctionSignature::static_inline(
                format!("MalType_{}", alias.name),
                format!("mal_{}_make_{index}", alias.name),
                parameters,
            ),
            Block::new([Statement::return_value(Expr::compound_literal(
                format!("MalType_{}", alias.name),
                [
                    Initializer::designated(
                        "tag",
                        Expr::identifier(format!("MAL_{}_TAG_{index}", alias.name)),
                    ),
                    Initializer::designated_path(
                        ["payload".into(), format!("variant_{index}")],
                        payload,
                    ),
                ],
            ))]),
        );
    }

    fn emit_sum_accessors(
        &self,
        output: &mut TranslationUnit,
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
                FunctionSignature::static_inline(
                    self.c_type(element),
                    format!("mal_{}_expect_{index}{suffix}", alias.name),
                    [
                        Parameter::named(TypeName::named("MalContext").pointer(), "context"),
                        Parameter::named(format!("MalType_{}", alias.name), "value"),
                    ],
                ),
                Block::new([
                    Statement::if_then(
                        Expr::logical_not(Expr::named_call(
                            format!("mal_{}_is_{index}", alias.name),
                            [Expr::identifier("value")],
                        )),
                        Block::new([Statement::expression(Expr::named_call(
                            "mal_trap",
                            [
                                Expr::identifier("context"),
                                Expr::string(format!("expected {} variant {index}", alias.name)),
                            ],
                        ))]),
                    ),
                    Statement::return_value(result),
                ]),
            );
        }
    }

    fn parameters(&self, elements: &[Type]) -> Vec<Parameter> {
        elements
            .iter()
            .enumerate()
            .map(|(index, element)| {
                Parameter::named(self.c_type(element), format!("value_{index}"))
            })
            .collect()
    }
}

fn unit() -> Expr {
    Expr::compound_literal(
        "MalType_Unit",
        [Initializer::designated(
            "unused",
            Expr::named_call("UINT8_C", [Expr::number("0")]),
        )],
    )
}

fn append_function(output: &mut TranslationUnit, signature: FunctionSignature, body: Block) {
    output.push(FunctionDefinition::from_signature(signature, body));
    output.blank_line();
}
