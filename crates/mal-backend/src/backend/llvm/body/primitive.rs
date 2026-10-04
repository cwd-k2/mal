//! Primitive scalar operations: unary and binary operators, and numeric conversion.

use crate::backend::llvm::syntax::emit_instruction;
use crate::backend::llvm::syntax::{BinaryOperator, CastOperator, UnaryOperator};
use crate::closure::ast::Atom;
use crate::control::ast::StateId;
use crate::core::ast::{BinaryPrimitive, UnaryPrimitive};
use crate::execution::ownership::BindingOperand;
use mal_frontend::check::ast::Type;

use super::scalar::{arithmetic_instruction, scalar_type};
use super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(super) fn emit_primitive_unary(
        &mut self,
        site: StateId,
        binding: usize,
        operator: &UnaryPrimitive,
        operand: &Atom,
    ) -> Option<EmittedValue> {
        self.require_binding_borrow(site, binding, BindingOperand::UnaryOperand, operand)?;
        let operand = self.atom(operand)?;
        let scalar = scalar_type(&operand.ty, self.types.index_size())?;
        let register = self.register();
        match operator {
            UnaryPrimitive::Negate if scalar.floating => {
                emit_instruction! {
                    self;
                    let { register.clone() } = unary {
                        operator: { UnaryOperator::FNeg },
                        value: ({ scalar.llvm_type() }, { operand.representation }),
                    };
                };
            }
            UnaryPrimitive::Negate => {
                emit_instruction! {
                    self;
                    let { register.clone() } = binary {
                        operator: { BinaryOperator::Sub },
                        ty: { scalar.llvm_type() },
                        left: "0",
                        right: { operand.representation },
                    };
                };
            }
            UnaryPrimitive::BitwiseNot if !scalar.floating => {
                emit_instruction! {
                    self;
                    let { register.clone() } = binary {
                        operator: { BinaryOperator::Xor },
                        ty: { scalar.llvm_type() },
                        left: { operand.representation },
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

    pub(super) fn emit_primitive_binary(
        &mut self,
        site: StateId,
        binding: usize,
        operator: &BinaryPrimitive,
        left: &Atom,
        right: &Atom,
        result_type: Option<&Type>,
    ) -> Option<EmittedValue> {
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
            let scalar = scalar_type(&left.ty, self.types.index_size())?;
            let (kind, predicate) =
                super::scalar::comparison_predicate(*operator)?.for_scalar(scalar);
            emit_instruction! {
                self;
                let { register.clone() } = compare {
                    kind: { kind },
                    predicate: { predicate },
                    ty: { scalar.llvm_type() },
                    left: { left.representation },
                    right: { right.representation },
                };
            };
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
            let { register.clone() } = binary {
                operator: { instruction },
                ty: { scalar.llvm_type() },
                left: { left.representation },
                right: { right.representation },
            };
        };
        Some(EmittedValue {
            ty: result_type.cloned().unwrap_or(left.ty),
            representation: register,
            owned: false,
        })
    }

    pub(super) fn emit_numeric_conversion(
        &mut self,
        site: StateId,
        binding: usize,
        operand: &Atom,
        result_type: Option<&Type>,
    ) -> Option<EmittedValue> {
        self.require_binding_borrow(site, binding, BindingOperand::NumericOperand, operand)?;
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
            let { register.clone() } = cast {
                operator: { instruction },
                value: ({ source.llvm_type() }, { operand.representation }),
                to: { target.llvm_type() },
            };
        };
        Some(EmittedValue {
            ty: result_type,
            representation: register,
            owned: false,
        })
    }
}
