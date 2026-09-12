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
    let mut marshalling = Marshalling::new(external.id.0, types, raw_types);
    let bridge = AbiFunction::external_bridge(external.id);
    let llvm_declaration = format!("declare {}", bridge.llvm_signature());
    let (mut statements, arguments) = match &external.parameter {
        Type::Unit => (
            vec![Statement::expression(Expr::cast(
                "void",
                identifier("mal_argument"),
            ))],
            Vec::new(),
        ),
        Type::Product(elements) => {
            let fields = types.product_fields(&external.parameter)?;
            let arguments = elements
                .iter()
                .zip(fields)
                .map(|(element, field)| {
                    marshalling.read(
                        element,
                        identifier("mal_argument"),
                        field.offset,
                        context_cast(),
                    )
                })
                .collect::<Option<Vec<_>>>()?;
            (Vec::new(), arguments)
        }
        ty => (
            Vec::new(),
            vec![marshalling.read(ty, identifier("mal_argument"), 0, context_cast())?],
        ),
    };
    let mut call_arguments = vec![context_cast()];
    call_arguments.extend(arguments);
    let call = Expr::named_call(format!("mal_ext_{}", external.name), call_arguments);
    match &external.result {
        Type::Unit => {
            statements.push(Statement::expression(call));
            statements.push(store(
                "uint8_t",
                identifier("mal_result"),
                Expr::named_call("UINT8_C", [number(0)]),
            ));
        }
        Type::Symbol => {
            statements.push(variable("MalType_Symbol", "result", Some(call)));
            statements.push(store(
                TypeName::named("void").pointer(),
                identifier("mal_result"),
                identifier("result").field("ownership"),
            ));
        }
        Type::Product(_) | Type::Sum(_) | Type::External { .. }
            if c_scalar_type(&external.result).is_none() =>
        {
            statements.push(variable(
                raw_types.c_type(&external.result),
                "result",
                Some(call),
            ));
            statements.extend(marshalling.write(
                &external.result,
                identifier("result"),
                0,
                context_cast(),
            )?);
        }
        ty => {
            statements.push(store(c_scalar_type(ty)?, identifier("mal_result"), call));
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
    types: body::types::Types,
    raw_types: &'a crate::backend::c::RawHostTypes,
    next_helper: usize,
    helpers: TranslationUnit,
}

impl<'a> Marshalling<'a> {
    fn new(
        external: u32,
        types: body::types::Types,
        raw_types: &'a crate::backend::c::RawHostTypes,
    ) -> Self {
        Self {
            external,
            types,
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

    fn read(&mut self, ty: &Type, base: Expr, offset: usize, context: Expr) -> Option<Expr> {
        let pointer = bridge_pointer(base.clone(), offset, true);
        match ty {
            Type::Unit => Some(Expr::compound_literal(
                "MalType_Unit",
                [Initializer::designated(
                    "unused",
                    Expr::named_call("UINT8_C", [number(0)]),
                )],
            )),
            Type::Symbol => {
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
            Type::External { .. } => Some(Expr::compound_literal(
                self.raw_types.c_type(ty),
                [Initializer::designated(
                    "bits",
                    load(TypeName::const_named("uintptr_t").pointer(), pointer),
                )],
            )),
            Type::Product(elements) => {
                let fields = self.types.product_fields(ty)?;
                let initializers = elements
                    .iter()
                    .zip(fields)
                    .enumerate()
                    .map(|(index, (element, field))| {
                        Some(Initializer::designated(
                            format!("field_{index}"),
                            self.read(
                                element,
                                base.clone(),
                                offset.checked_add(field.offset)?,
                                context.clone(),
                            )?,
                        ))
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(Expr::compound_literal(
                    self.raw_types.c_type(ty),
                    initializers,
                ))
            }
            Type::Sum(elements) if !body::types::is_bool(ty) => {
                self.read_sum(ty, elements, pointer, context)
            }
            _ => Some(load(
                TypeName::const_named(c_scalar_type(ty)?).pointer(),
                pointer,
            )),
        }
    }

    fn read_sum(
        &mut self,
        ty: &Type,
        elements: &[Type],
        pointer: Expr,
        context: Expr,
    ) -> Option<Expr> {
        let helper = self.helper_name("read");
        let c_type = self.raw_types.c_type(ty);
        let fields = self.types.sum_fields(ty)?;
        let tag_offset = fields.first()?.offset;
        let cases = elements
            .iter()
            .zip(fields.into_iter().skip(1))
            .enumerate()
            .map(|(index, (element, field))| {
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
                                    element,
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
        ty: &Type,
        value: Expr,
        offset: usize,
        context: Expr,
    ) -> Option<Vec<Statement>> {
        self.write_at(ty, identifier("mal_result"), value, offset, context)
    }

    fn write_at(
        &mut self,
        ty: &Type,
        base: Expr,
        value: Expr,
        offset: usize,
        context: Expr,
    ) -> Option<Vec<Statement>> {
        let pointer = bridge_pointer(base.clone(), offset, false);
        match ty {
            Type::Unit => Some(vec![store(
                "uint8_t",
                pointer,
                Expr::named_call("UINT8_C", [number(0)]),
            )]),
            Type::Symbol => Some(vec![store(
                TypeName::named("void").pointer(),
                pointer,
                value.field("ownership"),
            )]),
            Type::External { .. } => Some(vec![store("uintptr_t", pointer, value.field("bits"))]),
            Type::Product(elements) => {
                let fields = self.types.product_fields(ty)?;
                let mut statements = Vec::new();
                for (index, (element, field)) in elements.iter().zip(fields).enumerate() {
                    statements.extend(self.write_at(
                        element,
                        base.clone(),
                        value.clone().field(format!("field_{index}")),
                        offset.checked_add(field.offset)?,
                        context.clone(),
                    )?);
                }
                Some(statements)
            }
            Type::Sum(elements) if !body::types::is_bool(ty) => self
                .write_sum(ty, elements, value, pointer, context)
                .map(|call| vec![call]),
            _ => Some(vec![store(c_scalar_type(ty)?, pointer, value)]),
        }
    }

    fn write_sum(
        &mut self,
        ty: &Type,
        elements: &[Type],
        value: Expr,
        pointer: Expr,
        context: Expr,
    ) -> Option<Statement> {
        let helper = self.helper_name("write");
        let c_type = self.raw_types.c_type(ty);
        let fields = self.types.sum_fields(ty)?;
        let tag_offset = fields.first()?.offset;
        let mut cases = Vec::new();
        for (index, (element, field)) in elements.iter().zip(fields.into_iter().skip(1)).enumerate()
        {
            let mut statements = self.write_at(
                element,
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
    c_scalar_type(ty).is_some()
        || matches!(ty, Type::Unit | Type::Symbol)
        || matches!(ty, Type::External { .. })
        || matches!(ty, Type::Product(elements) if elements.iter().all(type_supported))
        || matches!(ty, Type::Sum(elements) if elements.iter().all(type_supported))
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
