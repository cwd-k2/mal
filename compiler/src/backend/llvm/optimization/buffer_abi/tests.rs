use super::*;
use crate::control::ast::Operation;
use crate::core::ast::PackedBuilderOperation;
use crate::source::{FileId, SourceFile};

#[test]
fn admits_only_closed_direct_buffer_abis() {
    let source = SourceFile::new(
        FileId::new(92),
        "direct-buffer-abi.mal",
        "read :: Buffer<Int64> -> Int64 := (buffer) -> { buffer.get(0usize); };\n\
         grow :: Buffer<Int64> -> Int64 := (buffer) -> { _ := buffer.new(2i64); buffer.get(0usize); };\n\
         forwardGrow :: Buffer<Int64> -> Int64 := (buffer) -> { grow(buffer); };\n\
         identity :: Buffer<Int64> -> Buffer<Int64> := (buffer) -> { buffer; };\n\
         capture :: Buffer<Int64> -> Int64 := (buffer) -> { nested :: (Unit -> Int64) := () -> { buffer.get(0usize); }; nested(); };\n\
         mixedRead :: Buffer<Int32> -> Int32 := (buffer) -> { buffer.get(0usize); };\n\
         mixedGrow :: Buffer<Int32> -> Int32 := (buffer) -> { _ := buffer.new(2i32); buffer.get(0usize); };\n\
         applyMixed :: ((Buffer<Int32> -> Int32), Buffer<Int32>) -> Int32 := (function, buffer) -> { function(buffer); };\n\
         framed :: ((Buffer<Int64>, Buffer<Int64>), Int32) -> Int64 := ((left, right), remaining) -> { if (remaining == 0i32) then { left.get(0usize) } else { prior := framed(((left, right), remaining - 1i32)); prior + right.get(0usize) }; };\n\
         main :: Unit -> Int32 := () -> { values := pack<Int64>((buffer) -> { _ := buffer.new(1i64); _ := read(buffer); _ := forwardGrow(buffer); _ := capture(buffer); _ := framed(((buffer, buffer), 1i32)); returned := identity(buffer); captured :: (Unit -> Int64) := () -> { returned.get(0usize); }; _ := captured(); (); }); other := pack<Int32>((buffer) -> { _ := buffer.new(1i32); _ := applyMixed((mixedRead, buffer)); _ := applyMixed((mixedGrow, buffer)); (); }); (values # 0usize).i32 + (other # 0usize) - 2i32; };"
            .into(),
    );
    let checked = crate::pipeline::check(&source).expect("check direct Buffer ABI fixture");
    let core = crate::core::lower(
        &crate::check::specialize(checked).expect("specialize direct Buffer ABI fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    let direct = plan(&execution);

    let get_only = execution
        .control
        .functions
        .iter()
        .filter(|function| {
            analysis::has_no_relocation(&execution, function.entry)
                && analysis::reachable_states(&execution.control, function.entry).any(|state| {
                    execution.control.states[state.0]
                        .bindings
                        .iter()
                        .any(|binding| {
                            matches!(
                                &binding.operation,
                                Operation::PackedBuilder {
                                    operation: PackedBuilderOperation::Get,
                                    ..
                                }
                            )
                        })
                })
        })
        .map(|function| function.id)
        .collect::<Vec<_>>();
    assert!(!direct.is_empty());
    assert!(direct.iter().all(|function| get_only.contains(function)));
    assert!(get_only.iter().any(|function| direct.contains(function)));
    assert!(get_only.iter().any(|function| !direct.contains(function)));

    for function in &execution.lowered.functions {
        if contains_buffer(&function.body.result.ty)
            || function
                .kind
                .captures()
                .is_some_and(|captures| captures.iter().any(|capture| contains_buffer(&capture.ty)))
        {
            assert!(!direct.contains(&function.id));
        }
    }
    for function in &execution.control.functions {
        if analysis::captures_buffer_in_nested_closure(&execution, function.entry) {
            assert!(!direct.contains(&function.id));
        }
    }
    let framed_multi_buffer = execution
        .control
        .functions
        .iter()
        .filter(|function| {
            execution
                .lowered
                .functions
                .iter()
                .find(|lowered| lowered.id == function.id)
                .is_some_and(|lowered| {
                    contains_buffer(&lowered.parameter.ty)
                        && !shape::has_single_buffer(&lowered.parameter.ty)
                })
                && !analysis::has_no_control_frame(&execution, function.entry)
        })
        .map(|function| function.id)
        .collect::<Vec<_>>();
    assert!(!framed_multi_buffer.is_empty());
    assert!(
        framed_multi_buffer
            .iter()
            .all(|function| !direct.contains(function))
    );

    let mixed_site = execution
        .applications
        .sites()
        .find(|(site, _)| {
            execution.applications.direct_target(*site).is_none()
                && execution
                    .applications
                    .targets(*site)
                    .is_some_and(|targets| targets.len() > 1)
                && analysis::call_argument(&execution.control.states[site.0].terminator)
                    .is_some_and(|argument| contains_buffer(&argument.ty))
        })
        .expect("mixed indirect Buffer site");
    assert!(
        execution
            .applications
            .targets(mixed_site.0)
            .expect("mixed targets")
            .iter()
            .all(|target| !direct.contains(target))
    );
}
