#[derive(Clone, Copy)]
enum PointerAccess {
    ReadOnly,
    ReadWrite,
}

struct PointerParameter {
    name: &'static str,
    access: PointerAccess,
}

pub(crate) struct Function {
    name: String,
    parameters: [PointerParameter; 3],
}

impl Function {
    pub(crate) fn program_entry() -> Self {
        Self {
            name: "mal_program_entry".into(),
            parameters: [
                PointerParameter {
                    name: "context",
                    access: PointerAccess::ReadWrite,
                },
                PointerParameter {
                    name: "argument",
                    access: PointerAccess::ReadOnly,
                },
                PointerParameter {
                    name: "result",
                    access: PointerAccess::ReadWrite,
                },
            ],
        }
    }

    pub(crate) fn external_bridge(id: mal_frontend::resolve::ast::ExternalOperationId) -> Self {
        let mut function = Self::program_entry();
        function.name = format!("mal_bridge_external_{}", id.0);
        function
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(in crate::backend) fn c_signature(&self) -> crate::backend::c::syntax::FunctionSignature {
        use crate::backend::c::syntax::{c_parameter, c_signature, c_type};

        let parameters = self
            .parameters
            .iter()
            .map(|parameter| {
                let ty = match parameter.access {
                    PointerAccess::ReadOnly => c_type!(ptr(const(named("void")))),
                    PointerAccess::ReadWrite => c_type!(ptr(named("void"))),
                };
                c_parameter!({{ format!("mal_{}", parameter.name) }} : {{ ty }})
            })
            .collect::<Vec<_>>();
        c_signature!(fn {{ self.name.clone() }}(
            ...{{ parameters }}
        ) -> named("void"))
    }

    fn llvm_parameters(&self) -> Vec<crate::backend::llvm::syntax::Parameter> {
        use crate::backend::llvm::syntax::llvm_parameter;

        self.parameters
            .iter()
            .map(|parameter| llvm_parameter!({{ format!("%mal_{}", parameter.name) }} : ptr))
            .collect()
    }

    fn llvm_unnamed_parameters(&self) -> Vec<crate::backend::llvm::syntax::Parameter> {
        use crate::backend::llvm::syntax::llvm_parameter;

        self.parameters
            .iter()
            .map(|_| llvm_parameter!(_ : ptr))
            .collect()
    }

    pub(in crate::backend) fn llvm_definition_signature(
        &self,
    ) -> crate::backend::llvm::syntax::FunctionSignature {
        use crate::backend::llvm::syntax::llvm_signature;

        let parameters = self.llvm_parameters();
        llvm_signature!(fn {{ self.name.clone() }}(
            ...{{ parameters }}
        ) -> void)
    }

    pub(in crate::backend) fn llvm_declaration(
        &self,
    ) -> crate::backend::llvm::syntax::FunctionDeclaration {
        use crate::backend::llvm::syntax::llvm_declaration;

        let parameters = self.llvm_unnamed_parameters();
        llvm_declaration!(fn {{ self.name.clone() }}(
            ...{{ parameters }}
        ) -> void;)
    }
}
