mod plan;

use super::body;
use crate::backend::abi::Function as AbiFunction;
use crate::backend::c::syntax::{
    Block, Expr, FunctionDefinition, FunctionSignature, FunctionSpecifier, Initializer, Parameter,
    Statement, SwitchCase, TranslationUnit, TypeName,
};
use crate::check::ast::Type;

pub(super) struct Bridge {
    pub(super) llvm_declaration: String,
    pub(super) c_definitions: TranslationUnit,
}

pub(super) fn generate(
    external: &crate::core::ast::ExternalOperation,
    pointer_size: usize,
    raw_types: &crate::backend::c::RawHostTypes,
) -> Option<Bridge> {
    let types = body::types::Types::new(pointer_size)?;
    let parameter = plan::Value::new(&external.parameter, types)?;
    let result = plan::Value::new(&external.result, types)?;
    let mut marshalling = Marshalling::new(external.id.0, raw_types);
    let bridge = AbiFunction::external_bridge(external.id);
    let llvm_declaration = format!("declare {}", bridge.llvm_signature());
    let (mut statements, arguments) = match &parameter.kind {
        plan::Kind::Unit => (
            vec![Statement::expression(Expr::cast(
                "void",
                identifier("mal_argument"),
            ))],
            Vec::new(),
        ),
        plan::Kind::Product(fields) => {
            let arguments = fields
                .iter()
                .map(|field| {
                    marshalling.read(
                        &field.value,
                        identifier("mal_argument"),
                        field.offset,
                        context_cast(),
                    )
                })
                .collect::<Option<Vec<_>>>()?;
            (Vec::new(), arguments)
        }
        _ => (
            Vec::new(),
            vec![marshalling.read(&parameter, identifier("mal_argument"), 0, context_cast())?],
        ),
    };
    let mut call_arguments = vec![context_cast()];
    call_arguments.extend(arguments);
    let call = Expr::named_call(format!("mal_ext_{}", external.name), call_arguments);
    match &result.kind {
        plan::Kind::Unit => {
            statements.push(Statement::expression(call));
            statements.push(store(
                "uint8_t",
                identifier("mal_result"),
                Expr::named_call("UINT8_C", [number(0)]),
            ));
        }
        plan::Kind::Symbol => {
            statements.push(variable("MalType_Symbol", "result", Some(call)));
            statements.push(store(
                TypeName::named("void").pointer(),
                identifier("mal_result"),
                identifier("result").field("ownership"),
            ));
        }
        plan::Kind::Product(_) | plan::Kind::Sum { .. } | plan::Kind::External => {
            statements.push(variable(
                raw_types.c_type(&external.result),
                "result",
                Some(call),
            ));
            statements.extend(marshalling.write(
                &result,
                identifier("result"),
                0,
                context_cast(),
            )?);
        }
        plan::Kind::Scalar => {
            statements.push(store(
                c_scalar_type(result.ty)?,
                identifier("mal_result"),
                call,
            ));
        }
    }
    marshalling.helpers.blank_line();
    marshalling.helpers.push(FunctionDefinition::from_signature(
        bridge.c_signature(),
        Block::new(statements),
    ));
    Some(Bridge {
        llvm_declaration,
        c_definitions: marshalling.helpers,
    })
}

struct Marshalling<'a> {
    external: u32,
    raw_types: &'a crate::backend::c::RawHostTypes,
    next_helper: usize,
    helpers: TranslationUnit,
}

impl<'a> Marshalling<'a> {
    fn new(external: u32, raw_types: &'a crate::backend::c::RawHostTypes) -> Self {
        Self {
            external,
            raw_types,
            next_helper: 0,
            helpers: TranslationUnit::default(),
        }
    }

    fn helper_name(&mut self, direction: &str) -> String {
        let helper = self.next_helper;
        self.next_helper += 1;
        format!(
            "mal_bridge_external_{}_{}_sum_{}",
            self.external, direction, helper
        )
    }

