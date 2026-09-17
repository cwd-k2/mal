use crate::check::ast::Type;
use crate::control::ast::Operation;
use crate::core::ast::UnaryPrimitive;

use super::scalar::{arithmetic_instruction, scalar_type};
use super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(super) fn emit_operation(
        &mut self,
        operation: &Operation,
        result_type: Option<&Type>,
        symbol_concat: super::super::optimization::SymbolConcatMode,
    ) -> Option<Option<EmittedValue>> {
        match operation {
            Operation::Atom(atom) => self.atom(atom).map(Some),
            Operation::MakeClosure { function, captures } => {
                let result_type = result_type?.clone();
                let Type::Function { .. } = &result_type else {
                    return None;
                };
                let closure_type = self.types.value(&result_type)?;
                let with_code = self.register();
                self.line(format!(
                    "  {with_code} = insertvalue {} zeroinitializer, ptr @{}, 0",
                    closure_type.llvm,
                    super::function_name(*function)?
                ));
                let target = *self.index.lowered_functions.get(function)?;
                if captures.len() != target.environment.len()
                    || captures
                        .iter()
                        .zip(&target.environment)
                        .any(|(capture, field)| capture.ty != field.ty)
                {
                    return None;
                }
                let closure = if captures.is_empty() {
                    with_code
                } else {
                    let environment_type = Type::Product(
                        target
                            .environment
                            .iter()
                            .map(|field| field.ty.clone())
                            .collect(),
                    );
                    let environment_value = self.emit_product(captures, &environment_type)?;
                    let environment_layout = self.types.value(&environment_type)?;
                    let environment = self.register();
                    self.line(format!(
                        "  {environment} = call ptr @mal_runtime_environment_allocate(ptr %mal_context, {} {}, ptr @mal_destroy_environment_{})",
                        self.types.pointer_integer()?,
                        environment_layout.size,
                        super::function_number(*function)?
                    ));
                    self.line(format!(
                        "  store {} {}, ptr {environment}, align {}",
                        environment_layout.llvm,
                        environment_value.representation,
                        environment_layout.alignment
                    ));
                    let closure = self.register();
                    self.line(format!(
                        "  {closure} = insertvalue {} {with_code}, ptr {environment}, 1",
                        closure_type.llvm
                    ));
                    closure
                };
                Some(Some(EmittedValue {
                    ty: result_type,
                    representation: closure,
                    owned: true,
                }))
            }
            Operation::PrimitiveUnary { operator, operand } => {
                let operand = self.atom(operand)?;
                let scalar = scalar_type(&operand.ty, self.types.index_size())?;
                let register = self.register();
                let instruction = match operator {
                    UnaryPrimitive::Negate if scalar.floating => {
                        format!("fneg {} {}", scalar.llvm, operand.representation)
                    }
                    UnaryPrimitive::Negate => {
                        format!("sub {} 0, {}", scalar.llvm, operand.representation)
                    }
                    UnaryPrimitive::BitwiseNot if !scalar.floating => {
                        format!("xor {} {}, -1", scalar.llvm, operand.representation)
                    }
                    UnaryPrimitive::BitwiseNot => return None,
                };
                self.line(format!("  {register} = {instruction}"));
                Some(Some(EmittedValue {
                    ty: operand.ty,
                    representation: register,
                    owned: false,
                }))
            }
            Operation::PrimitiveBinary {
                operator,
                left,
                right,
            } => {
                if *operator == crate::core::ast::BinaryPrimitive::Add
                    && left.ty == Type::Symbol
                    && right.ty == Type::Symbol
                {
                    return self
                        .emit_symbol_concatenate(left, right, symbol_concat)
                        .map(Some);
                }
                let left = self.atom(left)?;
                let right = self.atom(right)?;
                if left.ty == Type::Address && right.ty == Type::ByteSize {
                    let offset = match operator {
                        crate::core::ast::BinaryPrimitive::Add => right.representation,
                        crate::core::ast::BinaryPrimitive::Subtract => {
                            let negated = self.register();
                            self.line(format!(
                                "  {negated} = sub {} 0, {}",
                                self.types.pointer_integer()?,
                                right.representation
                            ));
                            negated
                        }
                        _ => return None,
                    };
                    let register = self.register();
                    self.line(format!(
                        "  {register} = getelementptr i8, ptr {}, {} {offset}",
                        left.representation,
                        self.types.pointer_integer()?
                    ));
                    return Some(Some(EmittedValue {
                        ty: Type::Address,
                        representation: register,
                        owned: false,
                    }));
                }
                let quantity_product = *operator == crate::core::ast::BinaryPrimitive::Multiply
                    && matches!(
                        (&left.ty, &right.ty),
                        (Type::ByteSize, Type::USize) | (Type::USize, Type::ByteSize)
                    );
                if left.ty != right.ty && !quantity_product {
                    return None;
                }
                if result_type.is_some_and(super::types::is_bool) {
                    let register = self.register();
                    if left.ty == Type::Symbol {
                        let equality = self.register();
                        self.line(format!(
                            "  {equality} = call i8 @mal_runtime_symbol_equal(ptr {}, ptr {})",
                            left.representation, right.representation
                        ));
                        let predicate = match operator {
                            crate::core::ast::BinaryPrimitive::Equal => "ne",
                            crate::core::ast::BinaryPrimitive::NotEqual => "eq",
                            _ => return None,
                        };
                        self.line(format!("  {register} = icmp {predicate} i8 {equality}, 0"));
                    } else if super::types::is_bool(&left.ty) {
                        let predicate = match operator {
                            crate::core::ast::BinaryPrimitive::Equal => "eq",
                            crate::core::ast::BinaryPrimitive::NotEqual => "ne",
                            _ => return None,
                        };
                        self.line(format!(
                            "  {register} = icmp {predicate} i1 {}, {}",
                            left.representation, right.representation
                        ));
                    } else {
                        let scalar = scalar_type(&left.ty, self.types.index_size())?;
                        let predicate =
                            super::scalar::comparison_predicate(*operator)?.for_scalar(scalar);
                        let instruction = if scalar.floating { "fcmp" } else { "icmp" };
                        self.line(format!(
                            "  {register} = {instruction} {predicate} {} {}, {}",
                            scalar.llvm, left.representation, right.representation
                        ));
                    }
                    return Some(Some(EmittedValue {
                        ty: result_type?.clone(),
                        representation: register,
                        owned: false,
                    }));
                }
                let scalar = scalar_type(&left.ty, self.types.index_size())?;
                let instruction = arithmetic_instruction(*operator, scalar)?;
                let register = self.register();
                self.line(format!(
                    "  {register} = {instruction} {} {}, {}",
                    scalar.llvm, left.representation, right.representation
                ));
                Some(Some(EmittedValue {
                    ty: result_type.cloned().unwrap_or(left.ty),
                    representation: register,
                    owned: false,
                }))
            }
            Operation::NumericConversion { operand } => {
                let operand = self.atom(operand)?;
                let source = scalar_type(&operand.ty, self.types.index_size())?;
                let result_type = result_type?.clone();
                let target = scalar_type(&result_type, self.types.index_size())?;
                if source.floating == target.floating && source.bits == target.bits {
                    return Some(Some(EmittedValue {
                        ty: result_type,
                        representation: operand.representation,
                        owned: false,
                    }));
                }
                let instruction = if source.floating && target.floating {
                    if source.bits > target.bits {
                        "fptrunc"
                    } else {
                        "fpext"
                    }
                } else if source.floating {
                    if target.signed { "fptosi" } else { "fptoui" }
                } else if target.floating {
                    if source.signed { "sitofp" } else { "uitofp" }
                } else if source.bits > target.bits {
                    "trunc"
                } else if source.signed {
                    "sext"
                } else {
                    "zext"
                };
                let register = self.register();
                self.line(format!(
                    "  {register} = {instruction} {} {} to {}",
                    source.llvm, operand.representation, target.llvm
                ));
                Some(Some(EmittedValue {
                    ty: result_type,
                    representation: register,
                    owned: false,
                }))
            }
            Operation::SymbolLength { value } => self.emit_symbol_length(value).map(Some),
            Operation::SymbolAt { argument } => self.emit_symbol_at(argument).map(Some),
            Operation::ExternalCall { id, argument } => self
                .emit_external_call(*id, argument, result_type?)
                .map(Some),
            Operation::Memory {
                primitive,
                argument,
            } => self
                .emit_memory(*primitive, argument, result_type?)
                .map(Some),
            Operation::Product(elements) => self.emit_product(elements, result_type?).map(Some),
            Operation::SumInjection { index, value } => {
                self.emit_sum(*index, value, result_type?).map(Some)
            }
        }
    }
}
