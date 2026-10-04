# 言語の範囲

Status: Accepted v0.7

## malが持つもの

- value、type、immutable binding
- lambda、application、self recursion
- product、sum、surface `if`、exhaustive sum continuation application
- direct block、direct result block、`when`、empty sum elimination
- fixed-width numeric、`ByteSize`、`USize`、logical、bit operation
- language-intrinsic immutable `Symbol`
- explicit parametric polymorphismとwhole-program specialization
- compilerがkindを推論するhigher-kinded type parameterとpartial type application
- file-local representation authorityを持つsource-defined opaque type
- mal-ownedで共有可変な`Buffer`
- C runtime extensionとしてのextern boundary
- optional process argument entry

何をもって最小とするかは[最小性の方針](../design/minimality.md)で定める。

## 実行環境

実行環境は`malc`のLLVM backend、C runtime、同じartifactへlinkするextern C implementationだけである。runtime carrierとextern C
ABIはこの環境のtarget ABIとdata layoutから決まり、他のbackendやhostへの対応規則は仕様に含めない。C extensionの規則は
[C runtime extension ABI](c-host-abi.md)、`malc`の対応環境は[`malc`利用contract](../development/compiler-usage.md)が定める。

## malが持たないもの

- mutable variable、reference、borrow checker
- resource ownershipを強制する型、destructor、finalizer
- record、class、method、nominal enum
- user-defined interface、trait、constraint、operator overload
- local generalization、first-class polymorphism、higher-rank kind polymorphism、type lambda、kind annotation、reflection、macro
- exception、async/await、effect system、first-class continuation
- mutable arrayと標準collection
- standard library、allocator、package manager

この一覧はprogramが依存できない規範的な範囲である。外した責務は、必要に応じてmal sourceまたはprogram固有のextern contractが
明示する。

## Named data

公開構造をそのままdata modelにする場合はtransparent alias、product、sumを使う。構築用の名前は通常のfunctionとしてbindingする。

```mal
Point :: (Float64, Float64);
Maybe<A> :: [Unit, A];

some<A> :: A -> Maybe<A> := (value) -> [none, some] => some(value);
```

field nameとimplicit constructorはない。representation invariantをfile内へ閉じる場合は
[file-local opaque type](types.md#file-local-opaque-type)を使い、通常のfunctionを公開operationにする。

## Memoryとmutable data

mal内に保持する有限mutable sequenceは`Buffer<A>`で表す。extern CはBufferをruntimeから構築してmalへ返し、借りたBufferの共有identityを
変更できる。mal sourceはraw pointer、汎用dereference、external storageのlayoutを持たない。

```mal
extern readInt64s :: File -> [UInt32, Buffer<Int64>];
```

file、socket、mapping、deviceはexternal opaque typeと型付きextern operationで表す。C libraryやsystem callが必要とするpointerは
C bodyがruntime carrierからcall中だけ取得する。Bufferの規則は[該当仕様](memory.md)、extern resourceは
[`extern`](extern.md)に定める。

## Standard library

Array、Map、File、Socket、JSON、Regex、HTTP、Unicode libraryを標準添付しない。必要なcodeは`.mal` fileからrequireするかhostが
externとして提供する。programとsource fileの規則は[プログラム構造](programs.md)に置く。

## 設計原則

機能は、基礎modelを理解した後に個別の型やcall siteで独立contractを再判断せず、一つの規則から挙動を導ける場合に共通mechanismへ
置く。program固有のpolicy、resource lifecycle、protocol encodingはsourceまたはextern contractへ残す。
