mod plan;

use std::collections::HashMap;

use super::body;
use crate::backend::abi::Function as AbiFunction;
use crate::backend::c::syntax::{
    Expr, Statement, TranslationUnit, TypeName, c_expr, c_function, c_initializer, c_statement,
    c_switch_case, c_type,
};
use mal_frontend::check::ast::{SharedTypeId, Type};

pub(super) struct Bridge {
    pub(super) llvm_declaration: super::syntax::FunctionDeclaration,
    pub(super) c_definitions: TranslationUnit,
}

pub(super) fn generate(
    external: &crate::core::ast::ExternalOperation,
    target: super::TargetLayout,
    raw_types: &crate::backend::c::RawHostTypes,
) -> Option<Bridge> {
    let types = body::types::Types::for_target(target);
    let parameter = plan::Value::new(&external.parameter, types.clone())?;
    let result = plan::Value::new(&external.result, types)?;
    let mut marshalling = Marshalling::new(external.id.0, raw_types);
    let bridge = AbiFunction::external_bridge(external.id);
    let llvm_declaration = bridge.llvm_declaration();
    let (mut statements, arguments) = match &parameter.kind {
        plan::Kind::Unit => (
            vec![c_statement!(expr (cast "void"; (id "mal_argument")))],
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
    let call = c_expr!(call format!("mal_ext_{}", external.name); {{ call_arguments }});
    match &result.kind {
        plan::Kind::Unit => {
            statements.push(c_statement!(expr { call }));
            statements.push(store(
                "uint8_t",
                identifier("mal_result"),
                c_expr!(call "UINT8_C"; (number 0)),
            ));
        }
        plan::Kind::Product(_) | plan::Kind::Sum { .. } | plan::Kind::External => {
            statements.push(c_statement!(
                var(raw_types.c_type(&external.result))("result") = { call }
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
    marshalling
        .helpers
        .push(c_function!(signature { bridge.c_signature() };
            block [{{ statements }}]
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
    read_helpers: HashMap<SharedTypeId, String>,
    write_helpers: HashMap<SharedTypeId, String>,
    helpers: TranslationUnit,
}

impl<'a> Marshalling<'a> {
    fn new(external: u32, raw_types: &'a crate::backend::c::RawHostTypes) -> Self {
        Self {
            external,
            raw_types,
            next_helper: 0,
            read_helpers: HashMap::new(),
            write_helpers: HashMap::new(),
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
            plan::Kind::Unit => Some(c_expr!(compound "MalType_Unit";
                (field "unused"; (call "UINT8_C"; (number 0))),
            )),
            plan::Kind::External => Some(c_expr!(compound self.raw_types.c_type(value.ty);
                (field "bits"; { load(c_type!(ptr(const(named("uintptr_t")))), pointer) }),
            )),
            plan::Kind::Product(fields) => {
                let initializers = fields
                    .iter()
                    .enumerate()
                    .map(|(index, field)| {
                        Some(c_initializer!(field format!("field_{index}");
                            { self.read(
                                &field.value,
                                base.clone(),
                                offset.checked_add(field.offset)?,
                                context.clone(),
                            )? }
                        ))
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(c_expr!(compound self.raw_types.c_type(value.ty);
                    {{ initializers }},
                ))
            }
            plan::Kind::Sum {
                tag_offset,
                variants,
            } => self.read_sum(value.ty, *tag_offset, variants, pointer, context),
            plan::Kind::Scalar => Some(load(
                c_type!(ptr(const(named(c_scalar_type(value.ty)?)))),
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
        if let Some(helper) = ty.shared_id().and_then(|id| self.read_helpers.get(&id)) {
            return Some(c_expr!(call helper.clone(); { context }, { pointer }));
        }
        let helper = self.helper_name("read");
        if let Some(id) = ty.shared_id() {
            self.read_helpers.insert(id, helper.clone());
        }
        let c_type = self.raw_types.c_type(ty);
        let cases = variants
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let payload = self.read(
                    &field.value,
                    identifier("value"),
                    field.offset,
                    identifier("context"),
                )?;
                Some(c_switch_case!(case (call "UINT32_C"; (number index)); [
                    (return (compound c_type.clone();
                        (field "tag"; (call "UINT32_C"; (number index))),
                        (path ["payload", &format!("variant_{index}")]; { payload }),
                    )),
                ]))
            })
            .collect::<Option<Vec<_>>>()?;
        let mut cases = cases;
        cases.push(c_switch_case!(default; [{ trap(
            identifier("context"),
            "invalid sum tag at LLVM bridge",
        ) }]));
        let tag = load(
            c_type!(ptr(const(named("uint32_t")))),
            c_expr!(add (id "value"); (number tag_offset)),
        );
        self.helpers.push(c_function!(
            static fn { helper.clone() }(
                "context": ptr(named("MalContext")),
                "value": ptr(const(named("uint8_t"))),
            ) -> { c_type }
            block [
                (var ("uint32_t") ("tag") = { tag }),
                (switch (id "tag"); [{{ cases }}]),
            ]
        ));
        self.helpers.blank_line();
        Some(c_expr!(call helper; { context }, { pointer }))
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
                c_expr!(call "UINT8_C"; (number 0)),
            )]),
            plan::Kind::External => Some(vec![store(
                "uintptr_t",
                pointer,
                c_expr!(field { value }; "bits"),
            )]),
            plan::Kind::Product(fields) => {
                let mut statements = Vec::new();
                for (index, field) in fields.iter().enumerate() {
                    statements.extend(self.write_at(
                        &field.value,
                        base.clone(),
                        c_expr!(field { value.clone() }; format!("field_{index}")),
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
        if let Some(helper) = ty.shared_id().and_then(|id| self.write_helpers.get(&id)) {
            return Some(c_statement!(call helper.clone();
                { context },
                { pointer },
                { value },
            ));
        }
        let helper = self.helper_name("write");
        if let Some(id) = ty.shared_id() {
            self.write_helpers.insert(id, helper.clone());
        }
        let c_type = self.raw_types.c_type(ty);
        let mut cases = Vec::new();
        for (index, field) in variants.iter().enumerate() {
            let mut statements = self.write_at(
                &field.value,
                identifier("value"),
                c_expr!(field
                    (field (id "input"); "payload");
                    format!("variant_{index}")
                ),
                field.offset,
                identifier("context"),
            )?;
            statements.push(c_statement!(return_void));
            cases.push(c_switch_case!(case (call "UINT32_C"; (number index)); [
                {{ statements }},
            ]));
        }
        cases.push(c_switch_case!(default; [{ trap(
            identifier("context"),
            "invalid sum tag at LLVM bridge",
        ) }]));
        let tag_store = store(
            "uint32_t",
            c_expr!(add (id "value"); (number tag_offset)),
            c_expr!(field (id "input"); "tag"),
        );
        self.helpers.push(c_function!(
            static fn { helper.clone() }(
                "context": ptr(named("MalContext")),
                "value": ptr(named("uint8_t")),
                "input": { c_type },
            ) -> named("void")
            block [
                { tag_store },
                (switch (field (id "input"); "tag"); [{{ cases }}]),
            ]
        ));
        self.helpers.blank_line();
        Some(c_statement!(call helper;
            { context },
            { pointer },
            { value },
        ))
    }
}

fn bridge_pointer(base: Expr, offset: usize, read_only: bool) -> Expr {
    let ty = if read_only {
        c_type!(ptr(const(named("uint8_t"))))
    } else {
        c_type!(ptr(named("uint8_t")))
    };
    let pointer = c_expr!(cast ty; { base });
    if offset == 0 {
        pointer
    } else {
        c_expr!(add { pointer }; (number offset))
    }
}

fn context_cast() -> Expr {
    c_expr!(cast c_type!(ptr(named("MalContext"))); (id "mal_context"))
}

fn load(ty: TypeName, pointer: Expr) -> Expr {
    c_expr!(dereference (cast ty; { pointer }))
}

fn store(ty: impl Into<TypeName>, pointer: Expr, value: Expr) -> Statement {
    c_statement!(expr (assign
        (dereference (cast c_type!({ ty.into().pointer() }); { pointer }));
        { value }
    ))
}

fn identifier(name: impl Into<crate::backend::c::syntax::Identifier>) -> Expr {
    c_expr!(id name)
}

fn trap(context: Expr, message: &str) -> Statement {
    c_statement!(call "mal_trap"; { context }, (string message))
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
        Type::Address => Some("MalType_Address"),
        Type::ByteSize => Some("MalType_ByteSize"),
        Type::USize => Some("MalType_USize"),
        _ => None,
    }
}
