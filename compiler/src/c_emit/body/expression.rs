use crate::check::ast::{MemoryPrimitive, MemoryScalar, Type};
use crate::closure::ast::{Atom, AtomKind, Operation, Reference};
use crate::core::ast::{BinaryPrimitive, UnaryPrimitive};

use crate::c_emit::scalar::integer_type;

use super::{BodyEmitter, function_name, value_name};

impl BodyEmitter<'_> {
    pub(super) fn emit_operation_expression(
        &mut self,
        operation: &Operation,
        result: &Type,
    ) -> String {
        match operation {
            Operation::Atom(atom) => self.emit_atom(atom),
            Operation::Call { callee, argument } => {
                let callee = self.emit_atom(callee);
                format!(
                    "{callee}.call(mal_context, {callee}.environment, {})",
                    self.emit_atom(argument)
                )
            }
            Operation::StringLength { value } => {
                format!("({}).length", self.emit_atom(value))
            }
            Operation::StringAt { argument } => {
                self.needs.string_at = true;
                let argument = self.emit_atom(argument);
                format!("mal_string_at(mal_context, {argument}.field_0, {argument}.field_1)")
            }
            Operation::Memory {
                primitive,
                argument,
            } => {
                self.needs.memory = true;
                let argument = self.emit_atom(argument);
                match primitive {
                    MemoryPrimitive::Offset => format!(
                        "mal_ptr_offset(mal_context, {argument}.field_0, {argument}.field_1)"
                    ),
                    MemoryPrimitive::Load(scalar) => {
                        format!("mal_load_{}({argument})", memory_scalar_name(*scalar))
                    }
                    MemoryPrimitive::Store(scalar) => format!(
                        "mal_store_{}({argument}.field_0, {argument}.field_1)",
                        memory_scalar_name(*scalar)
                    ),
                }
            }
            Operation::ExternalCall { id, argument } => self.emit_external_call(*id, argument),
            Operation::NumericConversion { operand } => {
                let source = &operand.ty;
                let operand = self.emit_atom(operand);
                if integer_type(source).is_some() && integer_type(result).is_some() {
                    let unsigned = integer_type(result).unwrap().unsigned;
                    self.wrap_integer(result, &format!("({unsigned})({operand})"))
                } else if is_float_type(source) && integer_type(result).is_some() {
                    let source_index = usize::from(*source == Type::Float64);
                    let target = integer_type(result).unwrap();
                    let target_index = target.index;
                    self.needs.float_to_integer |= 1_u32 << (source_index * 8 + target_index);
                    let target_name = target.name;
                    let source_name = if *source == Type::Float32 {
                        "f32"
                    } else {
                        "f64"
                    };
                    format!("mal_{source_name}_to_{target_name}(mal_context, {operand})")
                } else if is_float_type(result) {
                    format!("({})({operand})", self.types.c_type(result))
                } else {
                    unreachable!("type checking admits only numeric conversions")
                }
            }
            Operation::Product(elements) => format!(
                "({}){{ {} }}",
                self.types.c_type(result),
                elements
                    .iter()
                    .enumerate()
                    .map(|(index, element)| {
                        format!(".field_{index} = {}", self.emit_atom(element))
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Operation::SumInjection { index, value } => format!(
                "({}){{ .tag = UINT32_C({index}), .payload.variant_{index} = {} }}",
                self.types.c_type(result),
                self.emit_atom(value)
            ),
            Operation::PrimitiveUnary { operator, operand } => {
                let operand_text = self.emit_atom(operand);
                if matches!(operand.ty, Type::Float32 | Type::Float64) {
                    return match operator {
                        UnaryPrimitive::Negate => format!("-({operand_text})"),
                        UnaryPrimitive::BitwiseNot => {
                            unreachable!("bitwise not is not defined for Float")
                        }
                    };
                }
                let unsigned = integer_type(&operand.ty).unwrap().unsigned;
                let expression = match operator {
                    UnaryPrimitive::Negate => {
                        format!("({unsigned})0 - ({unsigned})({operand_text})")
                    }
                    UnaryPrimitive::BitwiseNot => format!("~({unsigned})({operand_text})"),
                };
                self.wrap_integer(&operand.ty, &expression)
            }
            Operation::PrimitiveBinary {
                operator,
                left,
                right,
            } => self.emit_binary(*operator, left, right, result),
            Operation::MakeClosure { .. } | Operation::Case { .. } => {
                unreachable!("structured operations are emitted as statements")
            }
        }
    }

    pub(super) fn emit_external_call(
        &self,
        id: crate::resolve::ast::ExternalOperationId,
        argument: &Atom,
    ) -> String {
        let external = self.external(id);
        let arguments = match &external.parameter {
            Type::Unit => String::new(),
            Type::Product(elements) => {
                let argument = self.emit_atom(argument);
                elements
                    .iter()
                    .enumerate()
                    .map(|(index, _)| format!(", {argument}.field_{index}"))
                    .collect()
            }
            _ => format!(", {}", self.emit_atom(argument)),
        };
        format!("mal_ext_{}(mal_context{arguments})", external.name)
    }

    fn emit_binary(
        &mut self,
        operator: BinaryPrimitive,
        left: &Atom,
        right: &Atom,
        result: &Type,
    ) -> String {
        let operand_type = left.ty.clone();
        let left = self.emit_atom(left);
        let right = self.emit_atom(right);
        match operator {
            BinaryPrimitive::Multiply | BinaryPrimitive::Add | BinaryPrimitive::Subtract => {
                if matches!(operand_type, Type::Float32 | Type::Float64) {
                    let symbol = match operator {
                        BinaryPrimitive::Multiply => "*",
                        BinaryPrimitive::Add => "+",
                        BinaryPrimitive::Subtract => "-",
                        _ => unreachable!(),
                    };
                    return format!("({left} {symbol} {right})");
                }
                let integer = integer_type(&operand_type).unwrap();
                let unsigned = integer.unsigned;
                let carrier = integer.carrier;
                let symbol = match operator {
                    BinaryPrimitive::Multiply => "*",
                    BinaryPrimitive::Add => "+",
                    BinaryPrimitive::Subtract => "-",
                    _ => unreachable!(),
                };
                let expression = format!(
                    "({carrier})({unsigned})({left}) {symbol} ({carrier})({unsigned})({right})"
                );
                self.wrap_integer(&operand_type, &expression)
            }
            BinaryPrimitive::Divide => {
                if matches!(operand_type, Type::Float32 | Type::Float64) {
                    return format!("({left} / {right})");
                }
                let integer = integer_type(&operand_type).unwrap();
                self.needs.divide |= integer.mask();
                let name = integer.name;
                format!("mal_{name}_divide(mal_context, {left}, {right})")
            }
            BinaryPrimitive::Remainder => {
                let integer = integer_type(&operand_type).unwrap();
                self.needs.remainder |= integer.mask();
                let name = integer.name;
                format!("mal_{name}_remainder(mal_context, {left}, {right})")
            }
            BinaryPrimitive::ShiftLeft | BinaryPrimitive::ShiftRight => {
                if operator == BinaryPrimitive::ShiftLeft {
                    self.needs.shift_left |= integer_type(&operand_type).unwrap().mask();
                } else {
                    self.needs.shift_right |= integer_type(&operand_type).unwrap().mask();
                }
                let integer = integer_type(&operand_type).unwrap();
                self.needs.wrap |= integer.mask();
                let name = integer.name;
                let direction = if operator == BinaryPrimitive::ShiftLeft {
                    "shift_left"
                } else {
                    "shift_right"
                };
                format!("mal_{name}_{direction}(mal_context, {left}, {right})")
            }
            BinaryPrimitive::BitwiseAnd
            | BinaryPrimitive::BitwiseXor
            | BinaryPrimitive::BitwiseOr => {
                let unsigned = integer_type(&operand_type).unwrap().unsigned;
                let symbol = match operator {
                    BinaryPrimitive::BitwiseAnd => "&",
                    BinaryPrimitive::BitwiseXor => "^",
                    BinaryPrimitive::BitwiseOr => "|",
                    _ => unreachable!(),
                };
                let expression = format!("({unsigned})({left}) {symbol} ({unsigned})({right})");
                self.wrap_integer(&operand_type, &expression)
            }
            BinaryPrimitive::Less
            | BinaryPrimitive::LessEqual
            | BinaryPrimitive::Greater
            | BinaryPrimitive::GreaterEqual
            | BinaryPrimitive::Equal
            | BinaryPrimitive::NotEqual => {
                if operand_type == Type::String {
                    self.needs.string_equality = true;
                    let equality = format!("mal_string_equal({left}, {right})");
                    let condition = if operator == BinaryPrimitive::Equal {
                        equality
                    } else {
                        format!("!{equality}")
                    };
                    return format!(
                        "({}){{ .tag = ({condition}) ? UINT32_C(1) : UINT32_C(0) }}",
                        self.types.c_type(result)
                    );
                }
                let symbol = match operator {
                    BinaryPrimitive::Less => "<",
                    BinaryPrimitive::LessEqual => "<=",
                    BinaryPrimitive::Greater => ">",
                    BinaryPrimitive::GreaterEqual => ">=",
                    BinaryPrimitive::Equal => "==",
                    BinaryPrimitive::NotEqual => "!=",
                    _ => unreachable!(),
                };
                format!(
                    "({}){{ .tag = ({left} {symbol} {right}) ? UINT32_C(1) : UINT32_C(0) }}",
                    self.types.c_type(result)
                )
            }
        }
    }

    fn wrap_integer(&mut self, ty: &Type, expression: &str) -> String {
        let integer = integer_type(ty).unwrap();
        let name = integer.name;
        let unsigned = integer.unsigned;
        if integer.signed() {
            self.needs.wrap |= integer.mask();
            format!("mal_{name}_from_{unsigned}(({unsigned})({expression}))")
        } else {
            format!("({unsigned})({expression})")
        }
    }

    pub(super) fn emit_atom(&self, atom: &Atom) -> String {
        match &atom.kind {
            AtomKind::Reference(Reference::Binding(id)) => value_name(*id),
            AtomKind::Reference(Reference::EnvironmentField(index)) => {
                format!("mal_environment_fields->field_{index}")
            }
            AtomKind::Reference(Reference::SelfClosure(function)) => format!(
                "({}){{ .call = {}, .environment = mal_environment }}",
                self.types.c_type(&atom.ty),
                function_name(*function)
            ),
            AtomKind::Integer(value) => {
                let integer = integer_type(&atom.ty).expect("integer atoms have integer types");
                if integer.minimum_value == Some(*value) {
                    integer.minimum.unwrap().into()
                } else {
                    let constant = integer.constant;
                    format!("{constant}({value})")
                }
            }
            AtomKind::Float(bits) => match atom.ty {
                Type::Float32 => format!("mal_float32_from_bits(UINT32_C({bits}))"),
                Type::Float64 => format!("mal_float64_from_bits(UINT64_C({bits}))"),
                _ => unreachable!("float atoms have Float32 or Float64 type"),
            },
            AtomKind::String(value) => {
                let bytes = value
                    .iter()
                    .map(|byte| format!("\\x{byte:02x}"))
                    .collect::<String>();
                format!(
                    "(MalString){{ (const uint8_t *)\"{bytes}\", UINT64_C({}) }}",
                    value.len()
                )
            }
            AtomKind::Unit => "(MalUnit){ UINT8_C(0) }".into(),
        }
    }
}

fn memory_scalar_name(scalar: MemoryScalar) -> &'static str {
    match scalar {
        MemoryScalar::Int8 => "int8",
        MemoryScalar::Int16 => "int16",
        MemoryScalar::Int32 => "int32",
        MemoryScalar::Int64 => "int64",
        MemoryScalar::UInt8 => "uint8",
        MemoryScalar::UInt16 => "uint16",
        MemoryScalar::UInt32 => "uint32",
        MemoryScalar::UInt64 => "uint64",
        MemoryScalar::Float32 => "float32",
        MemoryScalar::Float64 => "float64",
    }
}

fn is_float_type(ty: &Type) -> bool {
    matches!(ty, Type::Float32 | Type::Float64)
}
