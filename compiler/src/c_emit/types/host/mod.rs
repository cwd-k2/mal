use crate::c_emit::syntax::{
    AggregateDefinition, AggregateField, Block, Declaration, Directive, Expr, FunctionDefinition,
    FunctionSignature, Initializer, Parameter, Statement, SwitchCase, TranslationUnit, TypeName,
};
use crate::check::ast::Type;
use crate::core::ast::TypeAlias;

use super::{HostTypes, TypeRegistry, is_bool};

impl TypeRegistry {
    pub(in crate::c_emit) fn host_value_helpers(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for (index, ty) in self.aggregates.iter().enumerate() {
            if !host.contains(ty) || is_bool(ty) {
                continue;
            }
            match ty {
                Type::Product(_) => append_function(
                    &mut output,
                    FunctionSignature::static_inline(
                        self.c_type(ty),
                        format!("mal_repr_product_{index}_return"),
                        [
                            Parameter::named(TypeName::named("mal_call_t").pointer(), "call")
                                .maybe_unused(),
                            Parameter::named(format!("mal_repr_product_{index}_t"), "value"),
                        ],
                    ),
                    Block::new([Statement::return_value(self.host_to_raw_value(
                        ty,
                        Expr::identifier("call"),
                        Expr::identifier("value"),
                    ))]),
                ),
                Type::Sum(members) => {
                    output.extend(self.host_sum_conversion_helpers(index, ty));
                    self.append_host_sum_helpers(
                        &mut output,
                        &format!("repr_sum_{index}"),
                        &format!("mal_repr_sum_{index}_t"),
                        self.c_type(ty),
                        members,
                        &vec![None; members.len()],
                    );
                }
                _ => unreachable!("only aggregates have structural host helpers"),
            }
        }
        for name in &host.opaque_names {
            let host_type = format!("mal_{name}_t");
            append_function(
                &mut output,
                FunctionSignature::static_inline(
                    host_type.clone(),
                    format!("mal_{name}_from_bits"),
                    [Parameter::named("uintptr_t", "bits")],
                ),
                Block::new([Statement::return_value(Expr::compound_literal(
                    host_type.clone(),
                    [Initializer::designated(
                        "mal_detail_bits",
                        Expr::identifier("bits"),
                    )],
                ))]),
            );
            append_function(
                &mut output,
                FunctionSignature::static_inline(
                    "uintptr_t",
                    format!("mal_{name}_to_bits"),
                    [Parameter::named(host_type.clone(), "value")],
                ),
                Block::new([Statement::return_value(
                    Expr::identifier("value").field("mal_detail_bits"),
                )]),
            );
            append_function(
                &mut output,
                FunctionSignature::static_inline(
                    format!("MalType_{name}"),
                    format!("mal_{name}_return"),
                    [
                        Parameter::named(TypeName::named("mal_call_t").pointer(), "call")
                            .maybe_unused(),
                        Parameter::named(host_type, "value"),
                    ],
                ),
                Block::new([Statement::return_value(Expr::compound_literal(
                    format!("MalType_{name}"),
                    [Initializer::designated(
                        "bits",
                        Expr::identifier("value").field("mal_detail_bits"),
                    )],
                ))]),
            );
        }
        for alias in aliases {
            if !host.contains(&alias.ty) {
                continue;
            }
            if let Type::Sum(members) = &alias.ty {
                self.append_host_sum_helpers(
                    &mut output,
                    &alias.name,
                    &format!("mal_{}_t", alias.name),
                    self.header_c_type(&alias.ty, Some(&alias.name)),
                    members,
                    &alias.element_aliases,
                );
                continue;
            }
            append_function(
                &mut output,
                FunctionSignature::static_inline(
                    self.header_c_type(&alias.ty, Some(&alias.name)),
                    format!("mal_{}_return", alias.name),
                    [
                        Parameter::named(TypeName::named("mal_call_t").pointer(), "call")
                            .maybe_unused(),
                        Parameter::named(format!("mal_{}_t", alias.name), "value"),
                    ],
                ),
                Block::new([Statement::return_value(self.host_to_raw_value(
                    &alias.ty,
                    Expr::identifier("call"),
                    Expr::identifier("value"),
                ))]),
            );
        }
        output
    }

