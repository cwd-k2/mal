use super::TranslationUnit;
use crate::c_emit::syntax::{Declaration, Directive, TypeName};

#[test]
fn renders_items_and_owned_section_spacing() {
    let mut unit = TranslationUnit::default();
    unit.push(Directive::include_system("stdint.h"));
    unit.blank_line();
    unit.blank_line();
    unit.push(Declaration::type_alias(TypeName::named("uint8_t"), "Byte"));

    assert_eq!(
        unit.render(),
        "#include <stdint.h>\n\ntypedef uint8_t Byte;\n"
    );
}
