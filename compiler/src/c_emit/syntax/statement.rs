use super::{Directive, Expr, FunctionSignature, Identifier, MacroInvocation, VariableDeclaration};

mod render;

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
    Goto(Identifier),
    Break,
    Label {
        name: Identifier,
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
    Directive(Directive),
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

    pub(in crate::c_emit) fn call(
        name: impl Into<Identifier>,
        arguments: impl IntoIterator<Item = Expr>,
    ) -> Self {
        Self::expression(Expr::named_call(name, arguments))
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
        name: impl Into<Identifier>,
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

    pub(in crate::c_emit) fn goto(label: impl Into<Identifier>) -> Self {
        Self::Goto(label.into())
    }

    pub(in crate::c_emit) fn label(name: impl Into<Identifier>, body: Block) -> Self {
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

    pub(in crate::c_emit) fn directive(directive: Directive) -> Self {
        Self::Directive(directive)
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
}

impl ForInitializer {
    pub(in crate::c_emit) fn variable(
        ty: impl Into<super::TypeName>,
        name: impl Into<Identifier>,
        initializer: Expr,
    ) -> Self {
        Self {
            declaration: VariableDeclaration::new(ty, name),
            initializer,
        }
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
}

#[cfg(test)]
#[path = "statement_tests.rs"]
mod tests;
