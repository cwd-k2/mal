use crate::check::ast::Type;
use crate::closure::ast::{Atom, AtomKind, Operation, Reference};
use crate::core::ast::{BinaryPrimitive, UnaryPrimitive};

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
            Operation::ExternalCall { id, argument } => self.emit_external_call(*id, argument),
            Operation::IntegerConversion { operand } => {
                let operand = self.emit_atom(operand);
                let (_, unsigned, _, _) = integer_info(result);
                self.wrap_integer(result, &format!("({unsigned})({operand})"))
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
                let (_, unsigned, _, _) = integer_info(&operand.ty);
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
                let (_, unsigned, carrier, _) = integer_info(&operand_type);
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
                self.needs.divide |= integer_mask(&operand_type);
                let (name, _, _, _) = integer_info(&operand_type);
                format!("mal_{name}_divide(mal_context, {left}, {right})")
            }
            BinaryPrimitive::Remainder => {
                self.needs.remainder |= integer_mask(&operand_type);
                let (name, _, _, _) = integer_info(&operand_type);
                format!("mal_{name}_remainder(mal_context, {left}, {right})")
            }
            BinaryPrimitive::ShiftLeft | BinaryPrimitive::ShiftRight => {
                if operator == BinaryPrimitive::ShiftLeft {
                    self.needs.shift_left |= integer_mask(&operand_type);
                } else {
                    self.needs.shift_right |= integer_mask(&operand_type);
                }
                self.needs.wrap |= integer_mask(&operand_type);
                let (name, _, _, _) = integer_info(&operand_type);
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
                let (_, unsigned, _, _) = integer_info(&operand_type);
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
        let (name, unsigned, _, signed) = integer_info(ty);
        if signed {
            self.needs.wrap |= integer_mask(ty);
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
                let (constant, minimum) = match atom.ty {
                    Type::Int8 => ("INT8_C", Some(i128::from(i8::MIN))),
                    Type::Int16 => ("INT16_C", Some(i128::from(i16::MIN))),
                    Type::Int32 => ("INT32_C", Some(i128::from(i32::MIN))),
                    Type::Int64 => ("INT64_C", Some(i128::from(i64::MIN))),
                    Type::UInt8 => ("UINT8_C", None),
                    Type::UInt16 => ("UINT16_C", None),
                    Type::UInt32 => ("UINT32_C", None),
                    Type::UInt64 => ("UINT64_C", None),
                    _ => unreachable!("integer atoms have integer types"),
                };
                if minimum == Some(*value) {
                    format!("{}_MIN", &constant[..constant.len() - 2])
                } else {
                    format!("{constant}({value})")
                }
            }
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

fn integer_info(ty: &Type) -> (&'static str, &'static str, &'static str, bool) {
    match ty {
        Type::Int8 => ("i8", "uint8_t", "uint32_t", true),
        Type::Int16 => ("i16", "uint16_t", "uint32_t", true),
        Type::Int32 => ("i32", "uint32_t", "uint32_t", true),
        Type::Int64 => ("i64", "uint64_t", "uint64_t", true),
        Type::UInt8 => ("u8", "uint8_t", "uint32_t", false),
        Type::UInt16 => ("u16", "uint16_t", "uint32_t", false),
        Type::UInt32 => ("u32", "uint32_t", "uint32_t", false),
        Type::UInt64 => ("u64", "uint64_t", "uint64_t", false),
        _ => unreachable!("called only for integer types"),
    }
}

fn integer_mask(ty: &Type) -> u16 {
    match ty {
        Type::Int8 => 1 << 0,
        Type::Int16 => 1 << 1,
        Type::Int32 => 1 << 2,
        Type::Int64 => 1 << 3,
        Type::UInt8 => 1 << 4,
        Type::UInt16 => 1 << 5,
        Type::UInt32 => 1 << 6,
        Type::UInt64 => 1 << 7,
        _ => unreachable!("called only for integer types"),
    }
}
