use crate::backend::c::syntax::{
    Block, Expr, FunctionSignature, TranslationUnit, TypeName, c_block, c_directive, c_expr,
    c_function, c_initializer, c_parameters, c_signature, c_switch_case, c_type,
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
                    append_function(
                        &mut output,
                        c_signature! {
                            #[static] #[inline] fn #{ format!("mal_repr_product_{id}_return") }(
                                #[maybe_unused] "call": ptr(named("mal_call_t")),
                                "value": named(#{ format!("mal_repr_product_{id}_t") }),
                            ) -> #{ self.c_type(ty) }
                        },
                        c_block! {
                            return #{
                                    self.host_to_raw_value(
                                        ty,
                                        c_expr!(id("call")),
                                        c_expr!(id("value")),
                                    )
                                };
                        },
                    )
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
            append_function(
                &mut output,
                c_signature! {
                    #[static] #[inline] fn #{ format!("mal_{}_return", alias.name) }(
                        #[maybe_unused] "call": ptr(named("mal_call_t")),
                        "value": named(#{ format!("mal_{}_t", alias.name) }),
                    ) -> #{ self.header_c_type(&alias.ty, Some(&alias.name)) }
                },
                c_block! {
                    return #{
                            self.host_to_raw_value(
                                &alias.ty,
                                c_expr!(id("call")),
                                c_expr!(id("value")),
                            )
                        };
                },
            );
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
        for (variant, member) in members.iter().enumerate() {
            let tag_name = format!("mal_{public_name}_tag_{variant}");
            let macro_name = tag_name.clone();
            output.push(c_directive! {
                define #{ macro_name } =
                    (call("UINT32_C", [number(#{ variant })]));
            });
            let (parameters, payload) = if *member == Type::Unit {
                (
                    Vec::new(),
                    c_expr!(compound((named("mal_Unit_t")), [positional((number(0)))])),
                )
            } else {
                (
                    c_parameters! {
                        "value": #{
                            self.host_value_c_type(member, element_aliases[variant].as_deref())
                        }
                    },
                    c_expr!(id("value")),
                )
            };
            let host_value = c_expr! {
                compound(
                    #{ c_type!(named(#{ host_type })) },
                    [
                        field("tag", (id(#{ tag_name }))),
                        path(#{ ["payload".into(), format!("variant_{variant}")] }, #{ payload }),
                    ]
                )
            };
            append_function(
                output,
                c_signature! {
                    #[static] #[inline] fn #{ format!("mal_{public_name}_make_{variant}") }(
                        ...#{ parameters.clone() },
                    ) -> named(#{ host_type })
                },
                c_block! {
                    return #{ host_value.clone() };
                },
            );
            let mut return_parameters = c_parameters!("call": ptr(named("mal_call_t")));
            return_parameters.extend(parameters);
            append_function(
                output,
                c_signature! {
                    #[static] #[inline] fn #{ format!("mal_{public_name}_return_{variant}") }(
                        ...#{ return_parameters },
                    ) -> #{ raw_type.clone() }
                },
                c_block! {
                    return #{ self.host_to_raw_value(ty, c_expr!(id("call")), host_value) };
                },
            );
        }
    }

    fn host_sum_conversion_helpers(&self, index: RepresentationId, ty: &Type) -> TranslationUnit {
        let Type::Sum(members) = ty else {
            unreachable!("sum conversion requires a sum type")
        };
        let raw_type = self.c_type(ty);
        let host_type = self.host_value_c_type(ty, None);
        let mut to_host_cases = Vec::new();
        let mut to_raw_cases = Vec::new();
        for (variant, member) in members.iter().enumerate() {
            let tag = c_expr!(call("UINT32_C", [number(#{ variant })]));
            let payload = c_expr! {
                field((field((id("value")), "payload")), #{ format!("variant_{variant}") })
            };
            let host_payload =
                self.raw_to_host_value(member, None, c_expr!(id("call")), payload.clone());
            to_host_cases.push(c_switch_case! {
                #{ tag.clone() } => {
                    return (compound(#{ host_type.clone() }, [
                        field("tag", #{ tag.clone() }),
                        path(
                            #{ ["payload".into(), format!("variant_{variant}")] },
                            #{ host_payload }
                        ),
                    ]));
                }
            });
            let raw_payload = self.host_to_raw_value(member, c_expr!(id("call")), payload);
            to_raw_cases.push(c_switch_case! {
                #{ tag.clone() } => {
                    return (compound(#{ raw_type.clone() }, [
                        field("tag", #{ tag }),
                        path(
                            #{ ["payload".into(), format!("variant_{variant}")] },
                            #{ raw_payload }
                        ),
                    ]));
                }
            });
        }
        for cases in [&mut to_host_cases, &mut to_raw_cases] {
            cases.push(c_switch_case! {
                _ => {
                    call("mal_call_trap", [id("call"), string("invalid sum tag")]);
                }
            });
        }
        let mut output = TranslationUnit::default();
        output.push(c_function! {
            #[static] #[inline] fn #{ format!("mal_detail_to_host_{index}") }(
                "call": ptr(named("mal_call_t")),
                "value": #{ raw_type.clone() },
            ) -> #{ host_type.clone() } {
                switch (field((id("value")), "tag")) {
                    ...#{ to_host_cases },
                }
            }
        });
        output.blank_line();
        output.push(c_function! {
            #[static] #[inline] fn #{ format!("mal_detail_to_raw_{index}") }(
                "call": ptr(named("mal_call_t")),
                "value": #{ host_type },
            ) -> #{ raw_type } {
                switch (field((id("value")), "tag")) {
                    ...#{ to_raw_cases },
                }
            }
        });
        output.blank_line();
        output
    }

    fn host_to_raw_value(&self, ty: &Type, call: Expr, value: Expr) -> Expr {
        match ty {
            Type::Product(elements) => {
                let initializers = elements.iter().enumerate().map(|(field, element)| {
                    c_initializer! {
                        field(#{ format!("field_{field}") }, #{
                                self.host_to_raw_value(
                                    element,
                                    call.clone(),
                                    c_expr! {
                                        field(#{ value.clone() }, #{ format!("field_{field}") })
                                    },
                                )
                            })
                    }
                });
                c_expr!(compound(#{ self.c_type(ty) }, [...#{ initializers }]))
            }
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
            Type::Product(elements) => {
                let initializers = elements.iter().enumerate().map(|(field, element)| {
                    c_initializer! {
                        field(#{ format!("field_{field}") }, #{
                                self.raw_to_host_value(
                                    element,
                                    None,
                                    call.clone(),
                                    c_expr! {
                                        field(#{ value.clone() }, #{ format!("field_{field}") })
                                    },
                                )
                            })
                    }
                });
                c_expr! {
                    compound(
                        #{ self.host_value_c_type(ty, alias) },
                        [...#{ initializers }]
                    )
                }
            }
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
