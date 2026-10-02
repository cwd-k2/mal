//! Unary, binary, comparison, and cast operators with their mnemonics and valid combinations.

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

impl UnaryOperator {
    pub(in crate::backend::llvm::syntax) fn mnemonic(self) -> &'static str {
        match self {
            Self::FNeg => "fneg",
        }
    }
}

impl BinaryOperator {
    pub(super) fn mnemonic(self) -> &'static str {
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
    pub(super) fn mnemonic(self) -> &'static str {
        match self {
            Self::Integer => "icmp",
            Self::Floating => "fcmp",
        }
    }
}

impl ComparisonPredicate {
    pub(super) fn mnemonic(self) -> &'static str {
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
    pub(in crate::backend::llvm::syntax) fn mnemonic(self) -> &'static str {
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

pub(super) fn comparison_is_valid(kind: ComparisonKind, predicate: ComparisonPredicate) -> bool {
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
