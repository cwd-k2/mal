# 言語の範囲

Status: Accepted v0.6 profile

## malが持つもの

- value、type、immutable binding
- lambda、application、self recursion
- product、sum、surface `if`、exhaustive sum continuation application
- direct block、direct result block、`when`、empty sum elimination
- fixed-width numeric、`ByteSize`、`USize`、logical、bit operation
- language-intrinsic immutable `Symbol`
- explicit parametric polymorphismとwhole-program specialization
- `Address`、canonical memory layout、`Cursor`、`Region`によるexternal memory access
- mal-owned immutable sequence `Packed`とそのscoped構築・編集authority `Buffer`
- extern boundary
- optional process argument entry

何をもって最小とするかは[最小性の方針](../design/minimality.md)で定める。

## malが持たないもの

- mutable variable、reference、borrow checker
- resource ownershipを強制する型、destructor、finalizer
- record、class、method、nominal enum
- user-defined interface、trait、constraint、operator overload
- local generalization、first-class polymorphism、higher-kinded type、reflection、macro
- exception、async/await、effect system、first-class continuation
- mutable arrayと標準collection
- standard library、allocator、package manager

この一覧はprogramが依存できない規範的な範囲である。外した責務は、必要に応じてmal sourceまたはprogram固有のextern contractが
明示する。

## Named data

recordやenumの代わりにtransparent alias、product、sumを使う。構築用の名前は通常のfunctionとしてbindingする。

```mal
Point :: (Float64, Float64);
Maybe<A> :: [Unit, A];

some<A> :: A -> Maybe<A> := (value) -> [none, some] => some(value);
```

field name、implicit constructor、nominal identityはない。

## Memoryとmutable data

languageはexternal storageのallocation policyを持たない。host contractから受け取ったAddressをshapeでCursorへ置き、USizeを加えて
Regionを作る。exact placementとunaligned accessは共通mechanism、alignment、permission、lifetime、allocation failure policyは
必要なoperationのcontractが所有する。

```mal
extern allocate :: ByteSize -> Address;

readInt64 :: (Address, USize) -> Int64 := (base, index) -> {
    cursor := (base + index * #i64)@i64;
    <-cursor;
};
```

mal内に保持する有限sequenceは`Packed<A>`へadmitできる。更新を繰り返すbuffer、file、socket、deviceはRegionまたはexternal opaque
typeとextern operationで表す。`Region`と`Packed`の規則は[該当仕様](packed.md)に定める。

## Standard library

Array、Map、File、Socket、JSON、Regex、HTTP、Unicode libraryを標準添付しない。必要なcodeは`.mal` fileからrequireするかhostが
externとして提供する。programとsource fileの規則は[プログラム構造](programs.md)に置く。

## 設計原則

機能は、基礎modelを理解した後に個別の型やcall siteで独立contractを再判断せず、一つの規則から挙動を導ける場合に共通mechanismへ
置く。program固有のpolicy、resource lifecycle、protocol encodingはsourceまたはextern contractへ残す。
