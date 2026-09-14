# 言語の範囲

Status: Current v0.5 profile

## mal が持つもの

mal の責務は次に限定する。

- value and type
- immutable binding
- lambda and application
- product and sum
- surface `if` and exhaustive sum continuation application
- explicit lambda-local return binder、direct block、direct result block、`when`、empty sum elimination
- self recursion
- fixed-width numeric, logical, and bit operations
- language-intrinsic immutable `Symbol`
- `Symbol` byte concatenation
- typed numeric scalar and pointer access through untyped `Ptr`
- extern boundary
- optional process argument entry

何をもって最小とするかは[最小性の方針](../design/minimality.md)で定める。

## mal が持たないもの

v0.5は次を言語機能として持たない。

- mutable variable、typed `Ptr<T>`、reference
- GC、ownership、borrow
- struct、record、enum、class、method
- interface、trait
- generic、template、macro、reflection
- exception、async/await、effect system、first-class continuation
- operator overloading
- array and standard collections
- standard library and allocator
- package manager

この一覧は「実装がまだない」のではなく、v0.5 programが依存できないという規範的な範囲である。

## named data

record や enum の代わりに transparent alias、product、sum を使う。

```mal
Point :: (Float64, Float64);
MaybeInt32 :: [Unit, Int32];
```

構築用の名前が欲しければ、sum return binderを使う通常の関数をbindingする。

```mal
some :: Int32 -> MaybeInt32 := (value)[none, some] { some(value) };
```

mal は `Some` や field name に特別な意味を与えない。

## memory と mutable data

source languageは型なし`Ptr`と、byte offset、全numeric scalarと`Ptr`のload/store、およびSymbol byte copyを持つ。allocation、
deallocation、length、bounds、ownershipは組み込まず、program固有の`extern` contractに置く。完全な規則は
[memory primitive](memory.md)に定める。

```mal
extern alloc :: UInt64 -> Ptr;

readInt64 :: (Ptr, UInt64) -> Int64 :=
    (base, index) { Int64.load(base + index * Int64.size) };
```

mutable bytesが必要な場合も同じ境界を使う。次はpredefined APIではなく、program固有のhost contractの例である。

```mal
extern ByteBuffer;
extern bufferNew :: UInt64 -> ByteBuffer;
extern bufferWrite :: (ByteBuffer, UInt64, UInt8) -> Unit;
extern bufferToSymbol :: ByteBuffer -> Symbol;
extern bufferFree :: ByteBuffer -> Unit;
```

`bufferToSymbol`が返すbytesは[`extern`のadmission規則](extern.md#boundary-transport)によりmal-controlled storageへcopyされる。`ByteBuffer` handleの複製、bounds、freeの安全性はhost contractの責務である。

array は例えば `(Ptr, UInt64)` の alias と mal 関数で構成できる。

```mal
Int64Array :: (Ptr, UInt64);

arrayGet :: (Int64Array, UInt64) -> Int64 :=
    (array, index) {
        (memory, _) := array;
        Int64.load(memory + index * Int64.size);
    };
```

ただしbounds、allocation failure、deallocationはこのaliasだけでは保証されない。alignmentをscalar accessの
条件にはしない。storageのregionとlifetimeは[`extern` contract](extern.md)が定める。`[]` syntaxはない。

hash table、list、set も組み込み型ではない。必要な element type ごとに、product/sum と external storage から実装する。parametric polymorphism がないため、例えば `Int32Array` と `SymbolArray` は別実装になる。

## standard library と file

v0.5はArray、Map、File、Socket、JSON、Regex、HTTP、Unicode libraryを標準添付しない。必要なcodeは`.mal` fileから
requireするかhostが`extern`として提供する。

programとsource fileの規則は[プログラム構造](programs.md)に置く。

## 設計原則

機能追加の前に、lambda、application、binding、product、sum、primitive、extern の組合せで素直に書けるかを確認する。

ただし「外側へ置く」は仕様責任の消滅を意味しない。型付きの値が extern 境界を越えるなら、layout、lifetime、failure の contract は必ず必要になる。
