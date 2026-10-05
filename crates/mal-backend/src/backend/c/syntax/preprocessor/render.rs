use super::*;
use crate::backend::c::syntax::render::{MacroReplacementWriter, RenderWrite};

impl PreprocessorExpr {
    fn render(&self, output: &mut impl RenderWrite) {
        match self {
            Self::Defined(name) => {
                write!(output, "defined({name})").expect("writing generated C cannot fail");
            }
        }
    }
}

impl Directive {
    pub(in crate::backend) fn render(&self) -> String {
        match self {
            Self::IncludeQuoted(path) => format!("#include \"{path}\"\n"),
            Self::IncludeSystem(path) => format!("#include <{path}>\n"),
            Self::Define { name, value } => {
                let mut output = format!("#define {name}");
                if let Some(value) = value {
                    output.push(' ');
                    match value {
                        MacroValue::Expression(expression) => {
                            expression.render(&mut output);
                        }
                        MacroValue::Attribute(Attribute::Unused) => {
                            output.push_str("__attribute__((unused))");
                        }
                    }
                }
                output.push('\n');
                output
            }
            Self::ExpressionDefine {
                name,
                parameters,
                expression,
            } => {
                let mut output = format!("#define {name}(");
                render_macro_parameters(&mut output, parameters);
                output.push_str(") ");
                expression.render(&mut output);
                output.push('\n');
                output
            }
            Self::NamedTypeDefine => {
                "/** Expands a named closed Mal type to its C runtime carrier type. This carries no responsibility or runtime type metadata. */\n\
#define mal_type(name) MAL_DETAIL_NAMED_TYPE(name)\n\
#define MAL_DETAIL_NAMED_TYPE(name) MAL_DETAIL_NAMED_TYPE_EXPAND(name)\n\
#define MAL_DETAIL_NAMED_TYPE_EXPAND(name) mal_##name##_t\n"
                    .into()
            }
            Self::StructuralTypeDefine { name, selector } => format!(
                "#define {name}(...) __typeof__(*{selector}((void (*)(__VA_ARGS__))0))\n"
            ),
            Self::HostLifecycleDefines => {
                "/** Describes a carrier's size, alignment, and storage Share/Drop callbacks. Evaluating this performs no lifecycle operation. */\n\
#define mal_storage(type) mal_detail_storage((type *)0, sizeof(type), _Alignof(type))\n\
#define mal_share(call, value) __extension__ ({ \\\n    __auto_type mal_detail_shared = (value); \\\n    mal_detail_retain((call), &mal_detail_shared); \\\n    mal_detail_shared; \\\n})\n\
#define mal_move(value) __extension__ ({ \\\n    __auto_type *mal_detail_source = &(value); \\\n    __auto_type mal_detail_moved = *mal_detail_source; \\\n    memset(mal_detail_source, 0, sizeof(*mal_detail_source)); \\\n    mal_detail_moved; \\\n})\n\
#define mal_drop(value) ((void)__extension__ ({ \\\n    __auto_type *mal_detail_dropped = &(value); \\\n    mal_detail_release(mal_detail_dropped); \\\n    memset(mal_detail_dropped, 0, sizeof(*mal_detail_dropped)); \\\n}))\n"
                    .replace(
                        "#define mal_share",
                        "/** Returns a new owned responsibility while leaving the source responsibility live. */\n#define mal_share",
                    )
                    .replace(
                        "#define mal_move",
                        "/** Moves one responsibility out of an owned lvalue and leaves that lvalue vacant. */\n#define mal_move",
                    )
                    .replace(
                        "#define mal_drop",
                        "/** Drops one responsibility held by an owned lvalue and leaves that lvalue vacant. */\n#define mal_drop",
                    )
            }
            Self::AggregateLifecycleTemplates => {
                "#define MAL_DETAIL_PRODUCT_RETAIN_FIELD(context, index, member, type) mal_detail_retain(call, &value->member);\n\
#define MAL_DETAIL_PRODUCT_RELEASE_FIELD(context, index, member, type) mal_detail_release(&value->member);\n\
#define MAL_DETAIL_SUM_RETAIN_CASE(context, index, member, type) case UINT32_C(index): { mal_detail_retain(call, &value->payload.member); return; }\n\
#define MAL_DETAIL_SUM_RELEASE_CASE(context, index, member, type) case UINT32_C(index): { mal_detail_release(&value->payload.member); return; }\n\
#define MAL_DETAIL_DEFINE_PRODUCT_LIFECYCLE(host_type, fields, share_name, drop_name, cleanup_name) \\\nstatic inline __attribute__((overloadable)) void mal_detail_retain(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, host_type *value MAL_DETAIL_MAYBE_UNUSED) { fields(MAL_DETAIL_PRODUCT_RETAIN_FIELD, host_type) } \\\nstatic inline __attribute__((overloadable)) void mal_detail_release(host_type *value MAL_DETAIL_MAYBE_UNUSED) { fields(MAL_DETAIL_PRODUCT_RELEASE_FIELD, host_type) } \\\nstatic inline void share_name(MalContext *context, void *carrier) { mal_detail_retain(context, (host_type *)carrier); } \\\nstatic inline void drop_name(void *carrier) { mal_detail_release((host_type *)carrier); } \\\nstatic inline void cleanup_name(host_type *value) { mal_detail_release(value); } \\\nstatic inline __attribute__((overloadable)) mal_storage_descriptor_t mal_detail_storage(host_type *type_marker MAL_DETAIL_MAYBE_UNUSED, size_t size, size_t alignment) { return (mal_storage_descriptor_t){ .size = size, .alignment = alignment, .share = share_name, .drop = drop_name }; }\n\
#define MAL_DETAIL_DEFINE_SUM_LIFECYCLE(host_type, fields, share_name, drop_name, cleanup_name) \\\nstatic inline __attribute__((overloadable)) void mal_detail_retain(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, host_type *value MAL_DETAIL_MAYBE_UNUSED) { switch (value->tag) { fields(MAL_DETAIL_SUM_RETAIN_CASE, host_type) default: return; } } \\\nstatic inline __attribute__((overloadable)) void mal_detail_release(host_type *value MAL_DETAIL_MAYBE_UNUSED) { switch (value->tag) { fields(MAL_DETAIL_SUM_RELEASE_CASE, host_type) default: return; } } \\\nstatic inline void share_name(MalContext *context, void *carrier) { mal_detail_retain(context, (host_type *)carrier); } \\\nstatic inline void drop_name(void *carrier) { mal_detail_release((host_type *)carrier); } \\\nstatic inline void cleanup_name(host_type *value) { mal_detail_release(value); } \\\nstatic inline __attribute__((overloadable)) mal_storage_descriptor_t mal_detail_storage(host_type *type_marker MAL_DETAIL_MAYBE_UNUSED, size_t size, size_t alignment) { return (mal_storage_descriptor_t){ .size = size, .alignment = alignment, .share = share_name, .drop = drop_name }; }\n"
                    .into()
            }
            Self::OwnedTypeDefine => {
                "/** Declares a named managed local that owns one responsibility and drops it on lexical scope exit. */\n\
#define mal_owned(name) mal_type(name) __attribute__((cleanup(MAL_DETAIL_CLEANUP(name))))\n\
#define MAL_DETAIL_CLEANUP(name) MAL_DETAIL_CLEANUP_EXPAND(name)\n\
#define MAL_DETAIL_CLEANUP_EXPAND(name) MAL_DETAIL_CLEANUP_##name\n"
                    .into()
            }
            Self::BufferPushDefine => {
                "#define mal_push(call, buffer, element) __extension__ ({ \\\n    __auto_type mal_detail_element = (element); \\\n    mal_detail_buffer_push((call), (buffer), &mal_detail_element); \\\n})\n"
                    .into()
            }
            Self::BufferMutationDefines => {
                "#define mal_replace(call, buffer, index, element) __extension__ ({ \\\n    __auto_type mal_detail_element = (element); \\\n    mal_runtime_buffer_replace_move((call), (buffer), (index), &mal_detail_element); \\\n})\n\
#define mal_fill(call, buffer, offset, count, element) __extension__ ({ \\\n    __auto_type mal_detail_element = (element); \\\n    mal_runtime_buffer_fill_move((call), (buffer), (offset), (count), &mal_detail_element); \\\n})\n"
                    .into()
            }
            Self::FunctionItemsDefine {
                name,
                parameters,
                declarations,
                definitions,
                trailing_signature,
            } => {
                if declarations.is_empty() && definitions.is_empty() {
                    let mut output = format!("#define {name}(");
                    render_macro_parameters(&mut output, parameters);
                    output.push_str(") ");
                    trailing_signature.render_into(&mut output);
                    output.push('\n');
                    return output;
                }
                let mut prefix = format!("#define {name}(");
                render_macro_parameters(&mut prefix, parameters);
                prefix.push_str(") \\\n");
                let mut output = MacroReplacementWriter::new(prefix);
                for declaration in declarations {
                    declaration.render_into(&mut output);
                    output.push_str(";\n");
                }
                for definition in definitions {
                    definition.render_into(&mut output);
                }
                trailing_signature.render_multiline(&mut output);
                output.push('\n');
                output.finish()
            }
            Self::InvocationsDefine {
                name,
                parameters,
                invocations,
            } => {
                if invocations.is_empty() {
                    let mut output = format!("#define {name}(");
                    render_macro_parameters(&mut output, parameters);
                    output.push_str(")\n");
                    output
                } else {
                    render_replacement(name, parameters, |output| {
                        for invocation in invocations {
                            invocation.render_into(output);
                            output.push('\n');
                        }
                    })
                }
            }
            Self::RecordDefine {
                name,
                parameters,
                definition,
            } => render_replacement(name, parameters, |output| {
                definition.render_into(output);
            }),
            Self::RecordFieldsDefine {
                name,
                parameters,
                fields,
            } => render_replacement(name, parameters, |output| {
                crate::backend::c::syntax::unit::render::render_fields(output, fields, 0);
            }),
            #[cfg(test)]
            Self::InitializersDefine {
                name,
                parameters,
                initializers,
            } => render_replacement(name, parameters, |output| {
                for initializer in initializers {
                    initializer.render(output);
                    output.push_str(",\n");
                }
            }),
            Self::If(condition) => {
                let mut output = String::from("#if ");
                condition.render(&mut output);
                output.push('\n');
                output
            }
            Self::Ifndef(name) => format!("#ifndef {name}\n"),
            Self::Else => "#else\n".into(),
            Self::Endif => "#endif\n".into(),
        }
    }
}

fn render_replacement(
    name: &Identifier,
    parameters: &[MacroParameter],
    render: impl FnOnce(&mut MacroReplacementWriter),
) -> String {
    let mut prefix = format!("#define {name}(");
    render_macro_parameters(&mut prefix, parameters);
    prefix.push_str(") \\\n");
    let mut output = MacroReplacementWriter::new(prefix);
    render(&mut output);
    output.finish()
}

impl MacroInvocation {
    pub(in crate::backend::c::syntax) fn render_into(&self, output: &mut impl RenderWrite) {
        write!(output, "{}(", self.name).expect("writing generated C cannot fail");
        for (index, argument) in self.arguments.iter().enumerate() {
            if index != 0 {
                output.push_str(", ");
            }
            argument.render(output);
        }
        output.push(')');
    }
}

fn render_macro_parameters(output: &mut String, parameters: &[MacroParameter]) {
    for (index, parameter) in parameters.iter().enumerate() {
        if index != 0 {
            output.push_str(", ");
        }
        output.push_str(&parameter.0);
    }
}
