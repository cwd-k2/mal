use crate::backend::c::syntax::{
    Directive, Expr, MacroInvocation, Statement, TranslationUnit, c_block, c_directive, c_expr,
    c_macro_invocation, c_parameter, c_signature, c_statement, c_type,
};
use crate::backend::source_layout::SourceLayouts;
use crate::core::ast::TypeAlias;
use mal_frontend::check::ast::Type;

use super::super::{HostTypes, RepresentationId, TypeRegistry, is_bool};
use super::append_function;

impl TypeRegistry {
    pub(in crate::backend::c) fn memory_helpers(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
        layouts: SourceLayouts,
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        if !aliases.iter().any(|alias| alias.host_memory_access) {
            return output;
        }
        for ty in &self.aggregates {
            if host.memory_contains(ty) && !is_bool(ty) {
                let guard = format!("MAL_DETAIL_MEMORY_REPR_{}_HELPERS", self.index(ty));
                output.push(c_directive!(ifndef #{ guard.clone() }));
                output.push(c_directive!(define #{ guard };));
                match ty {
                    Type::Product(elements) => self.append_product_memory_template(
                        &mut output,
                        self.index(ty),
                        ty,
                        elements,
                        layouts,
                    ),
                    Type::Sum(members) => self.append_sum_memory_template(
                        &mut output,
                        self.index(ty),
                        ty,
                        members,
                        layouts,
                    ),
                    _ => unreachable!("only aggregate types have representation identities"),
                }
                output.push(c_directive!(endif));
                output.blank_line();
            }
        }
        for alias in aliases.iter().filter(|alias| alias.host_memory_access) {
            self.append_alias_memory_helpers(&mut output, alias, layouts);
        }
        output
    }

    fn append_product_memory_template(
        &self,
        output: &mut TranslationUnit,
        index: RepresentationId,
        ty: &Type,
        elements: &[Type],
        layouts: SourceLayouts,
    ) {
        let fields = layouts
            .product_fields(ty)
            .expect("checker-approved memory product has a layout");
        let descriptor = format!("MAL_DETAIL_MEMORY_FIELDS_{index}");
        let invocations =
            elements
                .iter()
                .zip(fields)
                .enumerate()
                .map(|(field, (element, layout))| {
                    let member = c_expr!(id(#{ format!("field_{field}") }));
                    if matches!(element, Type::Unit) {
                        return MacroInvocation::new("unit", [member]);
                    }
                    let helper = match element {
                        Type::Product(_) | Type::Sum(_) if !is_bool(element) => {
                            self.index(element).to_string()
                        }
                        _ => scalar_name(element).into(),
                    };
                    MacroInvocation::new(
                        "value",
                        [
                            member,
                            c_expr!(id(#{ format!("mal_detail_memory_read_{helper}") })),
                            c_expr!(id(#{ format!("mal_detail_memory_write_{helper}") })),
                            c_expr!(number(#{ layout.offset })),
                        ],
                    )
                });
        output.push(Directive::invocations_define(
            descriptor.clone(),
            ["unit", "value"],
            invocations,
        ));
        output.push(c_macro_invocation! {
            "MAL_DETAIL_DEFINE_MEMORY_PRODUCT"([
                id(#{ format!("mal_detail_memory_read_{index}") }),
                id(#{ format!("mal_detail_memory_write_{index}") }),
                id(#{ self.host_value_c_type(ty, None).to_string() }),
                id(#{ descriptor }),
            ])
        });
    }

    fn append_sum_memory_template(
        &self,
        output: &mut TranslationUnit,
        index: RepresentationId,
        ty: &Type,
        members: &[Type],
        layouts: SourceLayouts,
    ) {
        let layout = layouts
            .sum(ty)
            .expect("checker-approved memory sum has a layout");
        let tag_type = integer_type(layout.tag_bits);
        let tag_name = scalar_name(&tag_type);
        let value_type = self.host_value_c_type(ty, None).to_string();
        let descriptor = format!("MAL_DETAIL_MEMORY_MEMBERS_{index}");
        let invocations = members.iter().enumerate().map(|(variant, member)| {
            let helper = match member {
                Type::Unit => "Unit".into(),
                Type::Product(_) | Type::Sum(_) if !is_bool(member) => {
                    self.index(member).to_string()
                }
                _ => scalar_name(member).into(),
            };
            MacroInvocation::new(
                "member",
                [
                    c_expr!(id(#{ value_type.clone() })),
                    c_expr!(id(#{ format!("mal_{tag_name}_t") })),
                    c_expr!(id(#{ format!("mal_detail_memory_write_{tag_name}") })),
                    c_expr!(number(#{ variant })),
                    c_expr!(id(#{ format!("variant_{variant}") })),
                    c_expr!(id(#{ format!("mal_detail_memory_read_{helper}") })),
                    c_expr!(id(#{ format!("mal_detail_memory_write_{helper}") })),
                    c_expr!(number(#{ layout.payload_offset })),
                ],
            )
        });
        output.push(Directive::invocations_define(
            descriptor.clone(),
            ["member"],
            invocations,
        ));
        output.push(c_macro_invocation! {
            "MAL_DETAIL_DEFINE_MEMORY_SUM"([
                id(#{ format!("mal_detail_memory_read_{index}") }),
                id(#{ format!("mal_detail_memory_write_{index}") }),
                id(#{ value_type }),
                id(#{ format!("mal_detail_memory_read_{tag_name}") }),
                id(#{ descriptor }),
            ])
        });
    }

    pub(in crate::backend::c) fn common_scalar_memory_helpers(&self) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for ty in scalar_types() {
            self.append_scalar_memory_helpers(&mut output, &ty);
        }
        output
    }

    fn append_scalar_memory_helpers(&self, output: &mut TranslationUnit, ty: &Type) {
        let name = scalar_name(ty);
        let host_type = self.host_value_c_type(ty, None);
        let call = if matches!(ty, Type::Address) || is_bool(ty) {
            c_parameter!("call": ptr(named("mal_call_t")))
        } else {
            c_parameter!(#[maybe_unused] "call": ptr(named("mal_call_t")))
        };
        let value = if matches!(ty, Type::Address) {
            c_expr!(call("mal_Address_return", [id("call"), id("value")]))
        } else if is_bool(ty) {
            c_expr!(call("mal_Bool_return", [id("call"), id("value")]))
        } else {
            c_expr!(id("value"))
        };
        let read_body = c_block! {
            let "value": #{ host_type.clone() };
            call("memcpy", [
                address((id("value"))),
                id("source"),
                sizeof((id("value"))),
            ]);
            return #{ value };
        };
        append_function(
            output,
            c_signature! {
                #[static] #[inline] fn #{ format!("mal_detail_memory_read_{name}") }(
                    #{ call.clone() },
                    "source": ptr(const(named("uint8_t"))),
                ) -> #{ host_type.clone() }
            },
            read_body,
        );

        let validation = if matches!(ty, Type::Address) {
            Some(c_statement!(call("mal_Address_return", [id("call"), id("value")]);))
        } else if is_bool(ty) {
            Some(c_statement!(call("mal_Bool_return", [id("call"), id("value")]);))
        } else {
            None
        };
        let write_body = c_block! {
            ...#{ validation }
            call("memcpy", [
                id("destination"),
                address((id("value"))),
                sizeof((id("value"))),
            ]);
        };
        append_function(
            output,
            c_signature! {
                #[static] #[inline] fn #{ format!("mal_detail_memory_write_{name}") }(
                    #{ call },
                    "destination": ptr(named("uint8_t")),
                    "value": #{ host_type },
                ) -> named("void")
            },
            write_body,
        );
    }

    fn append_alias_memory_helpers(
        &self,
        output: &mut TranslationUnit,
        alias: &TypeAlias,
        layouts: SourceLayouts,
    ) {
        let stride = layouts
            .layout(&alias.ty)
            .expect("checker-approved memory alias has a layout")
            .stride;
        if !matches!(alias.ty, Type::Unit) {
            let helper = match &alias.ty {
                Type::Product(_) | Type::Sum(_) if !is_bool(&alias.ty) => {
                    self.index(&alias.ty).to_string()
                }
                _ => scalar_name(&alias.ty).into(),
            };
            output.push(c_macro_invocation! {
                "MAL_DETAIL_DEFINE_MEMORY_ALIAS"([
                    id(#{ format!("mal_{}_read", alias.name) }),
                    id(#{ format!("mal_{}_write", alias.name) }),
                    id(#{ format!("mal_{}_t", alias.name) }),
                    number(#{ stride }),
                    id(#{ format!("mal_detail_memory_read_{helper}") }),
                    id(#{ format!("mal_detail_memory_write_{helper}") }),
                ])
            });
            output.blank_line();
            return;
        }
        let source = offset(
            c_expr! {
                cast(
                    #{ c_type!(ptr(const(named("uint8_t")))) },
                    (id("address"))
                )
            },
            c_expr!(multiply((id("index")), (number(#{ stride })))),
        );
        let unused_index =
            (stride == 0).then(|| c_statement!(cast((named("void")), (id("index")));));
        let read_value = self.memory_read_value(&alias.ty, c_expr!(id("call")), source);
        let read_body = c_block! {
            ...#{ unused_index }
            call("mal_Address_return", [id("call"), id("address")]);
            return #{ read_value };
        };
        append_function(
            output,
            c_signature! {
                #[static] #[inline] fn #{ format!("mal_{}_read", alias.name) }(
                    "call": ptr(named("mal_call_t")),
                    "address": named("mal_Address_t"),
                    "index": named("mal_USize_t"),
                ) -> named(#{ format!("mal_{}_t", alias.name) })
            },
            read_body,
        );

        let destination = offset(
            c_expr! {
                cast(
                    #{ c_type!(ptr(named("uint8_t"))) },
                    (id("address"))
                )
            },
            c_expr!(multiply((id("index")), (number(#{ stride })))),
        );
        let unused_index =
            (stride == 0).then(|| c_statement!(cast((named("void")), (id("index")));));
        let write_value = self.memory_write_statement(
            &alias.ty,
            c_expr!(id("call")),
            destination,
            c_expr!(id("value")),
        );
        let write_body = c_block! {
            ...#{ unused_index }
            call("mal_Address_return", [id("call"), id("address")]);
            #{ write_value }
        };
        append_function(
            output,
            c_signature! {
                #[static] #[inline] fn #{ format!("mal_{}_write", alias.name) }(
                    "call": ptr(named("mal_call_t")),
                    "address": named("mal_Address_t"),
                    "index": named("mal_USize_t"),
                    "value": named(#{ format!("mal_{}_t", alias.name) }),
                ) -> named("void")
            },
            write_body,
        );
    }

    fn memory_read_value(&self, ty: &Type, call: Expr, source: Expr) -> Expr {
        match ty {
            Type::Unit => c_expr!(compound((named("mal_Unit_t")), [positional((number(0)))])),
            Type::Product(_) | Type::Sum(_) if !is_bool(ty) => c_expr! {
                call(
                    #{ format!("mal_detail_memory_read_{}", self.index(ty)) },
                    [#{ call }, #{ source }]
                )
            },
            _ => c_expr! {
                call(
                    #{ format!("mal_detail_memory_read_{}", scalar_name(ty)) },
                    [#{ call }, #{ source }]
                )
            },
        }
    }

    fn memory_write_statement(
        &self,
        ty: &Type,
        call: Expr,
        destination: Expr,
        value: Expr,
    ) -> Statement {
        if matches!(ty, Type::Unit) {
            return c_statement!(cast((named("void")), #{ value }););
        }
        let name = match ty {
            Type::Product(_) | Type::Sum(_) if !is_bool(ty) => self.index(ty).to_string(),
            _ => scalar_name(ty).into(),
        };
        c_statement! {
            call(#{ format!("mal_detail_memory_write_{name}") }, [
                #{ call },
                #{ destination },
                #{ value },
            ]);
        }
    }
}

fn scalar_types() -> Vec<Type> {
    vec![
        Type::Sum(vec![Type::Unit, Type::Unit].into()),
        Type::Int8,
        Type::Int16,
        Type::Int32,
        Type::Int64,
        Type::UInt8,
        Type::UInt16,
        Type::UInt32,
        Type::UInt64,
        Type::Float32,
        Type::Float64,
        Type::Address,
        Type::ByteSize,
        Type::USize,
    ]
}

fn scalar_name(ty: &Type) -> &'static str {
    if is_bool(ty) {
        return "Bool";
    }
    match ty {
        Type::Int8 => "Int8",
        Type::Int16 => "Int16",
        Type::Int32 => "Int32",
        Type::Int64 => "Int64",
        Type::UInt8 => "UInt8",
        Type::UInt16 => "UInt16",
        Type::UInt32 => "UInt32",
        Type::UInt64 => "UInt64",
        Type::Float32 => "Float32",
        Type::Float64 => "Float64",
        Type::Address => "Address",
        Type::ByteSize => "ByteSize",
        Type::USize => "USize",
        _ => unreachable!("only canonical scalar types have scalar memory helpers"),
    }
}

fn integer_type(bits: usize) -> Type {
    match bits {
        8 => Type::UInt8,
        16 => Type::UInt16,
        32 => Type::UInt32,
        64 => Type::UInt64,
        _ => unreachable!("canonical sum tag widths are fixed-width integers"),
    }
}

fn offset(base: Expr, offset: impl Into<Offset>) -> Expr {
    c_expr!(add(#{ base }, #{ offset.into().0 }))
}

struct Offset(Expr);

impl From<usize> for Offset {
    fn from(value: usize) -> Self {
        Self(c_expr!(number(#{ value })))
    }
}

impl From<Expr> for Offset {
    fn from(value: Expr) -> Self {
        Self(value)
    }
}
