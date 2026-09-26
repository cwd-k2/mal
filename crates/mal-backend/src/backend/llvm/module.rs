use super::body;
use super::syntax::{BasicBlock, FunctionDeclaration, FunctionDefinition, Module};
use super::{Target, TargetLayout, function_name};
use crate::backend::abi::Function as AbiFunction;

pub(super) fn render(
    body: &body::Output,
    target: Target<'_>,
    layout: TargetLayout,
    external_signatures: impl IntoIterator<Item = String>,
) -> Option<String> {
    let types = body::types::Types::for_target(layout);
    let mut module = Module::new(target.triple, target.data_layout);

    add_environment_declarations(&mut module, &types);
    if body.uses_control {
        add_control_declarations(&mut module, &types);
    }
    if body.uses_byte_runtime {
        add_byte_declarations(&mut module, &types);
    }
    for signature in external_signatures {
        module.declare(FunctionDeclaration::new(signature));
    }

    module.add_global_fragment(body.globals.clone());
    for definition in &body.definitions {
        module.define(definition.clone());
    }
    if body.uses_byte_runtime {
        module.add_metadata(buffer_alias_metadata());
    }
    module.define(entry_definition(body, &types)?);
    Some(module.render())
}

fn add_environment_declarations(module: &mut Module<'_>, types: &body::types::Types) {
    module.declare(FunctionDeclaration::new(format!(
        "ptr @mal_runtime_environment_allocate(ptr, {}, ptr)",
        types.index_integer()
    )));
    module.declare(FunctionDeclaration::new(
        "ptr @mal_runtime_environment_retain(ptr, ptr)",
    ));
    module.declare(FunctionDeclaration::new(
        "void @mal_runtime_environment_release(ptr)",
    ));
    module.declare(FunctionDeclaration::new(
        "i8 @mal_runtime_environment_is_unique(ptr) nofree nounwind willreturn memory(argmem: read)",
    ));
    module.declare(FunctionDeclaration::new(
        "ptr @llvm.invariant.start.p0(i64, ptr)",
    ));
    module.declare(FunctionDeclaration::new(format!(
        "ptr @llvm.ptrmask.p0.i{}(ptr, {})",
        types.pointer_size() * 8,
        types.pointer_representation_integer()
    )));
    module.declare(FunctionDeclaration::new(format!(
        "void @llvm.memcpy.p0.p0.i{}(ptr, ptr, {}, i1 immarg)",
        types.index_size() * 8,
        types.index_integer()
    )));
}

fn add_control_declarations(module: &mut Module<'_>, types: &body::types::Types) {
    let index = types.index_integer();
    for signature in [
        format!("ptr @mal_control_reserve_frame(ptr, {index}, {index})"),
        "ptr @mal_control_storage(ptr)".into(),
        format!("{index} @mal_control_capacity(ptr)"),
        "void @mal_native_stack_begin(ptr)".into(),
        "i8 @mal_native_stack_is_deep(ptr) nofree nounwind willreturn memory(argmem: read)".into(),
        "i1 @llvm.expect.i1(i1, i1)".into(),
    ] {
        module.declare(FunctionDeclaration::new(signature));
    }
}

