use super::*;

impl Lowerer {
    /// Case arms of a sum elimination without control paths. A function continuation is applied to the
    /// payload; a branch binds the payload in place and evaluates its body as part of the enclosing lambda.
    pub(super) fn lower_sum_arms(
        &mut self,
        members: &[checked::Type],
        continuations: &[checked::SumContinuation],
        result_type: &checked::Type,
    ) -> Vec<CaseArm> {
        continuations
            .iter()
            .zip(members)
            .enumerate()
            .map(|(index, (continuation, member))| {
                let payload_id = self.temporary();
                let span = sum_continuation_span(continuation);
                let value = match continuation {
                    checked::SumContinuation::Function(function) => Expression {
                        kind: ExpressionKind::Call {
                            callee: Box::new(self.lower_expression(function)),
                            argument: Box::new(self.reference(payload_id, member.clone(), span)),
                        },
                        ty: result_type.clone(),
                        span,
                    },
                    checked::SumContinuation::Branch(branch) => {
                        let body = self.lower_body(&branch.body.items, &branch.body.result);
                        self.bind_branch_payload(branch, payload_id, body)
                    }
                    checked::SumContinuation::Transfer(_) => {
                        unreachable!("a result transfer is lowered through the control paths")
                    }
                };
                CaseArm {
                    index,
                    pattern: Pattern::Binding {
                        id: payload_id,
                        ty: member.clone(),
                    },
                    value,
                    span,
                }
            })
            .collect()
    }

    /// Binds the branch parameter to the payload around the lowered branch body.
    pub(super) fn bind_branch_payload(
        &self,
        branch: &checked::SumBranch,
        payload_id: ValueId,
        body: Expression,
    ) -> Expression {
        let Some(pattern) = branch.parameter.as_deref() else {
            return body;
        };
        let ty = body.ty.clone();
        Expression {
            kind: ExpressionKind::Let {
                binding: Box::new(Binding {
                    pattern: self.lower_pattern(pattern),
                    value: self.reference(payload_id, branch.parameter_type.clone(), branch.span),
                    span: branch.span,
                }),
                body: Box::new(body),
            },
            ty,
            span: branch.span,
        }
    }

    /// Sends the payload to the result binder that a continuation names.
    pub(super) fn lower_sum_transfer(
        &self,
        transfer: &checked::SumTransfer,
        payload_id: ValueId,
        result_type: &checked::Type,
    ) -> Expression {
        let span = transfer.span;
        let payload = self.reference(payload_id, transfer.payload_type.clone(), span);
        let value = match transfer.variant {
            Some(index) => Expression {
                kind: ExpressionKind::SumInjection {
                    index,
                    value: Box::new(payload),
                },
                ty: transfer.result_type.clone(),
                span,
            },
            None => payload,
        };
        let target = *self
            .result_targets
            .get(&transfer.target)
            .expect("a result binder is lowered inside its result block");
        Expression {
            kind: ExpressionKind::Goto {
                target,
                value: Box::new(value),
            },
            ty: result_type.clone(),
            span,
        }
    }
}

impl Lowerer {
    /// When every variant of the scrutinee is sent to the same result binder group at the same position,
    /// the elimination only rebuilds the value it inspected, so it is the value itself sent to that
    /// result. Returns the join to jump to, which keeps a call in the scrutinee a tail call.
    pub(super) fn identity_forward_target(
        &self,
        scrutinee_type: &checked::Type,
        continuations: &[checked::SumContinuation],
    ) -> Option<ast::JoinId> {
        let mut target = None;
        for (index, continuation) in continuations.iter().enumerate() {
            let checked::SumContinuation::Transfer(transfer) = continuation else {
                return None;
            };
            if transfer.variant != Some(index)
                || transfer.result_type != *scrutinee_type
                || *target.get_or_insert(transfer.target) != transfer.target
            {
                return None;
            }
        }
        self.result_targets.get(&target?).copied()
    }
}

pub(super) fn sum_continuation_span(continuation: &checked::SumContinuation) -> Span {
    match continuation {
        checked::SumContinuation::Function(function) => function.span,
        checked::SumContinuation::Branch(branch) => branch.span,
        checked::SumContinuation::Transfer(transfer) => transfer.span,
    }
}
