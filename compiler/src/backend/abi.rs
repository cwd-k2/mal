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
    name: &'static str,
    parameters: [PointerParameter; 3],
}

impl Function {
    pub(crate) fn program_entry() -> Self {
        Self {
            name: "mal_program_entry",
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

    pub(crate) fn name(&self) -> &str {
        self.name
    }

    pub(crate) fn c_declaration(&self) -> String {
        let parameters = self
            .parameters
            .iter()
            .map(|parameter| {
                let qualifier = match parameter.access {
                    PointerAccess::ReadOnly => "const ",
                    PointerAccess::ReadWrite => "",
                };
                format!("{qualifier}void *mal_{}", parameter.name)
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("void {}({parameters});", self.name)
    }

    pub(crate) fn llvm_signature(&self) -> String {
        let parameters = self
            .parameters
            .iter()
            .map(|parameter| format!("ptr %mal_{}", parameter.name))
            .collect::<Vec<_>>()
            .join(", ");
        format!("void @{}({parameters})", self.name)
    }
}
