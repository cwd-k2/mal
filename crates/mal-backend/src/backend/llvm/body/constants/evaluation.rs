//! Evaluation of closed top-level operations and atoms into constants.

use super::*;

impl TopLevelConstants {
    pub(super) fn operation(
        &mut self,
        operation: &crate::closure::ast::Operation,
        result_type: &Type,
        values: &HashMap<ValueId, Constant>,
    ) -> Option<Constant> {
        use crate::closure::ast::Operation;

        let value = match operation {
            Operation::Atom(atom) => self.atom(atom, values)?,
            Operation::MakeClosure { function, captures } if captures.is_empty() => Constant {
                ty: result_type.clone(),
                kind: ConstantKind::Value(llvm_constant! {
                    structure([
                        typed(
                            (ptr),
                            atom(#{ format!("@{}", super::super::function_name(*function)) })
                        ),
                        typed((ptr), atom("null")),
                    ])
                }?),
            },
            Operation::NumericConversion { operand } => {
                let operand = self.atom(operand, values)?;
                numeric_conversion(operand, result_type, self.types.clone())?
            }
            Operation::Product(elements) => {
                let Type::Product(element_types) = result_type else {
                    return None;
                };
                if elements.len() != element_types.len() {
                    return None;
                }
                let elements = elements
                    .iter()
                    .zip(element_types.iter())
                    .map(|(atom, expected)| {
                        let value = self.atom(atom, values)?;
                        (value.ty == *expected).then_some(value)
                    })
                    .collect::<Option<Vec<_>>>()?;
                Constant {
                    ty: result_type.clone(),
                    kind: ConstantKind::Product(elements),
                }
            }
            Operation::SumInjection { index, value } => {
                let Type::Sum(members) = result_type else {
                    return None;
                };
                let member = members.get(*index)?;
                let value = self.atom(value, values)?;
                if value.ty != *member {
                    return None;
                }
                let kind = if super::super::types::is_bool(result_type) {
                    match index {
                        0 => ConstantKind::Value(llvm_constant!(atom("false"))?),
                        1 => ConstantKind::Value(llvm_constant!(atom("true"))?),
                        _ => return None,
                    }
                } else {
                    ConstantKind::Sum {
                        index: *index,
                        value: Box::new(value),
                    }
                };
                Constant {
                    ty: result_type.clone(),
                    kind,
                }
            }
            Operation::PrimitiveUnary { operator, operand } => {
                let operand = self.atom(operand, values)?;
                let scalar =
                    super::super::scalar::scalar_type(&operand.ty, self.types.index_size())?;
                let value = match operator {
                    crate::core::ast::UnaryPrimitive::Negate if scalar.floating => llvm_constant! {
                        unary {
                            operator: #{ UnaryOperator::FNeg },
                            operand: #{ operand.typed(self.types.clone())? },
                        }
                    }?,
                    crate::core::ast::UnaryPrimitive::Negate => {
                        let zero = llvm_typed_constant!(typed(#{ scalar.llvm_type() }, atom(0)))?;
                        llvm_constant! {
                            binary {
                                operator: #{ BinaryOperator::Sub },
                                left: #{ zero },
                                right: #{ operand.typed(self.types.clone())? },
                            }
                        }?
                    }
                    _ => return None,
                };
                Constant {
                    ty: operand.ty,
                    kind: ConstantKind::Value(value),
                }
            }
            Operation::PrimitiveBinary {
                operator,
                left,
                right,
            } => {
                let left = self.atom(left, values)?;
                let right = self.atom(right, values)?;
                if left.ty != right.ty {
                    return None;
                }
                let scalar = super::super::scalar::scalar_type(&left.ty, self.types.index_size())?;
                let instruction = super::super::scalar::arithmetic_instruction(*operator, scalar)?;
                let value = llvm_constant! {
                    binary {
                        operator: #{ instruction },
                        left: #{ left.typed(self.types.clone())? },
                        right: #{ right.typed(self.types.clone())? },
                    }
                }?;
                Constant {
                    ty: left.ty,
                    kind: ConstantKind::Value(value),
                }
            }
            _ => return None,
        };
        (value.ty == *result_type).then_some(value)
    }

    pub(super) fn atom(
        &mut self,
        atom: &crate::closure::ast::Atom,
        values: &HashMap<ValueId, Constant>,
    ) -> Option<Constant> {
        let value = match &atom.kind {
            AtomKind::Integer(value) => llvm_constant! {
                atom(#{ super::super::scalar::integer_literal(&atom.ty, *value, self.types.index_size())? })
            }?,
            AtomKind::Float(bits) if atom.ty == Type::Float32 => llvm_constant! {
                atom(#{ format!("0x{:016X}", (f32::from_bits(*bits as u32) as f64).to_bits()) })
            }?,
            AtomKind::Float(bits) if atom.ty == Type::Float64 => {
                llvm_constant!(atom(#{ format!("0x{bits:016X}") }))?
            }
            AtomKind::Symbol(bytes) if bytes.is_empty() => llvm_constant!(zero)?,
            AtomKind::Symbol(bytes) => {
                let name = format!("mal_top_symbol_{}", atom.id.0);
                self.globals
                    .push(super::super::symbol::literal_definition(&name, bytes)?);
                let address_constant = llvm_constant!(atom(#{ format!("@{name}") }))?;
                let address = || llvm_typed_constant!(typed((ptr), #{ address_constant.clone() }));
                llvm_constant! {
                    structure([
                        #{ address()? },
                        typed(
                            (ptr),
                            get_element_ptr {
                                element_type: (int(8_u16)),
                                pointer: #{ address()? },
                                indices: [typed(
                                    #{ self.types.index_llvm_type() },
                                    atom(#{ super::super::symbol::STATIC_OWNER_DATA_OFFSET })
                                )],
                            }
                        ),
                        typed(
                            #{ self.types.index_llvm_type() },
                            atom(#{ bytes.len() })
                        ),
                    ])
                }?
            }
            AtomKind::Unit if atom.ty == Type::Unit => llvm_constant!(atom(0))?,
            AtomKind::Reference(Reference::Binding(id)) => return values.get(id).cloned(),
            _ => return None,
        };
        Some(Constant {
            ty: atom.ty.clone(),
            kind: ConstantKind::Value(value),
        })
    }
}

fn numeric_conversion(operand: Constant, result_type: &Type, types: Types) -> Option<Constant> {
    let source = super::super::scalar::scalar_type(&operand.ty, types.index_size())?;
    let target = super::super::scalar::scalar_type(result_type, types.index_size())?;
    let representation = if source.floating == target.floating && source.bits == target.bits {
        operand.llvm()?.clone()
    } else if !source.floating && !target.floating {
        let value = operand.llvm()?.integer_value()?;
        let modulus = 1_i128 << target.bits;
        let residue = value.rem_euclid(modulus);
        if target.signed && residue >= modulus / 2 {
            llvm_constant!(atom(#{ residue - modulus }))?
        } else {
            llvm_constant!(atom(#{ residue }))?
        }
    } else {
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
        llvm_constant! {
            cast {
                operator: #{ instruction },
                operand: #{ operand.typed(types.clone())? },
                to: #{ target.llvm_type() },
            }
        }?
    };
    Some(Constant {
        ty: result_type.clone(),
        kind: ConstantKind::Value(representation),
    })
}
