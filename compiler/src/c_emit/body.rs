use crate::anf;
use crate::closure::ast::{self as closure, Binding, Block, Operation, Pattern};
use crate::resolve::ast::{ExternalOperationId, LambdaId};

use super::types::TypeRegistry;

mod expression;
mod function;
mod pattern;
mod statement;

use self::pattern::pattern_type;

#[derive(Clone, Copy, Default)]
pub(super) struct RuntimeNeeds {
    pub(super) wrap: u16,
    pub(super) divide: u16,
    pub(super) remainder: u16,
    pub(super) shift_left: u16,
    pub(super) shift_right: u16,
    pub(super) string_equality: bool,
    pub(super) string_at: bool,
    pub(super) float_to_integer: u32,
}

pub(super) struct BodyOutput {
    pub(super) environment_declarations: String,
    pub(super) globals: String,
    pub(super) function_declarations: String,
    pub(super) function_definitions: String,
    pub(super) initializer: String,
    pub(super) main: String,
    pub(super) needs: RuntimeNeeds,
}

pub(super) struct BodyEmitter<'a> {
    program: &'a closure::Program,
    types: &'a TypeRegistry,
    needs: RuntimeNeeds,
    next_discard: u32,
}

impl<'a> BodyEmitter<'a> {
    pub(super) fn new(program: &'a closure::Program, types: &'a TypeRegistry) -> Self {
        Self {
            program,
            types,
            needs: RuntimeNeeds::default(),
            next_discard: 0,
        }
    }

    pub(super) fn emit(&mut self, main: &crate::closure::ast::TopLevelBinding) -> BodyOutput {
        let environment_declarations = self.emit_environments();
        let globals = self.emit_globals();
        let function_declarations = self.emit_function_declarations();
        let function_definitions = self.emit_function_definitions();
        let initializer = self.emit_initializer();
        let main = self.emit_main(main);
        BodyOutput {
            environment_declarations,
            globals,
            function_declarations,
            function_definitions,
            initializer,
            main,
            needs: self.needs,
        }
    }

    fn external(&self, id: ExternalOperationId) -> &closure::ExternalOperation {
        self.program
            .externals
            .iter()
            .find(|external| external.id == id)
            .expect("closure conversion preserves external declarations")
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
}

fn value_name(id: anf::ast::ValueId) -> String {
    match id {
        anf::ast::ValueId::Core(crate::core::ast::ValueId::Source(id)) => {
            format!("mal_value_source_{}", id.0)
        }
        anf::ast::ValueId::Core(crate::core::ast::ValueId::Temporary(id)) => {
            format!("mal_value_core_{id}")
        }
        anf::ast::ValueId::Temporary(id) => format!("mal_value_anf_{id}"),
    }
}

fn function_name(id: LambdaId) -> String {
    format!("mal_function_{}", id.0)
}

fn environment_name(id: LambdaId) -> String {
    format!("MalEnvironment_{}", id.0)
}

fn has_direct_tail_call(block: &Block, function: LambdaId) -> bool {
    let Some(binding) = tail_binding(block) else {
        return false;
    };
    match &binding.operation {
        Operation::Call { callee, .. } => matches!(
            callee.kind,
            closure::AtomKind::Reference(closure::Reference::SelfClosure(id)) if id == function
        ),
        Operation::Case { arms, .. } => arms
            .iter()
            .any(|arm| has_direct_tail_call(&arm.value, function)),
        _ => false,
    }
}

fn tail_binding(block: &Block) -> Option<&Binding> {
    block.bindings.last().filter(|binding| {
        matches!(
            (&block.result.kind, &binding.pattern),
            (
                closure::AtomKind::Reference(closure::Reference::Binding(result)),
                Pattern::Binding { id, .. }
            ) if result == id
        )
    })
}
