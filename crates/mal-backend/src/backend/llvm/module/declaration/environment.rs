use super::{declaration, strings};
use crate::backend::llvm::body;
use crate::backend::llvm::syntax::Module;

pub(super) fn add(module: &mut Module<'_>, types: &body::types::Types) {
    let index = types.index_integer();
    module.declare(declaration(
        "ptr",
        "mal_runtime_environment_allocate",
        ["ptr".into(), index.clone(), "ptr".into()],
    ));
    module.declare(declaration(
        "ptr",
        "mal_runtime_environment_retain",
        strings(["ptr", "ptr"]),
    ));
    module.declare(declaration(
        "void",
        "mal_runtime_environment_release",
        strings(["ptr"]),
    ));
    module.declare(
        declaration("i8", "mal_runtime_environment_is_unique", strings(["ptr"])).with_attributes(
            strings(["nofree", "nounwind", "willreturn", "memory(argmem: read)"]),
        ),
    );
    module.declare(declaration(
        "ptr",
        "llvm.invariant.start.p0",
        strings(["i64", "ptr"]),
    ));
    module.declare(declaration(
        "ptr",
        format!("llvm.ptrmask.p0.i{}", types.pointer_size() * 8),
        ["ptr".into(), types.pointer_representation_integer()],
    ));
    module.declare(declaration(
        "void",
        format!("llvm.memcpy.p0.p0.i{}", types.index_size() * 8),
        ["ptr".into(), "ptr".into(), index, "i1 immarg".into()],
    ));
}
