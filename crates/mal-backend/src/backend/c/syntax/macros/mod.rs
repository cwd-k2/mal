//! Rust-shaped procedural entry macros for typed C syntax nodes.

pub(in crate::backend) use mal_backend_macros::{
    c_block, c_expr, c_function, c_invocation, c_items, c_parameter, c_parameters, c_record,
    c_record_fields, c_signature, c_statement, c_type,
};
#[cfg(test)]
pub(in crate::backend) use mal_backend_macros::{c_initializers, c_switch_cases};

#[cfg(test)]
mod tests;
