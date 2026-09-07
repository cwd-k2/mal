# C backendのEngram ownership

Status: Current implementation contract

この文書はreference C backendがmanaged Engramの保持と解放を生成する規約を定める。source-levelの意味と
authorityは[Engram仕様](../spec/engrams.md)、hostとの受け渡しは[C host ABI](../spec/c-host-abi.md)を正とする。

## 現在の状態

v0.5が現在受理するprogramとtrusted C adapter contractの範囲では、ownership correctnessに必要なcopy、transfer、
cleanup、local owned bindingのlast-use transfer、owned direct call、consuming `Symbol` concatは実装済みである。ownershipに関する残件は、
現行contractを変えないescape analysis、region化などの最適化であり、正しさを成立させるための未実装要件ではない。

将来、managed cycle、thread間共有、host resourceの自動解放などを言語またはABIへ追加する場合は、その新しい範囲に
対するownership設計を別途行う。これは現在のv0.5 ownership実装の未完成部分ではない。

## 対象

`Symbol`とcaptureを持つfunction valueはruntime storageを参照する。productとsumはfieldを再帰的に調べ、これらを
含む場合だけmanaged valueとして扱う。`Unit`、numeric scalar、`Ptr`、external opaque value、およびcapture-free
function valueは個別に解放するstorageを持たない。

Extern resourceのownershipはこの仕組みに含めない。`Ptr`のreferentやexternal opaque handleをcloseまたはfreeする
責務はoperation固有のhost contractに属する。

## generated Cの規約

値の受け渡しは次の二つへ統一する。

- function parameter、capture fieldの読取り、既存bindingへの参照、case payloadはborrowである。
- function result、runtime operation result、bindingが保持するmanaged valueはownである。

borrowを別のbinding、aggregate field、capture、branch result、function resultへ保存するときは、型ごとのcopy operationで
ownership shareを一つ増やす。binding、branch-local pattern、closure environment、top-level storageの終端では、生成と逆順に
型ごとのdestroy operationを呼ぶ。wildcardへ渡したowned resultも直ちにdestroyする。

expression emitterは各`Operation`のC式と`ResultOwnership`を同じinterfaceで返す。分類は`Operation`、
`MemoryPrimitive`、managed resultを作り得るprimitiveをwildcardなしで列挙し、新しいvariantの分類漏れをRustの
exhaustiveness checkで拒否する。structured operationもstatement emitterでowned resultを作る規約を明示する。

`c_emit`は型付きclosure-converted IRを逆向きに走査し、lexical blockとbranchごとにlocal owned bindingの最後の使用を
求める。binding、aggregate field、function result、direct tail callの次parameterへ保存する最後の使用ではdescriptorを
transferし、sourceを型に対応するzero状態にする。既存cleanupはzero状態を安全にdestroyできるため、branchごとにtransfer位置が
異なっても共通のlexical cleanupを維持できる。同じoperationまたは後続処理でaliasを再使用する場合はcopyを残す。

通常のparameter、environment field、case payloadの読取りはborrowであり、最後の使用というだけではtransferしない。direct tail
loopが明示的にcopyして所有するparameter slotは例外であり、slot全体をdestructureするときに各fieldへownershipを分配できる。
解析とmaterializationは`c_emit/body`に閉じ、lexer、parser、language IRへbackendのlifetime policyを追加しない。

known direct callでは、callerがmanaged argument全体を所有し、そのbindingの最後の使用である場合だけowned entryへdescriptorを
transferする。owned entryのparameterはlocal owned bindingと同じlast-use規則に従い、return、aggregate、primitive、次のknown
direct callへ再transferできる。owned sum全体のlast-useである`case`はactive payloadへownershipを移す。calleeを静的に
特定できないfunction value callと、call後にもargument bindingを使う経路は
borrowed entryを維持する。これはgenerated C内部のcalling conventionであり、source typeとC host ABIには露出しない。

