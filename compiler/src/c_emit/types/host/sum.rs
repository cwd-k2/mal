use crate::c_emit::syntax::{
    Block, Directive, Expr, FunctionSignature, Initializer, Parameter, Statement, TranslationUnit,
    TypeName,
};
use crate::check::ast::Type;
use crate::core::ast::TypeAlias;

use super::{TypeRegistry, append_function};

impl TypeRegistry {
    pub(super) fn emit_sum_helpers(
        &self,
        output: &mut TranslationUnit,
        alias: &TypeAlias,
        members: &[Type],
    ) {
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

    pub(super) fn emit_sum_constructor(
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

    pub(super) fn emit_sum_accessors(
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
                        Block::new([Statement::call(
                            "mal_trap",
                            [
                                Expr::identifier("context"),
                                Expr::string(format!("expected {} variant {index}", alias.name)),
                            ],
                        )]),
                    ),
                    Statement::return_value(result),
                ]),
            );
        }
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
