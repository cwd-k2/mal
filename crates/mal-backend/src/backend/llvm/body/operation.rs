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
    /// Emits one control binding operation and returns its materialized result.
    ///
    /// `None` rejects an inconsistent execution plan, unsupported target representation, ownership
    /// mismatch, or nested syntax emission failure. Even `Unit` has an `EmittedValue`, so successful
    /// operations never need a second sentinel for an absent result.
    pub(super) fn emit_operation(
        &mut self,
        site: crate::control::ast::StateId,
        binding: usize,
        operation: &Operation,
        result_type: Option<&Type>,
        symbol_concat: super::super::optimization::SymbolConcatMode,
    ) -> Option<EmittedValue> {
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
                    Some(prepared.value)
                }),
            Operation::MakeClosure { function, captures } => {
                let result_type = result_type?.clone();
                let Type::Function { .. } = &result_type else {
                    return None;
                };
                let closure_type = self.types.value(&result_type)?;
                let with_code = if self.execution.optimizations.omits_code_pointer(*function) {
                    "zeroinitializer".into()
                } else {
                    let with_code = self.register();
                    emit_instruction! {
                        self;
                        let #{ with_code.clone() } = insert_value {
                            aggregate: typed(#{ closure_type.llvm.clone() }, "zeroinitializer"),
                            element: typed((ptr), #{ format!("@{}", super::function_name(*function)) }),
                            indices: [0],
                        };
                    };
                    with_code
                };
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
                    emit_instruction! {
                        self;
                        let #{ environment.clone() } = call {
                            tail: false,
                            result_type: (ptr),
                            callee: direct("mal_runtime_environment_allocate"),
                            arguments: [
                                typed((ptr), "%mal_context"),
                                typed(#{ self.types.index_llvm_type() }, #{ environment_layout.size.to_string() }),
                                typed((ptr), #{ format!(
                                    "@mal_destroy_environment_{}",
                                    super::function_number(*function)
                                ) }),
                            ],
                        };
                    };
                    emit_instruction! {
                        self;
                        store {
                            value: typed(
                                #{ environment_layout.llvm },
                                #{ environment_value.value.representation.as_str() },
                            ),
                            pointer: #{ environment.as_str() },
                            alignment: #{ environment_layout.alignment },
                            metadata: [],
                        };
                    };
                    self.commit_consumes(&environment_value)?;
                    let closure = self.register();
                    emit_instruction! {
                        self;
                        let #{ closure.clone() } = insert_value {
                            aggregate: typed(#{ closure_type.llvm }, #{ with_code }),
                            element: typed((ptr), #{ environment }),
                            indices: [1],
                        };
                    };
                    closure
                };
                Some(EmittedValue {
                    ty: result_type,
                    representation: closure,
                    owned: true,
                })
            }
            Operation::PrimitiveUnary { operator, operand } => {
                self.require_binding_borrow(site, binding, BindingOperand::UnaryOperand, operand)?;
                let operand = self.atom(operand)?;
                let scalar = scalar_type(&operand.ty, self.types.index_size())?;
                let register = self.register();
                match operator {
                    UnaryPrimitive::Negate if scalar.floating => {
                        emit_instruction! {
                            self;
                            let #{ register.clone() } = unary {
                                operator: #{ UnaryOperator::FNeg },
                                value: typed(#{ scalar.llvm_type() }, #{ operand.representation }),
                            };
                        };
                    }
                    UnaryPrimitive::Negate => {
                        emit_instruction! {
                            self;
                            let #{ register.clone() } = binary {
                                operator: #{ BinaryOperator::Sub },
                                ty: #{ scalar.llvm_type() },
                                left: "0",
                                right: #{ operand.representation },
                            };
                        };
                    }
                    UnaryPrimitive::BitwiseNot if !scalar.floating => {
                        emit_instruction! {
                            self;
                            let #{ register.clone() } = binary {
                                operator: #{ BinaryOperator::Xor },
                                ty: #{ scalar.llvm_type() },
                                left: #{ operand.representation },
                                right: "-1",
                            };
                        };
                    }
                    UnaryPrimitive::BitwiseNot => return None,
                }
                Some(EmittedValue {
                    ty: operand.ty,
                    representation: register,
                    owned: false,
                })
            }
            Operation::PrimitiveBinary {
                operator,
                left,
                right,
            } => {
                self.require_binding_borrow(site, binding, BindingOperand::BinaryLeft, left)?;
                self.require_binding_borrow(site, binding, BindingOperand::BinaryRight, right)?;
                let left = self.atom(left)?;
                let right = self.atom(right)?;
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
                    if super::types::is_bool(&left.ty) {
                        let predicate = match operator {
                            crate::core::ast::BinaryPrimitive::Equal => ComparisonPredicate::Eq,
                            crate::core::ast::BinaryPrimitive::NotEqual => ComparisonPredicate::Ne,
                            _ => return None,
                        };
                        emit_instruction! {
                            self;
                            let #{ register.clone() } = compare {
                                kind: #{ ComparisonKind::Integer },
                                predicate: #{ predicate },
                                ty: (int(1_u16)),
                                left: #{ left.representation },
                                right: #{ right.representation },
                            };
                        };
                    } else {
                        let scalar = scalar_type(&left.ty, self.types.index_size())?;
                        let (kind, predicate) =
                            super::scalar::comparison_predicate(*operator)?.for_scalar(scalar);
                        emit_instruction! {
                            self;
                            let #{ register.clone() } = compare {
                                kind: #{ kind },
                                predicate: #{ predicate },
                                ty: #{ scalar.llvm_type() },
                                left: #{ left.representation },
                                right: #{ right.representation },
                            };
                        };
                    }
                    return Some(EmittedValue {
                        ty: result_type?.clone(),
                        representation: register,
                        owned: false,
                    });
                }
                let scalar = scalar_type(&left.ty, self.types.index_size())?;
                let instruction = arithmetic_instruction(*operator, scalar)?;
                let register = self.register();
                emit_instruction! {
                    self;
                    let #{ register.clone() } = binary {
                        operator: #{ instruction },
                        ty: #{ scalar.llvm_type() },
                        left: #{ left.representation },
                        right: #{ right.representation },
                    };
                };
                Some(EmittedValue {
                    ty: result_type.cloned().unwrap_or(left.ty),
                    representation: register,
                    owned: false,
                })
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
                    return Some(EmittedValue {
                        ty: result_type,
                        representation: operand.representation,
                        owned: false,
                    });
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
                emit_instruction! {
                    self;
                    let #{ register.clone() } = cast {
                        operator: #{ instruction },
                        value: typed(#{ source.llvm_type() }, #{ operand.representation }),
                        to: #{ target.llvm_type() },
                    };
                };
                Some(EmittedValue {
                    ty: result_type,
                    representation: register,
                    owned: false,
                })
            }
            Operation::Symbol {
                primitive,
                operands,
            } => {
                for (index, operand) in operands.iter().enumerate() {
                    self.require_binding_borrow(
                        site,
                        binding,
                        BindingOperand::SymbolOperand(index),
                        operand,
                    )?;
                }
                use mal_frontend::check::ast::SymbolPrimitive;
                match (primitive, operands.as_slice()) {
                    (SymbolPrimitive::Length, [value]) => self.emit_symbol_length(value),
                    (SymbolPrimitive::ByteAt, [symbol, index]) => {
                        self.emit_symbol_at(symbol, index)
                    }
                    (SymbolPrimitive::Concatenate, [left, right]) => {
                        self.emit_symbol_concatenate(left, right, symbol_concat)
                    }
                    (SymbolPrimitive::Prefix | SymbolPrimitive::Suffix, [symbol, index]) => {
                        self.emit_symbol_partition(symbol, index, *primitive)
                    }
                    (SymbolPrimitive::Equal | SymbolPrimitive::NotEqual, [left, right]) => {
                        self.emit_symbol_equality(left, right, *primitive)
                    }
                    _ => None,
                }
            }
            Operation::ExternalCall { id, argument } => {
                self.require_binding_borrow(
                    site,
                    binding,
                    BindingOperand::ExternalArgument,
                    argument,
                )?;
                self.emit_external_call(*id, argument, result_type?)
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
                Some(result)
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
                Some(prepared.value)
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
                Some(prepared.value)
            }
        }
    }
}
