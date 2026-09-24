# 言語の範囲

Status: Accepted v0.6

## malが持つもの

- value、type、immutable binding
- lambda、application、self recursion
- product、sum、surface `if`、exhaustive sum continuation application
- direct block、direct result block、`when`、empty sum elimination
- fixed-width numeric、`ByteSize`、`USize`、logical、bit operation
- language-intrinsic immutable `Symbol`
- explicit parametric polymorphismとwhole-program specialization
- opaqueな`Address` capabilityとcanonical memory layout
- mal-ownedで共有可変な`Buffer`
- extern boundary
- optional process argument entry

何をもって最小とするかは[最小性の方針](../design/minimality.md)で定める。

## 実行環境

実行環境は`malc`のLLVM backendとC hostだけである。`from<T>`、`buffer.into`、`extern`のC ABI、`Address`と`Buffer`の
canonical layoutはこの環境のtarget ABIとdata layoutから決まり、他のbackendやhostへの対応規則は仕様に含めない。
C hostの規則は[C host ABI](c-host-abi.md)、`malc`の対応環境は[`malc`利用contract](../development/compiler-usage.md)が定める。

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

languageはexternal storageのallocation policyを持たない。host contractから受け取った
`Address`、element offset、lengthを`from<T>`へ渡し、独立した`Buffer<T>`へcopyする。alignment、範囲、permission、
lifetime、allocation failure policyは必要なoperationのcontractが所有する。

```mal
extern allocate :: ByteSize -> Address;

readInt64 :: (Address, USize) -> Int64 := (base, index) -> {
    values := from<Int64>(base, index, 1usize);
    values.get(0usize);
};
```

mal内に保持する有限sequenceは`Buffer<A>`へcopyできる。file、socket、deviceはexternal opaque typeとextern operationで表す。
`Address`と`Buffer`の規則は[該当仕様](memory.md)に定める。

## Standard library

Array、Map、File、Socket、JSON、Regex、HTTP、Unicode libraryを標準添付しない。必要なcodeは`.mal` fileからrequireするかhostが
externとして提供する。programとsource fileの規則は[プログラム構造](programs.md)に置く。

## 設計原則

機能は、基礎modelを理解した後に個別の型やcall siteで独立contractを再判断せず、一つの規則から挙動を導ける場合に共通mechanismへ
置く。program固有のpolicy、resource lifecycle、protocol encodingはsourceまたはextern contractへ残す。
