//! Reading the LLVM bridge's raw byte carrier into public C carriers.

use super::*;

impl Marshalling<'_> {
    /// Reads the LLVM bridge's raw byte carrier into the public C carrier for `value`.
    ///
    /// `base` and `offset` address the LLVM-produced result layout; recursive product offsets are
    /// therefore plan offsets, not C field offsets.
    pub(super) fn read(
        &mut self,
        value: &plan::Value<'_>,
        base: Expr,
        offset: usize,
        context: Expr,
    ) -> Option<Expr> {
        let pointer = bridge_pointer(base.clone(), offset, true);
        match &value.kind {
            plan::Kind::Unit => Some(c_expr! {
                compound(
                    (named("MalType_Unit")),
                    [field("unused", (call("UINT8_C", [number(0)]))),]
                )
            }),
            plan::Kind::External => Some(c_expr! {
                compound(
                    #{ self.raw_types.c_type(value.ty) },
                    [field("bits", #{ load(c_type!(ptr(const(named("uintptr_t")))), pointer) })]
                )
            }),
            plan::Kind::Product(fields) => {
                let initializers = fields
                    .iter()
                    .enumerate()
                    .map(|(index, field)| {
                        Some(c_initializer! {
                            field(#{ format!("field_{index}") }, #{
                                    self.read(
                                        &field.value,
                                        base.clone(),
                                        offset.checked_add(field.offset)?,
                                        context.clone(),
                                    )?
                                })
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(c_expr! {
                    compound(
                        #{ self.raw_types.c_type(value.ty) },
                        [...#{ initializers }]
                    )
                })
            }
            plan::Kind::Sum {
                tag_offset,
                variants,
            } => self.read_sum(value.ty, *tag_offset, variants, pointer, context),
            plan::Kind::Scalar => Some(load(
                c_type!(ptr(const(named(#{ c_scalar_type(value.ty)? })))),
                pointer,
            )),
        }
    }

    /// Builds or reuses the C helper that validates a raw LLVM sum tag and constructs its public
    /// tagged carrier. Invalid tags trap instead of selecting an arbitrary union member.
    fn read_sum(
        &mut self,
        ty: &Type,
        tag_offset: usize,
        variants: &[plan::Field<'_>],
        pointer: Expr,
        context: Expr,
    ) -> Option<Expr> {
        if let Some(helper) = ty.shared_id().and_then(|id| self.read_helpers.get(&id)) {
            return Some(c_expr! {
                call(
                    #{ helper.clone() },
                    [#{ context }, #{ pointer }]
                )
            });
        }
        let helper = self.helper_name("read");
        if let Some(id) = ty.shared_id() {
            self.read_helpers.insert(id, helper.clone());
        }
        let c_type = self.raw_types.c_type(ty);
        let cases = variants
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let payload = self.read(
                    &field.value,
                    identifier("value"),
                    field.offset,
                    identifier("context"),
                )?;
                Some(c_switch_case! {
                    (call("UINT32_C", [number(#{ index })])) => {
                        return (compound(#{ c_type.clone() }, [
                            field("tag", (call("UINT32_C", [number(#{ index })]))),
                            path(
                                #{ ["payload".into(), format!("variant_{index}")] },
                                #{ payload }
                            ),
                        ]));
                    }
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let mut cases = cases;
        cases.push(c_switch_case! {
            _ => { #{ trap(
                identifier("context"),
                "invalid sum tag at LLVM bridge",
            ) } }
        });
        let tag = load(
            c_type!(ptr(const(named("uint32_t")))),
            c_expr!(add((id("value")), (number(#{ tag_offset })))),
        );
        self.helpers.push(c_function! {
            #[static] fn #{ helper.clone() }(
                "context": ptr(named("MalContext")),
                "value": ptr(const(named("uint8_t"))),
            ) -> #{ c_type } {
                let "tag": named("uint32_t") = #{ tag };
                switch (id("tag")) {
                    ...#{ cases },
                }
            }
        });
        self.helpers.blank_line();
        Some(c_expr! {
            call(
                #{ helper },
                [#{ context }, #{ pointer }]
            )
        })
    }
}
