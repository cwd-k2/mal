# EngramとExtern

Status: Accepted v0.7

## Authority

Engramはsource-levelの型名ではなく、mal program内で有効な値として構成されたものの総称である。`Unit`、numeric scalar、`Symbol`、
`Buffer`、product、sum、function valueはEngramであり、そのidentity、区別、到達可能性、source-level lifetime semanticsはmalが定める。
実装がstatic storage、reference counting、arena、region、tracingのどれを使うかは、観測できない限りsource semanticsではない。

Externはfilesystem、network、process、device、host allocationなどmal外部のstate、storage、resource、作用の領域である。external opaque
valueはExtern resourceへのcopyableなnominal carrierであり、copyしてもreferentのlifetimeを延長しない。close、free、permission、
alias、failureは個々のextern contractが定める。

EngramとExternはC境界へ出せる型を分ける分類ではない。extern Cはmal runtime implementationへ参加し、runtime carrierを直接扱う。
Engramの意味をmalが定めることと、C implementationがその表現を構築、観測、変更できることは両立する。host contractを破ってinvalid
carrierやlifecycleを作った後のbehaviorは保証しない。

## Managed responsibility

`Symbol`、`Buffer`、function closureと、それらを含むaggregateはmanaged responsibilityを持つ。mal sourceはretain、release、freeを
操作せず、compilerとruntimeが到達可能な値を保持し、不要になったresponsibilityを終了する。Buffer elementとして保持したmanaged
valueもBuffer placeのlifetime中保持する。

extern Cへ渡すmanaged parameterはcall中のborrowである。Cがcall後も保持するならruntime shareで独立したresponsibilityを作り、後に
dropする。managed resultはCからmalへ一つのresponsibilityをmoveする。このC lifecycle operationはsource primitiveではなく、同じ
artifactへ静的linkするruntime extension contractである。

Extern resourceのlifetimeはEngramの回収へ自動的に連動しない。external opaque carrierをSymbol、Buffer、aggregateへ格納しても、
referentのcloseやfreeを引き起こさず、lifetimeを延長しない。runtime-managedなnative resource typeを追加する場合は、そのdrop effectと
identityを新しいEngram leafとして明示的に定める。

## Representationとoperation

mal compilerはspecialization後のconcrete typeへruntime carrier layoutとmanaged lifecycleを与える。同じartifactのgenerated C headerは
このlayoutを、`mal.h`はSymbol/Bufferのshare/dropとmanaged Buffer callbackの構成要素を公開する。Cが値を構築することは新しいsource
constructorを追加せず、既存型のvalid carrierをruntimeへ渡すだけである。

productとsumのlifecycleはfieldごとに再帰する。sumはactive payloadだけを保持する。Buffer handleのcopyは同じmutable identityを指し、
Symbol carrierのcopyは同じimmutable byte値を与える。external opaque carrierのcopyはresourceをcloneしない。

function valueはEngramだがextern signatureへ現れない。closureをCへ渡すにはcode、environment、responsibility、callback中のexecution
control、C activationへのresumeを定める必要があり、raw representation公開だけから導けないためである。

## 外部memory

mal sourceはraw pointerと汎用dereference operationを持たない。同期C libraryやsystem callがpointerを要求する場合、C bodyがcall中に
SymbolまたはBufferのruntime carrierから取得する。kernelやlibraryがcall後も参照するstorageは、pin、completion、destructorを持つ
resource固有のexternal opaque typeまたはruntime-managed native objectで表す。

file、network、永続storageのencodingはruntime carrier layoutと同一視しない。program固有のformatはmalまたはCで明示的にencode、
decodeする。runtime layoutを別artifactへ保存し、後でEngramとして復元するportable contractは提供しない。

## Lifetimeとfailure

Engramがいつ回収可能になるかはmal compilerとruntimeが決める。source programが観測できるのは、到達可能な値の意味と共有identityが
保持されることだけである。extern Cが正しいshareを持つ値も到達可能として扱う。

allocation failureなど有効なEngram構築を完了できない場合はtrapする。extern Cがinvalid owner、tag、count、lifecycleを作った場合は
source-level trapを保証せず、その後のbehaviorを保証しない。resource固有failureをsum result、trap、process terminationのどれへ
写すかはextern contractが定める。

正確なextern signatureは[`extern`](extern.md)、C representationは[C runtime extension ABI](c-host-abi.md)、Buffer lifecycleは
[`Buffer`](memory.md)を正とする。採択理由は[D098](../history/decisions/active/D098.md)に記録する。