    fn append_host_sum_helpers(
        &self,
        output: &mut TranslationUnit,
        public_name: &str,
        host_type: &str,
        raw_type: TypeName,
        members: &[Type],
        element_aliases: &[Option<String>],
    ) {
        for (variant, member) in members.iter().enumerate() {
            let tag_name = format!("mal_{public_name}_tag_{variant}");
            output.push(Directive::define_expr(
                tag_name.clone(),
                Expr::named_call("UINT32_C", [Expr::number(variant.to_string())]),
            ));
            let (parameters, payload) = if *member == Type::Unit {
                (
                    Vec::new(),
                    Expr::compound_literal(
                        "mal_Unit_t",
                        [Initializer::positional(Expr::number("0"))],
                    ),
                )
            } else {
                (
                    vec![Parameter::named(
                        self.host_value_c_type(member, element_aliases[variant].as_deref()),
                        "value",
                    )],
                    Expr::identifier("value"),
                )
            };
            let host_value = Expr::compound_literal(
                host_type,
                [
                    Initializer::designated("tag", Expr::identifier(tag_name)),
                    Initializer::designated_path(
                        ["payload".into(), format!("variant_{variant}")],
                        payload,
                    ),
                ],
            );
            append_function(
                output,
                FunctionSignature::static_inline(
                    host_type,
                    format!("mal_{public_name}_make_{variant}"),
                    parameters.clone(),
                ),
                Block::new([Statement::return_value(host_value.clone())]),
            );
            let mut return_parameters = vec![Parameter::named(
                TypeName::named("mal_call_t").pointer(),
                "call",
            )];
            return_parameters.extend(parameters);
            append_function(
                output,
                FunctionSignature::static_inline(
                    raw_type.clone(),
                    format!("mal_{public_name}_return_{variant}"),
                    return_parameters,
                ),
                Block::new([Statement::return_value(self.host_to_raw_value(
                    &Type::Sum(members.to_vec()),
                    Expr::identifier("call"),
                    host_value,
                ))]),
            );
        }
    }

    fn host_sum_conversion_helpers(&self, index: usize, ty: &Type) -> TranslationUnit {
        let Type::Sum(members) = ty else {
            unreachable!("sum conversion requires a sum type")
        };
        let raw_type = self.c_type(ty);
        let host_type = self.host_value_c_type(ty, None);
        let mut to_host_cases = Vec::new();
        let mut to_raw_cases = Vec::new();
        for (variant, member) in members.iter().enumerate() {
            let tag = Expr::named_call("UINT32_C", [Expr::number(variant.to_string())]);
            to_host_cases.push(SwitchCase::case(
                tag.clone(),
                Block::new([Statement::return_value(Expr::compound_literal(
                    host_type.clone(),
                    [
                        Initializer::designated("tag", tag.clone()),
                        Initializer::designated_path(
                            ["payload".into(), format!("variant_{variant}")],
                            self.raw_to_host_value(
                                member,
                                None,
                                Expr::identifier("call"),
                                Expr::identifier("value")
                                    .field("payload")
                                    .field(format!("variant_{variant}")),
                            ),
                        ),
                    ],
                ))]),
            ));
            to_raw_cases.push(SwitchCase::case(
                tag.clone(),
                Block::new([Statement::return_value(Expr::compound_literal(
                    raw_type.clone(),
                    [
                        Initializer::designated("tag", tag),
                        Initializer::designated_path(
                            ["payload".into(), format!("variant_{variant}")],
                            self.host_to_raw_value(
                                member,
                                Expr::identifier("call"),
                                Expr::identifier("value")
                                    .field("payload")
                                    .field(format!("variant_{variant}")),
                            ),
                        ),
                    ],
                ))]),
            ));
        }
        for cases in [&mut to_host_cases, &mut to_raw_cases] {
            cases.push(SwitchCase::default(Block::new([Statement::call(
                "mal_call_trap",
                [Expr::identifier("call"), Expr::string("invalid sum tag")],
            )])));
        }
        let mut output = TranslationUnit::default();
        output.push(FunctionDefinition::from_signature(
            FunctionSignature::static_inline(
                host_type.clone(),
                format!("mal_detail_to_host_{index}"),
                [
                    Parameter::named(TypeName::named("mal_call_t").pointer(), "call"),
                    Parameter::named(raw_type.clone(), "value"),
                ],
            ),
            Block::new([Statement::switch(
                Expr::identifier("value").field("tag"),
                to_host_cases,
            )]),
        ));
        output.blank_line();
        output.push(FunctionDefinition::from_signature(
            FunctionSignature::static_inline(
                raw_type,
                format!("mal_detail_to_raw_{index}"),
                [
                    Parameter::named(TypeName::named("mal_call_t").pointer(), "call"),
                    Parameter::named(host_type, "value"),
                ],
            ),
            Block::new([Statement::switch(
                Expr::identifier("value").field("tag"),
                to_raw_cases,
            )]),
        ));
        output.blank_line();
        output
    }

