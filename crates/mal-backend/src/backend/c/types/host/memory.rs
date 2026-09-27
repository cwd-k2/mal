use crate::backend::c::syntax::{
    Block, Expr, Statement, TranslationUnit, c_block, c_expr, c_parameter, c_signature,
    c_statement, c_switch_case, c_type,
};
use crate::backend::source_layout::SourceLayouts;
use crate::core::ast::TypeAlias;
use mal_frontend::check::ast::Type;

use super::super::{HostTypes, TypeRegistry, is_bool};
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
        let tag_types = self
            .aggregates
            .iter()
            .filter(|ty| host.memory_contains(ty) && !is_bool(ty))
            .filter_map(|ty| layouts.sum(ty).map(|layout| integer_type(layout.tag_bits)))
            .collect::<Vec<_>>();
        for ty in scalar_types() {
            if host.memory_contains(&ty) || tag_types.contains(&ty) {
                self.append_scalar_memory_helpers(&mut output, &ty);
            }
        }
        for (index, ty) in self.aggregates.iter().enumerate() {
            if host.memory_contains(ty) && !is_bool(ty) {
                self.append_aggregate_memory_helpers(&mut output, index, ty, layouts);
            }
        }
        for alias in aliases.iter().filter(|alias| alias.host_memory_access) {
            self.append_alias_memory_helpers(&mut output, alias, layouts);
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

    fn append_aggregate_memory_helpers(
        &self,
        output: &mut TranslationUnit,
        index: usize,
        ty: &Type,
        layouts: SourceLayouts,
    ) {
        let host_type = self.host_value_c_type(ty, None);
        let read_body =
            match ty {
                Type::Product(elements) => {
                    let fields = layouts
                        .product_fields(ty)
                        .expect("checker-approved memory product has a layout");
                    let field_reads = elements.iter().zip(fields).enumerate().map(
                        |(field, (element, layout))| {
                            c_statement! {
                                assign(
                                    (field((id("value")), #{ format!("field_{field}") })),
                                    #{ self.memory_read_value(
                                        element,
                                        c_expr!(id("call")),
                                        offset(c_expr!(id("source")), layout.offset),
                                    ) }
                                );
                            }
                        },
                    );
                    c_block! {
                        let "value": #{ host_type.clone() };
                        ...#{ field_reads }
                        return (id("value"));
                    }
                }
                Type::Sum(members) => self.sum_memory_read_body(ty, members, layouts),
                _ => unreachable!("only aggregate types have representation identities"),
            };
        append_function(
            output,
            c_signature! {
                #[static] #[inline] fn #{ format!("mal_detail_memory_read_{index}") }(
                    #[maybe_unused] "call": ptr(named("mal_call_t")),
                    #[maybe_unused] "source": ptr(const(named("uint8_t"))),
                ) -> #{ host_type.clone() }
            },
            read_body,
        );

        let write_body = match ty {
            Type::Product(elements) => {
                let fields = layouts
                    .product_fields(ty)
                    .expect("checker-approved memory product has a layout");
                c_block! {
                    ...#{
                        elements.iter().zip(fields).enumerate().map(
                            |(field, (element, layout))| {
                                self.memory_write_statement(
                                    element,
                                    c_expr!(id("call")),
                                    offset(c_expr!(id("destination")), layout.offset),
                                    c_expr! {
                                        field(
                                            (id("value")),
                                            #{ format!("field_{field}") }
                                        )
                                    },
                                )
                            },
                        )
                    }
                }
            }
            Type::Sum(members) => self.sum_memory_write_body(ty, members, layouts),
            _ => unreachable!("only aggregate types have representation identities"),
        };
        append_function(
            output,
            c_signature! {
                #[static] #[inline] fn #{ format!("mal_detail_memory_write_{index}") }(
                    #[maybe_unused] "call": ptr(named("mal_call_t")),
                    #[maybe_unused] "destination": ptr(named("uint8_t")),
                    "value": #{ host_type },
                ) -> named("void")
            },
            write_body,
        );
    }

    fn sum_memory_read_body(&self, ty: &Type, members: &[Type], layouts: SourceLayouts) -> Block {
        let layout = layouts
            .sum(ty)
            .expect("checker-approved memory sum has a layout");
        let tag_type = integer_type(layout.tag_bits);
        let cases = members
            .iter()
            .enumerate()
            .map(|(variant, member)| {
                c_switch_case! {
                    (number(#{ variant })) => {
                        return (compound(#{ self.host_value_c_type(ty, None) }, [
                            field("tag", (call("UINT32_C", [number(#{ variant })]))),
                            path(#{ ["payload".into(), format!("variant_{variant}")] },
                                #{ self.memory_read_value(
                                    member,
                                    c_expr!(id("call")),
                                    offset(c_expr!(id("source")), layout.payload_offset),
                                ) }
                            ),
                        ]));
                    }
                }
            })
            .collect::<Vec<_>>();
        c_block! {
            switch #{ self.memory_read_value(
                &tag_type,
                c_expr!(id("call")),
                c_expr!(id("source")),
            ) } {
                ...#{ cases },
                _ => {
                    call("mal_call_trap", [
                        id("call"),
                        string("invalid canonical sum tag"),
                    ]);
                },
            }
        }
    }

    fn sum_memory_write_body(&self, ty: &Type, members: &[Type], layouts: SourceLayouts) -> Block {
        let layout = layouts
            .sum(ty)
            .expect("checker-approved memory sum has a layout");
        let tag_type = integer_type(layout.tag_bits);
        let cases: Vec<_> = members
            .iter()
            .enumerate()
            .map(|(variant, member)| {
                let tag_value = c_expr! {
                    cast(
                        #{ self.host_value_c_type(&tag_type, None) },
                        (field((id("value")), "tag"))
                    )
                };
                let payload = c_expr! {
                    field((field((id("value")), "payload")), #{ format!("variant_{variant}") })
                };
                c_switch_case! {
                    (call("UINT32_C", [number(#{ variant })])) => {
                        #{ self.memory_write_statement(
                            &tag_type,
                            c_expr!(id("call")),
                            c_expr!(id("destination")),
                            tag_value,
                        ) }
                        #{ self.memory_write_statement(
                            member,
                            c_expr!(id("call")),
                            offset(c_expr!(id("destination")), layout.payload_offset),
                            payload,
                        ) }
                        return;
                    }
                }
            })
            .chain([c_switch_case! {
                _ => {
                    call("mal_call_trap", [id("call"), string("invalid sum tag")]);
                }
            }])
            .collect();
        c_block! {
            switch (field((id("value")), "tag")) {
                ...#{ cases },
            }
        }
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
