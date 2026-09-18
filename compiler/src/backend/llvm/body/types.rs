use std::{collections::HashMap, sync::Arc};

use crate::check::ast::{SharedTypeId, Type};

use super::scalar::scalar_type;
use crate::backend::llvm::TargetLayout;

#[derive(Clone)]
pub(in crate::backend::llvm) struct ValueType {
    pub(in crate::backend::llvm) llvm: String,
    pub(in crate::backend::llvm) alignment: usize,
    pub(in crate::backend::llvm) size: usize,
}

#[derive(Clone, Copy)]
pub(in crate::backend::llvm) struct Field {
    pub(in crate::backend::llvm) offset: usize,
}

#[derive(Clone)]
pub(in crate::backend::llvm) struct Types {
    target: TargetLayout,
    compact_functions: Arc<[(Type, crate::closure::ast::FunctionId)]>,
}

impl Types {
    #[cfg(test)]
    pub(in crate::backend::llvm) fn new(pointer_size: usize) -> Option<Self> {
        Self::for_target(TargetLayout::natural(pointer_size, pointer_size)?)
    }

    pub(in crate::backend::llvm) fn for_target(target: TargetLayout) -> Option<Self> {
        TargetLayout::natural(target.pointer_size, target.index_size)?;
        Some(Self {
            target,
            compact_functions: Arc::new([]),
        })
    }

    pub(in crate::backend::llvm) fn for_program(
        target: TargetLayout,
        functions: &[crate::closure::ast::Function],
    ) -> Option<Self> {
        let mut types = Self::for_target(target)?;
        let mut candidates = Vec::new();
        for function in functions {
            let ty = Type::Function {
                parameter: function.parameter.ty.clone().into(),
                result: function.body.result.ty.clone().into(),
            };
            if !candidates.iter().any(|(candidate, _)| candidate == &ty) {
                candidates.push((ty, function.id));
            }
        }
        candidates.retain(|(ty, target)| {
            let Type::Function { parameter, result } = ty else {
                return false;
            };
            let mut matching = functions.iter().filter(|function| {
                function.parameter.ty == **parameter && function.body.result.ty == **result
            });
            matching.next().is_some_and(|function| {
                function.id == *target
                    && function.kind.has_scoped_environment()
                    && matching.next().is_none()
            })
        });
        types.compact_functions = candidates.into();
        Some(types)
    }

    pub(in crate::backend::llvm) fn function_is_compact(&self, ty: &Type) -> bool {
        self.compact_function(ty).is_some()
    }

    pub(in crate::backend::llvm) fn compact_function(
        &self,
        ty: &Type,
    ) -> Option<crate::closure::ast::FunctionId> {
        self.compact_functions
            .iter()
            .find_map(|(candidate, target)| (candidate == ty).then_some(*target))
    }

    pub(in crate::backend::llvm) fn value(&self, ty: &Type) -> Option<ValueType> {
        self.value_cached(ty, &mut HashMap::new())
    }

    fn value_cached(
        &self,
        ty: &Type,
        cache: &mut HashMap<SharedTypeId, ValueType>,
    ) -> Option<ValueType> {
        if let Some(value) = ty.shared_id().and_then(|id| cache.get(&id)) {
            return Some(value.clone());
        }
        if let Some(scalar) = scalar_type(ty, self.target.index_size) {
            return Some(ValueType {
                llvm: scalar.llvm.into(),
                alignment: self.target.scalar_alignment(scalar.bits, scalar.floating)?,
                size: usize::from(scalar.bits) / 8,
            });
        }
        let value = match ty {
            Type::Unit => Some(ValueType {
                llvm: "i8".into(),
                alignment: 1,
                size: 1,
            }),
            Type::Address | Type::Cursor(_) => Some(ValueType {
                llvm: "ptr".into(),
                alignment: self.target.pointer_alignment,
                size: self.target.pointer_size,
            }),
            Type::Symbol => self.byte_view(),
            Type::External { .. } => Some(ValueType {
                llvm: format!("i{}", self.target.pointer_size.checked_mul(8)?),
                alignment: self.target.pointer_alignment,
                size: self.target.pointer_size,
            }),
            Type::Function { .. } if self.function_is_compact(ty) => Some(ValueType {
                llvm: "ptr".into(),
                alignment: self.target.pointer_alignment,
                size: self.target.pointer_size,
            }),
            Type::Function { .. } => aggregate_type(vec![
                ValueType {
                    llvm: "ptr".into(),
                    alignment: self.target.pointer_alignment,
                    size: self.target.pointer_size,
                },
                ValueType {
                    llvm: "ptr".into(),
                    alignment: self.target.pointer_alignment,
                    size: self.target.pointer_size,
                },
            ]),
            Type::Region(_) => aggregate_type(vec![
                ValueType {
                    llvm: "ptr".into(),
                    alignment: self.target.pointer_alignment,
                    size: self.target.pointer_size,
                },
                ValueType {
                    llvm: self.pointer_integer()?,
                    alignment: self.index_alignment(),
                    size: self.target.index_size,
                },
            ]),
            Type::Packed(_) => self.byte_view(),
            Type::Product(elements) => self.product(elements, cache),
            Type::Sum(_) if is_bool(ty) => Some(ValueType {
                llvm: "i1".into(),
                alignment: 1,
                size: 1,
            }),
            Type::Sum(elements) => self.sum(elements, cache),
            _ => None,
        }?;
        if let Some(id) = ty.shared_id() {
            cache.insert(id, value.clone());
        }
        Some(value)
    }

