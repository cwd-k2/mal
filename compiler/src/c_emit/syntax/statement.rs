use super::{Expr, FunctionSignature, MacroInvocation, VariableDeclaration};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum Statement {
    VariableDeclaration {
        declaration: VariableDeclaration,
        initializer: Option<Expr>,
    },
    Expression(Expr),
    Assignment {
        target: Expr,
        value: Expr,
    },
    Return(Expr),
    Goto(String),
    Break,
    Label {
        name: String,
        body: Block,
    },
    If {
        condition: Expr,
        then: Block,
        otherwise: Option<Block>,
    },
    For {
        initializer: ForInitializer,
        condition: Expr,
        update: Expr,
        body: Block,
    },
    Switch {
        value: Expr,
        cases: Vec<SwitchCase>,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::c_emit) struct Block {
    statements: Vec<Statement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct FunctionDefinition {
    header: FunctionHeader,
    body: Block,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FunctionHeader {
    Signature(FunctionSignature),
    MacroInvocation(MacroInvocation),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct SwitchCase {
    label: Option<Expr>,
    body: Block,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct ForInitializer {
    declaration: VariableDeclaration,
    initializer: Expr,
}

impl Statement {
    pub(in crate::c_emit) fn expression(expression: Expr) -> Self {
        Self::Expression(expression)
    }

    pub(in crate::c_emit) fn variable_declaration(
        declaration: VariableDeclaration,
        initializer: Option<Expr>,
    ) -> Self {
        Self::VariableDeclaration {
            declaration,
            initializer,
        }
    }

    pub(in crate::c_emit) fn variable(
        ty: impl Into<super::TypeName>,
        name: impl Into<String>,
        initializer: Option<Expr>,
    ) -> Self {
        Self::variable_declaration(VariableDeclaration::new(ty, name), initializer)
    }

    pub(in crate::c_emit) fn assignment(target: Expr, value: Expr) -> Self {
        Self::Assignment { target, value }
    }

    pub(in crate::c_emit) fn return_value(value: Expr) -> Self {
        Self::Return(value)
    }

    pub(in crate::c_emit) fn goto(label: impl Into<String>) -> Self {
        Self::Goto(label.into())
    }

    pub(in crate::c_emit) fn label(name: impl Into<String>, body: Block) -> Self {
        Self::Label {
            name: name.into(),
            body,
        }
    }

    pub(in crate::c_emit) fn if_then(condition: Expr, then: Block) -> Self {
        Self::If {
            condition,
            then,
            otherwise: None,
        }
    }

    pub(in crate::c_emit) fn if_else(condition: Expr, then: Block, otherwise: Block) -> Self {
        Self::If {
            condition,
            then,
            otherwise: Some(otherwise),
        }
    }

    pub(in crate::c_emit) fn switch(value: Expr, cases: Vec<SwitchCase>) -> Self {
        Self::Switch { value, cases }
    }

    pub(in crate::c_emit) fn for_loop(
        initializer: ForInitializer,
        condition: Expr,
        update: Expr,
        body: Block,
    ) -> Self {
        Self::For {
            initializer,
            condition,
            update,
            body,
        }
    }

    pub(in crate::c_emit) fn render(&self, output: &mut String, depth: usize) {
        match self {
            Self::VariableDeclaration {
                declaration,
                initializer,
            } => {
                write_indent(output, depth);
                output.push_str(&declaration.render());
                if let Some(initializer) = initializer {
                    output.push_str(" = ");
                    output.push_str(&initializer.to_string());
                }
                output.push_str(";\n");
            }
            Self::Expression(expression) => {
                write_indent(output, depth);
                output.push_str(&expression.to_string());
                output.push_str(";\n");
            }
            Self::Assignment { target, value } => {
                write_indent(output, depth);
                output.push_str(&target.to_string());
                output.push_str(" = ");
                output.push_str(&value.to_string());
                output.push_str(";\n");
            }
            Self::Return(value) => {
                write_indent(output, depth);
                output.push_str("return ");
                output.push_str(&value.to_string());
                output.push_str(";\n");
            }
            Self::Goto(label) => {
                write_indent(output, depth);
                output.push_str("goto ");
                output.push_str(label);
                output.push_str(";\n");
            }
            Self::Break => {
                write_indent(output, depth);
                output.push_str("break;\n");
            }
            Self::Label { name, body } => {
                write_indent(output, depth);
                output.push_str(name);
                output.push_str(":\n");
                write_indent(output, depth);
                body.render_braced(output, depth);
            }
            Self::If {
                condition,
                then,
                otherwise,
            } => {
                write_indent(output, depth);
                output.push_str("if (");
                output.push_str(&condition.to_string());
                output.push_str(") ");
                then.render_braced(output, depth);
                if let Some(otherwise) = otherwise {
                    output.pop();
                    output.push_str(" else ");
                    otherwise.render_braced(output, depth);
                }
            }
            Self::For {
                initializer,
                condition,
                update,
                body,
            } => {
                write_indent(output, depth);
                output.push_str("for (");
                initializer.render(output);
                output.push_str("; ");
                output.push_str(&condition.to_string());
                output.push_str("; ");
                output.push_str(&update.to_string());
                output.push_str(") ");
                body.render_braced(output, depth);
            }
            Self::Switch { value, cases } => {
                write_indent(output, depth);
                output.push_str("switch (");
                output.push_str(&value.to_string());
                output.push_str(") {\n");
                for case in cases {
                    case.render(output, depth + 1);
                }
                write_indent(output, depth);
                output.push_str("}\n");
            }
        }
    }
}

impl ForInitializer {
    pub(in crate::c_emit) fn variable(
        ty: impl Into<super::TypeName>,
        name: impl Into<String>,
        initializer: Expr,
    ) -> Self {
        Self {
            declaration: VariableDeclaration::new(ty, name),
            initializer,
        }
    }

    fn render(&self, output: &mut String) {
        output.push_str(&self.declaration.render());
        output.push_str(" = ");
        output.push_str(&self.initializer.to_string());
    }
}

impl Block {
    pub(in crate::c_emit) fn new(statements: impl IntoIterator<Item = Statement>) -> Self {
        Self {
            statements: statements.into_iter().collect(),
        }
    }

    pub(in crate::c_emit) fn push(&mut self, statement: Statement) {
        self.statements.push(statement);
    }

    fn render_braced(&self, output: &mut String, depth: usize) {
        output.push_str("{\n");
        for statement in &self.statements {
            statement.render(output, depth + 1);
        }
        write_indent(output, depth);
        output.push_str("}\n");
    }
}

impl SwitchCase {
    pub(in crate::c_emit) fn case(label: Expr, body: Block) -> Self {
        Self {
            label: Some(label),
            body,
        }
    }

    pub(in crate::c_emit) fn default(body: Block) -> Self {
        Self { label: None, body }
    }

    fn render(&self, output: &mut String, depth: usize) {
        write_indent(output, depth);
        match &self.label {
            Some(label) => {
                output.push_str("case ");
                output.push_str(&label.to_string());
            }
            None => output.push_str("default"),
        }
        output.push_str(": {\n");
        for statement in &self.body.statements {
            statement.render(output, depth + 1);
        }
        write_indent(output, depth);
        output.push_str("}\n");
    }
}

impl FunctionDefinition {
    pub(in crate::c_emit) fn from_signature(signature: FunctionSignature, body: Block) -> Self {
        Self {
            header: FunctionHeader::Signature(signature),
            body,
        }
    }

    pub(in crate::c_emit) fn from_macro(invocation: MacroInvocation, body: Block) -> Self {
        Self {
            header: FunctionHeader::MacroInvocation(invocation),
            body,
        }
    }

    pub(in crate::c_emit) fn render(&self) -> String {
        let mut output = match &self.header {
            FunctionHeader::Signature(signature) => signature.render(),
            FunctionHeader::MacroInvocation(invocation) => invocation.render(),
        };
        output.push(' ');
        self.body.render_braced(&mut output, 0);
        output
    }
}

fn write_indent(output: &mut String, depth: usize) {
    for _ in 0..depth {
        output.push_str("    ");
    }
}

#[cfg(test)]
mod tests {
    use super::{Block, FunctionDefinition, Statement};
    use crate::c_emit::syntax::{Expr, FunctionSignature};

    #[test]
    fn renders_function_statements_and_blocks() {
        let definition = FunctionDefinition::from_signature(
            FunctionSignature::new("int", "choose", []),
            Block::new([
                Statement::variable("int", "result", Some(Expr::number("0"))),
                Statement::if_then(
                    Expr::identifier("ready"),
                    Block::new([Statement::assignment(
                        Expr::identifier("result"),
                        Expr::number("42"),
                    )]),
                ),
                Statement::return_value(Expr::identifier("result")),
            ]),
        );

        assert_eq!(
            definition.render(),
            "int choose(void) {\n    int result = 0;\n    if (ready) {\n        result = 42;\n    }\n    return result;\n}\n"
        );
    }
}
