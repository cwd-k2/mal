use crate::closure::ast::{Block, Operation, Program};

pub(in crate::call_pattern::parameter_lift) fn for_each_block<'a>(
    program: &'a Program,
    visit: &mut impl FnMut(&'a Block),
) {
    fn walk<'a>(block: &'a Block, visit: &mut impl FnMut(&'a Block)) {
        visit(block);
        for binding in &block.bindings {
            match &binding.operation {
                Operation::Case { arms, .. } => {
                    for arm in arms {
                        walk(&arm.value, visit);
                    }
                }
                Operation::PrimitiveBranch {
                    otherwise, then, ..
                } => {
                    walk(otherwise, visit);
                    walk(then, visit);
                }
                _ => {}
            }
        }
    }
    for binding in &program.bindings {
        walk(&binding.value, visit);
    }
    for function in &program.functions {
        walk(&function.body, visit);
        for join in &function.joins {
            walk(&join.body, visit);
        }
    }
}
