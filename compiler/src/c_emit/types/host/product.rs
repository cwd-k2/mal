use crate::c_emit::syntax::{
    Block, Expr, FunctionSignature, Initializer, Parameter, Statement, TranslationUnit,
};
use crate::check::ast::Type;
use crate::core::ast::TypeAlias;

use super::{TypeRegistry, append_function};

impl TypeRegistry {
    pub(super) fn emit_product_constructor(
        &self,
        output: &mut TranslationUnit,
        alias: &TypeAlias,
        elements: &[Type],
    ) {
        let parameters = self.parameters(elements);
        let fields = elements.iter().enumerate().map(|(index, _)| {
            Initializer::designated(
                format!("field_{index}"),
                Expr::identifier(format!("value_{index}")),
            )
        });
        append_function(
            output,
            FunctionSignature::static_inline(
                format!("MalType_{}", alias.name),
                format!("mal_{}_make", alias.name),
                parameters,
            ),
            Block::new([Statement::return_value(Expr::compound_literal(
                format!("MalType_{}", alias.name),
                fields,
            ))]),
        );
    }

    pub(super) fn emit_product_accessors(
        &self,
        output: &mut TranslationUnit,
        alias: &TypeAlias,
        elements: &[Type],
    ) {
        for (index, element) in elements.iter().enumerate() {
            append_function(
                output,
                FunctionSignature::static_inline(
                    self.c_type(element),
                    format!("mal_{}_get_{index}", alias.name),
                    [Parameter::named(format!("MalType_{}", alias.name), "value")],
                ),
                Block::new([Statement::return_value(
                    Expr::identifier("value").field(format!("field_{index}")),
                )]),
            );
        }
    }
}
