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
            plan::Kind::Unit => Some(c_expr!(mal_Unit_t { unused: UINT8_C(0) })),
            plan::Kind::External => Some(c_expr! {
                { self.raw_types.c_type(value.ty) } {
                    mal_detail_bits: { load(c_type!(*const uintptr_t), pointer) }
                }
            }),
            plan::Kind::Symbol {
                owner_offset,
                data_offset,
                length_offset,
            } => Some(c_expr! {
                mal_Symbol_t {
                    owner: { load(c_type!(*const *mut void), bridge_pointer(base.clone(), offset.checked_add(*owner_offset)?, true)) },
                    data: { load(c_type!(*const *const uint8_t), bridge_pointer(base.clone(), offset.checked_add(*data_offset)?, true)) },
                    length: { load(c_type!(*const size_t), bridge_pointer(base, offset.checked_add(*length_offset)?, true)) },
                }
            }),
            plan::Kind::Product(fields) => {
                let initializers = fields
                    .iter()
                    .enumerate()
                    .map(|(index, field)| {
                        Some(Initializer::designated(
                            format!("field_{index}"),
                            self.read(
                                &field.value,
                                base.clone(),
                                offset.checked_add(field.offset)?,
                                context.clone(),
                            )?,
                        ))
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(Expr::compound_literal(
                    self.raw_types.c_type(value.ty),
                    initializers,
                ))
            }
            plan::Kind::Sum {
                tag_offset,
                variants,
            } => self.read_sum(value.ty, *tag_offset, variants, pointer, context),
            plan::Kind::Scalar => Some(load(c_type!(*const { c_scalar_type(value.ty)? }), pointer)),
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
            return Some(c_expr!({ helper.clone() }({ context }, { pointer })));
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
                let tag = c_expr!(UINT32_C({ index }));
                let result = Expr::compound_literal(
                    c_type.clone(),
                    [
                        Initializer::designated("tag", tag.clone()),
                        Initializer::designated_path(
                            ["payload".into(), format!("variant_{index}")],
                            payload,
                        ),
                    ],
                );
                Some(c_switch_cases! {
                    { tag } => { return { result }; },
                })
            })
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let failure = trap(identifier("context"), "invalid sum tag at LLVM bridge");
        let tag = load(c_type!(*const uint32_t), c_expr!(value + { tag_offset }));
        let body = c_block! {
            let tag: uint32_t = { tag };
            match tag {
                ..{ cases },
                _ => { { failure }; },
            }
        };
        self.helpers.push(FunctionDefinition::from_signature(
            c_signature! {
                #[static] fn { helper.clone() }(
                    context: *mut MalContext,
                    value: *const uint8_t,
                ) -> { c_type }
            },
            body,
        ));
        self.helpers.blank_line();
        Some(c_expr!({ helper }({ context }, { pointer })))
    }
}