    fn host_to_raw_value(&self, ty: &Type, call: Expr, value: Expr) -> Expr {
        match ty {
            Type::Product(elements) => Expr::compound_literal(
                self.c_type(ty),
                elements.iter().enumerate().map(|(field, element)| {
                    Initializer::designated(
                        format!("field_{field}"),
                        self.host_to_raw_value(
                            element,
                            call.clone(),
                            value.clone().field(format!("field_{field}")),
                        ),
                    )
                }),
            ),
            Type::Sum(_) if !is_bool(ty) => Expr::named_call(
                format!("mal_detail_to_raw_{}", self.index(ty)),
                [call, value],
            ),
            Type::Symbol => Expr::named_call("mal_detail_Symbol_return", [call, value]),
            Type::Ptr => Expr::compound_literal(
                "MalType_Ptr",
                [Initializer::designated(
                    "address",
                    Expr::cast(TypeName::named("uint8_t").pointer(), value),
                )],
            ),
            Type::External { .. } => Expr::compound_literal(
                self.c_type(ty),
                [Initializer::designated(
                    "bits",
                    value.field("mal_detail_bits"),
                )],
            ),
            Type::Unit => {
                Expr::compound_literal("MalType_Unit", [Initializer::positional(Expr::number("0"))])
            }
            Type::Function { .. } => {
                unreachable!("type checking excludes functions from extern signatures")
            }
            _ => value,
        }
    }

    pub(in crate::c_emit) fn raw_to_host_value(
        &self,
        ty: &Type,
        alias: Option<&str>,
        call: Expr,
        value: Expr,
    ) -> Expr {
        match ty {
            Type::Product(elements) => Expr::compound_literal(
                self.host_value_c_type(ty, alias),
                elements.iter().enumerate().map(|(field, element)| {
                    Initializer::designated(
                        format!("field_{field}"),
                        self.raw_to_host_value(
                            element,
                            None,
                            call.clone(),
                            value.clone().field(format!("field_{field}")),
                        ),
                    )
                }),
            ),
            Type::Sum(_) if !is_bool(ty) => Expr::named_call(
                format!("mal_detail_to_host_{}", self.index(ty)),
                [call, value],
            ),
            Type::Symbol => Expr::compound_literal(
                "mal_Symbol_t",
                [
                    Initializer::designated("mal_detail_raw", value),
                    Initializer::designated(
                        "mal_detail_bytes",
                        Expr::compound_literal(
                            "mal_span_t",
                            [Initializer::positional(Expr::number("0"))],
                        ),
                    ),
                    Initializer::designated(
                        "mal_detail_source",
                        Expr::named_call("UINT8_C", [Expr::number("0")]),
                    ),
                ],
            ),
            Type::Ptr => Expr::cast("mal_Ptr_t", value.field("address")),
            Type::External { .. } => Expr::compound_literal(
                self.host_value_c_type(ty, alias),
                [Initializer::designated(
                    "mal_detail_bits",
                    value.field("bits"),
                )],
            ),
            Type::Function { .. } => {
                unreachable!("type checking excludes functions from extern signatures")
            }
            _ => value,
        }
    }

