mod declaration;
mod expression;
mod function;
mod preprocessor;
mod statement;

pub(in crate::backend) use declaration::{
    c_aggregate, c_aggregate_field, c_aggregate_fields, c_aggregate_fields_items, c_comment,
    c_declaration, c_parameter, c_parameter_attributes, c_parameters, c_parameters_items, c_scalar,
    c_signature, c_signature_from_parts, c_type, c_variable,
};
pub(in crate::backend) use expression::{
    c_expr, c_expr_child, c_exprs, c_exprs_items, c_initializer, c_initializers,
    c_initializers_items, c_type_child,
};
pub(in crate::backend) use function::{c_function, c_function_from_syntax};
pub(in crate::backend) use preprocessor::{c_directive, c_macro_invocation, c_preprocessor_expr};
pub(in crate::backend) use statement::{
    c_block, c_block_items, c_statement, c_switch_case, c_switch_cases, c_switch_cases_items,
};

#[cfg(test)]
mod tests;
