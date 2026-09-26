use super::{declaration, strings};
use crate::backend::llvm::body;
use crate::backend::llvm::syntax::Module;

pub(super) fn add(module: &mut Module<'_>, types: &body::types::Types) {
    let index = types.index_integer();
    module.declare(declaration(
        "ptr",
        "mal_control_reserve_frame",
        ["ptr".into(), index.clone(), index.clone()],
    ));
    module.declare(declaration("ptr", "mal_control_storage", strings(["ptr"])));
    module.declare(declaration(index, "mal_control_capacity", strings(["ptr"])));
    module.declare(declaration(
        "void",
        "mal_native_stack_begin",
        strings(["ptr"]),
    ));
    module.declare(
        declaration("i8", "mal_native_stack_is_deep", strings(["ptr"])).with_attributes(strings([
            "nofree",
            "nounwind",
            "willreturn",
            "memory(argmem: read)",
        ])),
    );
    module.declare(declaration("i1", "llvm.expect.i1", strings(["i1", "i1"])));
}
