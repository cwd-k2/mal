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
- `mal_type(Unit)`、numeric scalar、`ByteSize`、`USize`、`mal_type(Symbol)`、`mal_type(Buffer)`
- `mal_product(T, ...)`、`mal_sum(T, ...)`と、named closed typeに対する`mal_type(Name)`
- allocation、trap、storage descriptor、SymbolとBufferのruntime operation
- `mal_call_t`、`mal_share`、`mal_move`、`mal_drop`とoptional cleanup用の`mal_owned(Name)`

Symbol fieldとBuffer element storageはC implementationから直接参照、変更できる。Buffer object、owner header、byte ownerの内部layoutは
runtime implementation detailのまま、`mal.h`はdata/countと構築、growth、share/dropのoperationを公開する。helperは安全facadeではなく、
runtime invariantを正しく構成するcanonical operationである。公開pointerを不正にcastした操作、live elementのraw overwrite、owner、
count、data pointer、lifecycle callbackの破壊後の挙動は保証しない。

fixed-width integerは`stdint.h`の対応幅、`Float32`はbinary32 `float`、`Float64`はbinary64 `double`、`ByteSize`と`USize`はtarget
pointer index幅の`size_t`を使う。generated headerはsize、floating-point format、subnormal、`FLT_EVAL_METHOD`をcompile-time assertionで
検証する。C bodyはround-to-nearest, ties-to-evenを保持し、flush-to-zeroやdenormals-are-zeroを有効にしてreturnしてはならない。

## Program-specific carrier

generated file headerは、そのfileのextern signatureから到達するconcrete runtime carrierと、宣言元fileが所有するclosed aliasと
file-local opaque representationを出す。source aliasには`mal_type(Alias)`を用意し、anonymous aggregateは
`mal_product(T, ...)`または`mal_sum(T, ...)`で参照できる。実在するC identifierにはstructural fingerprintを使うが、host bodyは
そのmanglingを組み立てない。別fileの宣言追加やgraph load orderでfingerprintを変えない。

C type spellingはaliasとfile-local opaque representationを展開した後のruntime carrierを表し、Malのsurface type applicationを再現しない。
transparent generic aliasは展開するため`mal_generic`やaliasごとの`*_of`を生成しない。sourceで名前を与えたclosed aliasだけは
canonical carrierと互換な`mal_type(Name)`として残す。すべての`Buffer<T>`は`mal_type(Buffer)`へ写し、TはBuffer生成時に渡すstorage
contractにだけ残す。

primitive、Symbol、Buffer、product、sum、external opaque typeはLLVM moduleと同じruntime carrier layoutを持つ単一のC carrierを使う。productはsource orderの
field、sumはtagとvariant payload、Symbolはowner、active data、length、Bufferはstable runtime objectへのpointerである。generated
headerはpointerとindex幅、Symbolのsizeとfield offsetを`_Static_assert`し、aggregateは同じtarget C ABIのrecord layoutをLLVM側の
layout planにも使う。compiler-facing bridgeとhost bodyの間に別のnominal raw carrier、field-wise変換、return helperを置かない。
別のwire encodingやcanonical host memory layoutは導入しない。

`mal_false`と`mal_true`だけがvalidなBool carrierである。productは`.field_N`、sumは`.tag`と`.payload.variant_N`を持ち、sum tagはvariantの
0-based indexである。C bodyはcompound literalまたはinitializerでaggregateを直接構成する。Cがinvalid Bool、sum tag、inactive payload、
owner、Symbol viewを構成した後の挙動は保証しない。専用のproduct constructorやsum injection helper、境界validationは提供しない。

external opaque typeはpublicな`.bits` fieldを持つone-machine-word carrierであり、共通macro
`mal_from_bits(mal_type(T), bits)`と`mal_bits(value)`で`uintptr_t`へlosslessに変換する。generated headerは型別変換関数を生成しない。
resourceのallocate、clone、close、free、bit pattern validityはoperation固有contractが定める。

## Host operation

各external operationには`MAL_HAS_EXTERN_<name>`と`MAL_DEFINE_<name>`を生成する。`MAL_DEFINE_<name>`はpublic extern functionの
signatureへ直接展開し、別のbody functionや変換wrapperを生成しない。宣言されたoperationはapplicationの有無にかかわらずすべて定義する。未実装の
operationはstubのようにtrapするbodyで定義する。

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
by-valueの一carrierとして渡す。managed leafはcallerがbody完了まで保持するborrowであり、carrier自体のC copyは新しいresponsibilityを
作らない。