    pub(in crate::c_emit) fn host_value_declarations(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for name in &host.opaque_names {
            output.push(AggregateDefinition::typedef_structure(
                None,
                [AggregateField::variable("uintptr_t", "mal_detail_bits")],
                format!("mal_{name}_t"),
            ));
        }
        for (index, ty) in self.aggregates.iter().enumerate() {
            if !host.contains(ty) || is_bool(ty) {
                continue;
            }
            let kind = match ty {
                Type::Product(_) => "product",
                Type::Sum(_) => "sum",
                Type::Function { .. } => continue,
                _ => unreachable!("only aggregate types have representation identities"),
            };
            output.push(Declaration::type_alias(
                TypeName::structure(format!("mal_detail_repr_{kind}_{index}")),
                format!("mal_repr_{kind}_{index}_t"),
            ));
        }
        for alias in aliases {
            if host.contains(&alias.ty) {
                output.push(self.host_alias_declaration(alias));
            }
        }
        if !output.is_empty() {
            output.blank_line();
        }
        for (index, ty) in self.aggregates.iter().enumerate() {
            if !host.contains(ty) || is_bool(ty) {
                continue;
            }
            match ty {
                Type::Product(elements) => output.push(AggregateDefinition::structure(
                    format!("mal_detail_repr_product_{index}"),
                    elements.iter().enumerate().map(|(field, ty)| {
                        AggregateField::variable(
                            self.host_value_c_type(ty, None),
                            format!("field_{field}"),
                        )
                    }),
                )),
                Type::Sum(members) => output.push(AggregateDefinition::structure(
                    format!("mal_detail_repr_sum_{index}"),
                    [
                        AggregateField::variable("uint32_t", "tag"),
                        AggregateField::aggregate(
                            crate::c_emit::syntax::AggregateKind::Union,
                            members.iter().enumerate().map(|(variant, ty)| {
                                AggregateField::variable(
                                    self.host_value_c_type(ty, None),
                                    format!("variant_{variant}"),
                                )
                            }),
                            "payload",
                        ),
                    ],
                )),
                Type::Function { .. } => continue,
                _ => unreachable!("only aggregate types have representation identities"),
            }
            output.blank_line();
        }
        output
    }

    fn host_alias_declaration(&self, alias: &TypeAlias) -> Declaration {
        let source = alias.target_alias.as_deref().map_or_else(
            || self.host_value_c_type(&alias.ty, None),
            |target| TypeName::named(format!("mal_{target}_t")),
        );
        Declaration::type_alias(source, format!("mal_{}_t", alias.name))
    }

    pub(in crate::c_emit) fn header_declarations(&self, host: &HostTypes) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for name in &host.opaque_names {
            output.push(AggregateDefinition::typedef_structure(
                None,
                [AggregateField::variable("uintptr_t", "bits")],
                format!("MalType_{name}"),
            ));
        }
        if !host.opaque_names.is_empty() {
            output.blank_line();
        }
        output.extend(self.declarations(host, true));
        output
    }

    pub(in crate::c_emit) fn header_alias_declarations(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for alias in aliases {
            if host.contains(&alias.ty) {
                output.push(Declaration::type_alias(
                    self.c_type(&alias.ty),
                    format!("MalType_{}", alias.name),
                ));
            }
        }
        if !output.is_empty() {
            output.blank_line();
        }
        output
    }

    pub(in crate::c_emit) fn header_c_type(&self, ty: &Type, alias: Option<&str>) -> TypeName {
        alias.map_or_else(
            || self.c_type(ty),
            |alias| TypeName::named(format!("MalType_{alias}")),
        )
    }
}

fn append_function(output: &mut TranslationUnit, signature: FunctionSignature, body: Block) {
    output.push(FunctionDefinition::from_signature(signature, body));
    output.blank_line();
}
