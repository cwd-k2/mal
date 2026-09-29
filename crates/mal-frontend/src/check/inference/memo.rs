//! Non-committing probes and the per-call memo that lets a generic call's final check reuse them.

use std::collections::HashMap;

use crate::resolve::ast::{self as resolved, ValueId};
use mal_syntax::ast::Node;

use super::super::ast::{Expression, OperationRequirement, Type};
use super::super::{CheckResult, Checker};

/// Checked elaborations of the direct arguments of one generic call.
///
/// Every probe of an argument and the final check run under the same enclosing environment, so an argument
/// checked once against an expectation elaborates the same way when checked against it again. An argument checked
/// without a complete expectation to type `T` also elaborates the same way when later checked against `T`: the
/// expectation only adds a constraint that the first elaboration already satisfies.
pub(in crate::check) struct ArgumentMemo {
    entries: HashMap<usize, Vec<Elaboration>>,
}

struct Elaboration {
    /// The complete expectation the argument was checked against, or `None` when it had less information.
    expected: Option<Type>,
    expression: Expression,
    effects: Effects,
}

/// Checker state a probe changed and that a reused elaboration must replay.
pub(in crate::check) struct Effects {
    operations: Vec<OperationRequirement>,
    used_result_targets: Vec<ValueId>,
}

impl ArgumentMemo {
    pub(in crate::check) fn new(arguments: &[Node<resolved::Expression>]) -> Self {
        Self {
            entries: arguments
                .iter()
                .map(|argument| (address(argument), Vec::new()))
                .collect(),
        }
    }

    fn find(
        &self,
        argument: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> Option<&Elaboration> {
        self.entries.get(&address(argument))?.iter().find(|entry| {
            match (&entry.expected, expected) {
                (Some(checked), Some(expected)) => checked == expected,
                (None, Some(expected)) => entry.expression.ty == *expected,
                (None, None) => true,
                (Some(_), None) => false,
            }
        })
    }
}

fn address(argument: &Node<resolved::Expression>) -> usize {
    std::ptr::from_ref(argument) as usize
}

impl Checker {
    /// Runs `check` without committing the requirement, result-target, and alias-expansion state it changes.
    /// Identity counters and pure caches are kept.
    pub(in crate::check) fn transaction<T>(
        &mut self,
        check: impl FnOnce(&mut Self) -> T,
    ) -> (T, Effects) {
        let operations = self.active_operations.len();
        let used_result_targets = self.used_result_targets.clone();
        let expanding = self.expanding.clone();
        let result = check(self);
        let effects = Effects {
            operations: self.active_operations.split_off(operations),
            used_result_targets: self
                .used_result_targets
                .difference(&used_result_targets)
                .copied()
                .collect(),
        };
        self.used_result_targets = used_result_targets;
        self.expanding = expanding;
        (result, effects)
    }

    /// Probes a direct argument of the current generic call against `expected`, reusing and recording its
    /// elaboration in the call's memo.
    pub(in crate::check) fn probe_argument(
        &mut self,
        argument: &Node<resolved::Expression>,
        expected: Option<&Type>,
        check: impl FnOnce(&mut Self) -> CheckResult<Expression>,
    ) -> CheckResult<Expression> {
        if let Some(entry) = self
            .argument_memos
            .last()
            .and_then(|memo| memo.find(argument, expected))
        {
            return Ok(entry.expression.clone());
        }
        let (result, effects) = self.transaction(check);
        if let Ok(expression) = &result
            && let Some(entries) = self
                .argument_memos
                .last_mut()
                .and_then(|memo| memo.entries.get_mut(&address(argument)))
        {
            entries.push(Elaboration {
                expected: expected.cloned(),
                expression: expression.clone(),
                effects,
            });
        }
        result
    }

    /// Returns the memoized elaboration of a direct argument of the current generic call and commits its effects.
    pub(in crate::check) fn reuse_argument(
        &mut self,
        argument: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> Option<Expression> {
        let entry = self.argument_memos.last()?.find(argument, expected)?;
        let expression = entry.expression.clone();
        let operations = entry.effects.operations.clone();
        let used = entry.effects.used_result_targets.clone();
        for operation in operations {
            if !self.active_operations.contains(&operation) {
                self.active_operations.push(operation);
            }
        }
        self.used_result_targets.extend(used);
        Some(expression)
    }
}
