mod expression;
mod function;
mod statement;

pub(in crate::backend) use expression::{
    c_expr, c_initializer, c_initializers, c_initializers_item,
};
pub(in crate::backend) use function::c_function;
pub(in crate::backend) use statement::{
    c_block, c_block_item, c_statement, c_switch_case, c_switch_cases, c_switch_cases_item,
};

#[cfg(test)]
mod tests;
