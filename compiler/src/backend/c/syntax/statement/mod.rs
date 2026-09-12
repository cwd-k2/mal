use super::{Expr, FunctionSignature, Identifier, MacroInvocation, VariableDeclaration};

mod render;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) enum Statement {
    VariableDeclaration {
        declaration: VariableDeclaration,
        initializer: Option<Expr>,
    },
    Expression(Expr),
    Return(Expr),
    ReturnVoid,
    If {
        condition: Expr,
        then: Block,
    },
    For {
        initializer: VariableDeclaration,
        initial_value: Expr,
        condition: Expr,
        step: Expr,
        body: Block,
    },
    Switch {
        value: Expr,
        cases: Vec<SwitchCase>,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::backend) struct Block {
    statements: Vec<Statement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) struct FunctionDefinition {
    header: FunctionHeader,
    body: Block,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FunctionHeader {
    Signature(FunctionSignature),
    MacroInvocation(MacroInvocation),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend) struct SwitchCase {
    label: Option<Expr>,
    body: Block,
}

impl Statement {
    pub(in crate::backend) fn expression(expression: Expr) -> Self {
        Self::Expression(expression)
    }

    pub(in crate::backend) fn call(
        name: impl Into<Identifier>,
        arguments: impl IntoIterator<Item = Expr>,
    ) -> Self {
        Self::expression(Expr::named_call(name, arguments))
    }

    pub(in crate::backend) fn variable_declaration(
        declaration: VariableDeclaration,
        initializer: Option<Expr>,
    ) -> Self {
        Self::VariableDeclaration {
            declaration,
            initializer,
        }
    }

    pub(in crate::backend) fn variable(
        ty: impl Into<super::TypeName>,
        name: impl Into<Identifier>,
        initializer: Option<Expr>,
    ) -> Self {
        Self::variable_declaration(VariableDeclaration::new(ty, name), initializer)
    }

    pub(in crate::backend) fn return_value(value: Expr) -> Self {
        Self::Return(value)
    }

    pub(in crate::backend) fn return_void() -> Self {
        Self::ReturnVoid
    }

    pub(in crate::backend) fn if_then(condition: Expr, then: Block) -> Self {
        Self::If { condition, then }
    }

    pub(in crate::backend) fn switch(value: Expr, cases: Vec<SwitchCase>) -> Self {
        Self::Switch { value, cases }
    }

    pub(in crate::backend) fn for_loop(
        initializer: VariableDeclaration,
        initial_value: Expr,
        condition: Expr,
        step: Expr,
        body: Block,
    ) -> Self {
        Self::For {
            initializer,
            initial_value,
            condition,
            step,
            body,
        }
    }
}

impl Block {
    pub(in crate::backend) fn new(statements: impl IntoIterator<Item = Statement>) -> Self {
        Self {
            statements: statements.into_iter().collect(),
        }
    }

    pub(in crate::backend) fn push(&mut self, statement: Statement) {
        self.statements.push(statement);
    }
}

impl SwitchCase {
    pub(in crate::backend) fn case(label: Expr, body: Block) -> Self {
        Self {
            label: Some(label),
            body,
        }
    }

    pub(in crate::backend) fn default(body: Block) -> Self {
        Self { label: None, body }
    }
}

impl FunctionDefinition {
    pub(in crate::backend) fn from_signature(signature: FunctionSignature, body: Block) -> Self {
        Self {
            header: FunctionHeader::Signature(signature),
            body,
        }
    }

    pub(in crate::backend) fn from_macro(invocation: MacroInvocation, body: Block) -> Self {
        Self {
            header: FunctionHeader::MacroInvocation(invocation),
            body,
        }
    }
}

#[cfg(test)]
mod tests;
