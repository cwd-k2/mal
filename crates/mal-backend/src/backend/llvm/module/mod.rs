use super::body;
use super::syntax::{FunctionDeclaration, Module};
use super::{Target, TargetLayout};

mod declaration;
mod entry;
mod metadata;

pub(super) fn render(
    body: &body::Output,
    target: Target<'_>,
    layout: TargetLayout,
    external_declarations: impl IntoIterator<Item = FunctionDeclaration>,
) -> Option<String> {
    let types = body::types::Types::for_target(layout);
    let mut module = Module::new(target.triple, target.data_layout);

    declaration::add_environment(&mut module, &types);
    if body.uses_control {
        declaration::add_control(&mut module, &types);
    }
    if body.uses_byte_runtime {
        declaration::add_byte_runtime(&mut module, &types);
    }
    for declaration in external_declarations {
        module.declare(declaration);
    }

    for global in &body.globals {
        module.add_global(global.clone());
    }
    for definition in &body.definitions {
        module.define(definition.clone());
    }
    if body.uses_byte_runtime {
        module.add_metadata(metadata::buffer_alias());
    }
    module.define(entry::definition(body, &types)?);
    module.render()
}
