use std::{collections::HashMap, rc::Rc};

use mal_frontend::check::ast::{SharedTypeId, Type};

use super::body::types::{self, Types};

pub(super) struct Value<'a> {
    pub(super) ty: &'a Type,
    pub(super) kind: Kind<'a>,
}

pub(super) enum Kind<'a> {
    Unit,
    Scalar,
    External,
    Product(Vec<Field<'a>>),
    Sum {
        tag_offset: usize,
        variants: Vec<Field<'a>>,
    },
}

pub(super) struct Field<'a> {
    pub(super) offset: usize,
    pub(super) value: Rc<Value<'a>>,
}

impl<'a> Value<'a> {
    pub(super) fn new(ty: &'a Type, types: Types) -> Option<Rc<Self>> {
        Self::new_cached(ty, types, &mut HashMap::new())
    }

    fn new_cached(
        ty: &'a Type,
        types: Types,
        cache: &mut HashMap<SharedTypeId, Rc<Self>>,
    ) -> Option<Rc<Self>> {
        if let Some(value) = ty.shared_id().and_then(|id| cache.get(&id)) {
            return Some(value.clone());
        }
        let kind = match ty {
            Type::Unit => Kind::Unit,
            Type::Symbol => return None,
            Type::External { .. } => Kind::External,
            Type::Product(elements) => Kind::Product(product_fields(ty, elements, types, cache)?),
            Type::Sum(elements) if !types::is_bool(ty) => {
                let layouts = types.sum_fields(ty)?;
                Kind::Sum {
                    tag_offset: layouts.first()?.offset,
                    variants: elements
                        .iter()
                        .zip(layouts.into_iter().skip(1))
                        .map(|(element, layout)| {
                            Some(Field {
                                offset: layout.offset,
                                value: Self::new_cached(element, types.clone(), cache)?,
                            })
                        })
                        .collect::<Option<Vec<_>>>()?,
                }
            }
            Type::Function { .. } => return None,
            _ => Kind::Scalar,
        };
        let value = Rc::new(Self { ty, kind });
        if let Some(id) = ty.shared_id() {
            cache.insert(id, value.clone());
        }
        Some(value)
    }
}

fn product_fields<'a>(
    ty: &Type,
    elements: &'a [Type],
    types: Types,
    cache: &mut HashMap<SharedTypeId, Rc<Value<'a>>>,
) -> Option<Vec<Field<'a>>> {
    let layouts = types.product_fields(ty)?;
    elements
        .iter()
        .zip(layouts)
        .map(|(element, layout)| {
            Some(Field {
                offset: layout.offset,
                value: Value::new_cached(element, types.clone(), cache)?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_nested_product_offsets_before_c_emission() {
        let ty = Type::Product(
            vec![
                Type::UInt8,
                Type::UInt64,
                Type::Product(vec![Type::UInt16, Type::UInt32].into()),
            ]
            .into(),
        );
        let value = Value::new(&ty, Types::new(8).expect("target types")).expect("value plan");
        let Kind::Product(fields) = &value.kind else {
            panic!("product plan");
        };

        assert_eq!(
            fields.iter().map(|field| field.offset).collect::<Vec<_>>(),
            [0, 8, 16]
        );
        let Kind::Product(nested) = &fields[2].value.kind else {
            panic!("nested product plan");
        };
        assert_eq!(
            nested.iter().map(|field| field.offset).collect::<Vec<_>>(),
            [0, 4]
        );
    }

    #[test]
    fn plans_sum_tag_and_variant_offsets_before_c_emission() {
        let ty = Type::Sum(vec![Type::UInt8, Type::UInt64].into());
        let value = Value::new(&ty, Types::new(8).expect("target types")).expect("value plan");
        let Kind::Sum {
            tag_offset,
            variants,
        } = &value.kind
        else {
            panic!("sum plan");
        };

        assert_eq!(*tag_offset, 0);
        assert_eq!(
            variants
                .iter()
                .map(|field| field.offset)
                .collect::<Vec<_>>(),
            [4, 4]
        );
    }

    #[test]
    fn shares_repeated_marshalling_subplans() {
        let mut ty = Type::UInt8;
        for _ in 0..64 {
            ty = Type::Sum(vec![ty.clone(), ty].into());
        }
        let mut value = Value::new(&ty, Types::new(8).unwrap()).unwrap();

        for depth in 0..63 {
            let Kind::Sum { variants, .. } = &value.kind else {
                panic!("nested sum plan");
            };
            assert!(
                Rc::ptr_eq(&variants[0].value, &variants[1].value),
                "unshared plan at depth {depth}"
            );
            value = variants[0].value.clone();
        }
        let Kind::Sum { variants, .. } = &value.kind else {
            panic!("leaf sum plan");
        };
        assert!(
            variants
                .iter()
                .all(|variant| matches!(&variant.value.kind, Kind::Scalar))
        );
    }
}
