use std::collections::HashMap;

use crate::anf::ast::ValueId;
use crate::closure::ast::FunctionId;
use crate::control::ast::Program;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParameterDestination {
    Bind(ValueId),
    Discard,
}

pub(crate) struct ParameterPlan {
    destinations: HashMap<FunctionId, ParameterDestination>,
}

impl ParameterPlan {
    pub(crate) fn new(program: &Program) -> Self {
        let destinations = program
            .functions
            .iter()
            .map(|function| {
                (
                    function.id,
                    function
                        .parameter
                        .binding
                        .map_or(ParameterDestination::Discard, ParameterDestination::Bind),
                )
            })
            .collect();
        Self { destinations }
    }

    pub(crate) fn destination(&self, function: FunctionId) -> Option<ParameterDestination> {
        self.destinations.get(&function).copied()
    }

    pub(crate) fn is_valid(&self, program: &Program) -> bool {
        self.destinations.len() == program.functions.len()
            && program.functions.iter().all(|function| {
                self.destination(function.id)
                    == Some(
                        function
                            .parameter
                            .binding
                            .map_or(ParameterDestination::Discard, ParameterDestination::Bind),
                    )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mal_syntax::source::{FileId, SourceFile};

    #[test]
    fn validates_bound_and_discarded_parameter_destinations() {
        let source = SourceFile::new(
            FileId::new(81),
            "parameter-plan.mal",
            "bound :: Int32 -> Int32 := (value) -> { value; }; discarded :: Symbol -> Int32 := (_) -> { 0i32; };"
                .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check parameter plan fixture");
        let core = crate::core::lower(
            &crate::check::admit_monomorphic(checked).expect("specialize checked program"),
        );
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let control = crate::control::lower(&closure);
        let mut plan = ParameterPlan::new(&control);

        assert!(plan.is_valid(&control));
        let discarded = control
            .functions
            .iter()
            .find(|function| function.parameter.binding.is_none())
            .expect("discarded parameter function");
        plan.destinations.remove(&discarded.id);
        assert!(!plan.is_valid(&control));
    }
}
