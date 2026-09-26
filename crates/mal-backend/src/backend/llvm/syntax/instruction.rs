use super::Type;
use super::function::{is_single_line, is_valid_name};

#[derive(Clone)]
pub(in crate::backend) enum Instruction {
    Alloca {
        result: String,
        ty: Type,
        alignment: usize,
    },
    Load {
        result: String,
        ty: Type,
        pointer: String,
        alignment: usize,
        metadata: Vec<MetadataAttachment>,
    },
    Store {
        ty: Type,
        value: String,
        pointer: String,
        alignment: usize,
        metadata: Vec<MetadataAttachment>,
    },
    Call {
        result: Option<String>,
        tail: bool,
        result_type: Type,
        callee: Callee,
        arguments: Vec<TypedValue>,
    },
    Unary {
        result: String,
        operator: UnaryOperator,
        operand: TypedValue,
    },
    Binary {
        result: String,
        operator: BinaryOperator,
        ty: Type,
        left: String,
        right: String,
    },
    Compare {
        result: String,
        kind: ComparisonKind,
        predicate: ComparisonPredicate,
        ty: Type,
        left: String,
        right: String,
    },
    Cast {
        result: String,
        operator: CastOperator,
        operand: TypedValue,
        target: Type,
    },
    GetElementPtr {
        result: String,
        inbounds: bool,
        element_type: Type,
        pointer: String,
        indices: Vec<TypedValue>,
    },
    ExtractValue {
        result: String,
        aggregate: TypedValue,
        indices: Vec<usize>,
    },
    InsertValue {
        result: String,
        aggregate: TypedValue,
        element: TypedValue,
        indices: Vec<usize>,
    },
    Phi {
        result: String,
        ty: Type,
        incoming: Vec<(String, String)>,
    },
}

#[derive(Clone, Copy)]
pub(in crate::backend) enum UnaryOperator {
    FNeg,
}

#[derive(Clone, Copy)]
pub(in crate::backend) enum BinaryOperator {
    Add,
    Sub,
    Mul,
    UDiv,
    SDiv,
    FDiv,
    URem,
    SRem,
    Shl,
    LShr,
    AShr,
    And,
    Or,
    Xor,
    FAdd,
    FSub,
    FMul,
}

#[derive(Clone, Copy)]
pub(in crate::backend) enum ComparisonKind {
    Integer,
    Floating,
}

#[derive(Clone, Copy)]
pub(in crate::backend) enum ComparisonPredicate {
    Eq,
    Ne,
    Ult,
    Ule,
    Ugt,
    Uge,
    Slt,
    Sle,
    Sgt,
    Sge,
    Oeq,
    Une,
    Olt,
    Ole,
    Ogt,
    Oge,
}

#[derive(Clone, Copy)]
pub(in crate::backend) enum CastOperator {
    Trunc,
    ZExt,
    SExt,
    FPTrunc,
    FPExt,
    FPToUI,
    FPToSI,
    UIToFP,
    SIToFP,
}

#[derive(Clone)]
pub(in crate::backend) enum Callee {
    Direct(String),
    Indirect(String),
}

#[derive(Clone)]
pub(in crate::backend) struct TypedValue {
    ty: Type,
    value: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::backend) enum MetadataAttachment {
    Tbaa(usize),
    AliasScope(usize),
    NoAlias(usize),
}

impl Instruction {
    pub(in crate::backend::llvm) fn alloca(
        result: impl Into<String>,
        ty: Type,
        alignment: usize,
    ) -> Option<Self> {
        let result = result.into();
        local_name(&result)
            .is_some_and(is_valid_name)
            .then_some(Self::Alloca {
                result,
                ty,
                alignment,
            })
    }

    pub(in crate::backend::llvm) fn load(
        result: impl Into<String>,
        ty: Type,
        pointer: impl Into<String>,
        alignment: usize,
        metadata: impl IntoIterator<Item = MetadataAttachment>,
    ) -> Option<Self> {
        let result = result.into();
        let pointer = pointer.into();
        (local_name(&result).is_some_and(is_valid_name) && is_value(&pointer)).then_some(
            Self::Load {
                result,
                ty,
                pointer,
                alignment,
                metadata: metadata.into_iter().collect(),
            },
        )
    }

