//! Writing public C carriers into the LLVM bridge's raw byte carrier.

use super::*;

impl Marshalling<'_> {
    pub(super) fn write(
        &mut self,
        plan: &plan::Value<'_>,
        value: Expr,
        offset: usize,
        context: Expr,
    ) -> Option<Vec<Statement>> {
        self.write_at(plan, identifier("mal_result"), value, offset, context)
    }

    /// Writes a public C carrier into the raw byte layout consumed by LLVM.
    ///
    /// This is the reverse direction of `read`; `base` always denotes bridge storage rather than
    /// the address of the C aggregate passed as `value`.
    fn write_at(
        &mut self,
        plan: &plan::Value<'_>,
        base: Expr,
        value: Expr,
        offset: usize,
        context: Expr,
    ) -> Option<Vec<Statement>> {
        let pointer = bridge_pointer(base.clone(), offset, false);
        match &plan.kind {
            plan::Kind::Unit => Some(vec![store(
                "uint8_t",
                pointer,
                c_expr!(call("UINT8_C", [number(0)])),
            )]),
            plan::Kind::External => Some(vec![store(
                "uintptr_t",
                pointer,
                c_expr!(field(#{ value }, "bits")),
            )]),
            plan::Kind::Product(fields) => {
                let mut statements = Vec::new();
                for (index, field) in fields.iter().enumerate() {
                    statements.extend(self.write_at(
                        &field.value,
                        base.clone(),
                        c_expr! {
                            field(#{ value.clone() }, #{ format!("field_{index}") })
                        },
                        offset.checked_add(field.offset)?,
                        context.clone(),
                    )?);
                }
                Some(statements)
            }
            plan::Kind::Sum {
                tag_offset,
                variants,
            } => self
                .write_sum(plan.ty, *tag_offset, variants, value, pointer, context)
                .map(|call| vec![call]),
            plan::Kind::Scalar => Some(vec![store(c_scalar_type(plan.ty)?, pointer, value)]),
        }
    }

    /// Builds or reuses one writer per shared sum type and traps on an invalid public tag.
    fn write_sum(
        &mut self,
        ty: &Type,
        tag_offset: usize,
        variants: &[plan::Field<'_>],
        value: Expr,
        pointer: Expr,
        context: Expr,
    ) -> Option<Statement> {
        if let Some(helper) = ty.shared_id().and_then(|id| self.write_helpers.get(&id)) {
            return Some(c_statement! {
                call(#{ helper.clone() }, [
                    #{ context },
                    #{ pointer },
                    #{ value },
                ]);
            });
        }
        let helper = self.helper_name("write");
        if let Some(id) = ty.shared_id() {
            self.write_helpers.insert(id, helper.clone());
        }
        let c_type = self.raw_types.c_type(ty);
        let mut cases = Vec::new();
        for (index, field) in variants.iter().enumerate() {
            let mut statements = self.write_at(
                &field.value,
                identifier("value"),
                c_expr! {
                    field((field((id("input")), "payload")), #{ format!("variant_{index}") })
                },
                field.offset,
                identifier("context"),
            )?;
            statements.push(c_statement!(return;));
            cases.push(c_switch_case! {
                (call("UINT32_C", [number(#{ index })])) => {
                    ...#{ statements }
                }
            });
        }
        cases.push(c_switch_case! {
            _ => { #{ trap(
                identifier("context"),
                "invalid sum tag at LLVM bridge",
            ) } }
        });
        let tag_store = store(
            "uint32_t",
            c_expr!(add((id("value")), (number(#{ tag_offset })))),
            c_expr!(field((id("input")), "tag")),
        );
        self.helpers.push(c_function! {
            #[static] fn #{ helper.clone() }(
                "context": ptr(named("MalContext")),
                "value": ptr(named("uint8_t")),
                "input": #{ c_type },
            ) -> named("void") {
                #{ tag_store }
                switch (field((id("input")), "tag")) {
                    ...#{ cases },
                }
            }
        });
        self.helpers.blank_line();
        Some(c_statement! {
            call(#{ helper }, [
                #{ context },
                #{ pointer },
                #{ value },
            ]);
        })
    }
}
