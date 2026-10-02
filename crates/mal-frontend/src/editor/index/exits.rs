//! Where control leaves a result block and to which binders: the exits editors mark and the targets each reaches.

use super::*;
use crate::check::ast as checked;

impl Index {
    /// One side of a choice: whether every path through it leaves the block, and to which binders.
    pub(super) fn choice(&self, span: Span, completion: &checked::Completion) -> Choice {
        Choice {
            span,
            leaves: matches!(completion, checked::Completion::Abrupt(_)),
            targets: self.completion_targets(completion),
        }
    }

    pub(super) fn transfer_names(
        &self,
        target: resolved::ValueId,
        variant: Option<usize>,
    ) -> Vec<String> {
        self.result_binder_names
            .get(&(self.canonical_value(target), variant))
            .cloned()
            .into_iter()
            .collect()
    }

    /// The result binders a path may transfer to, in source order. An empty elimination names none.
    pub(super) fn completion_targets(&self, completion: &checked::Completion) -> Vec<String> {
        match completion {
            checked::Completion::Value(_) => Vec::new(),
            checked::Completion::Abrupt(abrupt) => self.abrupt_targets(abrupt),
        }
    }

    pub(super) fn abrupt_targets(&self, abrupt: &checked::AbruptExpression) -> Vec<String> {
        let mut targets = Vec::new();
        match &abrupt.kind {
            checked::AbruptExpressionKind::ResultTransfer {
                target, variant, ..
            } => {
                merge_targets(&mut targets, self.transfer_names(*target, *variant));
            }
            checked::AbruptExpressionKind::EmptyElimination { .. } => {}
            checked::AbruptExpressionKind::If {
                then_branch,
                else_branch,
                ..
            } => {
                merge_targets(&mut targets, self.completion_targets(&then_branch.result));
                merge_targets(&mut targets, self.completion_targets(&else_branch.result));
            }
            checked::AbruptExpressionKind::SumElimination { continuations, .. } => {
                for continuation in continuations {
                    let names = match continuation {
                        checked::SumContinuation::Function(_) => Vec::new(),
                        checked::SumContinuation::Branch(branch) => {
                            self.completion_targets(&branch.body.result)
                        }
                        checked::SumContinuation::Transfer(transfer) => {
                            self.transfer_names(transfer.target, transfer.variant)
                        }
                    };
                    merge_targets(&mut targets, names);
                }
            }
            checked::AbruptExpressionKind::Block(block) => {
                merge_targets(&mut targets, self.completion_targets(&block.result));
            }
        }
        targets
    }

    /// Records where a choice leaves the block. When some choices continue, the leaving ones are marked; when all of
    /// them leave, the whole choice is. Returns, for each side, whether a mark already covers where that side ends.
    pub(super) fn note_exits(&mut self, choices: &[Choice], whole: Span) -> Vec<bool> {
        let leaving = choices.iter().filter(|choice| choice.leaves).count();
        if leaving == 0 {
            return vec![false; choices.len()];
        }
        if leaving == choices.len() {
            let mut targets = Vec::new();
            for choice in choices {
                merge_targets(&mut targets, choice.targets.clone());
            }
            self.exits.push(Exit {
                span: whole,
                targets,
            });
        } else {
            self.exits.extend(
                choices
                    .iter()
                    .filter(|choice| choice.leaves)
                    .map(|choice| Exit {
                        span: choice.span,
                        targets: choice.targets.clone(),
                    }),
            );
        }
        choices.iter().map(|choice| choice.leaves).collect()
    }

    pub(super) fn note_branch_exits(
        &mut self,
        then_branch: &checked::ExpressionBlock,
        else_branch: &checked::ExpressionBlock,
        whole: Span,
    ) -> [bool; 2] {
        let then_choice = self.choice(then_branch.span, then_branch.result.as_ref());
        let mut else_choice = self.choice(else_branch.span, else_branch.result.as_ref());
        // `when` has no written else branch; its synthetic one spans the whole expression.
        else_choice.leaves &= else_branch.span != whole;
        let reported = self.note_exits(&[then_choice, else_choice], whole);
        [reported[0], reported[1]]
    }
}

/// Orders the exits by position and merges the ones that mark the same span.
///
/// Nested spans are kept: an exit among the statements of a leaving unit is a place where control may leave early. The
/// collection already leaves out the one hint that would repeat its parent, the choice that ends a marked unit.
pub(super) fn merged_exits(mut exits: Vec<Exit>) -> Vec<Exit> {
    exits.sort_by_key(|exit| {
        (
            exit.span.file().index(),
            exit.span.start(),
            std::cmp::Reverse(exit.span.end()),
        )
    });
    let mut merged: Vec<Exit> = Vec::new();
    for exit in exits {
        match merged.last_mut() {
            Some(last) if last.span == exit.span => merge_targets(&mut last.targets, exit.targets),
            _ => merged.push(exit),
        }
    }
    merged
}

pub(super) fn merge_targets(targets: &mut Vec<String>, more: Vec<String>) {
    for name in more {
        if !targets.contains(&name) {
            targets.push(name);
        }
    }
}

/// One side of a choice, for deciding whether the choice marks it or is marked as a whole.
pub(super) struct Choice {
    pub(super) span: Span,
    pub(super) leaves: bool,
    pub(super) targets: Vec<String>,
}