    pub(in crate::backend::llvm) fn store(
        ty: Type,
        value: impl Into<String>,
        pointer: impl Into<String>,
        alignment: usize,
        metadata: impl IntoIterator<Item = MetadataAttachment>,
    ) -> Option<Self> {
        let value = value.into();
        let pointer = pointer.into();
        (is_value(&value) && is_value(&pointer)).then_some(Self::Store {
            ty,
            value,
            pointer,
            alignment,
            metadata: metadata.into_iter().collect(),
        })
    }

    pub(in crate::backend) fn call(
        result: Option<impl Into<String>>,
        tail: bool,
        result_type: Type,
        callee: Callee,
        arguments: impl IntoIterator<Item = TypedValue>,
    ) -> Option<Self> {
        let result = result.map(Into::into);
        (result
            .as_deref()
            .is_none_or(|result| local_name(result).is_some_and(is_valid_name))
            && !(result.is_some() && matches!(result_type, Type::Void)))
        .then_some(Self::Call {
            result,
            tail,
            result_type,
            callee,
            arguments: arguments.into_iter().collect(),
        })
    }

    pub(in crate::backend::llvm) fn unary(
        result: impl Into<String>,
        operator: UnaryOperator,
        operand: TypedValue,
    ) -> Option<Self> {
        let result = result.into();
        local_name(&result)
            .is_some_and(is_valid_name)
            .then_some(Self::Unary {
                result,
                operator,
                operand,
            })
    }

    pub(in crate::backend::llvm) fn binary(
        result: impl Into<String>,
        operator: BinaryOperator,
        ty: Type,
        left: impl Into<String>,
        right: impl Into<String>,
    ) -> Option<Self> {
        let result = result.into();
        let left = left.into();
        let right = right.into();
        (local_name(&result).is_some_and(is_valid_name) && is_value(&left) && is_value(&right))
            .then_some(Self::Binary {
                result,
                operator,
                ty,
                left,
                right,
            })
    }

    pub(in crate::backend::llvm) fn compare(
        result: impl Into<String>,
        kind: ComparisonKind,
        predicate: ComparisonPredicate,
        ty: Type,
        left: impl Into<String>,
        right: impl Into<String>,
    ) -> Option<Self> {
        let result = result.into();
        let left = left.into();
        let right = right.into();
        (local_name(&result).is_some_and(is_valid_name)
            && is_value(&left)
            && is_value(&right)
            && comparison_is_valid(kind, predicate))
        .then_some(Self::Compare {
            result,
            kind,
            predicate,
            ty,
            left,
            right,
        })
    }

    pub(in crate::backend::llvm) fn cast(
        result: impl Into<String>,
        operator: CastOperator,
        operand: TypedValue,
        target: Type,
    ) -> Option<Self> {
        let result = result.into();
        local_name(&result)
            .is_some_and(is_valid_name)
            .then_some(Self::Cast {
                result,
                operator,
                operand,
                target,
            })
    }

    pub(in crate::backend::llvm) fn get_element_ptr(
        result: impl Into<String>,
        inbounds: bool,
        element_type: Type,
        pointer: impl Into<String>,
        indices: impl IntoIterator<Item = TypedValue>,
    ) -> Option<Self> {
        let result = result.into();
        let pointer = pointer.into();
        (local_name(&result).is_some_and(is_valid_name) && is_value(&pointer)).then_some(
            Self::GetElementPtr {
                result,
                inbounds,
                element_type,
                pointer,
                indices: indices.into_iter().collect(),
            },
        )
    }

    pub(in crate::backend::llvm) fn extract_value(
        result: impl Into<String>,
        aggregate: TypedValue,
        indices: impl IntoIterator<Item = usize>,
    ) -> Option<Self> {
        let result = result.into();
        let indices = indices.into_iter().collect::<Vec<_>>();
        (local_name(&result).is_some_and(is_valid_name) && !indices.is_empty()).then_some(
            Self::ExtractValue {
                result,
                aggregate,
                indices,
            },
        )
    }

