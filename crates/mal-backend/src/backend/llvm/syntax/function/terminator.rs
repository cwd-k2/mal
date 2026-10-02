//! Basic-block terminators.

use super::super::lexical::{is_atom, is_value};
use super::*;

#[derive(Clone)]
pub(in crate::backend::llvm) enum Terminator {
    Branch {
        target: String,
    },
    ConditionalBranch {
        condition: String,
        then_target: String,
        else_target: String,
    },
    ReturnVoid,
    Return {
        ty: Type,
        value: String,
    },
    Switch {
        ty: Type,
        value: String,
        default: String,
        cases: Vec<(String, String)>,
    },
    Unreachable,
}

impl Terminator {
    pub(in crate::backend::llvm) fn branch(target: impl Into<String>) -> Option<Self> {
        let target = target.into();
        is_valid_name(&target).then_some(Self::Branch { target })
    }

    pub(in crate::backend::llvm) fn conditional_branch(
        condition: impl Into<String>,
        then_target: impl Into<String>,
        else_target: impl Into<String>,
    ) -> Option<Self> {
        let condition = condition.into();
        let then_target = then_target.into();
        let else_target = else_target.into();
        (is_atom(&condition) && is_valid_name(&then_target) && is_valid_name(&else_target))
            .then_some(Self::ConditionalBranch {
                condition,
                then_target,
                else_target,
            })
    }

    pub(in crate::backend::llvm) fn return_void() -> Self {
        Self::ReturnVoid
    }

    pub(in crate::backend::llvm) fn return_value(
        ty: Type,
        value: impl Into<String>,
    ) -> Option<Self> {
        let value = value.into();
        is_value(&value).then_some(Self::Return { ty, value })
    }

    pub(in crate::backend::llvm) fn switch(
        ty: Type,
        value: impl Into<String>,
        default: impl Into<String>,
        cases: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Option<Self> {
        let value = value.into();
        let default = default.into();
        let cases = cases
            .into_iter()
            .map(|(value, target)| (value.into(), target.into()))
            .collect::<Vec<_>>();
        (is_atom(&value)
            && is_valid_name(&default)
            && cases
                .iter()
                .all(|(value, target)| is_atom(value) && is_valid_name(target)))
        .then_some(Self::Switch {
            ty,
            value,
            default,
            cases,
        })
    }

    pub(in crate::backend::llvm) fn unreachable() -> Self {
        Self::Unreachable
    }

    pub(super) fn render_into(&self, output: &mut String) {
        match self {
            Self::Branch { target } => output.push_str(&format!("br label %{target}")),
            Self::ConditionalBranch {
                condition,
                then_target,
                else_target,
            } => output.push_str(&format!(
                "br i1 {condition}, label %{then_target}, label %{else_target}"
            )),
            Self::ReturnVoid => output.push_str("ret void"),
            Self::Return { ty, value } => output.push_str(&format!("ret {ty} {value}")),
            Self::Switch {
                ty,
                value,
                default,
                cases,
            } => {
                output.push_str(&format!("switch {ty} {value}, label %{default} ["));
                for (case, target) in cases {
                    output.push_str(&format!("\n    {ty} {case}, label %{target}"));
                }
                output.push_str("\n  ]");
            }
            Self::Unreachable => output.push_str("unreachable"),
        }
    }

    pub(super) fn uses_byte_runtime(&self) -> bool {
        false
    }
}
