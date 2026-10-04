# C runtime extension ABI

Status: Accepted ABI 0x000a00 for mal v0.7

この文書はmal v0.7の`malc`が生成するC runtime extension interfaceを定める。`0x000a00`はC ABI自体のversionであり、source
languageのversionではない。source-level semanticsは[`extern`](extern.md)、managed valueは[Engram](engrams.md)と
[`Buffer`](memory.md)を正とする。

## Build model

toolchainはprogram非依存の`mal.h`を提供する。`malc emit header file.mal`はrequire graphを検査し、指定source fileが所有する
C interfaceをfile headerとして生成する。file headerは`mal.h`と、直接requireした`.mal` fileに対応するfile headerをincludeする。
C implementationは自身を所有するfile headerをincludeし、生成artifactと同じtarget ABI、C11 compiler、compile optionでbuildする。

`build`では必要なfile interfaceを持つ内部umbrella header、LLVM module、C shim、runtime、requireされたC sourceを構成する。今回生成した
umbrella headerをC translation unitへ先に読み込み、host sourceの隣にある保存済みfile headerが生成物を置き換えない。既存library、
object、archive、include path、macroは`malc`の明示的なbuild optionから渡し、source-level `require`はC package discoveryを行わない。

`mal.h`とgenerated headerは次を検査し、異なるversionを組み合わせない。

```c
#define MAL_C_ABI_VERSION 0x000a00u
```

ABIはcompiler/runtimeとのexact matchだけを保証する。version間のsource compatibilityとbinary compatibilityは保証せず、compiler更新後は
generated headerとC sourceを同じartifactとして再compileする。runtime `dlopen`、extension discovery、unload protocolは持たない。

`main :: Unit -> Int32`は`main(void)`へ、`main :: Buffer<Symbol> -> Int32`は`main(int, char **)`へlowerする。後者ではruntimeが
`argv + 1`の各C stringを終端NULを除くSymbolとして保持するmanaged Bufferを構成し、mal executionへ渡す。

## Common runtime header

`mal.h`はtarget-independentな名前と、そのartifactで使うtarget C ABIに従う公開carrierを宣言する。少なくとも次を含む。

- `MalContext`、Symbol carrier、Buffer handle
- `MalType_Unit`、numeric scalar、`ByteSize`、`USize`、`MalType_Symbol`、Buffer handle
- allocation、trap、owner share/drop、byte owner、Bufferのruntime operation
- `mal_call_t`とcommon carrierのreturn helper

Symbol fieldとBuffer element storageはC implementationから直接参照、変更できる。Buffer object、owner header、byte ownerの内部layoutは
runtime implementation detailのまま、`mal.h`はdata/countと構築、growth、share/dropのoperationを公開する。helperは安全facadeではなく、
runtime invariantを正しく構成するcanonical operationである。公開pointerを不正にcastした操作、live elementのraw overwrite、owner、
count、data pointer、lifecycle callbackの破壊後の挙動は保証しない。

fixed-width integerは`stdint.h`の対応幅、`Float32`はbinary32 `float`、`Float64`はbinary64 `double`、`ByteSize`と`USize`はtarget
pointer index幅の`size_t`を使う。generated headerはsize、floating-point format、subnormal、`FLT_EVAL_METHOD`をcompile-time assertionで
検証する。C bodyはround-to-nearest, ties-to-evenを保持し、flush-to-zeroやdenormals-are-zeroを有効にしてreturnしてはならない。

## Program-specific carrier

generated file headerは、そのfileのextern signatureから到達するconcrete runtime carrierと、宣言元fileが所有するaliasとfile-local
opaque representationを出す。source aliasがあれば`mal_<Alias>_t`、anonymous aggregateにはstructural fingerprintを持つ名前を使う。
別fileの宣言追加やgraph load orderでfingerprintを変えない。

primitive、Symbol、Buffer、product、sum、external opaque typeはLLVM moduleと同じruntime carrier layoutを使う。productはsource orderの
field、sumはtagとvariant payload、Symbolはowner、active data、length、Bufferはstable runtime objectへのpointerである。generated
headerはpointerとindex幅、Symbolのsizeとfield offsetを`_Static_assert`し、aggregateは同じtarget C ABIのrecord layoutをLLVM側の
layout planにも使う。`mal_<Alias>_t`とcompiler-facing `MalType_<Alias>`の間に生成するfield-wise helperはCのnominalなrecord型を
接続するだけで、別のwire encodingやcanonical host memory layoutを導入しない。

