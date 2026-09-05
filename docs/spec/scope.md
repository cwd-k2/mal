# 言語の範囲

Status: Current v0.5 profile

## mal が持つもの

mal の責務は次に限定する。

```text
value and type
immutable binding
lambda and application
product and sum
surface if and exhaustive case
self recursion
fixed-width numeric, logical, and bit operations
immutable byte string
typed scalar access through untyped Ptr
extern boundary
```

この集合は「実装 milestone M0」と同じではない。何をもって最小とするかは [最小性の方針](../design/minimality.md) で分けている。

## mal が持たないもの

v0.5は次を言語機能として持たない。

```text
mutable variable, typed Ptr<T>, reference
GC, ownership, borrow

struct, record, enum, class, method
interface, trait

generic, template, macro, reflection
exception, async/await, effect system
operator overloading

array and standard collections
standard library and allocator
module and package manager
```

この一覧は「実装がまだない」のではなく、v0.5 programが依存できないという規範的な範囲である。

## named data

record や enum の代わりに transparent alias、product、sum を使う。

```mal
Point :: (Float64, Float64);
MaybeInt32 :: [Unit, Int32];
```

constructor 名が欲しければ通常の関数を binding する。

```mal
some :: Int32 -> MaybeInt32 := \(value :: Int32) {
    return MaybeInt32[1](value);
};
```

mal は `Some` や field name に特別な意味を与えない。

## memory と mutable data

source languageは型なし`Ptr`と、byte offsetおよび全numeric scalarのload/storeを持つ。allocation、
deallocation、length、bounds、ownershipは組み込まず、program固有の`extern` contractに置く。完全な規則は
[memory primitive](memory.md)に定める。

```mal
extern alloc :: UInt64 -> Ptr;

readInt64 :: (Ptr, UInt64) -> Int64 := \(base :: Ptr, index :: UInt64) {
    return loadInt64(offset(base, index * 8u64));
};
```

mutable bytesが必要な場合も同じ境界を使う。次はpredefined APIではなく、program固有のhost contractの例である。

```mal
extern ByteBuffer;
extern bufferNew :: UInt64 -> ByteBuffer;
extern bufferWrite :: (ByteBuffer, UInt64, UInt8) -> Unit;
extern bufferToString :: ByteBuffer -> String;
extern bufferFree :: ByteBuffer -> Unit;
```

`bufferToString`が返すbytesは[`extern`のString copy規則](extern.md#stringのlifetime)によりmal-owned storageへcopyされる。`ByteBuffer` handleの複製、bounds、freeの安全性はhost contractの責務である。

array は例えば `(Ptr, UInt64)` の alias と mal 関数で構成できる。

```mal
Int64Array :: (Ptr, UInt64);

arrayGet :: (Int64Array, UInt64) -> Int64 :=
    \(array :: Int64Array, index :: UInt64) {
        (memory, _) := array;
        return loadInt64(offset(memory, index * 8u64));
    };
```

ただしbounds、allocation failure、deallocationはこのaliasだけでは保証されない。alignmentをscalar accessの
条件にはしない。storageのregionとlifetimeは[`extern` contract](extern.md)が定める。`[]` syntaxはない。

hash table、list、set も組み込み型ではない。必要な element type ごとに、product/sum と external storage から実装する。parametric polymorphism がないため、例えば `Int32Array` と `StringArray` は別実装になる。

## standard library と file

v0.5はArray、Map、File、Socket、JSON、Regex、HTTP、Unicode libraryを標準添付しない。必要なcodeはcompilation unitに含めるかhostが`extern`として提供する。

複数 file を一つの compilation unit にすることは compiler CLI の機能としてよいが、module/import/dependency resolution にはしない。詳細は [プログラム構造](programs.md) に置く。

## 設計原則

機能追加の前に、lambda、application、binding、product、sum、primitive、extern の組合せで素直に書けるかを確認する。

ただし「外側へ置く」は仕様責任の消滅を意味しない。型付きの値が extern 境界を越えるなら、layout、lifetime、failure の contract は必ず必要になる。
