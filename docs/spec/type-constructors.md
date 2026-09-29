# kindとtype constructor

Status: Accepted v0.6

この文書はkind inference、partial type application、canonical type-level termを定める。generic valueの型argument推論は
[parametric polymorphism](generics.md)、operation keyは[operation family](operation-families.md)を正とする。

compiler内部のkindは`Type`、function kind `K -> K`、推論変数からなる。runtime valueが持つ型のkindは`Type`である。
generic aliasとopaque typeのdeclaration parameterはtype-level abstractionを作り、`Buffer`はkind `Type -> Type`を持つ。
kindはdeclaration bodyとvalue signatureからoccurs check付きで推論し、sourceへkind annotationを要求しない。未制約なkindは
rank-1でgeneralizeし、declarationの参照ごとにfreshにinstantiateする。

type applicationは左からargumentを適用する。argument数をdeclaration arityと比較せず、kindがfunctionならpartial applicationを
認め、kind `Type`へさらに適用した場合とkindが一致しないargumentを拒否する。value signature、opaque representation、product、sum、
function、`Buffer` elementなどruntime typeを要求する位置は、正規化後のkindが`Type`でなければならない。

generic transparent aliasはtype-level abstractionとして展開し、applicationをbeta reduceする。alpha同値とeta同値をcanonical type termへ
正規化し、phantom parameterへ渡したargumentはforceしない。sourceにtype lambda、placeholder、kind annotation、type-level pattern match、
type constructorをruntime valueとして扱う構文はない。

```mal
Apply<F, A> :: F<A>;
Pair<A, B> :: (A, B);
PairWithInt32 :: Pair<Int32>;

value :: Apply<PairWithInt32, UInt8> := (42i32, 7u8);
```
