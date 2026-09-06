use crate::c_emit::syntax::{Block, Expr, FunctionDefinition, Initializer, Statement};
use crate::check::ast::MemoryScalar;

const SCALARS: [(MemoryScalar, &str, &str); 10] = [
    (MemoryScalar::Int8, "int8", "int8_t"),
    (MemoryScalar::Int16, "int16", "int16_t"),
    (MemoryScalar::Int32, "int32", "int32_t"),
    (MemoryScalar::Int64, "int64", "int64_t"),
    (MemoryScalar::UInt8, "uint8", "uint8_t"),
    (MemoryScalar::UInt16, "uint16", "uint16_t"),
    (MemoryScalar::UInt32, "uint32", "uint32_t"),
    (MemoryScalar::UInt64, "uint64", "uint64_t"),
    (MemoryScalar::Float32, "float32", "float"),
    (MemoryScalar::Float64, "float64", "double"),
];

pub(crate) fn scalar_mask(scalar: MemoryScalar) -> u16 {
    1 << scalar_index(scalar)
}

pub(crate) fn scalar_name(scalar: MemoryScalar) -> &'static str {
    SCALARS[scalar_index(scalar)].1
}

pub(super) fn emit(
    offsets: (bool, bool),
    loads: u16,
    stores: u16,
    load_ptr: bool,
    store_ptr: bool,
    load_symbol: bool,
    store_symbol: bool,
) -> String {
    let mut output = String::new();
    if offsets.0 {
        emit_offset(&mut output, "mal_ptr_offset", "+");
    }
    if offsets.1 {
        emit_offset(&mut output, "mal_ptr_offset_backward", "-");
    }
    for (index, (_, name, c_type)) in SCALARS.iter().enumerate() {
        let mask = 1 << index;
        if loads & mask != 0 {
            emit_load(&mut output, name, c_type);
        }
        if stores & mask != 0 {
            emit_store(&mut output, name, c_type);
        }
    }
    if load_ptr {
        emit_load(&mut output, "ptr", "MalType_Ptr");
    }
    if store_ptr {
        emit_store(&mut output, "ptr", "MalType_Ptr");
    }
    if load_symbol {
        append_function(
            &mut output,
            "static inline MalType_Symbol mal_load_symbol(MalContext *context, MalType_Ptr pointer, uint64_t length)",
            Block::new([Statement::return_value(Expr::named_call(
                "mal_Symbol_copy_from_bytes",
                [
                    Expr::identifier("context"),
                    Expr::identifier("pointer").field("address"),
                    Expr::identifier("length"),
                ],
            ))]),
        );
    }
    if store_symbol {
        append_function(
            &mut output,
            "static inline MalType_Unit mal_store_symbol(MalType_Ptr pointer, MalType_Symbol value)",
            Block::new([
                Statement::if_then(
                    Expr::binary(
                        "!=",
                        Expr::identifier("value").field("length"),
                        Expr::named_call("UINT64_C", [Expr::literal("0")]),
                    ),
                    Block::new([Statement::expression(Expr::named_call(
                        "memcpy",
                        [
                            Expr::identifier("pointer").field("address"),
                            Expr::identifier("value").field("data"),
                            Expr::cast("size_t", Expr::identifier("value").field("length")),
                        ],
                    ))]),
                ),
                Statement::return_value(unit()),
            ]),
        );
    }
    output
}

fn emit_offset(output: &mut String, name: &str, operator: &'static str) {
    append_function(
        output,
        format!(
            "static inline MalType_Ptr {name}(MalContext *context, MalType_Ptr pointer, uint64_t offset)"
        ),
        Block::new([
            Statement::if_then(
                Expr::binary(
                    ">",
                    Expr::identifier("offset"),
                    Expr::identifier("SIZE_MAX"),
                ),
                trap("pointer offset is not representable on this target"),
            ),
            Statement::return_value(Expr::compound_literal(
                "MalType_Ptr",
                [Initializer::positional(Expr::binary(
                    operator,
                    Expr::identifier("pointer").field("address"),
                    Expr::cast("size_t", Expr::identifier("offset")),
                ))],
            )),
        ]),
    );
}

fn emit_load(output: &mut String, name: &str, c_type: &str) {
    append_function(
        output,
        format!("static inline {c_type} mal_load_{name}(MalType_Ptr pointer)"),
        Block::new([
            Statement::declaration(format!("{c_type} value"), None),
            Statement::expression(Expr::named_call(
                "memcpy",
                [
                    Expr::unary("&", Expr::identifier("value")),
                    Expr::identifier("pointer").field("address"),
                    Expr::sizeof_type("value"),
                ],
            )),
            Statement::return_value(Expr::identifier("value")),
        ]),
    );
}

fn emit_store(output: &mut String, name: &str, c_type: &str) {
    append_function(
        output,
        format!("static inline MalType_Unit mal_store_{name}(MalType_Ptr pointer, {c_type} value)"),
        Block::new([
            Statement::expression(Expr::named_call(
                "memcpy",
                [
                    Expr::identifier("pointer").field("address"),
                    Expr::unary("&", Expr::identifier("value")),
                    Expr::sizeof_type("value"),
                ],
            )),
            Statement::return_value(unit()),
        ]),
    );
}

fn trap(message: &'static str) -> Block {
    Block::new([Statement::expression(Expr::named_call(
        "mal_trap",
        [
            Expr::identifier("context"),
            Expr::literal(format!("\"{message}\"")),
        ],
    ))])
}

fn unit() -> Expr {
    Expr::compound_literal(
        "MalType_Unit",
        [Initializer::positional(Expr::named_call(
            "UINT8_C",
            [Expr::literal("0")],
        ))],
    )
}

fn append_function(output: &mut String, signature: impl Into<String>, body: Block) {
    output.push_str(&FunctionDefinition::new(signature, body).render());
    output.push('\n');
}

fn scalar_index(scalar: MemoryScalar) -> usize {
    match scalar {
        MemoryScalar::Int8 => 0,
        MemoryScalar::Int16 => 1,
        MemoryScalar::Int32 => 2,
        MemoryScalar::Int64 => 3,
        MemoryScalar::UInt8 => 4,
        MemoryScalar::UInt16 => 5,
        MemoryScalar::UInt32 => 6,
        MemoryScalar::UInt64 => 7,
        MemoryScalar::Float32 => 8,
        MemoryScalar::Float64 => 9,
    }
}
