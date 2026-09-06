#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum Directive {
    IncludeQuoted(String),
    IncludeSystem(String),
    Define {
        name: String,
        replacement: String,
    },
    FunctionDefine {
        name: String,
        parameters: Vec<String>,
        replacement: Vec<String>,
    },
    If(String),
    Ifndef(String),
    Else,
    Endif,
    Error(String),
    Pragma(String),
}

impl Directive {
    pub(in crate::c_emit) fn define(
        name: impl Into<String>,
        replacement: impl Into<String>,
    ) -> Self {
        Self::Define {
            name: name.into(),
            replacement: replacement.into(),
        }
    }

    pub(in crate::c_emit) fn function_define(
        name: impl Into<String>,
        parameters: impl IntoIterator<Item = impl Into<String>>,
        replacement: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self::FunctionDefine {
            name: name.into(),
            parameters: parameters.into_iter().map(Into::into).collect(),
            replacement: replacement.into_iter().map(Into::into).collect(),
        }
    }

    pub(in crate::c_emit) fn render(&self) -> String {
        match self {
            Self::IncludeQuoted(path) => format!("#include \"{path}\"\n"),
            Self::IncludeSystem(path) => format!("#include <{path}>\n"),
            Self::Define { name, replacement } if replacement.is_empty() => {
                format!("#define {name}\n")
            }
            Self::Define { name, replacement } => format!("#define {name} {replacement}\n"),
            Self::FunctionDefine {
                name,
                parameters,
                replacement,
            } => {
                let mut output = format!("#define {name}({})", parameters.join(", "));
                if replacement.is_empty() {
                    output.push('\n');
                    return output;
                }
                if replacement.len() == 1 {
                    output.push(' ');
                    output.push_str(&replacement[0]);
                    output.push('\n');
                    return output;
                }
                output.push_str(" \\\n");
                for (index, line) in replacement.iter().enumerate() {
                    output.push_str("    ");
                    output.push_str(line);
                    if index + 1 != replacement.len() {
                        output.push_str(" \\\n");
                    } else {
                        output.push('\n');
                    }
                }
                output
            }
            Self::If(condition) => format!("#if {condition}\n"),
            Self::Ifndef(name) => format!("#ifndef {name}\n"),
            Self::Else => "#else\n".into(),
            Self::Endif => "#endif\n".into(),
            Self::Error(message) => format!("#error {message}\n"),
            Self::Pragma(value) => format!("#pragma {value}\n"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Directive;

    #[test]
    fn renders_multiline_function_macros() {
        let directive =
            Directive::function_define("APPLY", ["name", "value"], ["call(name,", "    value)"]);
        assert_eq!(
            directive.render(),
            "#define APPLY(name, value) \\\n    call(name, \\\n        value)\n"
        );
    }
}
