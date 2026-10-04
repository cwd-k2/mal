//! C declarations of the registered host types, and the representation descriptors that convert them.

use super::*;

impl TypeRegistry {
    pub(super) fn declarations(&self, host: &HostTypes, public: bool) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for ty in &self.aggregates {
            if host.external_contains(ty) != public {
                continue;
            }
            let kind = match ty {
                Type::Product(_) => "MalRepr_Product",
                Type::Sum(_) => "MalRepr_Sum",
                Type::Function { .. } => "MalRepr_Closure",
                Type::External { .. }
                | Type::Unit
                | Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64
                | Type::UInt8
                | Type::UInt16
                | Type::UInt32
                | Type::UInt64
                | Type::Float32
                | Type::Float64
                | Type::Symbol
                | Type::Buffer(_)
                | Type::ByteSize
                | Type::USize => unreachable!(),
                Type::Parameter { .. }
                | Type::Bound { .. }
                | Type::Application { .. }
                | Type::Abstraction { .. }
                | Type::Opaque { .. } => {
                    unreachable!("these types never enter the C host registry")
                }
            };
            let id = self.index(ty);
            let guard = format!("MAL_DETAIL_RAW_REPR_{id}_DECLARED");
            let name = format!("{kind}_{id}");
            output.extend(c_items! {
                if !defined({ guard.clone() }) {
                    define!({ guard });
                    type { name } = Struct<{ format!("{kind}_{id}") }>;
                }
            });
        }
        if !output.is_empty() {
            output.blank_line();
        }
        for ty in &self.aggregates {
            if host.external_contains(ty) != public {
                continue;
            }
            if matches!(ty, Type::Product(_) | Type::Sum(_)) {
                self.append_repr_descriptor(&mut output, ty);
            }
            let guard = format!("MAL_DETAIL_RAW_REPR_{}_DEFINED", self.index(ty));
            let mut guarded = c_items! { define!({ guard.clone() }); };
            match ty {
                Type::Product(_) => {
                    let tag = format!("MalRepr_Product_{}", self.index(ty));
                    guarded.push(c_invocation!(MAL_DETAIL_DEFINE_PRODUCT_REPR(
                        { tag },
                        { format!("MAL_DETAIL_REPR_FIELDS_{}", self.index(ty)) },
                        MAL_DETAIL_RAW_REPR_FIELD,
                    )));
                }
                Type::Sum(members) => {
                    let tag = format!("MalRepr_Sum_{}", self.index(ty));
                    let template = if members.is_empty() {
                        "MAL_DETAIL_DEFINE_EMPTY_SUM_REPR"
                    } else {
                        "MAL_DETAIL_DEFINE_SUM_REPR"
                    };
                    let mut arguments = vec![c_expr!({ tag })];
                    if !members.is_empty() {
                        arguments.push(c_expr!({
                            format!("MAL_DETAIL_REPR_FIELDS_{}", self.index(ty))
                        }));
                        arguments.push(c_expr!(MAL_DETAIL_RAW_REPR_FIELD));
                    }
                    guarded.push(c_invocation!({ template }(..{ arguments })));
                }
                Type::Function { parameter, result } => {
                    let tag = format!("MalRepr_Closure_{}", self.index(ty));
                    let parameter = self.c_type(parameter);
                    let result = self.c_type(result);
                    guarded.extend(c_items! {
                        struct { tag } {
                            call: fn(
                                _: *mut MalContext,
                                _: *const void,
                                _: { parameter },
                            ) -> { result },
                            environment: *const void,
                            destroy_environment: fn(
                                _: *mut MalContext,
                                _: *const void,
                            ) -> void,
                        }
                    });
                }
                Type::External { .. }
                | Type::Unit
                | Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64
                | Type::UInt8
                | Type::UInt16
                | Type::UInt32
                | Type::UInt64
                | Type::Float32
                | Type::Float64
                | Type::Symbol
                | Type::Buffer(_)
                | Type::ByteSize
                | Type::USize => unreachable!(),
                Type::Parameter { .. }
                | Type::Bound { .. }
                | Type::Application { .. }
                | Type::Abstraction { .. }
                | Type::Opaque { .. } => {
                    unreachable!("these types never enter the C host registry")
                }
            }
            output.extend(c_items! {
                if !defined({ guard }) {
                    ..{ guarded }
                }
            });
            output.blank_line();
        }
        output
    }

    pub(super) fn append_repr_descriptor(&self, output: &mut TranslationUnit, ty: &Type) {
        const FIELDS_PER_CHUNK: usize = 32;

        let (member_prefix, elements) = match ty {
            Type::Product(elements) => ("field", elements.as_ref()),
            Type::Sum(members) => ("variant", members.as_ref()),
            _ => unreachable!("only products and sums have field descriptors"),
        };
        let id = self.index(ty);
        let descriptor = format!("MAL_DETAIL_REPR_FIELDS_{id}");
        let guard = format!("{descriptor}_DEFINED");
        let mut guarded = c_items! { define!({ guard.clone() }); };

        let fields = elements
            .iter()
            .enumerate()
            .map(|(index, element)| {
                let (to_host, to_raw) = self.repr_field_conversions(ty, element);
                c_invocation!(field(
                    context,
                    { index },
                    { format!("{member_prefix}_{index}") },
                    { self.c_type(element).to_string() },
                    { self.host_value_c_type(element, None).to_string() },
                    { to_host },
                    { to_raw },
                ))
            })
            .collect::<Vec<_>>();

        if fields.len() <= FIELDS_PER_CHUNK {
            guarded.push(Directive::invocations_define(
                descriptor,
                ["field", "context"],
                fields,
            ));
        } else {
            let mut chunks = Vec::new();
            for (chunk_index, fields) in fields.chunks(FIELDS_PER_CHUNK).enumerate() {
                let chunk = format!("{descriptor}_{chunk_index}");
                guarded.push(Directive::invocations_define(
                    chunk.clone(),
                    ["field", "context"],
                    fields.iter().cloned(),
                ));
                chunks.push(c_invocation!({ chunk }(field, context)));
            }
            guarded.push(Directive::invocations_define(
                descriptor,
                ["field", "context"],
                chunks,
            ));
        }
        output.extend(c_items! {
            if !defined({ guard }) {
                ..{ guarded }
            }
        });
    }

    fn repr_field_conversions(&self, aggregate: &Type, element: &Type) -> (String, String) {
        let identity = || "MAL_DETAIL_REPR_IDENTITY".to_string();
        let aggregate_to_host = || match element {
            Type::Product(_) | Type::Sum(_) if !is_bool(element) => {
                format!("mal_detail_to_host_{}", self.index(element))
            }
            Type::External { name, .. } => format!("mal_detail_to_host_{name}"),
            _ => identity(),
        };
        let aggregate_to_raw = || match element {
            Type::Unit => "mal_detail_convert_Unit".into(),
            Type::Product(_) => format!("mal_repr_product_{}_return", self.index(element)),
            Type::Sum(_) if !is_bool(element) => {
                format!("mal_detail_to_raw_{}", self.index(element))
            }
            Type::External { name, .. } => format!("mal_{name}_return"),
            Type::Symbol => "mal_Symbol_return_move".into(),
            Type::Buffer(_) => "mal_Buffer_return_move".into(),
            _ => identity(),
        };
        if matches!(aggregate, Type::Product(_)) {
            return (aggregate_to_host(), aggregate_to_raw());
        }

        let scalar = |ty: &Type| {
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
                Type::ByteSize => "ByteSize",
                Type::USize => "USize",
                _ => unreachable!("only host scalar types have builtin conversion helpers"),
            }
        };
        let sum_conversion = |direction: &str| match element {
            Type::Unit => "mal_detail_convert_Unit".into(),
            Type::Product(_) => {
                if direction == "host" {
                    format!("mal_detail_to_host_{}", self.index(element))
                } else {
                    format!("mal_repr_product_{}_return", self.index(element))
                }
            }
            Type::Sum(_) if !is_bool(element) => {
                format!("mal_detail_to_{direction}_{}", self.index(element))
            }
            Type::External { name, .. } => {
                if direction == "host" {
                    format!("mal_detail_to_host_{name}")
                } else {
                    format!("mal_{name}_return")
                }
            }
            Type::Symbol => {
                if direction == "host" {
                    "MAL_DETAIL_REPR_IDENTITY".into()
                } else {
                    "mal_Symbol_return_move".into()
                }
            }
            Type::Buffer(_) => {
                if direction == "host" {
                    "MAL_DETAIL_REPR_IDENTITY".into()
                } else {
                    "mal_Buffer_return_move".into()
                }
            }
            _ => format!("mal_{}_return", scalar(element)),
        };
        (sum_conversion("host"), sum_conversion("raw"))
    }
}