    fn read(
        &mut self,
        value: &plan::Value<'_>,
        base: Expr,
        offset: usize,
        context: Expr,
    ) -> Option<Expr> {
        let pointer = bridge_pointer(base.clone(), offset, true);
        match &value.kind {
            plan::Kind::Unit => Some(Expr::compound_literal(
                "MalType_Unit",
                [Initializer::designated(
                    "unused",
                    Expr::named_call("UINT8_C", [number(0)]),
                )],
            )),
            plan::Kind::Symbol => {
                let ownership = load(TypeName::named("void").const_pointer().pointer(), pointer);
                Some(Expr::named_call(
                    "mal_symbol_materialize",
                    [
                        context,
                        Expr::compound_literal(
                            "MalType_Symbol",
                            [
                                Initializer::designated("data", identifier("NULL")),
                                Initializer::designated(
                                    "length",
                                    Expr::named_call(
                                        "mal_runtime_symbol_length",
                                        [ownership.clone()],
                                    ),
                                ),
                                Initializer::designated("ownership", ownership),
                            ],
                        ),
                    ],
                ))
            }
            plan::Kind::External => Some(Expr::compound_literal(
                self.raw_types.c_type(value.ty),
                [Initializer::designated(
                    "bits",
                    load(TypeName::const_named("uintptr_t").pointer(), pointer),
                )],
            )),
            plan::Kind::Product(fields) => {
                let initializers = fields
                    .iter()
                    .enumerate()
                    .map(|(index, field)| {
                        Some(Initializer::designated(
                            format!("field_{index}"),
                            self.read(
                                &field.value,
                                base.clone(),
                                offset.checked_add(field.offset)?,
                                context.clone(),
                            )?,
                        ))
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(Expr::compound_literal(
                    self.raw_types.c_type(value.ty),
                    initializers,
                ))
            }
            plan::Kind::Sum {
                tag_offset,
                variants,
            } => self.read_sum(value.ty, *tag_offset, variants, pointer, context),
            plan::Kind::Scalar => Some(load(
                TypeName::const_named(c_scalar_type(value.ty)?).pointer(),
                pointer,
            )),
        }
    }

    fn read_sum(
        &mut self,
        ty: &Type,
        tag_offset: usize,
        variants: &[plan::Field<'_>],
        pointer: Expr,
        context: Expr,
    ) -> Option<Expr> {
        let helper = self.helper_name("read");
        let c_type = self.raw_types.c_type(ty);
        let cases = variants
            .iter()
            .enumerate()
            .map(|(index, field)| {
                Some(SwitchCase::case(
                    Expr::named_call("UINT32_C", [number(index)]),
                    Block::new([Statement::return_value(Expr::compound_literal(
                        c_type.clone(),
                        [
                            Initializer::designated(
                                "tag",
                                Expr::named_call("UINT32_C", [number(index)]),
                            ),
                            Initializer::designated_path(
                                ["payload", &format!("variant_{index}")],
                                self.read(
                                    &field.value,
                                    identifier("value"),
                                    field.offset,
                                    identifier("context"),
                                )?,
                            ),
                        ],
                    ))]),
                ))
            })
            .collect::<Option<Vec<_>>>()?;
        let mut cases = cases;
        cases.push(SwitchCase::default(Block::new([trap(
            identifier("context"),
            "invalid sum tag at LLVM bridge",
        )])));
        self.helpers.push(FunctionDefinition::from_signature(
            FunctionSignature::new(
                c_type,
                helper.clone(),
                [
                    Parameter::named(TypeName::named("MalContext").pointer(), "context"),
                    Parameter::named(TypeName::const_named("uint8_t").pointer(), "value"),
                ],
            )
            .with_specifiers([FunctionSpecifier::Static]),
            Block::new([
                variable(
                    "uint32_t",
                    "tag",
                    Some(load(
                        TypeName::const_named("uint32_t").pointer(),
                        Expr::add(identifier("value"), number(tag_offset)),
                    )),
                ),
                Statement::switch(identifier("tag"), cases),
            ]),
        ));
        self.helpers.blank_line();
        Some(Expr::named_call(helper, [context, pointer]))
    }

    fn write(
        &mut self,
        plan: &plan::Value<'_>,
        value: Expr,
        offset: usize,
        context: Expr,
    ) -> Option<Vec<Statement>> {
        self.write_at(plan, identifier("mal_result"), value, offset, context)
    }

    fn write_at(
        &mut self,
        plan: &plan::Value<'_>,
        base: Expr,
        value: Expr,
        offset: usize,
        context: Expr,
    ) -> Option<Vec<Statement>> {
        let pointer = bridge_pointer(base.clone(), offset, false);
        match &plan.kind {
            plan::Kind::Unit => Some(vec![store(
                "uint8_t",
                pointer,
                Expr::named_call("UINT8_C", [number(0)]),
            )]),
            plan::Kind::Symbol => Some(vec![store(
                TypeName::named("void").pointer(),
                pointer,
                value.field("ownership"),
            )]),
            plan::Kind::External => Some(vec![store("uintptr_t", pointer, value.field("bits"))]),
            plan::Kind::Product(fields) => {
                let mut statements = Vec::new();
                for (index, field) in fields.iter().enumerate() {
                    statements.extend(self.write_at(
                        &field.value,
                        base.clone(),
                        value.clone().field(format!("field_{index}")),
                        offset.checked_add(field.offset)?,
                        context.clone(),
                    )?);
                }
                Some(statements)
            }
            plan::Kind::Sum {
                tag_offset,
                variants,
            } => self
                .write_sum(plan.ty, *tag_offset, variants, value, pointer, context)
                .map(|call| vec![call]),
            plan::Kind::Scalar => Some(vec![store(c_scalar_type(plan.ty)?, pointer, value)]),
        }
    }

    fn write_sum(
        &mut self,
        ty: &Type,
        tag_offset: usize,
        variants: &[plan::Field<'_>],
        value: Expr,
        pointer: Expr,
        context: Expr,
    ) -> Option<Statement> {
        let helper = self.helper_name("write");
        let c_type = self.raw_types.c_type(ty);
        let mut cases = Vec::new();
        for (index, field) in variants.iter().enumerate() {
            let mut statements = self.write_at(
                &field.value,
                identifier("value"),
                identifier("input")
                    .field("payload")
                    .field(format!("variant_{index}")),
                field.offset,
                identifier("context"),
            )?;
            statements.push(Statement::return_void());
            cases.push(SwitchCase::case(
                Expr::named_call("UINT32_C", [number(index)]),
                Block::new(statements),
            ));
        }
        cases.push(SwitchCase::default(Block::new([trap(
            identifier("context"),
            "invalid sum tag at LLVM bridge",
        )])));
        self.helpers.push(FunctionDefinition::from_signature(
            FunctionSignature::new(
                "void",
                helper.clone(),
                [
                    Parameter::named(TypeName::named("MalContext").pointer(), "context"),
                    Parameter::named(TypeName::named("uint8_t").pointer(), "value"),
                    Parameter::named(c_type, "input"),
                ],
            )
            .with_specifiers([FunctionSpecifier::Static]),
            Block::new([
                store(
                    "uint32_t",
                    Expr::add(identifier("value"), number(tag_offset)),
                    identifier("input").field("tag"),
                ),
                Statement::switch(identifier("input").field("tag"), cases),
            ]),
        ));
        self.helpers.blank_line();
        Some(Statement::call(helper, [context, pointer, value]))
    }
}

fn bridge_pointer(base: Expr, offset: usize, read_only: bool) -> Expr {
    let ty = if read_only {
        TypeName::const_named("uint8_t").pointer()
    } else {
        TypeName::named("uint8_t").pointer()
    };
    let pointer = Expr::cast(ty, base);
    if offset == 0 {
        pointer
    } else {
        Expr::add(pointer, number(offset))
    }
}

fn context_cast() -> Expr {
    Expr::cast(
        TypeName::named("MalContext").pointer(),
        identifier("mal_context"),
    )
}

fn load(ty: TypeName, pointer: Expr) -> Expr {
    Expr::dereference(Expr::cast(ty, pointer))
}

fn store(ty: impl Into<TypeName>, pointer: Expr, value: Expr) -> Statement {
    Statement::expression(Expr::assign(
        Expr::dereference(Expr::cast(ty.into().pointer(), pointer)),
        value,
    ))
}

fn variable(ty: impl Into<TypeName>, name: &str, initializer: Option<Expr>) -> Statement {
    Statement::variable(ty, name, initializer)
}

fn identifier(name: impl Into<crate::backend::c::syntax::Identifier>) -> Expr {
    Expr::identifier(name)
}

fn number(value: impl ToString) -> Expr {
    Expr::number(value.to_string())
}

fn trap(context: Expr, message: &str) -> Statement {
    Statement::call("mal_trap", [context, Expr::string(message)])
}

pub(super) fn type_supported(ty: &Type) -> bool {
    ty.data_subtypes().all(|ty| {
        c_scalar_type(ty).is_some()
            || matches!(
                ty,
                Type::Unit | Type::Symbol | Type::External { .. } | Type::Product(_) | Type::Sum(_)
            )
    })
}

fn c_scalar_type(ty: &Type) -> Option<&'static str> {
    if body::types::is_bool(ty) {
        return Some("MalType_Bool");
    }
    match ty {
        Type::Int8 => Some("MalType_Int8"),
        Type::Int16 => Some("MalType_Int16"),
        Type::Int32 => Some("MalType_Int32"),
        Type::Int64 => Some("MalType_Int64"),
        Type::UInt8 => Some("MalType_UInt8"),
        Type::UInt16 => Some("MalType_UInt16"),
        Type::UInt32 => Some("MalType_UInt32"),
        Type::UInt64 => Some("MalType_UInt64"),
        Type::Float32 => Some("MalType_Float32"),
        Type::Float64 => Some("MalType_Float64"),
        Type::Ptr => Some("MalType_Ptr"),
        _ => None,
    }
}
