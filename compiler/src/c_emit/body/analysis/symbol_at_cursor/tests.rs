use super::*;
use crate::source::{FileId, SourceFile};
use crate::{anf, check, closure, core, parser, resolve};

#[test]
fn admits_only_access_to_a_symbol_preserved_by_every_tail_edge() {
    let preserved = lower(
        "scan :: (Symbol, UInt64) -> UInt8 := \\(value :: Symbol, index :: UInt64) {\n\
           if (index + 1u64 == #value)\n\
           then { value # index }\n\
           else { byte := value # index; scan(value, index + UInt64(byte)); };\n\
         };\n\
         main :: Unit -> Int32 := \\() { Int32(scan(\"a\", 0u64)); };",
    );
    let mut plan = SymbolAtCursorPlan::new(&preserved);
    assert!(plan.is_valid(&preserved));
    assert_eq!(plan.sites.values().map(Vec::len).sum::<usize>(), 2);

    let function = *plan.sites.keys().next().expect("planned function");
    plan.sites.remove(&function);
    assert!(!plan.is_valid(&preserved));

    let replaced = lower(
        "scan :: (Symbol, UInt64) -> UInt8 := \\(value :: Symbol, index :: UInt64) {\n\
           if (index + 1u64 == #value)\n\
           then { value # index }\n\
           else { byte := value # index; scan(value + \"x\", index + UInt64(byte)); };\n\
         };\n\
         main :: Unit -> Int32 := \\() { Int32(scan(\"a\", 0u64)); };",
    );
    assert!(SymbolAtCursorPlan::new(&replaced).sites.is_empty());
}

fn lower(source: &str) -> crate::closure::ast::Program {
    let source = SourceFile::new(FileId::new(91), "symbol-at-cursor-plan.mal", source.into());
    let parsed = parser::parse(&source).expect("parse cursor plan fixture");
    let resolved = resolve::resolve(&parsed).expect("resolve cursor plan fixture");
    let checked = check::check(&resolved).expect("check cursor plan fixture");
    let core = core::lower(&checked);
    let anf = anf::lower(&core);
    closure::convert(&anf)
}
