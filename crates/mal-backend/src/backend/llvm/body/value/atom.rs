use crate::backend::llvm::syntax::emit_instruction;
use crate::closure::ast::{Atom, AtomKind, Reference};
use mal_frontend::check::ast::Type;

use super::super::scalar::{integer_literal, scalar_type};
use super::super::{EmittedValue, FunctionEmitter, ValueResponsibility};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn atom(&mut self, atom: &Atom) -> Option<EmittedValue> {
        match (&atom.ty, &atom.kind) {
            (ty, AtomKind::Integer(value))
                if scalar_type(ty, self.types.index_size()).is_some() =>
            {
                Some(EmittedValue {
                    ty: ty.clone(),
                    representation: integer_literal(ty, *value, self.types.index_size())?,
                    responsibility: ValueResponsibility::borrowed(ty),
                })
            }
            (Type::Float32, AtomKind::Float(bits)) => Some(EmittedValue {
                ty: Type::Float32,
                representation: format!(
                    "0x{:016X}",
                    (f32::from_bits(*bits as u32) as f64).to_bits()
                ),
                responsibility: ValueResponsibility::None,
            }),
            (Type::Float64, AtomKind::Float(bits)) => Some(EmittedValue {
                ty: Type::Float64,
                representation: format!("0x{bits:016X}"),
                responsibility: ValueResponsibility::None,
            }),
            (Type::Symbol, AtomKind::Symbol(bytes)) => {
                if bytes.is_empty() {
                    return self.make_byte_view(
                        &Type::Symbol,
                        "null",
                        "null",
                        "0",
                        ValueResponsibility::Owned,
                    );
                }
                let name = format!(
                    "mal_symbol_literal_{}_{}",
                    super::super::function_number(self.function.id),
                    atom.id.0
                );
                self.globals
                    .push(super::super::symbol::literal_definition(&name, bytes)?);
                let data = self.register();
                emit_instruction! {
                    self;
                    let { data.clone() } = get_element_ptr {
                        inbounds: false,
                        element_type: int(8_u16),
                        pointer: { format!("@{name}") },
                        indices: [(
                            int(64_u16),
                            { super::super::symbol::STATIC_OWNER_DATA_OFFSET.to_string() },
                        )],
                    };
                };
                self.make_byte_view(
                    &Type::Symbol,
                    &format!("@{name}"),
                    &data,
                    &bytes.len().to_string(),
                    ValueResponsibility::Owned,
                )
            }
            (Type::Function { .. }, AtomKind::Reference(Reference::SelfClosure(function))) => {
                let value_type = self.types.value(&atom.ty)?;
                let with_code = self.register();
                emit_instruction! {
                    self;
                    let { with_code.clone() } = insert_value {
                        aggregate: ({ value_type.llvm.clone() }, "zeroinitializer"),
                        element: (ptr, { format!("@{}", super::super::function_name(*function)) }),
                        indices: [0],
                    };
                };
                let environment = self.active_environment();
                let closure = self.register();
                emit_instruction! {
                    self;
                    let { closure.clone() } = insert_value {
                        aggregate: ({ value_type.llvm }, { with_code }),
                        element: (ptr, { environment }),
                        indices: [1],
                    };
                };
                Some(EmittedValue {
                    ty: atom.ty.clone(),
                    representation: closure,
                    responsibility: ValueResponsibility::borrowed(&atom.ty),
                })
            }
            (ty, AtomKind::Reference(Reference::Capture(index))) => {
                let pointer = self.capture_pointer(*index, ty)?;
                let value_type = self.types.value(ty)?;
                let value = self.register();
                emit_instruction! {
                    self;
                    let { value.clone() } = load {
                        ty: { value_type.llvm },
                        pointer: { pointer },
                        alignment: { value_type.alignment },
                        metadata: [],
                    };
                };
                Some(EmittedValue {
                    ty: ty.clone(),
                    representation: value,
                    responsibility: ValueResponsibility::borrowed(ty),
                })
            }
            (ty, AtomKind::Reference(Reference::Binding(id))) if !self.slots.contains_key(id) => {
                let constant = self.top_levels.get(*id)?.clone();
                if constant.ty != *ty {
                    return None;
                }
                self.constant(constant)
            }
            (ty, AtomKind::Reference(Reference::Binding(id))) if self.types.value(ty).is_some() => {
                let slot = self.slots.get(id)?.clone();
                if slot.ty != *ty {
                    return None;
                }
                let register = self.load_slot(&slot)?;
                Some(EmittedValue {
                    ty: ty.clone(),
                    representation: register,
                    responsibility: ValueResponsibility::borrowed(ty),
                })
            }
            (Type::Unit, AtomKind::Unit) => Some(EmittedValue {
                ty: Type::Unit,
                representation: "0".into(),
                responsibility: ValueResponsibility::None,
            }),
            _ => None,
        }
    }

    pub(in crate::backend::llvm::body) fn capture_pointer(
        &mut self,
        index: usize,
        ty: &Type,
    ) -> Option<String> {
        let function = *self.index.lowered_functions.get(&self.current_function)?;
        let captures = &function.captures;
        let field = captures.get(index)?;
        if field.ty != *ty {
            return None;
        }
        let environment_type =
            Type::Product(captures.iter().map(|field| field.ty.clone()).collect());
        let fields = self.types.product_fields(&environment_type)?;
        let offset = fields.get(index)?.offset;
        let environment = self.active_environment();
        let pointer = self.register();
        emit_instruction! {
            self;
            let { pointer.clone() } = get_element_ptr {
                inbounds: false,
                element_type: int(8_u16),
                pointer: { environment },
                indices: [(int(64_u16), { offset.to_string() })],
            };
        };
        Some(pointer)
    }

    fn constant(&mut self, constant: super::super::constants::Constant) -> Option<EmittedValue> {
        if let Some(value) = constant.value() {
            return Some(EmittedValue {
                ty: constant.ty.clone(),
                representation: value,
                responsibility: ValueResponsibility::borrowed(&constant.ty),
            });
        }
        if let Some(elements) = constant.product() {
            let Type::Product(types) = &constant.ty else {
                return None;
            };
            if elements.len() != types.len() {
                return None;
            }
            let aggregate_type = self.types.value(&constant.ty)?;
            let mut aggregate = "poison".to_owned();
            for (index, (element, expected)) in elements.iter().zip(types.iter()).enumerate() {
                let element = self.constant(element.clone())?;
                if element.ty != *expected {
                    return None;
                }
                let element_type = self.types.value(expected)?;
                let register = self.register();
                emit_instruction! {
                    self;
                    let { register.clone() } = insert_value {
                        aggregate: ({ aggregate_type.llvm.clone() }, { aggregate }),
                        element: ({ element_type.llvm }, { element.representation }),
                        indices: [{ index }],
                    };
                };
                aggregate = register;
            }
            let responsibility = ValueResponsibility::borrowed(&constant.ty);
            return Some(EmittedValue {
                ty: constant.ty,
                representation: aggregate,
                responsibility,
            });
        }
        let (index, value) = constant.sum()?;
        let value = self.constant(value.clone())?;
        self.emit_sum_value(index, value, &constant.ty, false)
    }
}