bodyはCの`return`で一度完了する。owned localをmanaged resultへ渡す場合は`return mal_move(value);`、borrowから独立したresultを作る場合は
`return mal_share(call, value);`とする。その場で構成したowned rvalueとtrivial valueは直接returnできる。Unitは`mal_unit`をreturnする。
空直和は正常にreturnできる値を持たない。

`mal_call_t`は`MalContext`のpublic aliasであり、extern ABIの先頭parameterとして直接渡す同期runtime capabilityである。runtime allocation、
share、trapに利用できるが、program固有continuation、現在のcontrol state、Cからmal closureをapplicationするauthorityを持たない。
pointerまたは内部stateをcall後に保持しない。

## Lifecycleとstorage

公開helperはtype form、storage query operator、lifecycle operatorという別の層を表す。

### Type form

| form | 表すもの | responsibility |
|---|---|---|
| `mal_type(T)` | named closed Mal型のC runtime carrier型 | 持たない |
| `mal_owned(T)` | 同じcarrier型にlexical cleanupを付けたowned localの宣言形 | initializerから一つ受け取る |

`mal_owned(Name)`は`mal_type(Name)`と異なるwrapper型ではない。managed responsibilityを持つ名前付きclosed typeのC localへoptionalな
lexical cleanupを付ける。builtin managed leafは`mal.h`のcleanupを使い、generated headerはmanaged aggregate representationごとのcleanupと
named aliasからそれへの一行macro mappingだけを生成する。scope終了、early return、明示drop、moveを同じvacant規則で扱うが、
`mal_call_trap`はstack unwindingしない。external opaque resourceやtrivial typeへ`mal_owned`を提供せず、正しさをautomatic cleanupだけへ
依存させない。

### Storage query operator

`mal_storage(T)`はstorageのallocation、carrierの格納、Share、Move、Dropを実行しない。値やresponsibilityを含まないstaticな説明であり、
type-erasedなBuffer objectが後のoperationでcarrierを扱うために保持する。具体的なoperationがeffectを選び、copyはdescriptorの`share`、
place終了は`drop`を呼び、格納とrelocationのMoveはcallbackを呼ばずcarrierを移す。typedな`mal_owned(T)` localではC compilerがcleanupを
静的に選べるため、local自身がstorage descriptorを保持するわけではない。

`mal_storage(T)`はCのoperatorではなくdescriptor値を返すmacroだが、利用上の位置づけは`sizeof(T)`や`_Alignof(T)`に近い。型から情報を
取り出す式であり、storage objectや要素へのpointerを返さない。`sizeof`と`_Alignof`に加えて、その型をstorage内でShare / Dropするcallbackも
まとめたものと捉えられる。

### Borrowとlifecycle operator

Borrowにはoperatorがない。managed extern parameter、primitive表でborrowと記したoperand、responsibilityを持つ値の有効期間内だけ使う
carrier copyは、いずれも新しいresponsibilityを作らない通常のCの値として渡す。Share、Move、Dropだけが明示的なlifecycle operatorである。

| mode | C上の形 | sourceの状態 | resultのresponsibility |
|---|---|---|---|
| Borrow | 通常のparameter渡し、観測、carrier copy | liveのまま | 作らない |
| Share | `mal_share(call, value)` | liveのまま | 新しい一つを作る |
| Move | `mal_move(value)` | vacantになる | sourceの一つを移す |
| Drop | `mal_drop(value)` | vacantになる | 返さず一つを終了する |

Copyは独立したlifecycle modeではない。trivial carrierは通常のC assignmentでbitsをcopyし、managed carrierを独立して保持するための論理的な
copyはShareとして新しいresponsibilityを作る。Shareは同じmanaged identityを共有し、汎用的なdeep cloneを意味しない。Buffer primitiveの
`mal_copy`はrange名であり、汎用value copy operatorではない。trivial elementにはbitsのcopy、managed elementには要素ごとのShareを行う。

borrowしたcarrierはsourceを保持するresponsibilityより長く使わず、`mal_move`、`mal_drop`、owned operandをconsumeするprimitiveへ渡さない。
call後のC-owned storageへ保存する場合やmanaged resultとして返す場合は、borrow中に`mal_share`して独立したresponsibilityを作る。
carrier bitsだけをcopyしてもsourceのlifetimeは延びず、そのcopyをowned valueとして扱えない。C compilerとruntimeはこの区別を追跡しないため、
違反後の挙動は保証しない。

