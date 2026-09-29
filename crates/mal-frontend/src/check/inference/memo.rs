//! Non-committing probes and the per-call memo that lets a generic call's final check reuse them.

use std::collections::HashMap;

use crate::resolve::ast::{self as resolved, ValueId};
use mal_syntax::ast::Node;

use super::super::ast::{Expression, OperationRequirement, Type};
use super::super::{CheckResult, Checker};

/// Checked outcomes of the direct arguments of one generic call.
///
/// Every probe of an argument and the final check run under the same enclosing environment, so an argument
/// checked once under an expectation has the same outcome, success or failure, when checked under it again. An
/// argument that succeeded with less information, reaching type `T`, also elaborates the same way when later checked
/// against `T`: the expectation only adds a constraint that the first elaboration already satisfies. A failure is
/// reused only under the same expectation, because more information may let the argument check.
pub(in crate::check) struct ArgumentMemo {
    entries: HashMap<usize, Vec<Outcome>>,
}

/// What an argument was checked against. A lambda whose parameter is known but whose result is not is checked
/// against that parameter, which an untyped check of the same lambda is not.
#[derive(Clone, PartialEq)]
pub(in crate::check) enum Expectation {
    Untyped,
    Parameter(Type),
    Expected(Type),
}

impl Expectation {
    pub(in crate::check) fn from_expected(expected: Option<&Type>) -> Self {
        expected.map_or(Self::Untyped, |ty| Self::Expected(ty.clone()))
    }
}

struct Outcome {
    expectation: Expectation,
    result: CheckResult<Expression>,
    effects: Effects,
}

/// Checker state a probe changed and that a reused outcome must replay.
struct Effects {
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
        expectation: &Expectation,
    ) -> Option<&Outcome> {
        self.entries.get(&address(argument))?.iter().find(|entry| {
            entry.expectation == *expectation
                || match (&entry.expectation, expectation, &entry.result) {
                    (
                        Expectation::Untyped | Expectation::Parameter(_),
                        Expectation::Expected(expected),
                        Ok(expression),
                    ) => expression.ty == *expected,
                    _ => false,
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
    pub(in crate::check) fn transaction<T>(&mut self, check: impl FnOnce(&mut Self) -> T) -> T {
        self.recorded_transaction(check).0
    }

    fn recorded_transaction<T>(&mut self, check: impl FnOnce(&mut Self) -> T) -> (T, Effects) {
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

    /// Probes a direct argument of the current generic call under `expectation`, reusing and recording its outcome
    /// in the call's memo.
    pub(in crate::check) fn probe_argument(
        &mut self,
        argument: &Node<resolved::Expression>,
        expectation: Expectation,
        check: impl FnOnce(&mut Self) -> CheckResult<Expression>,
    ) -> CheckResult<Expression> {
        if let Some(entry) = self
            .argument_memos
            .last()
            .and_then(|memo| memo.find(argument, &expectation))
        {
            return entry.result.clone();
        }
        let (result, effects) = self.recorded_transaction(check);
        if let Some(entries) = self
            .argument_memos
            .last_mut()
            .and_then(|memo| memo.entries.get_mut(&address(argument)))
        {
            entries.push(Outcome {
                expectation,
                result: result.clone(),
                effects,
            });
        }
        result
    }

    /// Returns the memoized outcome of a direct argument of the current generic call and commits its effects.
    pub(in crate::check) fn reuse_argument(
        &mut self,
        argument: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> Option<CheckResult<Expression>> {
        let entry = self
            .argument_memos
            .last()?
            .find(argument, &Expectation::from_expected(expected))?;
        let result = entry.result.clone();
        let operations = entry.effects.operations.clone();
        let used = entry.effects.used_result_targets.clone();
        for operation in operations {
            if !self.active_operations.contains(&operation) {
                self.active_operations.push(operation);
            }
        }
        self.used_result_targets.extend(used);
        Some(result)
    }
}
