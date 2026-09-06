use crate::anf;
use crate::check::ast::{MemoryPrimitive, Type};
use crate::closure::ast::{self as closure, Binding, Block, FunctionId, Operation, Pattern};
use crate::core::ast::ExternalOperation;
use crate::resolve::ast::ExternalOperationId;

use super::syntax::TranslationUnit;
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
    pub(super) symbol_equality: bool,
    pub(super) symbol_at: bool,
    pub(super) symbol_concatenate: bool,
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
    pub(super) globals: TranslationUnit,
    pub(super) function_declarations: TranslationUnit,
    pub(super) function_definitions: TranslationUnit,
    pub(super) initializer: TranslationUnit,
    pub(super) main: TranslationUnit,
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

    fn external(&self, id: ExternalOperationId) -> &ExternalOperation {
        self.program
            .interface
            .externals
            .iter()
            .find(|external| external.id == id)
            .expect("closure conversion preserves external declarations")
    }

    fn function(&self, id: FunctionId) -> &closure::Function {
        self.program
            .functions
            .iter()
            .find(|function| function.id == id)
            .expect("closure conversion preserves function identities")
    }

    fn direct_function(&self, callee: &closure::Atom) -> Option<(FunctionId, &'static str)> {
        match callee.kind {
            closure::AtomKind::Reference(closure::Reference::SelfClosure(function)) => {
                Some((function, "mal_environment"))
            }
            closure::AtomKind::Reference(closure::Reference::Binding(id)) => {
                self.program.bindings.iter().find_map(|binding| {
                    let closure::TopLevelPattern::Binding {
                        id: top_level_id, ..
                    } = binding.pattern
                    else {
                        return None;
                    };
                    if top_level_id != id {
                        return None;
                    }
                    let closure::AtomKind::Reference(closure::Reference::Binding(result_id)) =
                        binding.value.result.kind
                    else {
                        return None;
                    };
                    binding.value.bindings.iter().find_map(|value| {
                        let closure::Pattern::Binding { id, .. } = value.pattern else {
                            return None;
                        };
                        match &value.operation {
                            closure::Operation::MakeClosure { function, captures }
                                if id == result_id && captures.is_empty() =>
                            {
                                Some((*function, "NULL"))
                            }
                            _ => None,
                        }
                    })
                })
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
        anf::ast::ValueId::MemoryParameter(primitive) => {
            format!("mal_memory_parameter_{}", memory_primitive_name(primitive))
        }
        anf::ast::ValueId::MemoryResult(primitive) => {
            format!("mal_memory_result_{}", memory_primitive_name(primitive))
        }
    }
}

fn function_name(id: FunctionId) -> String {
    match id {
        FunctionId::Lambda(id) => format!("mal_function_{}", id.0),
        FunctionId::Memory(primitive) => {
            format!("mal_memory_function_{}", memory_primitive_name(primitive))
        }
    }
}

fn direct_function_name(id: FunctionId) -> String {
    match id {
        FunctionId::Lambda(id) => format!("mal_direct_function_{}", id.0),
        FunctionId::Memory(primitive) => format!(
            "mal_direct_memory_function_{}",
            memory_primitive_name(primitive)
        ),
    }
}

const MAX_DIRECT_PARAMETERS: usize = 16;

fn has_direct_product_entry(ty: &Type) -> bool {
    matches!(ty, Type::Product(_)) && flattened_product_types(ty).len() <= MAX_DIRECT_PARAMETERS
}

fn flattened_product_types(ty: &Type) -> Vec<&Type> {
    match ty {
        Type::Product(elements) => elements.iter().flat_map(flattened_product_types).collect(),
        _ => vec![ty],
    }
}

fn flattened_product_values(
    ty: &Type,
    value: crate::c_emit::syntax::Expr,
) -> Vec<crate::c_emit::syntax::Expr> {
    match ty {
        Type::Product(elements) => elements
            .iter()
            .enumerate()
            .flat_map(|(index, element)| {
                flattened_product_values(element, value.clone().field(format!("field_{index}")))
            })
            .collect(),
        _ => vec![value],
    }
}

fn environment_name(id: FunctionId) -> String {
    match id {
        FunctionId::Lambda(id) => format!("MalEnvironment_{}", id.0),
        FunctionId::Memory(primitive) => {
            format!("MalMemoryEnvironment_{}", memory_primitive_name(primitive))
        }
    }
}

fn has_direct_tail_call(block: &Block, function: FunctionId) -> bool {
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
        Operation::PrimitiveBranch {
            otherwise, then, ..
        } => has_direct_tail_call(otherwise, function) || has_direct_tail_call(then, function),
        _ => false,
    }
}

fn memory_primitive_name(primitive: MemoryPrimitive) -> &'static str {
    use crate::check::ast::MemoryScalar;

    match primitive {
        MemoryPrimitive::Load(scalar) => match scalar {
            MemoryScalar::Int8 => "load_int8",
            MemoryScalar::Int16 => "load_int16",
            MemoryScalar::Int32 => "load_int32",
            MemoryScalar::Int64 => "load_int64",
            MemoryScalar::UInt8 => "load_uint8",
            MemoryScalar::UInt16 => "load_uint16",
            MemoryScalar::UInt32 => "load_uint32",
            MemoryScalar::UInt64 => "load_uint64",
            MemoryScalar::Float32 => "load_float32",
            MemoryScalar::Float64 => "load_float64",
        },
        MemoryPrimitive::Store(scalar) => match scalar {
            MemoryScalar::Int8 => "store_int8",
            MemoryScalar::Int16 => "store_int16",
            MemoryScalar::Int32 => "store_int32",
            MemoryScalar::Int64 => "store_int64",
            MemoryScalar::UInt8 => "store_uint8",
            MemoryScalar::UInt16 => "store_uint16",
            MemoryScalar::UInt32 => "store_uint32",
            MemoryScalar::UInt64 => "store_uint64",
            MemoryScalar::Float32 => "store_float32",
            MemoryScalar::Float64 => "store_float64",
        },
        MemoryPrimitive::LoadPtr => "load_ptr",
        MemoryPrimitive::StorePtr => "store_ptr",
        MemoryPrimitive::LoadSymbol => "load_symbol",
        MemoryPrimitive::StoreSymbol => "store_symbol",
        MemoryPrimitive::OffsetForward | MemoryPrimitive::OffsetBackward => {
            unreachable!("pointer offsets are operators, not function values")
        }
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