fn add_byte_declarations(module: &mut Module<'_>, types: &body::types::Types) {
    let index = types.index_integer();
    for signature in [
        "ptr @mal_runtime_bytes_data(ptr) nofree nounwind willreturn memory(argmem: read)".into(),
        format!("ptr @mal_runtime_bytes_read(ptr, ptr, {index})"),
        "ptr @mal_runtime_bytes_retain(ptr, ptr)".into(),
        "void @mal_runtime_bytes_release(ptr)".into(),
        format!("void @mal_runtime_bytes_write(ptr, ptr, {index}, {index})"),
        format!("ptr @mal_runtime_buffer_make(ptr, {index}, {index})"),
        format!("ptr @mal_runtime_buffer_make_managed(ptr, {index}, {index}, ptr, ptr)"),
        format!("{index} @mal_runtime_buffer_new_managed(ptr, ptr, ptr, {index})"),
        format!("void @mal_runtime_buffer_fill_managed(ptr, ptr, {index}, {index}, ptr, {index})"),
        format!(
            "void @mal_runtime_buffer_copy_managed(ptr, ptr, {index}, ptr, {index}, {index}, {index})"
        ),
        format!("{index} @mal_runtime_buffer_new(ptr, ptr, ptr, {index})"),
        format!("void @mal_runtime_buffer_fill(ptr, ptr, {index}, {index}, ptr, {index})"),
        format!("void @mal_runtime_buffer_copy(ptr, ptr, {index}, ptr, {index}, {index}, {index})"),
        "ptr @mal_runtime_buffer_data_slot(ptr) nofree nounwind willreturn memory(none)".into(),
        format!(
            "{index} @mal_runtime_buffer_count(ptr) nofree nounwind willreturn memory(argmem: read)"
        ),
        format!("ptr @mal_runtime_buffer_from(ptr, ptr, {index}, {index}, {index})"),
        format!("void @mal_runtime_buffer_into(ptr, ptr, ptr, {index}, {index}, {index})"),
        format!("i8 @mal_runtime_symbol_at(ptr, {index})"),
        format!(
            "void @mal_runtime_symbol_concatenate(ptr, ptr, ptr, ptr, {index}, ptr, ptr, {index})"
        ),
        format!(
            "void @mal_runtime_symbol_concatenate_consuming_left(ptr, ptr, ptr, ptr, {index}, ptr, ptr, {index})"
        ),
        format!(
            "void @mal_runtime_symbol_concatenate_consuming_right(ptr, ptr, ptr, ptr, {index}, ptr, ptr, {index})"
        ),
        format!("i8 @mal_runtime_symbol_equal(ptr, {index}, ptr, {index})"),
    ] {
        module.declare(FunctionDeclaration::new(signature));
    }
}

fn entry_definition(body: &body::Output, types: &body::types::Types) -> Option<FunctionDefinition> {
    let mut instructions = Vec::new();
    let control_top = if body.uses_control {
        instructions.extend([
            format!(
                "%mal_control_top = alloca {}, align {}",
                types.index_integer(),
                types.index_alignment()
            ),
            format!(
                "store {} 0, ptr %mal_control_top, align {}",
                types.index_integer(),
                types.index_alignment()
            ),
            "call void @mal_native_stack_begin(ptr %mal_context)".into(),
        ]);
        "%mal_control_top"
    } else {
        "null"
    };
    let call = match &body.main_parameter {
        mal_frontend::check::ast::Type::Unit => format!(
            "call i32 @{}(ptr %mal_context, ptr {control_top}, ptr null)",
            function_name(body.main)
        ),
        ty => {
            let value = types.value(ty)?;
            instructions.push(format!(
                "%mal_entry_argument = load {}, ptr %mal_argument, align {}",
                value.llvm, value.alignment
            ));
            format!(
                "call i32 @{}(ptr %mal_context, ptr {control_top}, ptr null, {} %mal_entry_argument)",
                function_name(body.main),
                value.llvm
            )
        }
    };
    instructions.extend([
        format!("%mal_entry_result = {call}"),
        "store i32 %mal_entry_result, ptr %mal_result, align 4".into(),
        "ret void".into(),
    ]);
    Some(FunctionDefinition::new(
        AbiFunction::program_entry().llvm_signature(),
        vec![BasicBlock::new("entry", instructions)?],
    ))
}

fn buffer_alias_metadata() -> &'static str {
    // Buffer object fields and element storage are distinct allocations. Their TBAA types preserve
    // that boundary across inlining. Runtime slot writes do not carry this metadata, so growth still
    // invalidates an active data pointer.
    "!0 = !{!\"Simple C/C++ TBAA\"}\n\
     !1 = !{!\"omnipotent char\", !0, i64 0}\n\
     !2 = !{!\"mal buffer element storage\", !1, i64 0}\n\
     !3 = !{!2, !2, i64 0}\n\
     !4 = distinct !{!4, !\"mal buffer object allocation\"}\n\
     !5 = distinct !{!5, !4, !\"mal buffer object metadata\"}\n\
     !6 = !{!5}\n\
     !7 = !{!\"mal buffer object field\", !1, i64 0}\n\
     !8 = !{!7, !7, i64 0}"
}