direct self tail callではfunction parameterをloop全体のowned slotとして保持する。各tail edgeは次のparameterを先にcopyまたは
last-use transferで確保し、そのpathでliveなbindingを内側から逆順にdestroyして現在のparameterをdestroyした後、次のparameterを
slotへtransferしてloop entryへ戻る。通常returnもresultを先にcopyまたはtransferしてから同じcleanupを行う。これによりmanaged valueを
含む場合も、参照先を早く解放せず、iterationごとのownership shareを残さず、C stackを増やさない。

## 型ごとのoperation

| 型 | copy | destroy |
|---|---|---|
| `Symbol` | ownership pointerをretain | ownership pointerをreleaseし、最後ならbytesを解放 |
| captureを持つfunction | environmentをretain | environmentをreleaseし、最後ならcaptureを逆順にdestroyして解放 |
| product | managed fieldをsource orderでcopy | managed fieldを逆順にdestroy |
| sum | active payloadだけをcopy | active payloadだけをdestroy |
| その他 | C value copy | no-op |

`Symbol` literalはstatic storageを参照しownership pointerを持たない。runtime生成Symbolのownershipはreference count付きの
flat allocationまたはrope nodeを指す。borrowed operandを受ける連結が既存descriptorを返す場合は、result contractを満たすため
retainする。last-useのowned flat operandはreference countが1なら、leftでは末尾capacity、rightでは先頭余白を再利用し、
不足時は幾何的に拡張する。static、共有中、ropeのoperandはin-placeに変更しない。

共有された大きなconcatはAVL-balanced rope nodeとして両operandをretainする。comparison、byte access、memory store、extern
parameterがbytesを要求したときだけflattenし、そのcacheはrope nodeと共に解放する。extern aggregate内のSymbolも型再帰で
materializeする。いずれの表現もsourceからは新しいimmutable byte sequenceとしてだけ観測され、node、cache、capacityは
C host ABIのopaque ownership内部に留まる。

closure valueはcode pointer、environment pointer、environment destructorの組である。destructorはcapture型を知る生成function
であり、generic reference-count runtimeはenvironment layoutを解釈しない。

call以外へ流出しないlocal closureはdescriptorをmaterializeせず、environment structをstack上に置いて外側の
bindingをborrowする。単純alias chainとclosure本体のself referenceを合わせて調べ、全referenceがcallee位置に限られる
場合だけこの表現を使う。self closureを別functionのargumentなどの値として使う場合を含め、それ以外はreference count付き
heap environmentへfallbackする。stack environmentはretainもdestroyもしない。

## programとhost境界

top-level initializerの一時値は各initializerの終了時にdestroyし、保存したtop-level値は`main`のreturn後に逆順でdestroyする。
argument descriptor列のruntime allocationもsource-level `main`のreturn後に解放する。

extern parameterはcall中だけborrowされる。managed resultの各fieldはownership shareを一つmalへtransferしなければならない。
Symbol resultは`mal_Symbol_copy_from_bytes`で作る。malはextern resultをowned valueとして受け取り、通常のbinding cleanupへ接続する。
C adapter内のclone、move、dropとaggregate constructor/accessorの規約は[C host ABI](../spec/c-host-abi.md)を正とする。

generated C自身は`MAL_CLONE`、`MAL_MOVE`、`MAL_DROP`を内部ownership primitiveとして使わない。compilerは
typed IR上のborrow/ownを静的に知り、hostへ公開されないanonymous aggregateとclosureも含めて内部copy/destroyへ
直接loweringする。ABI macroはその静的情報を持たない手書きadapterへ同じ意味契約を提供する境界APIである。

## 最適化との境界

immutabilityによりcopyはreferentの複製ではなくretainでよく、cleanup順序によって値の内容は変わらない。closureの
local-use解析や将来のregion化も、この文書のborrow/result contractを変えずに行う。

slice、hash cache、operation memoizationは値表現または計算量の最適化であり、ownershipの正しさとは分離する。rope nodeと
flatten cacheは上記のcopy/destroy contractに従う。descriptor addressの同一性はsourceから観測できず、再利用可能性もあるため、
memoization keyのsource-level意味には使わない。

測定baseline、着手順、安全条件は[managed Engram性能評価](../development/ownership-performance.md)に置く。
