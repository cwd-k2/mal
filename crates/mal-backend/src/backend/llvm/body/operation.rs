use crate::backend::llvm::syntax::{
    BinaryOperator, CastOperator, ComparisonKind, ComparisonPredicate, UnaryOperator,
};
use crate::control::ast::Operation;
use crate::core::ast::UnaryPrimitive;
use mal_frontend::check::ast::Type;

use super::scalar::{arithmetic_instruction, scalar_type};
use super::{EmittedValue, FunctionEmitter};
use crate::execution::ownership::BindingOperand;

impl FunctionEmitter<'_> {
    pub(super) fn emit_operation(
        &mut self,
        site: crate::control::ast::StateId,
        binding: usize,
        operation: &Operation,
        result_type: Option<&Type>,
        symbol_concat: super::super::optimization::SymbolConcatMode,
    ) -> Option<Option<EmittedValue>> {
        match operation {
            Operation::Atom(atom) => self
                .prepare_atom_for_use(
                    atom,
                    self.ownership.binding_operand_use(
                        site,
                        binding,
                        BindingOperand::Atom,
                        atom,
                    )?,
                )
                .and_then(|prepared| {
                    self.commit_consumes(&prepared)?;
                    Some(Some(prepared.value))
                }),
            Operation::MakeClosure { function, captures } => {
                let result_type = result_type?.clone();
                let Type::Function { .. } = &result_type else {
                    return None;
                };
                let closure_type = self.types.value(&result_type)?;
                let with_code = self.register();
                emit_instruction!(
                    self;
                    insert_value { with_code.clone() };
                    { closure_type.llvm.clone() } => "zeroinitializer",
                    (ptr) => { format!("@{}", super::function_name(*function)) },
                    [0]
                );
                let target = *self.index.lowered_functions.get(function)?;
                let environment = &target.captures;
                if captures.len() != environment.len()
                    || captures
                        .iter()
                        .zip(environment)
                        .any(|(capture, field)| capture.ty != field.ty)
                {
                    return None;
                }
                let closure = if captures.is_empty() {
                    with_code
                } else {
                    let environment_type =
                        Type::Product(environment.iter().map(|field| field.ty.clone()).collect());
                    let effects = captures
                        .iter()
                        .enumerate()
                        .map(|(index, atom)| {
                            self.ownership.binding_operand_use(
                                site,
                                binding,
                                BindingOperand::Capture(index),
                                atom,
                            )
                        })
                        .collect::<Option<Vec<_>>>()?;
                    let environment_value =
                        self.emit_product(captures, &environment_type, &effects)?;
                    let environment_layout = self.types.value(&environment_type)?;
                    let environment = self.register();
                    emit_instruction!(
                        self;
                        call { Some(environment.clone()) },
                        false,
                        (ptr),
                        direct "mal_runtime_environment_allocate";
                        [
                            (typed (ptr) => "%mal_context"),
                            (typed { self.types.index_llvm_type() } => { environment_layout.size.to_string() }),
                            (typed (ptr) => { format!(
                                                            "@mal_destroy_environment_{}",
                                                            super::function_number(*function)
                                                        ) }),
                        ]
                    );
                    emit_instruction!(
                        self;
                        store { environment_layout.llvm },
                        { environment_value.value.representation.as_str() },
                        { environment.as_str() },
                        { environment_layout.alignment },
                        []
                    );
                    self.commit_consumes(&environment_value)?;
                    let closure = self.register();
                    emit_instruction!(
                        self;
                        insert_value { closure.clone() };
                        { closure_type.llvm } => { with_code },
                        (ptr) => { environment },
                        [1]
                    );
                    closure
                };
                Some(Some(EmittedValue {
                    ty: result_type,
                    representation: closure,
                    owned: true,
                }))
            }
            Operation::PrimitiveUnary { operator, operand } => {
                self.require_binding_borrow(site, binding, BindingOperand::UnaryOperand, operand)?;
                let operand = self.atom(operand)?;
                let scalar = scalar_type(&operand.ty, self.types.index_size())?;
                let register = self.register();
                match operator {
                    UnaryPrimitive::Negate if scalar.floating => {
                        emit_instruction!(
                            self;
                            unary { register.clone() },
                            { UnaryOperator::FNeg };
                            { scalar.llvm_type() } => { operand.representation }
                        );
                    }
                    UnaryPrimitive::Negate => {
                        emit_instruction!(
                            self;
                            binary { register.clone() },
                            { BinaryOperator::Sub },
                            { scalar.llvm_type() },
                            "0",
                            { operand.representation }
                        );
                    }
                    UnaryPrimitive::BitwiseNot if !scalar.floating => {
                        emit_instruction!(
                            self;
                            binary { register.clone() },
                            { BinaryOperator::Xor },
                            { scalar.llvm_type() },
                            { operand.representation },
                            "-1"
                        );
                    }
                    UnaryPrimitive::BitwiseNot => return None,
                }
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
                    self.require_binding_borrow(site, binding, BindingOperand::BinaryLeft, left)?;
                    self.require_binding_borrow(site, binding, BindingOperand::BinaryRight, right)?;
                    return self
                        .emit_symbol_concatenate(left, right, symbol_concat)
                        .map(Some);
                }
                self.require_binding_borrow(site, binding, BindingOperand::BinaryLeft, left)?;
                self.require_binding_borrow(site, binding, BindingOperand::BinaryRight, right)?;
                let left = self.atom(left)?;
                let right = self.atom(right)?;
                if left.ty == Type::Address && right.ty == Type::ByteSize {
                    let offset = match operator {
                        crate::core::ast::BinaryPrimitive::Add => right.representation,
                        crate::core::ast::BinaryPrimitive::Subtract => {
                            let negated = self.register();
                            emit_instruction!(
                                self;
                                binary { negated.clone() },
                                { BinaryOperator::Sub },
                                { self.types.index_llvm_type() },
                                "0",
                                { right.representation }
                            );
                            negated
                        }
                        _ => return None,
                    };
                    let register = self.register();
                    emit_instruction!(
                        self;
                        get_element_ptr { register.clone() },
                        false,
                        (int(8_u16)),
                        { left.representation };
                        [
                            (typed { self.types.index_llvm_type() } => { offset }),
                        ]
                    );
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
                        let left = self.byte_view_fields(&left)?;
                        let right = self.byte_view_fields(&right)?;
                        let equality = self.register();
                        emit_instruction!(
                            self;
                            call { Some(equality.clone()) },
                            false,
                            (int(8_u16)),
                            direct "mal_runtime_symbol_equal";
                            [
                                (typed (ptr) => { left.data }),
                                (typed { self.types.index_llvm_type() } => { left.count }),
                                (typed (ptr) => { right.data }),
                                (typed { self.types.index_llvm_type() } => { right.count }),
                            ]
                        );
                        let predicate = match operator {
                            crate::core::ast::BinaryPrimitive::Equal => ComparisonPredicate::Ne,
                            crate::core::ast::BinaryPrimitive::NotEqual => ComparisonPredicate::Eq,
                            _ => return None,
                        };
                        emit_instruction!(
                            self;
                            compare { register.clone() },
                            { ComparisonKind::Integer },
                            { predicate },
                            (int(8_u16)),
                            { equality },
                            "0"
                        );
                    } else if super::types::is_bool(&left.ty) {
                        let predicate = match operator {
                            crate::core::ast::BinaryPrimitive::Equal => ComparisonPredicate::Eq,
                            crate::core::ast::BinaryPrimitive::NotEqual => ComparisonPredicate::Ne,
                            _ => return None,
                        };
                        emit_instruction!(
                            self;
                            compare { register.clone() },
                            { ComparisonKind::Integer },
                            { predicate },
                            (int(1_u16)),
                            { left.representation },
                            { right.representation }
                        );
                    } else {
                        let scalar = scalar_type(&left.ty, self.types.index_size())?;
                        let (kind, predicate) =
                            super::scalar::comparison_predicate(*operator)?.for_scalar(scalar);
                        emit_instruction!(
                            self;
                            compare { register.clone() },
                            { kind },
                            { predicate },
                            { scalar.llvm_type() },
                            { left.representation },
                            { right.representation }
                        );
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
                emit_instruction!(
                    self;
                    binary { register.clone() },
                    { instruction },
                    { scalar.llvm_type() },
                    { left.representation },
                    { right.representation }
                );
                Some(Some(EmittedValue {
                    ty: result_type.cloned().unwrap_or(left.ty),
                    representation: register,
                    owned: false,
                }))
            }
            Operation::NumericConversion { operand } => {
                self.require_binding_borrow(
                    site,
                    binding,
                    BindingOperand::NumericOperand,
                    operand,
                )?;
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
                        CastOperator::FPTrunc
                    } else {
                        CastOperator::FPExt
                    }
                } else if source.floating {
                    if target.signed {
                        CastOperator::FPToSI
                    } else {
                        CastOperator::FPToUI
                    }
                } else if target.floating {
                    if source.signed {
                        CastOperator::SIToFP
                    } else {
                        CastOperator::UIToFP
                    }
                } else if source.bits > target.bits {
                    CastOperator::Trunc
                } else if source.signed {
                    CastOperator::SExt
                } else {
                    CastOperator::ZExt
                };
                let register = self.register();
                emit_instruction!(
                    self;
                    cast { register.clone() },
                    { instruction };
                    { source.llvm_type() } => { operand.representation },
                    { target.llvm_type() }
                );
                Some(Some(EmittedValue {
                    ty: result_type,
                    representation: register,
                    owned: false,
                }))
            }
            Operation::SymbolLength { value } => {
                self.require_binding_borrow(site, binding, BindingOperand::SymbolLength, value)?;
                self.emit_symbol_length(value).map(Some)
            }
            Operation::SymbolAt { argument } => {
                self.require_binding_borrow(site, binding, BindingOperand::SymbolAt, argument)?;
                self.emit_symbol_at(argument).map(Some)
            }
            Operation::ExternalCall { id, argument } => {
                self.require_binding_borrow(
                    site,
                    binding,
                    BindingOperand::ExternalArgument,
                    argument,
                )?;
                self.emit_external_call(*id, argument, result_type?)
                    .map(Some)
            }
            Operation::Memory {
                primitive,
                operands,
            } => {
                let effects = operands
                    .iter()
                    .enumerate()
                    .map(|(index, operand)| {
                        self.ownership.binding_operand_use(
                            site,
                            binding,
                            BindingOperand::MemoryOperand(index),
                            operand,
                        )
                    })
                    .collect::<Option<Vec<_>>>()?;
                let prepared = if let [operand] = operands.as_slice() {
                    self.prepare_atom_for_use(operand, effects[0])?
                } else {
                    let argument_type =
                        Type::Product(operands.iter().map(|operand| operand.ty.clone()).collect());
                    self.emit_product(operands, &argument_type, &effects)?
                };
                let result = self.emit_memory(*primitive, &prepared.value, result_type?)?;
                self.commit_consumes(&prepared)?;
                Some(Some(result))
            }
            Operation::Buffer {
                operation,
                element,
                operands,
            } => {
                let operands = operands
                    .iter()
                    .enumerate()
                    .map(|(index, operand)| {
                        self.require_binding_borrow(
                            site,
                            binding,
                            BindingOperand::BufferOperand(index),
                            operand,
                        )?;
                        self.atom(operand)
                    })
                    .collect::<Option<Vec<_>>>()?;
                self.emit_buffer(*operation, element, &operands, result_type?)
                    .map(Some)
            }
            Operation::Product(elements) => {
                let effects = elements
                    .iter()
                    .enumerate()
                    .map(|(index, atom)| {
                        self.ownership.binding_operand_use(
                            site,
                            binding,
                            BindingOperand::ProductElement(index),
                            atom,
                        )
                    })
                    .collect::<Option<Vec<_>>>()?;
                let prepared = self.emit_product(elements, result_type?, &effects)?;
                self.commit_consumes(&prepared)?;
                Some(Some(prepared.value))
            }
            Operation::SumInjection { index, value } => {
                let effect = self.ownership.binding_operand_use(
                    site,
                    binding,
                    BindingOperand::SumValue,
                    value,
                )?;
                let prepared = self.emit_sum(*index, value, result_type?, effect)?;
                self.commit_consumes(&prepared)?;
                Some(Some(prepared.value))
            }
        }
    }
}