    pub(in crate::backend::llvm) fn insert_value(
        result: impl Into<String>,
        aggregate: TypedValue,
        element: TypedValue,
        indices: impl IntoIterator<Item = usize>,
    ) -> Option<Self> {
        let result = result.into();
        let indices = indices.into_iter().collect::<Vec<_>>();
        (local_name(&result).is_some_and(is_valid_name) && !indices.is_empty()).then_some(
            Self::InsertValue {
                result,
                aggregate,
                element,
                indices,
            },
        )
    }

    pub(in crate::backend::llvm) fn phi(
        result: impl Into<String>,
        ty: Type,
        incoming: impl IntoIterator<Item = (String, String)>,
    ) -> Option<Self> {
        let result = result.into();
        let incoming = incoming.into_iter().collect::<Vec<_>>();
        (local_name(&result).is_some_and(is_valid_name)
            && !incoming.is_empty()
            && incoming
                .iter()
                .all(|(value, block)| is_value(value) && is_valid_name(block)))
        .then_some(Self::Phi {
            result,
            ty,
            incoming,
        })
    }

    pub(super) fn render_into(&self, output: &mut String) {
        match self {
            Self::Alloca {
                result,
                ty,
                alignment,
            } => output.push_str(&format!("{result} = alloca {ty}, align {alignment}")),
            Self::Load {
                result,
                ty,
                pointer,
                alignment,
                metadata,
            } => {
                output.push_str(&format!(
                    "{result} = load {ty}, ptr {pointer}, align {alignment}"
                ));
                render_metadata(output, metadata);
            }
            Self::Store {
                ty,
                value,
                pointer,
                alignment,
                metadata,
            } => {
                output.push_str(&format!(
                    "store {ty} {value}, ptr {pointer}, align {alignment}"
                ));
                render_metadata(output, metadata);
            }
            Self::Call {
                result,
                tail,
                result_type,
                callee,
                arguments,
            } => {
                if let Some(result) = result {
                    output.push_str(result);
                    output.push_str(" = ");
                }
                if *tail {
                    output.push_str("tail ");
                }
                output.push_str("call ");
                output.push_str(&result_type.to_string());
                output.push(' ');
                callee.render_into(output);
                output.push('(');
                for (index, argument) in arguments.iter().enumerate() {
                    if index != 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&argument.ty.to_string());
                    output.push(' ');
                    output.push_str(&argument.value);
                }
                output.push(')');
            }
            Self::Unary {
                result,
                operator,
                operand,
            } => output.push_str(&format!(
                "{result} = {} {} {}",
                operator.mnemonic(),
                operand.ty,
                operand.value
            )),
            Self::Binary {
                result,
                operator,
                ty,
                left,
                right,
            } => output.push_str(&format!(
                "{result} = {} {ty} {left}, {right}",
                operator.mnemonic()
            )),
            Self::Compare {
                result,
                kind,
                predicate,
                ty,
                left,
                right,
            } => output.push_str(&format!(
                "{result} = {} {} {ty} {left}, {right}",
                kind.mnemonic(),
                predicate.mnemonic()
            )),
            Self::Cast {
                result,
                operator,
                operand,
                target,
            } => output.push_str(&format!(
                "{result} = {} {} {} to {target}",
                operator.mnemonic(),
                operand.ty,
                operand.value
            )),
            Self::GetElementPtr {
                result,
                inbounds,
                element_type,
                pointer,
                indices,
            } => {
                output.push_str(&format!("{result} = getelementptr"));
                if *inbounds {
                    output.push_str(" inbounds");
                }
                output.push_str(&format!(" {element_type}, ptr {pointer}"));
                for index in indices {
                    output.push_str(&format!(", {} {}", index.ty, index.value));
                }
            }
            Self::ExtractValue {
                result,
                aggregate,
                indices,
            } => output.push_str(&format!(
                "{result} = extractvalue {} {}, {}",
                aggregate.ty,
                aggregate.value,
                render_indices(indices)
            )),
            Self::InsertValue {
                result,
                aggregate,
                element,
                indices,
            } => output.push_str(&format!(
                "{result} = insertvalue {} {}, {} {}, {}",
                aggregate.ty,
                aggregate.value,
                element.ty,
                element.value,
                render_indices(indices)
            )),
            Self::Phi {
                result,
                ty,
                incoming,
            } => {
                output.push_str(&format!("{result} = phi {ty} "));
                for (index, (value, block)) in incoming.iter().enumerate() {
                    if index != 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&format!("[{value}, %{block}]"));
                }
            }
        }
    }

    pub(super) fn uses_byte_runtime(&self) -> bool {
        match self {
            Self::Alloca { .. }
            | Self::Load { .. }
            | Self::Store { .. }
            | Self::Unary { .. }
            | Self::Binary { .. }
            | Self::Compare { .. }
            | Self::Cast { .. }
            | Self::GetElementPtr { .. }
            | Self::ExtractValue { .. }
            | Self::InsertValue { .. }
            | Self::Phi { .. } => false,
            Self::Call { callee, .. } => callee.uses_byte_runtime(),
        }
    }
}

