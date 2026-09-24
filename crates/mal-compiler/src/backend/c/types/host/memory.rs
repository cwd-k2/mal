use crate::backend::c::syntax::{
    Block, Expr, FunctionSignature, Initializer, Parameter, Statement, SwitchCase, TranslationUnit,
    TypeName,
};
use crate::backend::source_layout::SourceLayouts;
use crate::check::ast::Type;
use crate::core::ast::TypeAlias;

use super::super::{HostTypes, TypeRegistry, is_bool};
use super::append_function;

impl TypeRegistry {
    pub(in crate::backend::c) fn memory_helpers(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
        layouts: SourceLayouts,
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        if !aliases.iter().any(|alias| alias.host_memory_access) {
            return output;
        }
        let tag_types = self
            .aggregates
            .iter()
            .filter(|ty| host.memory_contains(ty) && !is_bool(ty))
            .filter_map(|ty| layouts.sum(ty).map(|layout| integer_type(layout.tag_bits)))
            .collect::<Vec<_>>();
        for ty in scalar_types() {
            if host.memory_contains(&ty) || tag_types.contains(&ty) {
                self.append_scalar_memory_helpers(&mut output, &ty);
            }
        }
        for (index, ty) in self.aggregates.iter().enumerate() {
            if host.memory_contains(ty) && !is_bool(ty) {
                self.append_aggregate_memory_helpers(&mut output, index, ty, layouts);
            }
        }
        for alias in aliases.iter().filter(|alias| alias.host_memory_access) {
            self.append_alias_memory_helpers(&mut output, alias, layouts);
        }
        output
    }

    fn append_scalar_memory_helpers(&self, output: &mut TranslationUnit, ty: &Type) {
        let name = scalar_name(ty);
        let host_type = self.host_value_c_type(ty, None);
        let call = if matches!(ty, Type::Address) || is_bool(ty) {
            Parameter::named(TypeName::named("mal_call_t").pointer(), "call")
        } else {
            Parameter::named(TypeName::named("mal_call_t").pointer(), "call").maybe_unused()
        };
        let mut read_body = Block::new([Statement::variable(host_type.clone(), "value", None)]);
        read_body.push(Statement::call(
            "memcpy",
            [
                Expr::address_of(Expr::identifier("value")),
                Expr::identifier("source"),
                Expr::sizeof_value(Expr::identifier("value")),
            ],
        ));
        let value = if matches!(ty, Type::Address) {
            Expr::named_call(
                "mal_Address_return",
                [Expr::identifier("call"), Expr::identifier("value")],
            )
        } else if is_bool(ty) {
            Expr::named_call(
                "mal_Bool_return",
                [Expr::identifier("call"), Expr::identifier("value")],
            )
        } else {
            Expr::identifier("value")
        };
        read_body.push(Statement::return_value(value));
        append_function(
            output,
            FunctionSignature::static_inline(
                host_type.clone(),
                format!("mal_detail_memory_read_{name}"),
                [
                    call.clone(),
                    Parameter::named(TypeName::const_named("uint8_t").pointer(), "source"),
                ],
            ),
            read_body,
        );

        let mut write_body = Block::default();
        if matches!(ty, Type::Address) {
            write_body.push(Statement::call(
                "mal_Address_return",
                [Expr::identifier("call"), Expr::identifier("value")],
            ));
        } else if is_bool(ty) {
            write_body.push(Statement::call(
                "mal_Bool_return",
                [Expr::identifier("call"), Expr::identifier("value")],
            ));
        }
        write_body.push(Statement::call(
            "memcpy",
            [
                Expr::identifier("destination"),
                Expr::address_of(Expr::identifier("value")),
                Expr::sizeof_value(Expr::identifier("value")),
            ],
        ));
        append_function(
            output,
            FunctionSignature::static_inline(
                "void",
                format!("mal_detail_memory_write_{name}"),
                [
                    call,
                    Parameter::named(TypeName::named("uint8_t").pointer(), "destination"),
                    Parameter::named(host_type, "value"),
                ],
            ),
            write_body,
        );
    }

