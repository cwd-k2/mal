use crate::check::ast::Type;

use super::body::types::Types;

pub(super) fn entry_main(parameter: &Type, types: Types, entry: &str) -> Option<String> {
    if *parameter == Type::Unit {
        return Some(format!(
            "int main(void) {{\n    MalContext context = {{0}};\n    int32_t result;\n    {entry}(&context, NULL, &result);\n    mal_control_destroy(&context);\n    return result;\n}}\n"
        ));
    }
    let fields = types.product_fields(parameter)?;
    let count_offset = fields.first()?.offset;
    let pointer_offset = fields.get(1)?.offset;
    let value = types.value(parameter)?;
    let descriptor_stride = types
        .value(&Type::Ptr)?
        .size
        .checked_add(types.value(&Type::UInt64)?.size)?;
    Some(format!(
        "int main(int mal_argc, char **mal_argv) {{\n    MalContext context = {{0}};\n    size_t argument_count = mal_argc > 1 ? (size_t)(mal_argc - 1) : 0;\n    if ((size_t)(uint64_t)argument_count != argument_count || argument_count > SIZE_MAX / {descriptor_stride}) {{\n        mal_trap(&context, \"argument descriptor size overflow\");\n    }}\n    unsigned char *storage = mal_runtime_allocate(&context, argument_count * {descriptor_stride});\n    for (size_t index = 0; index < argument_count; ++index) {{\n        size_t length = strlen(mal_argv[index + 1]);\n        if ((size_t)(uint64_t)length != length) {{\n            mal_trap(&context, \"argument length overflow\");\n        }}\n        unsigned char *slot = storage + index * {descriptor_stride};\n        void *data = mal_argv[index + 1];\n        uint64_t length_u64 = (uint64_t)length;\n        memcpy(slot, &data, sizeof(data));\n        memcpy(slot + sizeof(data), &length_u64, sizeof(length_u64));\n    }}\n    _Alignas({}) unsigned char argument[{}] = {{0}};\n    uint64_t count_u64 = (uint64_t)argument_count;\n    void *descriptor = storage;\n    memcpy(argument + {count_offset}, &count_u64, sizeof(count_u64));\n    memcpy(argument + {pointer_offset}, &descriptor, sizeof(descriptor));\n    int32_t result;\n    {entry}(&context, argument, &result);\n    mal_runtime_deallocate(storage);\n    mal_control_destroy(&context);\n    return result;\n}}\n",
        value.alignment, value.size
    ))
}
