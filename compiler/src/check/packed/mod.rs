mod buffer;
mod build;

use super::ast::Type;

fn callback_type(element: &Type) -> Type {
    Type::Function {
        parameter: Type::Buffer(element.clone().into()).into(),
        result: Type::Unit.into(),
    }
}