    fn byte_view(&self) -> Option<ValueType> {
        aggregate_type(vec![
            ValueType {
                llvm: "ptr".into(),
                alignment: self.target.pointer_alignment,
                size: self.target.pointer_size,
            },
            ValueType {
                llvm: self.pointer_integer()?,
                alignment: self.index_alignment(),
                size: self.target.index_size,
            },
            ValueType {
                llvm: self.pointer_integer()?,
                alignment: self.index_alignment(),
                size: self.target.index_size,
            },
        ])
    }

    pub(in crate::backend::llvm) fn pointer_integer(&self) -> Option<String> {
        Some(format!("i{}", self.target.index_size.checked_mul(8)?))
    }

    pub(in crate::backend::llvm) fn pointer_representation_integer(&self) -> Option<String> {
        Some(format!("i{}", self.target.pointer_size.checked_mul(8)?))
    }

    pub(in crate::backend::llvm) fn pointer_size(&self) -> usize {
        self.target.pointer_size
    }

    pub(in crate::backend::llvm) fn index_size(&self) -> usize {
        self.target.index_size
    }

    pub(in crate::backend::llvm) fn pointer_alignment(&self) -> usize {
        self.target.pointer_alignment
    }

    pub(in crate::backend::llvm) fn index_alignment(&self) -> usize {
        self.target
            .scalar_alignment((self.target.index_size * 8) as u8, false)
            .expect("supported index width has an integer ABI alignment")
    }

    pub(in crate::backend::llvm) fn maximum_value_alignment(&self) -> usize {
        self.target
            .integer_alignments
            .into_iter()
            .chain(self.target.float_alignments)
            .chain([self.target.pointer_alignment])
            .max()
            .expect("target layout has scalar and pointer alignments")
    }

    fn product(
        &self,
        elements: &[Type],
        cache: &mut HashMap<SharedTypeId, ValueType>,
    ) -> Option<ValueType> {
        aggregate_type(self.fields(elements, cache)?)
    }

    fn sum(
        &self,
        elements: &[Type],
        cache: &mut HashMap<SharedTypeId, ValueType>,
    ) -> Option<ValueType> {
        let mut fields = vec![ValueType {
            llvm: "i32".into(),
            alignment: 4,
            size: 4,
        }];
        if let Some(payload) = self.sum_payload(elements, cache)? {
            fields.push(payload);
        }
        aggregate_type(fields)
    }

    pub(in crate::backend::llvm) fn product_fields(&self, ty: &Type) -> Option<Vec<Field>> {
        let Type::Product(elements) = ty else {
            return None;
        };
        field_layouts(&self.fields(elements, &mut HashMap::new())?)
    }

    pub(in crate::backend::llvm) fn sum_fields(&self, ty: &Type) -> Option<Vec<Field>> {
        let Type::Sum(elements) = ty else {
            return None;
        };
        let tag = ValueType {
            llvm: "i32".into(),
            alignment: 4,
            size: 4,
        };
        let Some(payload) = self.sum_payload(elements, &mut HashMap::new())? else {
            return field_layouts(&[tag]);
        };
        let layouts = field_layouts(&[tag, payload])?;
        let payload_offset = layouts.get(1)?.offset;
        let mut variants = vec![layouts[0]];
        variants.extend((0..elements.len()).map(|_| Field {
            offset: payload_offset,
        }));
        Some(variants)
    }

    fn fields(
        &self,
        elements: &[Type],
        cache: &mut HashMap<SharedTypeId, ValueType>,
    ) -> Option<Vec<ValueType>> {
        elements
            .iter()
            .map(|element| self.value_cached(element, cache))
            .collect()
    }

    fn sum_payload(
        &self,
        elements: &[Type],
        cache: &mut HashMap<SharedTypeId, ValueType>,
    ) -> Option<Option<ValueType>> {
        let size = elements
            .iter()
            .map(|element| self.value_cached(element, cache).map(|value| value.size))
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .max();
        Some(size.map(|size| ValueType {
            llvm: format!("[{size} x i8]"),
            alignment: 1,
            size,
        }))
    }
}

