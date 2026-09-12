use crate::check::ast::Type;

use super::body::types::{self, Types};

pub(super) struct Value<'a> {
    pub(super) ty: &'a Type,
    pub(super) kind: Kind<'a>,
}

pub(super) enum Kind<'a> {
    Unit,
    Scalar,
    Symbol,
    External,
    Product(Vec<Field<'a>>),
    Sum {
        tag_offset: usize,
        variants: Vec<Field<'a>>,
    },
}

pub(super) struct Field<'a> {
    pub(super) offset: usize,
    pub(super) value: Value<'a>,
}

impl<'a> Value<'a> {
    pub(super) fn new(ty: &'a Type, types: Types) -> Option<Self> {
        let kind = match ty {
            Type::Unit => Kind::Unit,
            Type::Symbol => Kind::Symbol,
            Type::External { .. } => Kind::External,
            Type::Product(elements) => Kind::Product(product_fields(ty, elements, types)?),
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
                                value: Value::new(element, types)?,
                            })
                        })
                        .collect::<Option<Vec<_>>>()?,
                }
            }
            Type::Function { .. } => return None,
            _ => Kind::Scalar,
        };
        Some(Self { ty, kind })
    }
}

fn product_fields<'a>(ty: &Type, elements: &'a [Type], types: Types) -> Option<Vec<Field<'a>>> {
    let layouts = types.product_fields(ty)?;
    elements
        .iter()
        .zip(layouts)
        .map(|(element, layout)| {
            Some(Field {
                offset: layout.offset,
                value: Value::new(element, types)?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_nested_product_offsets_before_c_emission() {
        let ty = Type::Product(vec![
            Type::UInt8,
            Type::UInt64,
            Type::Product(vec![Type::UInt16, Type::UInt32]),
        ]);
        let value = Value::new(&ty, Types::new(8).expect("target types")).expect("value plan");
        let Kind::Product(fields) = value.kind else {
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
        let ty = Type::Sum(vec![Type::UInt8, Type::UInt64]);
        let value = Value::new(&ty, Types::new(8).expect("target types")).expect("value plan");
        let Kind::Sum {
            tag_offset,
            variants,
        } = value.kind
        else {
            panic!("sum plan");
        };

        assert_eq!(tag_offset, 0);
        assert_eq!(
            variants
                .iter()
                .map(|field| field.offset)
                .collect::<Vec<_>>(),
            [4, 8]
        );
    }
}
