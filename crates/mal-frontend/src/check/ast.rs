use crate::resolve::ast::{
    ExternalOperationId, LambdaId, TypeBinding, TypeId, ValueBinding, ValueId, ValueReference,
};
use mal_syntax::ast::{BinaryOperator, Node, UnaryOperator};
use mal_syntax::source::Span;
use std::{collections::HashSet, sync::Arc};

#[derive(Clone, Debug)]
pub enum Type {
    Unit,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Symbol,
    Address,
    ByteSize,
    USize,
    Parameter {
        id: TypeId,
        name: String,
    },
    Buffer(Arc<Type>),
    External {
        id: TypeId,
        name: String,
    },
    Product(Arc<[Type]>),
    Sum(Arc<[Type]>),
    Function {
        parameter: Arc<Type>,
        result: Arc<Type>,
    },
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        let mut pending = vec![(self, other)];
        let mut compared = HashSet::new();

        while let Some((left, right)) = pending.pop() {
            if std::ptr::eq(left, right) {
                continue;
            }
            if let (Some(left), Some(right)) = (shared_id(left), shared_id(right))
                && !compared.insert((left, right))
            {
                continue;
            }

            match (left, right) {
                (Self::Unit, Self::Unit)
                | (Self::Int8, Self::Int8)
                | (Self::Int16, Self::Int16)
                | (Self::Int32, Self::Int32)
                | (Self::Int64, Self::Int64)
                | (Self::UInt8, Self::UInt8)
                | (Self::UInt16, Self::UInt16)
                | (Self::UInt32, Self::UInt32)
                | (Self::UInt64, Self::UInt64)
                | (Self::Float32, Self::Float32)
                | (Self::Float64, Self::Float64)
                | (Self::Symbol, Self::Symbol)
                | (Self::Address, Self::Address)
                | (Self::ByteSize, Self::ByteSize)
                | (Self::USize, Self::USize) => {}
                (
                    Self::Parameter {
                        id: left_id,
                        name: left_name,
                    },
                    Self::Parameter {
                        id: right_id,
                        name: right_name,
                    },
                ) if left_id == right_id && left_name == right_name => {}
                (Self::Buffer(left), Self::Buffer(right)) => pending.push((left, right)),
                (
                    Self::External {
                        id: left_id,
                        name: left_name,
                    },
                    Self::External {
                        id: right_id,
                        name: right_name,
                    },
                ) if left_id == right_id && left_name == right_name => {}
                (Self::Product(left), Self::Product(right))
                | (Self::Sum(left), Self::Sum(right))
                    if left.len() == right.len() =>
                {
                    if !Arc::ptr_eq(left, right) {
                        pending.extend(left.iter().zip(right.iter()));
                    }
                }
                (
                    Self::Function {
                        parameter: left_parameter,
                        result: left_result,
                    },
                    Self::Function {
                        parameter: right_parameter,
                        result: right_result,
                    },
                ) => {
                    if !Arc::ptr_eq(left_parameter, right_parameter) {
                        pending.push((left_parameter, right_parameter));
                    }
                    if !Arc::ptr_eq(left_result, right_result) {
                        pending.push((left_result, right_result));
                    }
                }
                _ => return false,
            }
        }

        true
    }
}

impl Eq for Type {}

impl Type {
    pub fn data_subtypes(&self) -> DataSubtypes<'_> {
        DataSubtypes {
            pending: vec![self],
            visited: HashSet::new(),
        }
    }

    pub fn shared_id(&self) -> Option<SharedTypeId> {
        shared_id(self)
    }
}

pub struct DataSubtypes<'a> {
    pending: Vec<&'a Type>,
    visited: HashSet<SharedTypeId>,
}

impl<'a> Iterator for DataSubtypes<'a> {
    type Item = &'a Type;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(ty) = self.pending.pop() {
            if let Some(identity) = shared_id(ty)
                && !self.visited.insert(identity)
            {
                continue;
            }
            if let Type::Product(elements) | Type::Sum(elements) = ty {
                self.pending.extend(elements.iter().rev());
            }
            return Some(ty);
        }
        None
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum SharedTypeId {
    Product(*const Type),
    Sum(*const Type),
    Function(*const Type, *const Type),
}

