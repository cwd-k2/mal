use crate::anf;
use crate::check::ast::MemoryPrimitive;
use crate::closure::ast::FunctionId;

pub(super) fn value_name(id: anf::ast::ValueId) -> String {
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

pub(super) fn function_name(id: FunctionId) -> String {
    match id {
        FunctionId::Lambda(id) => format!("mal_function_{}", id.0),
        FunctionId::Memory(primitive) => {
            format!("mal_memory_function_{}", memory_primitive_name(primitive))
        }
    }
}

pub(super) fn direct_function_name(id: FunctionId) -> String {
    match id {
        FunctionId::Lambda(id) => format!("mal_direct_function_{}", id.0),
        FunctionId::Memory(primitive) => format!(
            "mal_direct_memory_function_{}",
            memory_primitive_name(primitive)
        ),
    }
}

pub(super) fn environment_name(id: FunctionId) -> String {
    match id {
        FunctionId::Lambda(id) => format!("MalEnvironment_{}", id.0),
        FunctionId::Memory(primitive) => {
            format!("MalMemoryEnvironment_{}", memory_primitive_name(primitive))
        }
    }
}

pub(super) fn stack_environment_name(id: anf::ast::ValueId) -> String {
    format!("mal_stack_environment_{}", value_name(id))
}

pub(super) fn environment_destroy_name(id: FunctionId) -> String {
    match id {
        FunctionId::Lambda(id) => format!("mal_destroy_environment_{}", id.0),
        FunctionId::Memory(primitive) => format!(
            "mal_destroy_memory_environment_{}",
            memory_primitive_name(primitive)
        ),
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
