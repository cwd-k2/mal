use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::{self as closure, FunctionId, Pattern};
use crate::core::ast::ExternalOperation;
use crate::resolve::ast::ExternalOperationId;

use super::syntax::TranslationUnit;
use super::types::TypeRegistry;

mod analysis;
mod call;
mod control;
mod entry;
mod expression;
mod function;
mod name;
mod pattern;
mod statement;

use self::analysis::{ControlFramePlan, OwnedCallPlan, OwnershipPlan, SymbolAtCursorPlan};
use self::call::{
    flattened_product_types, flattened_product_values, has_direct_product_entry,
    has_direct_tail_call,
};
use self::expression::ResultOwnership;
use self::name::environment_destroy_name;
use self::name::{
    direct_function_name, environment_name, function_name, owned_function_name,
    stack_environment_name, value_name,
};
use self::pattern::pattern_type;
use crate::execution::{
    self, ApplicationGraph, ClosureUsePlan, ControlCallPlan, ControlRegionPlan, TailCallPlan,
    direct_function_id,
};

#[derive(Clone, Copy, Default)]
pub(super) struct RuntimeNeeds {
    pub(super) control_arenas: usize,
    pub(super) homogeneous_control: bool,
    pub(super) heterogeneous_control: bool,
    pub(super) wrap: u16,
    pub(super) divide: u16,
    pub(super) remainder: u16,
    pub(super) shift_left: u16,
    pub(super) shift_right: u16,
    pub(super) symbol_equality: bool,
    pub(super) symbol_at: bool,
    pub(super) symbol_at_cursor: bool,
    pub(super) symbol_concatenate: bool,
    pub(super) symbol_concatenate_consuming_left: bool,
    pub(super) symbol_concatenate_consuming_right: bool,
    pub(super) memory_offset_forward: bool,
    pub(super) memory_offset_backward: bool,
    pub(super) memory_load: u16,
    pub(super) memory_store: u16,
    pub(super) memory_load_ptr: bool,
    pub(super) memory_store_ptr: bool,
    pub(super) memory_load_symbol: bool,
    pub(super) memory_store_symbol: bool,
    pub(super) float_to_integer: u32,
}

pub(super) struct BodyOutput {
    pub(super) environment_declarations: TranslationUnit,
    pub(super) environment_definitions: TranslationUnit,
    pub(super) globals: TranslationUnit,
    pub(super) function_declarations: TranslationUnit,
    pub(super) function_definitions: TranslationUnit,
    pub(super) control_frames: TranslationUnit,
    pub(super) initializer: TranslationUnit,
    pub(super) program_destroy: TranslationUnit,
    pub(super) main: TranslationUnit,
    pub(super) needs: RuntimeNeeds,
}

pub(super) struct BodyEmitter<'a> {
    program: &'a closure::Program,
    types: &'a TypeRegistry,
    needs: RuntimeNeeds,
    next_discard: u32,
    ownership: OwnershipPlan,
    symbol_at_cursors: SymbolAtCursorPlan,
    active_symbol_at_cursors: HashMap<ValueId, usize>,
    closure_uses: ClosureUsePlan,
    owned_calls: OwnedCallPlan,
    ephemeral_bindings: HashSet<ValueId>,
    borrowed_bindings: HashSet<ValueId>,
    direct_borrow_sources: HashSet<ValueId>,
    parameter_owned: bool,
    control: crate::control::ast::Program,
    applications: ApplicationGraph,
    tail_calls: TailCallPlan,
    control_calls: ControlCallPlan,
    control_regions: ControlRegionPlan,
    control_frames: ControlFramePlan,
}

impl<'a> BodyEmitter<'a> {
    pub(super) fn new(program: &'a closure::Program, types: &'a TypeRegistry) -> Self {
        let ownership = OwnershipPlan::new(program);
        let symbol_at_cursors = SymbolAtCursorPlan::new(program);
        debug_assert!(ownership.is_valid(program));
        debug_assert!(symbol_at_cursors.is_valid(program));
        let execution::Program {
            control,
            closure_uses,
            applications,
            tail_calls,
            control_calls,
            control_regions,
        } = execution::lower(program);
        let owned_calls = OwnedCallPlan::new(program, types, &ownership, &closure_uses);
        let control_frames = ControlFramePlan::new(
            &control,
            &control_regions,
            &control_calls,
            types,
            &closure_uses,
        );
        debug_assert!(control_frames.is_valid(
            &control,
            &control_regions,
            &control_calls,
            types,
            &closure_uses
        ));
        let needs = RuntimeNeeds {
            control_arenas: control_frames.arena_count(),
            homogeneous_control: control_frames.has_homogeneous_arenas(),
            heterogeneous_control: control_frames.has_heterogeneous_arenas(),
            ..RuntimeNeeds::default()
        };
        Self {
            program,
            types,
            needs,
            next_discard: 0,
            ownership,
            symbol_at_cursors,
            active_symbol_at_cursors: HashMap::new(),
            closure_uses,
            owned_calls,
            ephemeral_bindings: HashSet::new(),
            borrowed_bindings: HashSet::new(),
            direct_borrow_sources: HashSet::new(),
            parameter_owned: false,
            control,
            applications,
            tail_calls,
            control_calls,
            control_regions,
            control_frames,
        }
    }

