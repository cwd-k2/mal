mod declaration;
mod expression;
mod function;
mod statement;

pub(in crate::backend) use declaration::{
    c_parameter, c_parameter_attributes, c_parameters, c_parameters_items, c_signature,
    c_signature_from_parts, c_type,
};
pub(in crate::backend) use expression::{
    c_expr, c_exprs, c_exprs_item, c_initializer, c_initializers, c_initializers_item,
};
pub(in crate::backend) use function::c_function;
pub(in crate::backend) use statement::{
    c_block, c_block_item, c_statement, c_switch_case, c_switch_cases, c_switch_cases_item,
};

#[cfg(test)]
mod tests;
