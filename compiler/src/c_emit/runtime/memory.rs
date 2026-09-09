use crate::c_emit::syntax::{
    BinaryOperator, Block, Expr, FunctionDefinition, FunctionSignature, Initializer, Parameter,
    Statement, TranslationUnit, TypeName,
};
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
) -> TranslationUnit {
    let mut output = TranslationUnit::default();
    if offsets.0 {
        emit_offset(&mut output, "mal_ptr_offset", BinaryOperator::Add);
    }
    if offsets.1 {
        emit_offset(
            &mut output,
            "mal_ptr_offset_backward",
            BinaryOperator::Subtract,
        );
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
            FunctionSignature::static_inline(
                "MalType_Symbol",
                "mal_load_symbol",
                [
                    Parameter::named(TypeName::named("MalContext").pointer(), "context"),
                    Parameter::named("MalType_Ptr", "pointer"),
                    Parameter::named("uint64_t", "length"),
                ],
            ),
            Block::new([Statement::return_value(Expr::named_call(
                "mal_symbol_copy_from_bytes",
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
            FunctionSignature::static_inline(
                "MalType_Unit",
                "mal_store_symbol",
                [
                    Parameter::named(TypeName::named("MalContext").pointer(), "context"),
                    Parameter::named("MalType_Ptr", "pointer"),
                    Parameter::named("MalType_Symbol", "value"),
                ],
            ),
            Block::new([
                Statement::expression(Expr::cast("void", Expr::identifier("context"))),
                Statement::call(
                    "mal_symbol_copy_into",
                    [
                        Expr::identifier("value"),
                        Expr::identifier("pointer").field("address"),
                    ],
                ),
                Statement::return_value(unit()),
            ]),
        );
    }
    output
}

fn emit_offset(output: &mut TranslationUnit, name: &str, operator: BinaryOperator) {
    // Representability and region membership belong to the contract that
    // supplied the external capability.
    append_function(
        output,
        FunctionSignature::static_inline(
            "MalType_Ptr",
            name,
            [
                Parameter::named("MalType_Ptr", "pointer"),
                Parameter::named("uint64_t", "offset"),
            ],
        ),
        Block::new([Statement::return_value(Expr::compound_literal(
            "MalType_Ptr",
            [Initializer::positional(Expr::binary(
                operator,
                Expr::identifier("pointer").field("address"),
                Expr::cast("size_t", Expr::identifier("offset")),
            ))],
        ))]),
    );
}

fn emit_load(output: &mut TranslationUnit, name: &str, c_type: &str) {
    append_function(
        output,
        FunctionSignature::static_inline(
            c_type,
            format!("mal_load_{name}"),
            [Parameter::named("MalType_Ptr", "pointer")],
        ),
        Block::new([
            Statement::variable(c_type, "value", None),
            Statement::call(
                "memcpy",
                [
                    Expr::address_of(Expr::identifier("value")),
                    Expr::identifier("pointer").field("address"),
                    Expr::sizeof_expr(Expr::identifier("value")),
                ],
            ),
            Statement::return_value(Expr::identifier("value")),
        ]),
    );
}

fn emit_store(output: &mut TranslationUnit, name: &str, c_type: &str) {
    append_function(
        output,
        FunctionSignature::static_inline(
            "MalType_Unit",
            format!("mal_store_{name}"),
            [
                Parameter::named("MalType_Ptr", "pointer"),
                Parameter::named(c_type, "value"),
            ],
        ),
        Block::new([
            Statement::call(
                "memcpy",
                [
                    Expr::identifier("pointer").field("address"),
                    Expr::address_of(Expr::identifier("value")),
                    Expr::sizeof_expr(Expr::identifier("value")),
                ],
            ),
            Statement::return_value(unit()),
        ]),
    );
}

fn unit() -> Expr {
    Expr::compound_literal(
        "MalType_Unit",
        [Initializer::positional(Expr::named_call(
            "UINT8_C",
            [Expr::number("0")],
        ))],
    )
}

fn append_function(output: &mut TranslationUnit, signature: FunctionSignature, body: Block) {
    output.push(FunctionDefinition::from_signature(signature, body));
    output.blank_line();
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