fn aggregate_type(fields: Vec<ValueType>) -> Option<ValueType> {
    let alignment = fields
        .iter()
        .map(|field| field.alignment)
        .max()
        .unwrap_or(1);
    let mut size = 0usize;
    for field in &fields {
        size = align(size, field.alignment)?;
        size = size.checked_add(field.size)?;
    }
    Some(ValueType {
        llvm: format!(
            "{{ {} }}",
            fields
                .iter()
                .map(|field| field.llvm.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        alignment,
        size: align(size, alignment)?,
    })
}

fn field_layouts(fields: &[ValueType]) -> Option<Vec<Field>> {
    let mut offset = 0usize;
    fields
        .iter()
        .map(|value_type| {
            offset = align(offset, value_type.alignment)?;
            let field = Field { offset };
            offset = offset.checked_add(value_type.size)?;
            Some(field)
        })
        .collect()
}

pub(in crate::backend::llvm) fn is_bool(ty: &Type) -> bool {
    matches!(ty, Type::Sum(elements) if elements.as_ref() == [Type::Unit, Type::Unit])
}

pub(super) fn align(value: usize, alignment: usize) -> Option<usize> {
    value
        .checked_add(alignment.checked_sub(1)?)
        .map(|value| value & !(alignment - 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{FileId, SourceFile};

    fn lowered(source: &str) -> crate::closure::ast::Program {
        let source = SourceFile::new(FileId::new(102), "compact-functions.mal", source.into());
        let checked = crate::pipeline::check(&source).expect("check compact function fixture");
        let specialized = crate::check::specialize(checked).expect("specialize compact fixture");
        crate::closure::convert(&crate::anf::lower(&crate::core::lower(&specialized)))
    }

    fn reader_type() -> Type {
        Type::Function {
            parameter: Type::USize.into(),
            result: Type::Int64.into(),
        }
    }

    #[test]
    fn sums_use_one_maximum_sized_payload_region() {
        let types = Types::new(8).unwrap();
        let ty = Type::Sum(vec![Type::UInt8, Type::UInt64].into());

        let value = types.value(&ty).unwrap();
        let fields = types.sum_fields(&ty).unwrap();

        assert_eq!(value.llvm, "{ i32, [8 x i8] }");
        assert_eq!(value.size, 12);
        assert_eq!(
            fields.iter().map(|field| field.offset).collect::<Vec<_>>(),
            [0, 4, 4]
        );
    }

    #[test]
    fn lays_out_shared_sum_dags_once_per_node() {
        let mut ty = Type::Unit;
        for _ in 0..64 {
            ty = Type::Sum(vec![ty.clone(), ty].into());
        }

        let value = Types::new(8).unwrap().value(&ty).unwrap();

        assert_eq!(value.size, 256);
        assert!(value.llvm.len() < 1_500);
    }

    #[test]
    fn uses_target_abi_alignment_for_runtime_scalars_and_pointers() {
        let target = crate::backend::llvm::target_layout("e-p:64:32-i16:32-i64:32")
            .expect("synthetic target layout");
        let types = Types::for_target(target).unwrap();

        assert_eq!(types.value(&Type::UInt16).unwrap().alignment, 4);
        assert_eq!(types.value(&Type::UInt64).unwrap().alignment, 4);
        assert_eq!(types.value(&Type::Address).unwrap().alignment, 4);
        assert_eq!(types.value(&Type::Address).unwrap().size, 8);
    }

    #[test]
    fn compacts_a_function_type_only_for_one_scoped_inhabitant() {
        let scoped = lowered(
            "fill :: ((Int64 -> USize), (USize -> Int64), ((USize, Int64) -> Unit)) -> Unit := (_, get, _) -> { _ := get(0usize); (); };\n\
             main :: Unit -> Int32 := () -> { _ := pack<Int64>(fill); 0i32; };",
        );
        let target = crate::backend::llvm::target_layout("e-p:64:64").unwrap();
        let types = Types::for_program(target, &scoped.functions).unwrap();
        assert_eq!(types.value(&reader_type()).unwrap().llvm, "ptr");

        let mixed = lowered(
            "read :: USize -> Int64 := (_) -> { 0i64 };\n\
             fill :: ((Int64 -> USize), (USize -> Int64), ((USize, Int64) -> Unit)) -> Unit := (_, get, _) -> { _ := get(0usize); (); };\n\
             main :: Unit -> Int32 := () -> { _ := pack<Int64>(fill); _ := read(0usize); 0i32; };",
        );
        let types = Types::for_program(target, &mixed.functions).unwrap();
        assert_eq!(types.value(&reader_type()).unwrap().llvm, "{ ptr, ptr }");
    }
}
