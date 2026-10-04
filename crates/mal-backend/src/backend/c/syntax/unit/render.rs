use super::*;

impl Declaration {
    pub(in crate::backend) fn render(&self) -> String {
        let declaration = match self {
            Self::Function(signature) => signature.render(),
            Self::TypeAlias { source, alias } => {
                format!("typedef {}", source.render_declarator(alias))
            }
            Self::FunctionPointerTypeAlias {
                result,
                alias,
                parameters,
            } => format!(
                "typedef {}",
                VariableDeclaration::function_pointer(
                    result.clone(),
                    alias.clone(),
                    parameters.clone(),
                )
                .render()
            ),
            Self::StaticAssert { condition, message } => {
                format!("_Static_assert({condition}, {})", Expr::string(message))
            }
        };
        format!("{declaration};\n")
    }
}

impl Comment {
    pub(in crate::backend) fn render(&self) -> String {
        format!("/* {} */\n", self.0.replace("*/", "* /"))
    }
}

impl RecordDefinition {
    pub(in crate::backend) fn render(&self) -> String {
        let mut output = String::new();
        self.render_into(&mut output);
        output
    }

    pub(in crate::backend::c::syntax) fn render_into(
        &self,
        output: &mut impl crate::backend::c::syntax::render::RenderWrite,
    ) {
        if self.is_typedef {
            output.push_str("typedef ");
        }
        output.push_str(self.kind.keyword());
        if let Some(tag) = &self.tag {
            output.push(' ');
            output.push_str(tag);
        }
        if self.tag.is_none()
            && self.is_typedef
            && self
                .fields
                .iter()
                .all(|field| matches!(field, RecordField::Declaration(_)))
        {
            output.push_str(" { ");
            for field in &self.fields {
                let RecordField::Declaration(declaration) = field else {
                    unreachable!()
                };
                output.push_str(&declaration.render());
                output.push_str("; ");
            }
            output.push('}');
            if let Some(alias) = &self.alias {
                output.push(' ');
                output.push_str(alias);
            }
            output.push_str(";\n");
            return;
        }

        output.push_str(" {\n");
        render_fields(output, &self.fields, 1);
        output.push('}');
        if let Some(alias) = &self.alias {
            output.push(' ');
            output.push_str(alias);
        }
        output.push_str(";\n");
    }
}

pub(in crate::backend::c::syntax) fn render_fields(
    output: &mut impl crate::backend::c::syntax::render::RenderWrite,
    fields: &[RecordField],
    depth: usize,
) {
    for field in fields {
        for _ in 0..depth {
            output.push_str("    ");
        }
        match field {
            RecordField::Declaration(declaration) => {
                output.push_str(&declaration.render());
                output.push_str(";\n");
            }
            RecordField::MacroInvocation(invocation) => {
                invocation.render_into(output);
                output.push('\n');
            }
            RecordField::Record { kind, fields, name } => {
                output.push_str(kind.keyword());
                output.push_str(" {\n");
                render_fields(output, fields, depth + 1);
                for _ in 0..depth {
                    output.push_str("    ");
                }
                output.push_str("} ");
                output.push_str(name);
                output.push_str(";\n");
            }
        }
    }
}

impl RecordKind {
    fn keyword(self) -> &'static str {
        match self {
            Self::Struct => "struct",
            Self::Union => "union",
        }
    }
}
