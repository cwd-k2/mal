use std::fmt::{self, Display, Formatter};

use super::*;

impl TypeName {
    pub(in crate::backend) fn render_declarator(&self, declarator: &str) -> String {
        let mut output = String::new();
        if self.is_const {
            output.push_str("const ");
        }
        self.base.render(&mut output);
        if self.pointer_const.is_empty() {
            output.push(' ');
        } else {
            output.push(' ');
            for is_const in &self.pointer_const {
                output.push('*');
                if *is_const {
                    output.push_str("const ");
                }
            }
        }
        output.push_str(declarator);
        output
    }
}

impl Display for TypeName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        if self.is_const {
            formatter.write_str("const ")?;
        }
        match &self.base {
            TypeBase::Named(name) => formatter.write_str(name)?,
            TypeBase::Struct(tag) => write!(formatter, "struct {tag}")?,
        }
        if !self.pointer_const.is_empty() {
            formatter.write_str(" ")?;
            for is_const in &self.pointer_const {
                formatter.write_str("*")?;
                if *is_const {
                    formatter.write_str("const ")?;
                }
            }
        }
        Ok(())
    }
}

impl TypeBase {
    fn render(&self, output: &mut String) {
        match self {
            Self::Named(name) => output.push_str(name),
            Self::Struct(tag) => {
                output.push_str("struct ");
                output.push_str(tag);
            }
        }
    }
}

impl Declarator {
    fn render(&self, ty: &TypeName) -> String {
        match self {
            Self::Identifier(name) => ty.render_declarator(name),
            Self::Array { name, size } => format!("{}[{size}]", ty.render_declarator(name)),
            Self::FunctionPointer { name, parameters } => {
                format!("{} (*{name})({})", ty, render_parameters(parameters))
            }
        }
    }
}

impl VariableDeclaration {
    pub(in crate::backend) fn render(&self) -> String {
        let declaration = self.declarator.render(&self.ty);
        let mut output = String::new();
        if let Some(alignment) = &self.alignment {
            output.push_str(&format!("_Alignas({alignment}) "));
        }
        if self.is_static {
            output.push_str("static ");
        }
        output.push_str(&declaration);
        output
    }
}

impl Parameter {
    pub(in crate::backend) fn render(&self) -> String {
        let mut output = match &self.name {
            Some(name) => self.ty.render_declarator(name),
            None => self.ty.to_string(),
        };
        if self.maybe_unused {
            output.push_str(" MAL_DETAIL_MAYBE_UNUSED");
        }
        output
    }
}

impl FunctionSpecifier {
    fn spelling(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Inline => "inline",
            Self::NoReturn => "_Noreturn",
        }
    }
}

impl FunctionSignature {
    pub(in crate::backend) fn render(&self) -> String {
        let mut output = String::new();
        for specifier in &self.specifiers {
            output.push_str(match specifier {
                FunctionSpecifier::Static => "static ",
                FunctionSpecifier::Inline => "inline ",
                FunctionSpecifier::NoReturn => "_Noreturn ",
            });
        }
        output.push_str(&self.result.render_declarator(&self.name));
        output.push('(');
        output.push_str(&render_parameters(&self.parameters));
        output.push(')');
        output
    }

    pub(in crate::backend) fn render_macro(&self, output: &mut String) {
        for specifier in &self.specifiers {
            output.push_str(specifier.spelling());
            output.push(' ');
        }
        output.push_str(&self.result.render_declarator(&self.name));
        if self.parameters.is_empty() {
            output.push_str("(void)");
            return;
        }
        output.push_str("( \\\n");
        for (index, parameter) in self.parameters.iter().enumerate() {
            output.push_str("    ");
            output.push_str(&parameter.render());
            if index + 1 != self.parameters.len() {
                output.push(',');
            }
            output.push_str(" \\\n");
        }
        output.push(')');
    }
}

fn render_parameters(parameters: &[Parameter]) -> String {
    if parameters.is_empty() {
        return "void".into();
    }
    let mut output = String::new();
    for (index, parameter) in parameters.iter().enumerate() {
        if index != 0 {
            output.push_str(", ");
        }
        output.push_str(&parameter.render());
    }
    output
}