    pub(super) fn emit(&mut self, main: &crate::closure::ast::TopLevelBinding) -> BodyOutput {
        let environment_declarations = self.emit_environments();
        let environment_definitions = self.emit_environment_definitions();
        let globals = self.emit_globals();
        let function_declarations = self.emit_function_declarations();
        let function_definitions = self.emit_function_definitions();
        let control_frames = self.emit_control_frames();
        let initializer = self.emit_initializer();
        let program_destroy = self.emit_program_destroy();
        let main = self.emit_main(main);
        BodyOutput {
            environment_declarations,
            environment_definitions,
            globals,
            function_declarations,
            function_definitions,
            control_frames,
            initializer,
            program_destroy,
            main,
            needs: self.needs,
        }
    }

    fn external(&self, id: ExternalOperationId) -> &ExternalOperation {
        self.program
            .interface
            .externals
            .iter()
            .find(|external| external.id == id)
            .expect("closure conversion preserves external declarations")
    }

    fn uses_common_control(&self, function: closure::FunctionId) -> bool {
        self.control_regions
            .function_region(function)
            .is_some_and(|region| self.control_calls.requires_common_control(region))
    }

    fn function(&self, id: FunctionId) -> &closure::Function {
        self.program
            .functions
            .iter()
            .find(|function| function.id == id)
            .expect("closure conversion preserves function identities")
    }

    fn direct_function(&self, callee: &closure::Atom) -> Option<(FunctionId, super::syntax::Expr)> {
        let function = direct_function_id(&self.closure_uses, callee)?;
        match callee.kind {
            closure::AtomKind::Reference(closure::Reference::SelfClosure(function)) => {
                Some((function, super::syntax::Expr::identifier("mal_environment")))
            }
            closure::AtomKind::Reference(closure::Reference::Binding(id)) => {
                if let Some(target) = self.closure_uses.direct_closure(id) {
                    let environment = if self.function(function).environment.is_empty() {
                        super::syntax::Expr::identifier("NULL")
                    } else if self
                        .control_frames
                        .closure_crosses_suspension(target.creator)
                    {
                        super::syntax::Expr::identifier(value_name(target.creator))
                            .field("environment")
                    } else {
                        super::syntax::Expr::address_of(super::syntax::Expr::identifier(
                            stack_environment_name(target.creator),
                        ))
                    };
                    return Some((function, environment));
                }
                Some((function, super::syntax::Expr::identifier("NULL")))
            }
            _ => None,
        }
    }

    fn top_level_function_name(&self, function: FunctionId) -> Option<&str> {
        self.program.bindings.iter().find_map(|binding| {
            let closure::TopLevelPattern::Binding { name, .. } = &binding.pattern else {
                return None;
            };
            let closure::AtomKind::Reference(closure::Reference::Binding(result_id)) =
                binding.value.result.kind
            else {
                return None;
            };
            binding.value.bindings.iter().find_map(|value| {
                let closure::Pattern::Binding { id, .. } = value.pattern else {
                    return None;
                };
                matches!(
                    value.operation,
                    closure::Operation::MakeClosure {
                        function: candidate,
                        ..
                    } if id == result_id && candidate == function
                )
                .then_some(name.as_str())
            })
        })
    }

    fn result_target(&mut self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Binding { id, .. } => value_name(*id),
            Pattern::Wildcard { .. } | Pattern::Product { .. } => {
                let name = format!("mal_discard_{}", self.next_discard);
                self.next_discard += 1;
                name
            }
        }
    }

    fn can_transfer(&self, atom: &closure::Atom) -> bool {
        if matches!(
            atom.kind,
            closure::AtomKind::Reference(closure::Reference::Binding(id))
                if self.borrowed_bindings.contains(&id)
        ) {
            return false;
        }
        self.ownership.can_transfer(atom, self.parameter_owned)
    }

    fn uses_stack_environment(&self, id: ValueId) -> bool {
        self.closure_uses.direct_closure(id).is_some_and(|target| {
            target.creator == id
                && !self
                    .control_frames
                    .closure_crosses_suspension(target.creator)
        })
    }

    fn is_borrowed(&self, atom: &closure::Atom) -> bool {
        matches!(
            atom.kind,
            closure::AtomKind::Reference(closure::Reference::Binding(id))
                if self.borrowed_bindings.contains(&id)
                    || self.direct_borrow_sources.contains(&id)
        )
    }

    fn elides_top_level(&self, id: ValueId) -> bool {
        self.closure_uses.is_direct_top_level(id) && !self.control_calls.needs_closure_binding(id)
    }
}
