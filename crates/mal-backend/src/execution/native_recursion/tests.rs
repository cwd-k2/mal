use mal_syntax::source::{FileId, SourceFile};

fn lower(source: &str) -> super::super::Program {
    let source = SourceFile::new(FileId::new(108), "native-parameter.mal", source.into());
    let checked = mal_frontend::analysis::check(&source).expect("check native parameter fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize native parameter fixture"),
    );
    let anf = crate::anf::lower(&core);
    super::super::lower(
        crate::closure::convert(&anf),
        super::super::OptimizationSet::production(),
    )
}

#[test]
fn borrows_a_managed_parameter_preserved_by_every_native_self_edge() {
    let execution = lower(
        "walk :: (Buffer<Int32>, Int32) -> Int32 := (values, depth) -> {
           if (depth == 0i32) then { values.get(0usize) } else {
             child := walk(values, depth - 1i32);
             child + values.get(0usize);
           };
         };
         main :: Unit -> Int32 := () -> {
           values := make<Int32>(1usize);
           values.new(1i32);
           walk(values, 2i32) - 3i32;
         };",
    );
    let function = execution
        .control
        .functions
        .iter()
        .find(|function| execution.native_recursion.has_native_version(function.id))
        .expect("native recursive function");

    assert!(execution.native_recursion.borrows_parameter(function.id));
    let parameter = execution
        .native_recursion
        .scalar_parameter(function.id)
        .expect("native worker with the changing depth leaf");
    assert_eq!(parameter.varying.len(), 1);
    assert_eq!(parameter.varying[0].path, [1]);
}

#[test]
fn owns_a_managed_parameter_changed_by_a_native_self_edge() {
    let execution = lower(
        "walk :: (Symbol, Int32) -> Int32 := (text, depth) -> {
           if (depth == 0i32) then { (#text).i32 } else {
             next := text + \"x\";
             child := walk(next, depth - 1i32);
             child + (#text).i32;
           };
         };
         main :: Unit -> Int32 := () -> { walk(\"a\", 2i32) - 6i32; };",
    );
    let function = execution
        .control
        .functions
        .iter()
        .find(|function| execution.native_recursion.has_native_version(function.id))
        .expect("native recursive function");

    assert!(!execution.native_recursion.borrows_parameter(function.id));
}

#[test]
fn scalarizes_only_changing_leaves_of_a_native_parameter() {
    let execution = lower(
        "walk :: (Int64, Int64, Int64) -> Int64 := (fixed, scale, depth) -> {
           if (depth == 0i64) then { fixed } else {
             child := walk(fixed, scale, depth - 1i64);
             child + scale;
           };
         };
         main :: Unit -> Int32 := () -> { walk(1i64, 2i64, 2i64).i32 - 5i32; };",
    );
    let function = execution
        .control
        .functions
        .iter()
        .find(|function| execution.native_recursion.has_native_version(function.id))
        .expect("native recursive function");
    let parameter = execution
        .native_recursion
        .scalar_parameter(function.id)
        .expect("scalar native parameter");

    assert_eq!(parameter.varying.len(), 1);
    assert_eq!(parameter.varying[0].path, [2]);
}

#[test]
fn keeps_a_changed_managed_leaf_out_of_the_scalar_worker_plan() {
    let execution = lower(
        "walk :: (Symbol, Int64) -> Int64 := (text, depth) -> {
           if (depth == 0i64) then { 0i64 } else {
             child := walk(text + \"x\", depth - 1i64);
             child + 1i64;
           };
         };
         main :: Unit -> Int32 := () -> { walk(\"a\", 2i64).i32 - 2i32; };",
    );
    let function = execution
        .control
        .functions
        .iter()
        .find(|function| execution.native_recursion.has_native_version(function.id))
        .expect("native recursive function");

    assert!(
        execution
            .native_recursion
            .scalar_parameter(function.id)
            .is_none()
    );
}

#[test]
fn borrows_nested_preserved_buffers_with_a_discarded_alias() {
    let execution = lower(
        "walk :: ((Buffer<Int32>, Buffer<Int32>), Int32) -> Int32 := (index, depth) -> {
           (values, _) := index;
           if (depth == 0i32) then { values.get(0usize) } else {
             child := walk(index, depth - 1i32);
             child + values.get(0usize);
           };
         };
         main :: Unit -> Int32 := () -> {
           values := make<Int32>(1usize);
           unused := make<Int32>(1usize);
           values.new(1i32);
           unused.new(0i32);
           walk((values, unused), 2i32) - 3i32;
         };",
    );
    let function = execution
        .control
        .functions
        .iter()
        .find(|function| execution.native_recursion.has_native_version(function.id))
        .expect("native recursive function");

    assert!(execution.native_recursion.borrows_parameter(function.id));
}
