# C runtime extension ABI

Status: Accepted ABI 0x000a00 for mal v0.7

この文書はmal v0.7の`malc`が生成するC runtime extensionのbuild、header、call boundaryを定める。C carrier、responsibility、
storage descriptor、公開primitiveは[C host value API](c-host-api.md)、source-level semanticsは[`extern`](extern.md)、実装方法は
[C runtime extension guide](../guide/c-runtime-extension.md)を正とする。

## Build model

toolchainはprogram非依存の`mal.h`を提供する。`malc emit header file.mal`はrequire graphを検査し、指定source fileが所有する
C interfaceをfile headerとして生成する。file headerは`mal.h`と、そのC interfaceが参照するaliasまたはexternal typeを所有する、
直接requireした`.mal` fileのfile headerをincludeする。C interfaceから参照しないsource-level requirementはincludeしない。
C implementationは自身を所有するfile headerをincludeし、生成artifactと同じtarget ABI、C11 compiler、compile optionでbuildする。

`build`では必要なfile interfaceを持つ内部umbrella header、LLVM module、C shim、runtime、requireされたC sourceを構成する。今回生成した
umbrella headerをC translation unitへ先に読み込み、host sourceの隣にある保存済みfile headerが生成物を置き換えない。既存library、
object、archive、include path、macroは`malc`の明示的なbuild optionから渡し、source-level `require`はC package discoveryを行わない。

`mal.h`とgenerated headerは次を検査し、異なるversionを組み合わせない。

```c
#define MAL_C_ABI_VERSION 0x000a00u
```

`0x000a00`はC ABI自体のversionであり、source languageのversionではない。ABIはcompiler/runtimeとのexact matchだけを保証する。
version間のsource compatibilityとbinary compatibilityは保証せず、compiler更新後はgenerated headerとC sourceを同じartifactとして
recompileする。runtime `dlopen`、extension discovery、unload protocolは持たない。

`main :: Unit -> Int32`は`main(void)`へ、`main :: Buffer<Symbol> -> Int32`は`main(int, char **)`へlowerする。後者ではruntimeが
`argv + 1`の各C stringを終端NULを除くSymbolとして保持するmanaged Bufferを構成し、mal executionへ渡す。

## Common runtime header

`mal.h`はtarget-independentな名前と、そのartifactで使うtarget C ABIに従う公開surfaceを宣言する。

- builtin carrier、`MalContext`、`mal_call_t`
- named、product、sumの[type form](c-host-api.md#type-form)
- [storage queryとlifecycle operator](c-host-api.md#storage-query-operator)
- allocation、trap、[SymbolとBufferのprimitive](c-host-api.md#symbol-primitive-functions)
- generated aggregate declarationとlifecycle glueが使うtemplate

generated file headerはextern signatureから到達するprogram固有carrier、closed alias、file-local opaque representation、external operationの
declarationを加える。helperは安全facadeではなく、runtime invariantを正しく構成するcanonical operationである。公開pointerの不正なcast、
live elementのraw overwrite、ownerやcallbackの破壊後の挙動は保証しない。

fixed-width integerの幅、`Float32`のbinary32、`Float64`のbinary64、`ByteSize`と`USize`のtarget pointer index幅は
[C host value API](c-host-api.md#runtime-carrier)が定める。generated headerはtarget固有のsizeとoffsetを検査し、`mal.h`はfloating-point
format、subnormal、`FLT_EVAL_METHOD`をcompile-time assertionで検査する。C bodyはround-to-nearest, ties-to-evenを保持し、
flush-to-zeroやdenormals-are-zeroを有効にしてreturnしてはならない。

## Host operation

各external operationには`MAL_HAS_EXTERN_<name>`と`MAL_DEFINE_<name>`を生成する。`MAL_DEFINE_<name>`はpublic extern functionの
signatureへ直接展開し、別のbody functionや変換wrapperを生成しない。宣言されたoperationはapplicationの有無にかかわらずすべて定義する。
未実装のoperationはstubのようにtrapするbodyで定義する。

```mal
extern appendNewline :: Buffer<UInt8> -> Buffer<UInt8>;
```

```c
MAL_DEFINE_appendNewline(call, buffer) {
    mal_push(call, buffer, UINT8_C('\n'));
    return mal_share(call, buffer);
}
```

body parameterは先頭の`mal_call_t *call`と、source-level parameterがUnitでない場合の一つのruntime carrierである。productもflattenせず
by-valueの一carrierとして渡す。managed parameterはcallerがbody完了まで保持するBorrowであり、carrier copyは新しいresponsibilityを
作らない。managed resultはC bodyからmalへ一つのresponsibilityをMoveする。正確な規則は
[Borrowとlifecycle operator](c-host-api.md#borrowとlifecycle-operator)が定める。

bodyはCの`return`で一度完了する。その場で構成したowned rvalueとtrivial valueは直接returnできる。Unitは`mal_unit`をreturnする。
空直和は正常にreturnできる値を持たない。

`mal_call_t`は`MalContext`のpublic aliasであり、extern ABIの先頭parameterとして直接渡す同期runtime capabilityである。runtime allocation、
Share、trapに利用できるが、program固有continuation、現在のcontrol state、Cからmal closureをapplicationするauthorityを持たない。
pointerまたは内部stateをcall後に保持しない。

## Failure、effect、concurrency

allocation failure、target sizeで表現できないlayoutやlength、hostが明示したrecover不能failureはtrapする。`mal_call_trap`はprocessを
終了し、一般的なstack unwindingとrollbackを行わない。invalid carrierを境界で検査してtrapへ変換する保証はない。

C bodyは引数から到達するmanaged identity、以前Shareして保持したidentity、外部stateを自由に観測、変更できる。LLVM moduleは
extern callを未知のmemory clobberとして扱い、effect順序を保持する。

runtime contextとmanaged valueはthread-confinedである。同じcall capabilityまたはmanaged carrierへ複数threadから同時にaccessしては
ならない。worker threadへexternal bytesを渡す場合もbody return前にjoinし、managed resultは元のthreadで構成する。

`mal_`と`MAL_` prefixはgenerated headerとruntime用に予約する。uppercaseはmacro、lowercaseは型、function、constantに使う。
採択理由は[D098](../history/decisions/active/D098.md)に記録する。