    fn append_aggregate_memory_helpers(
        &self,
        output: &mut TranslationUnit,
        index: usize,
        ty: &Type,
        layouts: SourceLayouts,
    ) {
        let host_type = self.host_value_c_type(ty, None);
        let read_body = match ty {
            Type::Product(elements) => {
                let fields = layouts
                    .product_fields(ty)
                    .expect("checker-approved memory product has a layout");
                let mut body = Block::new([Statement::variable(host_type.clone(), "value", None)]);
                for (field, (element, layout)) in elements.iter().zip(fields).enumerate() {
                    body.push(Statement::expression(Expr::assign(
                        Expr::identifier("value").field(format!("field_{field}")),
                        self.memory_read_value(
                            element,
                            Expr::identifier("call"),
                            offset(Expr::identifier("source"), layout.offset),
                        ),
                    )));
                }
                body.push(Statement::return_value(Expr::identifier("value")));
                body
            }
            Type::Sum(members) => self.sum_memory_read_body(ty, members, layouts),
            _ => unreachable!("only aggregate types have representation identities"),
        };
        append_function(
            output,
            FunctionSignature::static_inline(
                host_type.clone(),
                format!("mal_detail_memory_read_{index}"),
                [
                    Parameter::named(TypeName::named("mal_call_t").pointer(), "call")
                        .maybe_unused(),
                    Parameter::named(TypeName::const_named("uint8_t").pointer(), "source")
                        .maybe_unused(),
                ],
            ),
            read_body,
        );

        let write_body = match ty {
            Type::Product(elements) => {
                let fields = layouts
                    .product_fields(ty)
                    .expect("checker-approved memory product has a layout");
                Block::new(elements.iter().zip(fields).enumerate().map(
                    |(field, (element, layout))| {
                        self.memory_write_statement(
                            element,
                            Expr::identifier("call"),
                            offset(Expr::identifier("destination"), layout.offset),
                            Expr::identifier("value").field(format!("field_{field}")),
                        )
                    },
                ))
            }
            Type::Sum(members) => self.sum_memory_write_body(ty, members, layouts),
            _ => unreachable!("only aggregate types have representation identities"),
        };
        append_function(
            output,
            FunctionSignature::static_inline(
                "void",
                format!("mal_detail_memory_write_{index}"),
                [
                    Parameter::named(TypeName::named("mal_call_t").pointer(), "call")
                        .maybe_unused(),
                    Parameter::named(TypeName::named("uint8_t").pointer(), "destination")
                        .maybe_unused(),
                    Parameter::named(host_type, "value"),
                ],
            ),
            write_body,
        );
    }

    fn sum_memory_read_body(&self, ty: &Type, members: &[Type], layouts: SourceLayouts) -> Block {
        let layout = layouts
            .sum(ty)
            .expect("checker-approved memory sum has a layout");
        let tag_type = integer_type(layout.tag_bits);
        let mut cases = members
            .iter()
            .enumerate()
            .map(|(variant, member)| {
                SwitchCase::case(
                    Expr::number(variant.to_string()),
                    Block::new([Statement::return_value(Expr::compound_literal(
                        self.host_value_c_type(ty, None),
                        [
                            Initializer::designated(
                                "tag",
                                Expr::named_call("UINT32_C", [Expr::number(variant.to_string())]),
                            ),
                            Initializer::designated_path(
                                ["payload".into(), format!("variant_{variant}")],
                                self.memory_read_value(
                                    member,
                                    Expr::identifier("call"),
                                    offset(Expr::identifier("source"), layout.payload_offset),
                                ),
                            ),
                        ],
                    ))]),
                )
            })
            .collect::<Vec<_>>();
        cases.push(SwitchCase::default(Block::new([Statement::call(
            "mal_call_trap",
            [
                Expr::identifier("call"),
                Expr::string("invalid canonical sum tag"),
            ],
        )])));
        Block::new([Statement::switch(
            self.memory_read_value(
                &tag_type,
                Expr::identifier("call"),
                Expr::identifier("source"),
            ),
            cases,
        )])
    }

    fn sum_memory_write_body(&self, ty: &Type, members: &[Type], layouts: SourceLayouts) -> Block {
        let layout = layouts
            .sum(ty)
            .expect("checker-approved memory sum has a layout");
        let tag_type = integer_type(layout.tag_bits);
        let cases = members
            .iter()
            .enumerate()
            .map(|(variant, member)| {
                SwitchCase::case(
                    Expr::named_call("UINT32_C", [Expr::number(variant.to_string())]),
                    Block::new([
                        self.memory_write_statement(
                            &tag_type,
                            Expr::identifier("call"),
                            Expr::identifier("destination"),
                            Expr::cast(
                                self.host_value_c_type(&tag_type, None),
                                Expr::identifier("value").field("tag"),
                            ),
                        ),
                        self.memory_write_statement(
                            member,
                            Expr::identifier("call"),
                            offset(Expr::identifier("destination"), layout.payload_offset),
                            Expr::identifier("value")
                                .field("payload")
                                .field(format!("variant_{variant}")),
                        ),
                        Statement::return_void(),
                    ]),
                )
            })
            .chain([SwitchCase::default(Block::new([Statement::call(
                "mal_call_trap",
                [Expr::identifier("call"), Expr::string("invalid sum tag")],
            )]))])
            .collect();
        Block::new([Statement::switch(
            Expr::identifier("value").field("tag"),
            cases,
        )])
    }

