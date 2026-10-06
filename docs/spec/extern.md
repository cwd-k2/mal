# `extern`境界

Status: Accepted v0.7

## 目的

I/O、filesystem、network、clock、randomness、process、thread、system call、host resource、native codecなど、mal sourceだけで
実装しないoperationはprogram固有の`extern` declarationに置く。C implementationは同じcompiler/runtime revisionへ結合する
runtime extensionであり、mal valueのruntime carrierを直接構築、観測、変更できる。

```mal
extern File;
extern open :: Symbol -> [UInt32, File];
extern read :: (File, USize) -> [UInt32, Buffer<UInt8>];
extern write :: (File, Buffer<UInt8>) -> [UInt32, USize];
extern close :: File -> Unit;
```

external operationは通常のtop-level function valueとしてscopeへ入る。参照、binding、受け渡しだけではhost operationを実行せず、
applicationしたときに宣言されたC bodyを一度呼ぶ。local bindingによるshadowingも通常のlexical scopeに従う。

malはeffect systemを持たず、通常の関数型はpure/impureを区別しない。compilerはextern applicationを外部stateと未知のmemoryを
観測、変更し得るoperationとして扱い、sourceで観測できる評価順序を変えてはならない。

## Admitted type

extern parameterとresultはaliasを展開し、file-local opaque typeをhidden representationへ正規化した後、closed concrete typeで
なければならない。次を直接または再帰的に含められる。

```text
Unit、Bool、numeric scalar、ByteSize、USize
external opaque type
Symbol
Buffer<A> where Storable(A)
productとsum
```

function型を直接またはaggregate、Buffer element、opaque representationを通して含めてはならない。Cからmal closureを呼んで元の
C activationへ戻るcallback ABI、environment responsibility、program固有continuationを定めないためである。empty sumはparameterや
aggregate memberとしてcarrierを持てるがvalid valueを持たず、正常なresultとして構築できない。

generic extern declarationは認めない。generic bindingはwhole-program specializationで単相化されるが、一つのC definitionへopen type
parameterを渡すruntime descriptorは存在しない。利用者は必要なconcrete external operationを宣言するか、genericな処理をmalで書く。

このadmissionはhost implementationの安全性を示さない。C bodyがruntime contractを破った後のprogram behaviorは保証しない。

## Source-level semantics

applicationではargumentを通常の式と同じく左から右へ評価し、continuationであるcalleeをその後に評価する。host bodyが正常に
resultを一度返した後、その型のmal valueを得たものとして評価を続ける。host bodyがtrapまたはprocess terminationしたpathは
正常resultを返さない。

scalar、product、sum、Symbol descriptorのcarrier自体はby-valueでC bodyへ渡す。Bufferは共有identityへのhandleであり、productや
sumに含まれるmanaged leafも同じreferentを指す。Cがborrow中にBuffer identityを変更すれば、malのaliasから変更を観測できる。
Symbolのbyte値はimmutableであり、Cが完成済みstorageを書き換えた後の挙動は保証しない。

## Managed responsibility

managed parameterはhost bodyが正常returnまたはtrapするまでcallerが保持するborrowである。C bodyはborrow responsibilityをdropしては
ならない。call後も値を保持する場合は`mal.h`のSymbol/Buffer share operationをmanaged leafへ再帰的に適用し、独立したresponsibilityを
C-owned storageへ保存する。保存したresponsibilityは同じruntimeとthreadのcontractに従って後にdropする。carrier bitsだけのcopyは
lifetimeを延長しない。

managed resultはC bodyが所有する一つのresponsibilityをCの`return`でmalへmoveする。owned localは`mal_move`して元をvacantにし、
その場で構成したowned rvalueは直接returnする。productとsumではactiveなmanaged leafへ再帰的にこの規則を適用する。hostがruntime allocationや別のshareで
取得し、resultにもC-owned storageにも渡さなかったtemporary responsibilityはhostがdropする。

`mal.h`とgenerated headerは`mal_share`、`mal_move`、`mal_drop`、storage contractを保持するBuffer operationを提供し、aggregate lifecycleは
compilerと同じconcrete type recursionから生成する。productとsumは公開fieldをC initializerで直接構成する。二重move、borrowのdrop、
live placeのraw overwrite、invalid ownerなどcontract違反後の結果は保証しない。

## External opaque type

external opaque typeはone-machine-wordのcopyable carrierである。malのcopy、binding、discard、Buffer storageはcarrier bitsだけを扱い、
resourceのallocate、retain、release、close、freeを暗黙に実行しない。Cは`mal_from_bits(mal_type(T), bits)`と`mal_to_bits(value)`で`uintptr_t`へlosslessに
変換できる。zeroを含むvalid bit pattern、resource identity、permission、lifetime、failureはoperation固有contractが定める。

file、socket、mapping、device allocationなどcall後にも存在するhost resourceはnominalなexternal opaque typeで表せる。raw pointerを
sourceへ公開する万能型と、任意memoryをdereferenceするpredefined operationは持たない。runtime-managed native objectを新しい
source typeとして追加する場合は、その型のidentityとshare/dropを別途仕様化する。

## Host contractとfailure

型だけではoperation固有の意味を定義できない。C implementationとそのmal-facing APIは少なくとも次を定める。

- resource、byte encoding、system call、partial transferの意味
- argumentとして受け取るshared identityへの変更
- resultに含めるexternal resourceのpermissionとlifetime
- recoverable failureをsum result、trap、process terminationのどれへ写すか
- hostがshareして保持するmanaged valueと、そのdrop point
- temporary external resourceとmanaged responsibilityのcleanup

`mal_trap`はrecover不能なfailureでprocessを終了する。現在のruntimeはtrapから回復しないため一般的なstack unwindingとrollbackを
提供しない。hostはtrapし得るhelperより前に取得したtemporary external resourceを残さない構成にするか、operation固有のcleanupを
行う。contract違反を境界で検査することは要求しない。

## Concurrency

runtime contextとmanaged valueはthread-confinedである。C bodyは同じcall capabilityまたはmanaged carrierへ複数threadから同時に
accessしてはならない。worker threadがexternal bytesだけを処理する場合も、managed resultの構築とreturnは元のruntime threadで行う。
cross-thread managed sharing、async callback、Cからmalへのreentryはこの境界に含まれない。

正確なC representationは[C host value API](c-host-api.md)、build規則は[C ABI](c-host-abi.md)、Buffer lifecycleは[`Buffer`](memory.md)、値のauthorityは
[EngramとExtern](engrams.md)を正とする。採択理由は[D098](../history/decisions/active/D098.md)に記録する。