impl UnaryOperator {
    fn mnemonic(self) -> &'static str {
        match self {
            Self::FNeg => "fneg",
        }
    }
}

impl BinaryOperator {
    fn mnemonic(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::UDiv => "udiv",
            Self::SDiv => "sdiv",
            Self::FDiv => "fdiv",
            Self::URem => "urem",
            Self::SRem => "srem",
            Self::Shl => "shl",
            Self::LShr => "lshr",
            Self::AShr => "ashr",
            Self::And => "and",
            Self::Or => "or",
            Self::Xor => "xor",
            Self::FAdd => "fadd",
            Self::FSub => "fsub",
            Self::FMul => "fmul",
        }
    }
}

impl std::fmt::Display for BinaryOperator {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.mnemonic())
    }
}

impl ComparisonKind {
    fn mnemonic(self) -> &'static str {
        match self {
            Self::Integer => "icmp",
            Self::Floating => "fcmp",
        }
    }
}

impl ComparisonPredicate {
    fn mnemonic(self) -> &'static str {
        match self {
            Self::Eq => "eq",
            Self::Ne => "ne",
            Self::Ult => "ult",
            Self::Ule => "ule",
            Self::Ugt => "ugt",
            Self::Uge => "uge",
            Self::Slt => "slt",
            Self::Sle => "sle",
            Self::Sgt => "sgt",
            Self::Sge => "sge",
            Self::Oeq => "oeq",
            Self::Une => "une",
            Self::Olt => "olt",
            Self::Ole => "ole",
            Self::Ogt => "ogt",
            Self::Oge => "oge",
        }
    }
}

impl CastOperator {
    fn mnemonic(self) -> &'static str {
        match self {
            Self::Trunc => "trunc",
            Self::ZExt => "zext",
            Self::SExt => "sext",
            Self::FPTrunc => "fptrunc",
            Self::FPExt => "fpext",
            Self::FPToUI => "fptoui",
            Self::FPToSI => "fptosi",
            Self::UIToFP => "uitofp",
            Self::SIToFP => "sitofp",
        }
    }
}

fn comparison_is_valid(kind: ComparisonKind, predicate: ComparisonPredicate) -> bool {
    match kind {
        ComparisonKind::Integer => matches!(
            predicate,
            ComparisonPredicate::Eq
                | ComparisonPredicate::Ne
                | ComparisonPredicate::Ult
                | ComparisonPredicate::Ule
                | ComparisonPredicate::Ugt
                | ComparisonPredicate::Uge
                | ComparisonPredicate::Slt
                | ComparisonPredicate::Sle
                | ComparisonPredicate::Sgt
                | ComparisonPredicate::Sge
        ),
        ComparisonKind::Floating => matches!(
            predicate,
            ComparisonPredicate::Oeq
                | ComparisonPredicate::Une
                | ComparisonPredicate::Olt
                | ComparisonPredicate::Ole
                | ComparisonPredicate::Ogt
                | ComparisonPredicate::Oge
        ),
    }
}