    fn append_alias_memory_helpers(
        &self,
        output: &mut TranslationUnit,
        alias: &TypeAlias,
        layouts: SourceLayouts,
    ) {
        let stride = layouts
            .layout(&alias.ty)
            .expect("checker-approved memory alias has a layout")
            .stride;
        let source = offset(
            Expr::cast(
                TypeName::const_named("uint8_t").pointer(),
                Expr::identifier("address"),
            ),
            Expr::multiply(Expr::identifier("index"), Expr::number(stride.to_string())),
        );
        let mut read_body = Block::default();
        if stride == 0 {
            read_body.push(Statement::expression(Expr::cast(
                "void",
                Expr::identifier("index"),
            )));
        }
        read_body.push(Statement::call(
            "mal_Address_return",
            [Expr::identifier("call"), Expr::identifier("address")],
        ));
        read_body.push(Statement::return_value(self.memory_read_value(
            &alias.ty,
            Expr::identifier("call"),
            source,
        )));
        append_function(
            output,
            FunctionSignature::static_inline(
                format!("mal_{}_t", alias.name),
                format!("mal_{}_read", alias.name),
                [
                    Parameter::named(TypeName::named("mal_call_t").pointer(), "call"),
                    Parameter::named("mal_Address_t", "address"),
                    Parameter::named("mal_USize_t", "index"),
                ],
            ),
            read_body,
        );

        let destination = offset(
            Expr::cast(
                TypeName::named("uint8_t").pointer(),
                Expr::identifier("address"),
            ),
            Expr::multiply(Expr::identifier("index"), Expr::number(stride.to_string())),
        );
        let mut write_body = Block::default();
        if stride == 0 {
            write_body.push(Statement::expression(Expr::cast(
                "void",
                Expr::identifier("index"),
            )));
        }
        write_body.push(Statement::call(
            "mal_Address_return",
            [Expr::identifier("call"), Expr::identifier("address")],
        ));
        write_body.push(self.memory_write_statement(
            &alias.ty,
            Expr::identifier("call"),
            destination,
            Expr::identifier("value"),
        ));
        append_function(
            output,
            FunctionSignature::static_inline(
                "void",
                format!("mal_{}_write", alias.name),
                [
                    Parameter::named(TypeName::named("mal_call_t").pointer(), "call"),
                    Parameter::named("mal_Address_t", "address"),
                    Parameter::named("mal_USize_t", "index"),
                    Parameter::named(format!("mal_{}_t", alias.name), "value"),
                ],
            ),
            write_body,
        );
    }

    fn memory_read_value(&self, ty: &Type, call: Expr, source: Expr) -> Expr {
        match ty {
            Type::Unit => {
                Expr::compound_literal("mal_Unit_t", [Initializer::positional(Expr::number("0"))])
            }
            Type::Product(_) | Type::Sum(_) if !is_bool(ty) => Expr::named_call(
                format!("mal_detail_memory_read_{}", self.index(ty)),
                [call, source],
            ),
            _ => Expr::named_call(
                format!("mal_detail_memory_read_{}", scalar_name(ty)),
                [call, source],
            ),
        }
    }

    fn memory_write_statement(
        &self,
        ty: &Type,
        call: Expr,
        destination: Expr,
        value: Expr,
    ) -> Statement {
        if matches!(ty, Type::Unit) {
            return Statement::expression(Expr::cast("void", value));
        }
        let name = match ty {
            Type::Product(_) | Type::Sum(_) if !is_bool(ty) => self.index(ty).to_string(),
            _ => scalar_name(ty).into(),
        };
        Statement::call(
            format!("mal_detail_memory_write_{name}"),
            [call, destination, value],
        )
    }
}

fn scalar_types() -> Vec<Type> {
    vec![
        Type::Sum(vec![Type::Unit, Type::Unit].into()),
        Type::Int8,
        Type::Int16,
        Type::Int32,
        Type::Int64,
        Type::UInt8,
        Type::UInt16,
        Type::UInt32,
        Type::UInt64,
        Type::Float32,
        Type::Float64,
        Type::Address,
        Type::ByteSize,
        Type::USize,
    ]
}

fn scalar_name(ty: &Type) -> &'static str {
    if is_bool(ty) {
        return "Bool";
    }
    match ty {
        Type::Int8 => "Int8",
        Type::Int16 => "Int16",
        Type::Int32 => "Int32",
        Type::Int64 => "Int64",
        Type::UInt8 => "UInt8",
        Type::UInt16 => "UInt16",
        Type::UInt32 => "UInt32",
        Type::UInt64 => "UInt64",
        Type::Float32 => "Float32",
        Type::Float64 => "Float64",
        Type::Address => "Address",
        Type::ByteSize => "ByteSize",
        Type::USize => "USize",
        _ => unreachable!("only canonical scalar types have scalar memory helpers"),
    }
}

fn integer_type(bits: usize) -> Type {
    match bits {
        8 => Type::UInt8,
        16 => Type::UInt16,
        32 => Type::UInt32,
        64 => Type::UInt64,
        _ => unreachable!("canonical sum tag widths are fixed-width integers"),
    }
}

fn offset(base: Expr, offset: impl Into<Offset>) -> Expr {
    Expr::add(base, offset.into().0)
}

struct Offset(Expr);

impl From<usize> for Offset {
    fn from(value: usize) -> Self {
        Self(Expr::number(value.to_string()))
    }
}

impl From<Expr> for Offset {
    fn from(value: Expr) -> Self {
        Self(value)
    }
}
