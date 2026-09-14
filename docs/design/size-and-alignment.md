# `Size`とaligned memory accessの試案

Status: Discussion draft (2026-09-14)

この文書はtarget依存のbyte量を表す`Size`と、programmerがalignmentを保証するmemory accessの候補を記録する。
現行の規範は[`types`](../spec/types.md)、[`Symbol`](../spec/symbols.md)、
[`memory primitive`](../spec/memory.md)、[`program`](../spec/programs.md)、
[`C host ABI`](../spec/c-host-abi.md)を正とし、この試案だけを根拠にsourceやABIを変更しない。

## 問題

現行仕様はstorage size、pointerのbyte offset、`Symbol`のlengthとindex、process argumentのcountとlengthを
`UInt64`で表す。この固定幅には64-bit targetへ限定する意味はなく、targetのaddress計算で表現できない値を
contract違反として除外する。一方、targetが実際に扱うobject sizeとaddress indexの範囲をsource typeへ反映せず、
固定幅を選ぶ積極的な理由も記録されていない。

通常のscalar memory accessはalignmentを要求しないため、LLVM backendは`align 1`でload/storeする。実際のaddressが
揃っていても、その事実をprogrammerからbackendへ伝えるsource operationはない。

## `Size`

`Size`をdefault address spaceでobject size、要素数、byte offsetを表すtarget依存のunsigned numeric typeとする候補を採る。
pointerのmemory representation幅とは独立に定め、LLVM backendではtarget data layoutのpointer index幅、public C ABIでは
`size_t`に対応させる。

```text
source type          Size
literal suffix       size
public C ABI type    mal_Size_t
C representation    size_t
LLVM representation default address spaceのpointer index幅を持つinteger
```

`Size`は他のnumeric typeと同じくliteral、算術、比較、明示的conversionの対象にする。`10size`のような明示的literalが
targetの範囲に収まらなければcompile-time errorとする。suffixのないinteger literalは周辺型が`Size`なら`Size`になる。

採択する場合、target上のmemoryまたは実在する有限byte sequenceの量を表す次のoperationを`UInt64`から`Size`へ揃える。

```mal
T.size       :: Size
+            :: (Ptr, Size) -> Ptr
-            :: (Ptr, Size) -> Ptr
#symbol      :: Size
symbol # i   :: UInt8  // i :: Size
Symbol.read  :: (Ptr, Size) -> Symbol
main         :: (Size, Ptr) -> Int32
```

process argument descriptorは`Ptr`と`Size`をpaddingなしで並べ、argument countと各argumentのbyte lengthも`Size`で表す。
file format、network protocol、hashなど幅そのものに意味がある値には、引き続き`UInt32`や`UInt64`を使う。

generated C headerは`mal_Size_t`を`size_t`のtypedefとして公開する。LLVM moduleとC shimは同じtargetを使い、compilerは
Cの`size_t`幅とLLVM default address spaceのpointer index幅が一致することをABI admission時に検証する。

## Alignment

memory representationを持つ型`T`に、targetのABI alignmentを返すconstantを追加する候補を採る。

```mal
T.alignment :: Size
```

通常のoperationはalignmentを要求しない現在のcontractを維持する。別に、programmerが`pointer`のaddressを
`T.alignment`の倍数だと保証するoperationを追加する。

```mal
T.load          :: Ptr -> T
T.store         :: (Ptr, T) -> Unit
T.aligned.load  :: Ptr -> T
T.aligned.store :: (Ptr, T) -> Unit
```

aligned operationはalignmentを検査せず、通常版と同じmemory representationを読み書きする。追加preconditionを満たさない
programはregion、permission、lifetime違反と同じcontract違反であり、実行結果を保証しない。LLVM backendは通常版を
`align 1`、aligned版を`align T.alignment`のload/storeへ変換できる。alignmentは性能hintではなくcode generatorが
正しく申告しなければならない保証である。

programmerは、alignmentを保証するallocatorまたは`extern` resultをbaseとし、加えるbyte offsetが`T.alignment`の倍数で
あることから派生pointerのalignmentを示せる。連続する`T`を`T.size`間隔で置く場合は、少なくとも
`T.size mod T.alignment == 0`をmemory layoutのinvariantにする。

`T.aligned.load`と`T.aligned.store`は一般のmember、namespace、propertyを導入せず、memory primitive専用のatomicな
qualified expressionとする。従来のmemory functionと同様にfirst-class functionとして扱う。

## Backendの入力

LLVM backendはtarget data layoutから次を別々に取得する。

- default address spaceのpointer representation幅
- default address spaceのpointer index幅
- 各memory representationのABI alignment

pointer representation幅とindex幅は一致するとは限らない。LLVMのdata layoutではpointer specificationが両方を別に持ち、
GEPのinteger operandはindex幅へsign extensionまたはtruncationされる。alignment付きload/storeのalignmentは2の冪で、
過大申告はundefined behaviorになる。正確なLLVM contractは
[`Data Layout`](https://llvm.org/docs/LangRef.html#data-layout)、
[`getelementptr`](https://llvm.org/docs/LangRef.html#getelementptr-instruction)、
[`load`](https://llvm.org/docs/LangRef.html#load-instruction)、
[`store`](https://llvm.org/docs/LangRef.html#store-instruction)を正とする。

現行compilerはpointer representation幅を読む一方、pointer index幅を独立に保持せず、byte offsetを`i64`のGEPへ変換する。
scalar alignmentもtarget data layoutから取得せず型幅から構成する。試案の採択時には三つのtarget propertyを分離し、
LLVM moduleとC shimが共有するABI planで検証する。

## この試案に含めない判断

productとsumへcanonicalなexternal memory representationを与えること、aggregateの`T.size`、`T.alignment`、load/storeを
公開することはこの試案に含めない。これらはpadding、tag、inactive payload、managed valueを含む型のadmissionとownershipを
別に決める必要がある。

typed pointer、alignmentを型で証明するcapability、bounds、allocator、atomic accessも追加しない。`Ptr`は引き続き
要素型、length、ownership、alignment証明を持たず、aligned operationのpreconditionはprogramとhost contractが所有する。
