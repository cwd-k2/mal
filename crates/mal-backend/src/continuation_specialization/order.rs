//! Deterministic ordering keys for identities recorded in continuation plans.

use crate::anf::ast::ValueId;
use crate::closure::ast::FunctionId;

pub(super) fn function(function: FunctionId) -> u32 {
    let FunctionId::Lambda(mal_frontend::resolve::ast::LambdaId(number)) = function;
    number
}

pub(super) fn value(value: ValueId) -> u64 {
    match value {
        ValueId::Core(crate::core::ast::ValueId::Source(id)) => u64::from(id.0),
        ValueId::Core(crate::core::ast::ValueId::Temporary(id)) => 1_u64 << 32 | u64::from(id),
        ValueId::Temporary(id) => 2_u64 << 32 | u64::from(id),
    }
}
