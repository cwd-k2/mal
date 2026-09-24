use super::Type;

pub(in crate::check) fn type_name(ty: &Type) -> String {
    const LIMIT: usize = 4096;

    let mut output = String::new();
    let mut pending = vec![TypeNamePart::Type(ty)];
    while let Some(part) = pending.pop() {
        let text = match part {
            TypeNamePart::Text(text) => text,
            TypeNamePart::Type(ty) => match ty {
                Type::Unit => "Unit",
                Type::Int8 => "Int8",
                Type::Int16 => "Int16",
                Type::Int32 => "Int32",
                Type::Int64 => "Int64",
                Type::UInt8 => "UInt8",
                Type::UInt16 => "UInt16",
                Type::UInt32 => "UInt32",
                Type::UInt64 => "UInt64",
                Type::Float32 => "Float32",
                Type::Float64 => "Float64",
                Type::Symbol => "Symbol",
                Type::Address => "Address",
                Type::ByteSize => "ByteSize",
                Type::USize => "USize",
                Type::Parameter { name, .. } => name,
                Type::Buffer(element) => {
                    pending.push(TypeNamePart::Text(">"));
                    pending.push(TypeNamePart::Type(element));
                    "Buffer<"
                }
                Type::External { name, .. } => name,
                Type::Product(elements) => {
                    push_aggregate_name(&mut pending, elements, ")");
                    "("
                }
                Type::Sum(members) if members.as_ref() == [Type::Unit, Type::Unit] => "Bool",
                Type::Sum(members) => {
                    push_aggregate_name(&mut pending, members, "]");
                    "["
                }
                Type::Function { parameter, result } => {
                    pending.push(TypeNamePart::Type(result));
                    pending.push(TypeNamePart::Text(" -> "));
                    pending.push(TypeNamePart::Type(parameter));
                    continue;
                }
            },
        };
        if output.len().saturating_add(text.len()) > LIMIT {
            output.push('…');
            break;
        }
        output.push_str(text);
    }
    output
}

enum TypeNamePart<'a> {
    Type(&'a Type),
    Text(&'a str),
}

fn push_aggregate_name<'a>(
    pending: &mut Vec<TypeNamePart<'a>>,
    elements: &'a [Type],
    close: &'static str,
) {
    pending.push(TypeNamePart::Text(close));
    for (index, element) in elements.iter().enumerate().rev() {
        pending.push(TypeNamePart::Type(element));
        if index != 0 {
            pending.push(TypeNamePart::Text(", "));
        }
    }
}
