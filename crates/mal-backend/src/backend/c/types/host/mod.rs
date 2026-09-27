use crate::backend::c::syntax::{
    Block, Directive, Expr, FunctionSignature, MacroInvocation, TranslationUnit, TypeName, c_block,
    c_directive, c_expr, c_function, c_macro_invocation, c_signature, c_type,
};
use crate::core::ast::TypeAlias;
use mal_frontend::check::ast::Type;

use super::{HostTypes, RepresentationId, TypeRegistry, is_bool};

mod declaration;
mod memory;

impl TypeRegistry {
    pub(in crate::backend::c) fn host_value_helpers(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for name in &host.opaque_names {
            output.push(crate::backend::c::syntax::c_declaration! {
                fn #{ c_signature! {
                    #[static] #[inline] fn #{ format!("mal_detail_to_host_{name}") }(
                        #[maybe_unused] "call": ptr(named("mal_call_t")),
                        "value": named(#{ format!("MalType_{name}") }),
                    ) -> named(#{ format!("mal_{name}_t") })
                } };
            });
            output.push(crate::backend::c::syntax::c_declaration! {
                fn #{ c_signature! {
                    #[static] #[inline] fn #{ format!("mal_{name}_return") }(
                        #[maybe_unused] "call": ptr(named("mal_call_t")),
                        "value": named(#{ format!("mal_{name}_t") }),
                    ) -> named(#{ format!("MalType_{name}") })
                } };
            });
        }
        if !host.opaque_names.is_empty() {
            output.blank_line();
        }
        for ty in &self.aggregates {
            if !host.external_contains(ty) || is_bool(ty) {
                continue;
            }
            let guard = format!("MAL_DETAIL_HOST_REPR_{}_HELPERS", self.index(ty));
            output.push(c_directive!(ifndef #{ guard.clone() }));
            output.push(c_directive!(define #{ guard };));
            match ty {
                Type::Product(_) => {
                    let id = self.index(ty);
                    output.push(c_macro_invocation! {
                        "MAL_DETAIL_DEFINE_PRODUCT_CONVERSIONS"([
                            id(#{ format!("mal_detail_to_host_{id}") }),
                            id(#{ format!("mal_repr_product_{id}_return") }),
                            id(#{ format!("MalRepr_Product_{id}") }),
                            id(#{ format!("mal_repr_product_{id}_t") }),
                            id(#{ format!("MAL_DETAIL_REPR_FIELDS_{id}") }),
                        ])
                    });
                }
                Type::Sum(members) => {
                    let id = self.index(ty);
                    output.extend(self.host_sum_conversion_helpers(id, ty));
                    self.append_host_sum_helpers(
                        &mut output,
                        &format!("repr_sum_{id}"),
                        &format!("mal_repr_sum_{id}_t"),
                        self.c_type(ty),
                        ty,
                        &vec![None; members.len()],
                    );
                }
                _ => unreachable!("only aggregates have structural host helpers"),
            }
            output.push(c_directive!(endif));
            output.blank_line();
        }
        for name in &host.opaque_names {
            let host_type = format!("mal_{name}_t");
            append_function(
                &mut output,
                c_signature! {
                    #[static] #[inline] fn #{ format!("mal_{name}_from_bits") }(
                        "bits": named("uintptr_t"),
                    ) -> named(#{ host_type.clone() })
                },
                c_block! {
                    return (compound(
                        #{ c_type!(named(#{ host_type.clone() })) },
                        [field("mal_detail_bits", (id("bits")))],
                    ));
                },
            );
            append_function(
                &mut output,
                c_signature! {
                    #[static] #[inline] fn #{ format!("mal_detail_to_host_{name}") }(
                        #[maybe_unused] "call": ptr(named("mal_call_t")),
                        "value": named(#{ format!("MalType_{name}") }),
                    ) -> named(#{ host_type.clone() })
                },
                c_block! {
                    return (call(#{ format!("mal_{name}_from_bits") }, [
                        field((id("value")), "bits"),
                    ]));
                },
            );
            append_function(
                &mut output,
                c_signature! {
                    #[static] #[inline] fn #{ format!("mal_{name}_to_bits") }(
                        "value": named(#{ host_type.clone() }),
                    ) -> named("uintptr_t")
                },
                c_block! {
                    return (field((id("value")), "mal_detail_bits"));
                },
            );
            append_function(
                &mut output,
                c_signature! {
                    #[static] #[inline] fn #{ format!("mal_{name}_return") }(
                        #[maybe_unused] "call": ptr(named("mal_call_t")),
                        "value": named(#{ host_type }),
                    ) -> named(#{ format!("MalType_{name}") })
                },
                c_block! {
                    return (compound(
                        #{ c_type!(named(#{ format!("MalType_{name}") })) },
                        [field("bits", (field((id("value")), "mal_detail_bits")))],
                    ));
                },
            );
        }
        for alias in aliases {
            if !host.exposes_external_alias(alias) {
                continue;
            }
            if matches!(&alias.ty, Type::Sum(_)) {
                self.append_host_sum_helpers(
                    &mut output,
                    &alias.name,
                    &format!("mal_{}_t", alias.name),
                    self.header_c_type(&alias.ty, Some(&alias.name)),
                    &alias.ty,
                    &alias.element_aliases,
                );
                continue;
            }
            if matches!(&alias.ty, Type::Product(_)) {
                output.push(c_macro_invocation! {
                    "MAL_DETAIL_DEFINE_CONVERTING_RETURN"([
                        id(#{ format!("mal_{}_return", alias.name) }),
                        id(#{ format!("MalType_{}", alias.name) }),
                        id(#{ format!("mal_{}_t", alias.name) }),
                        id(#{ format!("mal_repr_product_{}_return", self.index(&alias.ty)) }),
                    ])
                });
                continue;
            }
            let conversion = {
                let conversion = format!("MAL_DETAIL_TO_RAW_ALIAS_{}", alias.name);
                output.push(c_directive! {
                    define #{ conversion.clone() } = #{
                        self.host_to_raw_value(
                            &alias.ty,
                            c_expr!(id("call")),
                            c_expr!(id("value")),
                        )
                    };
                });
                conversion
            };
            output.push(c_macro_invocation! {
                "MAL_DETAIL_DEFINE_CONVERSION"([
                    id(#{ format!("mal_{}_return", alias.name) }),
                    id(#{ format!("MalType_{}", alias.name) }),
                    id(#{ format!("mal_{}_t", alias.name) }),
                    id(#{ conversion }),
                ])
            });
        }
        output
    }

    fn append_host_sum_helpers(
        &self,
        output: &mut TranslationUnit,
        public_name: &str,
        host_type: &str,
        raw_type: TypeName,
        ty: &Type,
        element_aliases: &[Option<String>],
    ) {
        let Type::Sum(members) = ty else {
            unreachable!("sum helpers require a sum type")
        };
        let mut variants = Vec::new();
        for (variant, member) in members.iter().enumerate() {
            let tag_name = format!("mal_{public_name}_tag_{variant}");
            let macro_name = tag_name.clone();
            output.push(c_directive! {
                define #{ macro_name } =
                    (call("UINT32_C", [number(#{ variant })]));
            });
            let make_name = format!("mal_{public_name}_make_{variant}");
            let return_name = format!("mal_{public_name}_return_{variant}");
            let member_name = format!("variant_{variant}");
            let to_raw = format!("mal_detail_to_raw_{}", self.index(ty));
            let invocation = if *member == Type::Unit {
                MacroInvocation::new(
                    "unit",
                    [
                        c_expr!(id(#{ make_name })),
                        c_expr!(id(#{ return_name })),
                        c_expr!(id(#{ host_type })),
                        c_expr!(id(#{ raw_type.to_string() })),
                        c_expr!(id(#{ tag_name })),
                        c_expr!(id(#{ member_name })),
                        c_expr!(id(#{ to_raw })),
                    ],
                )
            } else {
                MacroInvocation::new(
                    "value",
                    [
                        c_expr!(id(#{ make_name })),
                        c_expr!(id(#{ return_name })),
                        c_expr!(id(#{ host_type })),
                        c_expr!(id(#{ raw_type.to_string() })),
                        c_expr!(id(#{
                            self.host_value_c_type(
                                member,
                                element_aliases[variant].as_deref(),
                            ).to_string()
                        })),
                        c_expr!(id(#{ tag_name })),
                        c_expr!(id(#{ member_name })),
                        c_expr!(id(#{ to_raw })),
                    ],
                )
            };
            variants.push(invocation);
        }
        let descriptor = format!("MAL_DETAIL_SUM_API_{public_name}");
        output.push(Directive::invocations_define(
            descriptor.clone(),
            ["unit", "value"],
            variants,
        ));
        output.push(c_macro_invocation! {
            #{ descriptor }([id("MAL_DETAIL_DEFINE_SUM_UNIT_API"), id("MAL_DETAIL_DEFINE_SUM_VALUE_API")])
        });
    }

    fn host_sum_conversion_helpers(&self, index: RepresentationId, ty: &Type) -> TranslationUnit {
        let Type::Sum(_) = ty else {
            unreachable!("sum conversion requires a sum type")
        };
        let mut output = TranslationUnit::default();
        output.push(c_macro_invocation! {
            "MAL_DETAIL_DEFINE_SUM_CONVERSIONS"([
                id(#{ format!("mal_detail_to_host_{index}") }),
                id(#{ format!("mal_detail_to_raw_{index}") }),
                id(#{ self.c_type(ty).to_string() }),
                id(#{ self.host_value_c_type(ty, None).to_string() }),
                id(#{ format!("MAL_DETAIL_REPR_FIELDS_{index}") }),
            ])
        });
        output.blank_line();
        output
    }

    fn host_to_raw_value(&self, ty: &Type, call: Expr, value: Expr) -> Expr {
        match ty {
            Type::Product(_) => c_expr! {
                call(
                    #{ format!("mal_repr_product_{}_return", self.index(ty)) },
                    [#{ call }, #{ value }]
                )
            },
            Type::Sum(_) if !is_bool(ty) => c_expr! {
                call(
                    #{ format!("mal_detail_to_raw_{}", self.index(ty)) },
                    [#{ call }, #{ value }]
                )
            },
            Type::Address => c_expr!(call("mal_Address_return", [#{ call }, #{ value }])),
            Type::External { .. } => c_expr! {
                compound(
                    #{ self.c_type(ty) },
                    [field("bits", (field(#{ value }, "mal_detail_bits"))),]
                )
            },
            Type::Unit => c_expr!(compound((named("MalType_Unit")), [positional((number(0)))])),
            Type::Symbol | Type::Function { .. } => {
                unreachable!("type checking excludes functions from extern signatures")
            }
            _ => value,
        }
    }

    pub(in crate::backend::c) fn raw_to_host_value(
        &self,
        ty: &Type,
        alias: Option<&str>,
        call: Expr,
        value: Expr,
    ) -> Expr {
        match ty {
            Type::Product(_) => c_expr! {
                call(
                    #{ format!("mal_detail_to_host_{}", self.index(ty)) },
                    [#{ call }, #{ value }]
                )
            },
            Type::Sum(_) if !is_bool(ty) => c_expr! {
                call(
                    #{ format!("mal_detail_to_host_{}", self.index(ty)) },
                    [#{ call }, #{ value }]
                )
            },
            Type::Address => value,
            Type::External { .. } => c_expr! {
                compound(
                    #{ self.host_value_c_type(ty, alias) },
                    [field("mal_detail_bits", (field(#{ value }, "bits")))]
                )
            },
            Type::Symbol | Type::Function { .. } => {
                unreachable!("type checking excludes functions from extern signatures")
            }
            _ => value,
        }
    }
}

fn append_function(output: &mut TranslationUnit, signature: FunctionSignature, body: Block) {
    output.push(c_function!(signature #{ signature } body #{ body }));
    output.blank_line();
}
