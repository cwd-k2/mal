use crate::backend::c::syntax::{
    Block, Expr, FunctionDefinition, FunctionSignature, FunctionSpecifier, Parameter, Statement,
    SwitchCase, TranslationUnit, TypeName, c_block, c_expr, c_function, c_initializers,
    c_signature,
};
use crate::core::ast::TypeAlias;
use mal_frontend::check::ast::Type;

use super::super::{HostTypes, TypeRegistry, is_bool};

impl TypeRegistry {
    pub(in crate::backend::c) fn host_lifecycle_helpers(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for ty in &self.aggregates {
            if !host.contains(ty) || is_bool(ty) {
                continue;
            }
            let id = self.index(ty);
            let guard = format!("MAL_DETAIL_HOST_REPR_{id}_LIFECYCLE");
            let mut helpers = TranslationUnit::default();
            let host_type = self.host_value_c_type(ty, None);
            helpers.push(FunctionDefinition::from_signature(
                lifecycle_signature("mal_detail_retain", host_type.clone(), true),
                self.aggregate_lifecycle_body(ty, true),
            ));
            helpers.push(FunctionDefinition::from_signature(
                lifecycle_signature("mal_detail_release", host_type.clone(), false),
                self.aggregate_lifecycle_body(ty, false),
            ));
            if self.has_managed_leaf(ty) {
                helpers.extend(self.aggregate_storage_helpers(ty, host_type));
            }
            output.extend(crate::backend::c::syntax::c_items! {
                if !defined({ guard.clone() }) {
                    define!({ guard });
                    ..{ helpers }
                }
            });
        }
        for name in host.opaque_names.iter().chain(
            aliases
                .iter()
                .filter(|alias| host.exposes_alias(alias))
                .map(|alias| &alias.name),
        ) {
            output.push(c_function! {
                #[static] #[inline] fn { format!("mal_detail_cleanup_{name}") }(
                    value: *mut { format!("mal_{name}_t") },
                ) -> void {
                    mal_detail_release(value);
                    memset(value, 0, sizeof(*value));
                }
            });
        }
        output
    }

    fn aggregate_lifecycle_body(&self, ty: &Type, retain: bool) -> Block {
        match ty {
            Type::Product(elements) => {
                let mut body = Block::default();
                for index in 0..elements.len() {
                    body.push(lifecycle_statement(index, false, retain));
                }
                body
            }
            Type::Sum(members) => {
                if members.is_empty() {
                    return Block::default();
                }
                let cases: Vec<_> = (0..members.len())
                    .map(|index| {
                        let mut body = Block::default();
                        body.push(lifecycle_statement(index, true, retain));
                        body.push(Statement::return_void());
                        SwitchCase::case(c_expr!(UINT32_C({ index })), body)
                    })
                    .chain(std::iter::once(SwitchCase::default(c_block!(return;))))
                    .collect();
                c_block! {
                    match (*value).tag {
                        ..{ cases },
                    }
                }
            }
            _ => unreachable!("only products and sums have aggregate lifecycle"),
        }
    }

    fn aggregate_storage_helpers(&self, ty: &Type, host_type: TypeName) -> TranslationUnit {
        let id = self.index(ty);
        let share = format!("mal_detail_storage_share_{id}");
        let drop = format!("mal_detail_storage_drop_{id}");
        let mut output = TranslationUnit::default();
        output.push(c_function! {
            #[static] #[inline] fn { share.clone() }(
                context: *mut MalContext,
                carrier: *mut void,
            ) -> void {
                let call: mal_call_t = mal_call_t { mal_detail_context: context };
                mal_detail_retain(&call, carrier as *mut { host_type.clone() });
            }
        });
        output.push(c_function! {
            #[static] #[inline] fn { drop.clone() }(
                carrier: *mut void,
            ) -> void {
                mal_detail_release(carrier as *mut { host_type.clone() });
            }
        });
        let descriptor = Expr::compound_literal(
            TypeName::named("mal_storage_descriptor_t"),
            c_initializers! {
                size: { c_expr!(size) },
                alignment: { c_expr!(alignment) },
                share: { Expr::identifier(share) },
                drop: { Expr::identifier(drop) },
            },
        );
        output.push(FunctionDefinition::from_signature(
            c_signature! {
                #[static] #[inline] #[overloadable] fn mal_detail_storage(
                    #[maybe_unused] type_marker: *mut { host_type },
                    size: size_t,
                    alignment: size_t,
                ) -> mal_storage_descriptor_t
            },
            c_block!(return { descriptor };),
        ));
        output
    }

    fn has_managed_leaf(&self, ty: &Type) -> bool {
        let mut pending = vec![ty];
        while let Some(ty) = pending.pop() {
            match ty {
                Type::Symbol | Type::Buffer(_) => return true,
                Type::Product(elements) | Type::Sum(elements) => {
                    pending.extend(elements.iter());
                }
                _ => {}
            }
        }
        false
    }
}

fn lifecycle_signature(name: &str, ty: TypeName, retain: bool) -> FunctionSignature {
    let mut parameters = Vec::new();
    if retain {
        parameters
            .push(Parameter::named(TypeName::named("mal_call_t").pointer(), "call").maybe_unused());
    }
    parameters.push(Parameter::named(ty.pointer(), "value").maybe_unused());
    FunctionSignature::new(TypeName::named("void"), name, parameters).with_specifiers([
        FunctionSpecifier::Static,
        FunctionSpecifier::Inline,
        FunctionSpecifier::Overloadable,
    ])
}

fn lifecycle_statement(index: usize, sum: bool, retain: bool) -> Statement {
    let value = Expr::dereference(Expr::identifier("value"));
    let member = if sum {
        value.field("payload").field(format!("variant_{index}"))
    } else {
        value.field(format!("field_{index}"))
    };
    let mut arguments = Vec::new();
    if retain {
        arguments.push(Expr::identifier("call"));
    }
    arguments.push(Expr::address_of(member));
    Statement::expression(Expr::call(
        Expr::identifier(if retain {
            "mal_detail_retain"
        } else {
            "mal_detail_release"
        }),
        arguments,
    ))
}