fn render_indices(indices: &[usize]) -> String {
    indices
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

impl Callee {
    pub(in crate::backend) fn direct(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        is_valid_name(&name).then_some(Self::Direct(name))
    }

    pub(in crate::backend) fn indirect(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        is_value(&value).then_some(Self::Indirect(value))
    }

    fn render_into(&self, output: &mut String) {
        match self {
            Self::Direct(name) => output.push_str(&format!("@{name}")),
            Self::Indirect(value) => output.push_str(value),
        }
    }

    fn uses_byte_runtime(&self) -> bool {
        matches!(self, Self::Direct(name) if name.starts_with("mal_runtime_bytes_")
            || name.starts_with("mal_runtime_buffer_")
            || name.starts_with("mal_runtime_symbol_"))
    }
}

impl TypedValue {
    pub(in crate::backend) fn new(ty: Type, value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        is_value(&value).then_some(Self { ty, value })
    }
}

fn local_name(value: &str) -> Option<&str> {
    value.strip_prefix('%')
}

fn is_value(value: &str) -> bool {
    is_single_line(value)
}

fn render_metadata(output: &mut String, metadata: &[MetadataAttachment]) {
    for attachment in metadata {
        let (name, node) = match attachment {
            MetadataAttachment::Tbaa(node) => ("tbaa", node),
            MetadataAttachment::AliasScope(node) => ("alias.scope", node),
            MetadataAttachment::NoAlias(node) => ("noalias", node),
        };
        output.push_str(&format!(", !{name} !{node}"));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BinaryOperator, CastOperator, ComparisonKind, ComparisonPredicate, Instruction,
        MetadataAttachment, Type, TypedValue,
    };

    #[test]
    fn renders_alloca_from_typed_fields() {
        let instruction = Instruction::alloca(
            "%storage",
            Type::structure([Type::integer(32_u16), Type::Pointer]),
            8,
        )
        .unwrap();
        let mut output = String::new();
        instruction.render_into(&mut output);

        assert_eq!(output, "%storage = alloca { i32, ptr }, align 8");
    }

    #[test]
    fn renders_memory_operations_and_metadata_from_fields() {
        let mut output = String::new();
        Instruction::load(
            "%value",
            Type::integer(32_u16),
            "%address",
            4,
            [MetadataAttachment::Tbaa(3), MetadataAttachment::NoAlias(6)],
        )
        .unwrap()
        .render_into(&mut output);
        assert_eq!(
            output,
            "%value = load i32, ptr %address, align 4, !tbaa !3, !noalias !6"
        );

        output.clear();
        Instruction::store(Type::Pointer, "null", "%address", 8, std::iter::empty())
            .unwrap()
            .render_into(&mut output);
        assert_eq!(output, "store ptr null, ptr %address, align 8");
    }

    #[test]
    fn renders_value_and_pointer_operations_from_typed_fields() {
        let mut rendered = Vec::new();
        for instruction in [
            Instruction::binary(
                "%sum",
                BinaryOperator::Add,
                Type::integer(32_u16),
                "%a",
                "1",
            )
            .unwrap(),
            Instruction::compare(
                "%small",
                ComparisonKind::Integer,
                ComparisonPredicate::Ult,
                Type::integer(32_u16),
                "%sum",
                "8",
            )
            .unwrap(),
            Instruction::cast(
                "%wide",
                CastOperator::ZExt,
                TypedValue::new(Type::integer(1_u16), "%small").unwrap(),
                Type::integer(8_u16),
            )
            .unwrap(),
            Instruction::get_element_ptr(
                "%field",
                true,
                Type::structure([Type::integer(32_u16), Type::Pointer]),
                "%storage",
                [
                    TypedValue::new(Type::integer(32_u16), "0").unwrap(),
                    TypedValue::new(Type::integer(32_u16), "1").unwrap(),
                ],
            )
            .unwrap(),
        ] {
            let mut output = String::new();
            instruction.render_into(&mut output);
            rendered.push(output);
        }

        assert_eq!(
            rendered,
            [
                "%sum = add i32 %a, 1",
                "%small = icmp ult i32 %sum, 8",
                "%wide = zext i1 %small to i8",
                "%field = getelementptr inbounds { i32, ptr }, ptr %storage, i32 0, i32 1",
            ]
        );
    }

    #[test]
    fn instruction_macro_builds_typed_calls() {
        let instruction = super::super::llvm_instruction!(
            call Some("%result".to_owned()), false, Type::integer(8_u16), direct "observe",
            [(Type::Pointer, "%value".to_owned())]
        )
        .unwrap();
        let mut output = String::new();
        instruction.render_into(&mut output);

        assert_eq!(output, "%result = call i8 @observe(ptr %value)");
    }
}