`mal_move`と`mal_drop`はowned lvalueだけを受け、一度だけ評価する。productではmanaged field、sumではactive payloadだけへgenerated glueが
再帰し、trivial fieldにはoperationを行わない。Cが保持したresponsibilityは、後の同じthread上のhost operationかhost cleanupで
`mal_drop`する。

### Symbol primitive functions

| primitive | source responsibility | result / effect |
|---|---|---|
| `mal_symbol(call, source, length)` | `source`が指すbyte rangeをcall中だけborrow | byteをcopyした独立したowned Symbolを返す |
| `mal_snapshot(call, buffer)` | `Buffer<UInt8>`とそのactive elementをborrow | 後のBuffer mutationから独立したowned Symbolを返す |

どちらもsourceのlifetimeを延長せず、返したSymbolがcopy先のbytesを所有する。`mal_snapshot`の`buffer`は
`mal_storage(mal_type(UInt8))`で構築されていなければならない。

### Buffer primitive functions

Buffer primitiveは生成時に明示されたstorage contractをobjectへ保持し、以後の要素operationで利用する。表中の`buffer`、`source`、
`destination`はborrowであり、Buffer自体のresponsibilityを移さない。

| primitive | element responsibility | result / effect |
|---|---|---|
| `mal_buffer(call, storage, capacity)` | `storage`を値として保持 | count 0、指定capacityのowned Bufferを返す |
| `mal_data(buffer)` | なし | element storageへの`void *`をborrowする |
| `mal_count(buffer)` | なし | 現在のelement countを返す |
| `mal_push(call, buffer, element)` | owned `element`を末尾へMove | 追加したindexを返し、countを1増やす |
| `mal_replace(call, buffer, index, element)` | 既存要素をDropし、owned `element`をMove | countを変えず指定indexを置換する |
| `mal_fill(call, buffer, offset, count, element)` | owned `element`をconsumeし、必要なら追加分をShare。既存要素はDrop | `[offset, offset + count)`を同じ値で埋め、末尾を越える場合はcountを伸ばす |
| `mal_copy(call, destination, destination_offset, source, source_offset, count)` | source rangeをborrowし、managed要素の新しいresponsibilityをdestinationへ作る。置換される要素はDrop | Buffer間でrangeをcopyし、必要ならdestinationのcountを伸ばす |
| `mal_append(call, buffer, source, count)` | `source`が指すelement rangeをborrowし、managed要素の新しいresponsibilityを作る | rangeを末尾へ追加する |
| `mal_extend(call, buffer, count)` | trivial elementだけを許可 | countを増やし、未初期化の新しい末尾への`void *`を返す |
| `mal_truncate(buffer, count)` | 取り除くmanaged要素をDrop | countを縮める。現在値以上なら何もしない |
| `mal_reserve(call, buffer, capacity)` | なし | element capacityを確保し、countは変えない |

`mal_copy`の二つのBufferは同じstorage contractを持たなければならない。`mal_append`のsource pointerはdestination Bufferのstorage contractと
一致するelementを`count`個指さなければならない。registry、型名検索、dynamic type equalityはないため、runtimeが異なるcontractを
照合するわけではない。

growthし得るoperation後は以前のelement pointerを使用せず再取得する。descriptorと異なるpointer型でのaccess、異なるdescriptorを持つ
Buffer間のcopy、managed elementのraw overwrite、`extend`で作った未初期化placeの観測、runtime以外によるBuffer object/data allocationの
freeはcontract違反である。

## Failure、effect、concurrency

allocation failure、target sizeで表現できないlayoutやlength、hostが明示したrecover不能failureはtrapする。`mal_call_trap`はprocessを
終了し、一般的なstack unwindingとrollbackを行わない。invalid carrierを境界で検査してtrapへ変換する保証はない。

C bodyは引数から到達するmanaged identity、以前shareして保持したidentity、外部stateを自由に観測、変更できる。LLVM moduleは
extern callを未知のmemory clobberとして扱い、effect順序を保持する。

runtime contextとmanaged valueはthread-confinedである。同じcall capabilityまたはmanaged carrierへ複数threadから同時にaccessしては
ならない。worker threadへexternal bytesを渡す場合もbody return前にjoinし、managed resultは元のthreadで構成する。

`mal_`と`MAL_` prefixはgenerated headerとruntime用に予約する。uppercaseはmacro、lowercaseは型、function、constantに使う。
採択理由は[D098](../history/decisions/active/D098.md)に記録する。
