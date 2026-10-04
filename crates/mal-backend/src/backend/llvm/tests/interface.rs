use super::super::*;
use mal_syntax::source::{FileId, SourceFile};

#[test]
fn admits_product_external_calls() {
    for (index, source) in [
        "extern inspect :: (UInt64, UInt64) -> UInt64; main :: Unit -> Int32 := () -> { inspect(1u64, 2u64).i32; };",
        "extern inspect :: Bool -> Bool; main :: Unit -> Int32 := () -> { if (inspect(true)) then { 0 } else { 1 }; };",
        "extern text :: Unit -> Symbol; extern inspect :: (UInt64, Symbol) -> UInt64; main :: Unit -> Int32 := () -> { inspect(1u64, text()).i32; };",
        "extern text :: Unit -> Symbol; extern inspect :: (Symbol, USize) -> (Symbol, USize); main :: Unit -> Int32 := () -> { (_, length) := inspect(text(), 1usize); length.i32; };",
        "extern text :: Unit -> Symbol; Packet :: (Symbol, USize); extern exchange :: Packet -> Packet; main :: Unit -> Int32 := () -> { (_, length) := exchange(text(), 2usize); length.i32; };",
    ]
    .into_iter()
    .enumerate()
    {
        let source = SourceFile::new(FileId::new(76), "product-extern.mal", source.into());
        let checked = mal_frontend::analysis::check(&source).expect("check product extern fixture");
        let core = crate::core::lower(&mal_frontend::check::specialize(checked).expect("specialize checked program"));
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution = crate::execution::lower(
            closure,
            crate::execution::OptimizationSet::production(),
        );
        assert!(supports(&execution), "unsupported fixture {index}");
    }
}

#[test]
fn admits_sum_external_calls_recursively() {
    for (index, source) in [
        "Choice :: [Symbol, USize]; extern inspect :: Choice -> Choice; main :: Unit -> Int32 := () -> { 0; };",
        "Choice :: [Unit, (Buffer<UInt8>, USize)]; Envelope :: (UInt8, Choice); extern inspect :: Envelope -> Envelope; main :: Unit -> Int32 := () -> { 0; };",
        "extern Handle; Choice :: [Unit, (UInt64, Handle)]; extern inspect :: Choice -> Choice; main :: Unit -> Int32 := () -> { 0; };",
    ]
    .into_iter()
    .enumerate()
    {
        let source = SourceFile::new(FileId::new(77), "sum-extern.mal", source.into());
        let checked = mal_frontend::analysis::check(&source).expect("check sum extern fixture");
        let core = crate::core::lower(&mal_frontend::check::specialize(checked).expect("specialize checked program"));
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution = crate::execution::lower(
            closure,
            crate::execution::OptimizationSet::production(),
        );
        assert!(supports(&execution), "unsupported fixture {index}");
    }
}