`mal_false`と`mal_true`だけがvalidなBool carrierである。sum tagはvariantの0-based indexである。Cがinvalid Bool、sum tag、owner、
Symbol viewを構成した後の挙動は保証しない。constructorとprojection helperはvalid carrierを作る便宜であり、境界validationではない。

external opaque typeはone-machine-word carrierであり、generated `mal_<T>_from_bits(uintptr_t)`と`mal_<T>_to_bits(value)`でlosslessに
変換する。resourceのallocate、clone、close、free、bit pattern validityはoperation固有contractが定める。

## Host operation

各external operationには`MAL_HAS_EXTERN_<name>`と`MAL_DEFINE_<name>`を生成する。C implementationは`MAL_DEFINE_<name>`だけでbodyを
定義し、compiler-facing wrapperを直接定義しない。宣言されたoperationはapplicationの有無にかかわらずすべて定義する。未実装の
operationはstubのようにtrapするbodyで定義する。

```mal
extern appendNewline :: Buffer<UInt8> -> Buffer<UInt8>;
```

```c
MAL_DEFINE_appendNewline(call, buffer) {
    mal_Buffer_UInt8_push(call, buffer, UINT8_C('\n'));
    return mal_Buffer_UInt8_return_move(call, mal_Buffer_UInt8_share(call, buffer));
}
```

body parameterは先頭の`mal_call_t *call`と、source-level parameterがUnitでない場合の一つのruntime carrierである。productもflattenせず
by-valueの一carrierとして渡す。managed leafはcallerがbody完了まで保持するborrowであり、carrier自体のC copyは新しいresponsibilityを
作らない。

bodyはresult型に対応するterminal return helperで一度完了する。managed resultではhostが所有するresponsibilityをhelperへmoveし、
helper後に使用またはdropしない。Unit、scalar、trivial aggregateのhelperも同じbody shapeを保つ。sumはvariant-specific constructorと
terminal helperを生成する。空直和はnormal return helperを持たない。

`mal_call_t`は同期call中のruntime capabilityである。runtime allocation、share、trapに利用できるが、program固有continuation、現在の
control state、Cからmal closureをapplicationするauthorityを持たない。pointerまたは内部stateをcall後に保持しない。

## Lifecycle helper

`mal.h`はSymbolとBufferについて`share`、`drop`、`return_move`を提供する。productはmanaged field、sumはactive payloadだけへhostが
再帰し、trivial fieldにはoperationを行わない。Cがparameterをcall後も保持する場合はbody中に独立したresponsibilityを作り、後の同じ
thread上のhost operationかhost cleanupでdropする。保存したcarrierだけではlifetimeを延長しない。

`mal.h`はBufferのdata pointer、count、trivial element用make/newと、retain/release callbackを受け取るmanaged element用
make/new-moveを提供する。hostはcarrier fieldとleaf helperを使って型別callbackを実装できる。growthし得るoperation後は以前のelement
pointerを使用せず再取得する。managed elementのraw overwrite、countを進める前の未初期化place公開、runtime以外によるBuffer
object/data allocationのfreeはcontract違反である。

## Failure、effect、concurrency

allocation failure、target sizeで表現できないlayoutやlength、hostが明示したrecover不能failureはtrapする。`mal_call_trap`はprocessを
終了し、一般的なstack unwindingとrollbackを行わない。invalid carrierを境界で検査してtrapへ変換する保証はない。

C bodyは引数から到達するmanaged identity、以前shareして保持したidentity、外部stateを自由に観測、変更できる。generated wrapperと
LLVM moduleはextern callを未知のmemory clobberとして扱い、effect順序を保持する。

runtime contextとmanaged valueはthread-confinedである。同じcall capabilityまたはmanaged carrierへ複数threadから同時にaccessしては
ならない。worker threadへexternal bytesを渡す場合もbody return前にjoinし、managed resultは元のthreadで構成する。

`mal_`と`MAL_` prefixはgenerated headerとruntime用に予約する。uppercaseはmacro、lowercaseは型、function、constantに使う。
採択理由は[D098](../history/decisions/active/D098.md)に記録する。