fn shared_id(ty: &Type) -> Option<SharedTypeId> {
    match ty {
        Type::Product(elements) => Some(SharedTypeId::Product(elements.as_ptr())),
        Type::Sum(elements) => Some(SharedTypeId::Sum(elements.as_ptr())),
        Type::Function { parameter, result } => Some(SharedTypeId::Function(
            std::ptr::from_ref(parameter.as_ref()),
            std::ptr::from_ref(result.as_ref()),
        )),
        _ => None,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub items: Vec<Node<TopItem>>,
    pub span: Span,
    pub entry: Option<EntryPoint>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntryPoint {
    pub binding: ValueId,
    pub parameter: EntryParameter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryParameter {
    Unit,
    ProcessArguments,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MonomorphicProgram(Program);

impl MonomorphicProgram {
    pub fn new(program: Program) -> Self {
        Self(program)
    }

    pub fn program(&self) -> &Program {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TopItem {
    TypeAlias {
        binding: TypeBinding,
        ty: Type,
        element_aliases: Vec<Option<String>>,
        host_memory_access: bool,
    },
    ExternalType {
        binding: TypeBinding,
    },
    ExternalOperation {
        id: ExternalOperationId,
        binding: ValueBinding,
        lambda_id: LambdaId,
        parameter: Type,
        parameter_alias: Option<String>,
        parameter_aliases: Vec<Option<String>>,
        result: Type,
        result_alias: Option<String>,
    },
    GenericBinding(Box<GenericBinding>),
    Binding(Box<Binding>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericBinding {
    pub binding: ValueBinding,
    pub parameters: Vec<TypeBinding>,
    pub ty: Type,
    pub value: Expression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    pub pattern: Pattern,
    pub annotation: Option<Type>,
    pub value: Expression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pattern {
    Binding {
        binding: ValueBinding,
        ty: Type,
    },
    Wildcard {
        ty: Type,
        span: Span,
    },
    Product {
        elements: Vec<Pattern>,
        ty: Type,
        span: Span,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Completion {
    Value(Expression),
    Abrupt(AbruptExpression),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbruptExpression {
    pub preceding: Vec<Expression>,
    pub kind: AbruptExpressionKind,
    pub span: Span,
}

impl AbruptExpression {
    pub fn preceded_by(mut self, mut values: Vec<Expression>) -> Self {
        values.append(&mut self.preceding);
        self.preceding = values;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbruptExpressionKind {
    ResultTransfer {
        target: ValueId,
        value: Box<Expression>,
    },
    EmptyElimination {
        scrutinee: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        then_branch: ExpressionBlock,
        else_branch: ExpressionBlock,
    },
    Block(ExpressionBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpressionKind {
    Reference(ValueReference),
    GenericReference {
        reference: ValueReference,
        arguments: Vec<Type>,
    },
    Integer(i128),
    Float(u64),
    Symbol(Vec<u8>),
    Unit,
    Product(Vec<Expression>),
    Parenthesized(Box<Expression>),
    Block(ExpressionBlock),
    ResultBlock {
        target: ValueId,
        result_binders: Vec<ResultBinder>,
        body: ExpressionBlock,
    },
    Lambda(Lambda),
    Call {
        callee: Box<Expression>,
        argument: Box<Expression>,
    },
    SymbolLength {
        value: Box<Expression>,
    },
    SymbolAt {
        argument: Box<Expression>,
    },
    Memory {
        primitive: MemoryPrimitive,
        operands: Vec<Expression>,
    },
    NumericConversion {
        value: Box<Expression>,
    },
    SumElimination {
        scrutinee: Box<Expression>,
        continuations: Vec<Expression>,
    },
    SumInjection {
        index: usize,
        value: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        then_branch: ExpressionBlock,
        else_branch: ExpressionBlock,
    },
    Unary {
        operator: Node<UnaryOperator>,
        operand: Box<Expression>,
    },
    Binary {
        operator: Node<BinaryOperator>,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MemoryPrimitive {
    BufferMake,
    BufferFromAddress,
    BufferIntoAddress,
    ViewLength,
    BufferToSymbol,
    SymbolToBuffer,
    BufferNew,
    BufferGet,
    BufferPut,
    BufferFill,
    BufferCopy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lambda {
    pub id: LambdaId,
    pub self_binding: Option<ValueId>,
    pub captures: Vec<Capture>,
    pub parameter: Option<Box<Pattern>>,
    pub parameter_type: Type,
    pub result_type: Type,
    pub body: LambdaBody,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResultBinder {
    pub binding: ValueBinding,
    pub parameter_type: Type,
    pub variant: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capture {
    pub source: ValueReference,
    pub binding: ValueBinding,
    pub ty: Type,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LambdaBody {
    pub items: Vec<BodyItem>,
    pub result: Box<Completion>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpressionBlock {
    pub items: Vec<BodyItem>,
    pub result: Box<Completion>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BodyItem {
    Binding(Binding),
    Expression(Expression),
}
