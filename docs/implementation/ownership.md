# C backendのEngram ownership

Status: Current implementation contract

この文書はreference C backendがmanaged Engramの保持と解放を生成する規約を定める。source-levelの意味と
authorityは[Engram仕様](../spec/engrams.md)、hostとの受け渡しは[C host ABI](../spec/c-host-abi.md)を正とする。

## 保証範囲

v0.5が受理するprogramとtrusted C adapter contractの範囲では、各managed ownerはlocal slot、closure environment、
top-level storage、typed continuation frameのいずれか一箇所に属する。copyはownerを一つ増やし、transferは移動元をzero状態にし、
scope、activation、programの終端では対応するownerを一度だけ解放する。last-use transfer、owned direct call、consuming `Symbol` concatも
この規約の特殊化であり、別のownership authorityを持たない。

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
解析中は各`Atom` occurrenceへbackend-localなdense identityを割り当て、transfer集合はこのidentityだけを保持する。
Rust object addressはimmutable program内のoccurrenceを照合する索引に限り、last-use authorityそのものにはしない。
debug buildではemission開始前に元programからplan全体を再導出し、occurrence集合とtransfer集合のexact matchを検査する。

通常のparameter、environment field、case payloadの読取りはborrowであり、最後の使用というだけではtransferしない。direct tail
loopが明示的にcopyして所有するparameter slotは例外であり、slot全体をdestructureするときに各fieldへownershipを分配できる。
解析とmaterializationは`c_emit/body`に閉じ、lexer、parser、language IRへbackendのlifetime policyを追加しない。

known direct callでは、callerがmanaged argument全体を所有し、そのbindingの最後の使用である場合だけowned entryへdescriptorを
transferする。owned entryのparameterはlocal owned bindingと同じlast-use規則に従い、return、aggregate、primitive、次のknown
direct callへ再transferできる。owned sum全体のlast-useである`case`はactive payloadへownershipを移す。calleeを静的に
特定できないfunction value callと、call後にもargument bindingを使う経路は
borrowed entryを維持する。borrowed entryはmanaged parameterと、その冒頭でdestructureしたfieldをcopyせずに参照し、resultなどへ
escapeするときだけcopyする。これはgenerated C内部のcalling conventionであり、source typeとC host ABIには露出しない。

direct self tail callではfunction parameterをloop全体のowned slotとして保持する。各tail edgeは次のparameterを先にcopyまたは
last-use transferで確保し、そのpathでliveなbindingを内側から逆順にdestroyして現在のparameterをdestroyした後、次のparameterを
slotへtransferしてloop entryへ戻る。通常returnもresultを先にcopyまたはtransferしてから同じcleanupを行う。これによりmanaged valueを
含む場合も、参照先を早く解放せず、iterationごとのownership shareを残さず、C stackを増やさない。
すべてのtail edgeが同じslotをそのまま次状態へ渡す場合、そのslotと冒頭でdestructureしたfieldはloop中のknown direct callへ
borrowできる。slot自身のownershipとtail edgeでのtransferは維持する。

## control frame

application control loweringでhandlerがnon-tail callによりsuspendすると、callerのC activationはdispatcherへreturnする。
resume stateのlive-inにある値だけをcall-site固有frameへ保存し、top-level bindingはprogram storageから再取得する。

managed live bindingは型ごとのcopy operationでframe-owned fieldへ保存してから、activation側のinitialized slotを
destroyする。resume時はfieldをzero状態にしてownerをlocal slotへmoveし、frame自体をpopする。これにより、parameterやcase payloadも
suspension中は独立したownerを持つ。後からlast-use情報によりcopyとactivation側destroyを一つのtransferへ
まとめてよいが、frame前後のowner数を変えてはならない。

resume後にenvironment fieldまたはself closureを使う場合、frameはcaller environmentのownership shareも保持する。callee entryへ
渡すenvironmentは、caller frameとは別のshareを確保してからcurrent activationを終了する。tail applicationではcaller frameを
作らず、callee argumentとenvironmentを次entryへ移した後にcaller localをdestroyする。

call-only local closureのC stack配置は、そのclosureとborrowed captureが同じhandler activation内だけで使われる場合に限る。
closure bindingまたはそのenvironmentがsuspensionをまたぐ場合はheap environmentへfallbackする。control frameのbyte storageが
移動し得るため、frame内captureのaddressをclosure environmentとして公開する最適化は行わない。

trapではcontrol frameをunwindせず、processを直ちに異常終了する。control storageのcapacity不足は
Engram allocation trapではなくimplementation resource failureであり、managed fieldの通常cleanupを開始しない。

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

共有された大きなconcatはAVL-balanced rope nodeとして両operandをretainする。comparisonとbyte accessはropeを直接走査し、
`storeSymbol`は外部storageへleaf bytesを直接copyする。連続領域を要求するextern parameterだけをcall前にflattenし、そのcacheは
rope nodeと共に解放する。extern aggregate内のSymbolも型再帰でmaterializeする。いずれの表現もsourceからは新しいimmutable
byte sequenceとしてだけ観測され、node、cache、capacityはC host ABIのopaque ownership内部に留まる。
comparisonのleaf cursorはdescriptorとrope nodeをborrowし、retain、release、allocation、cache mutationを行わない。

closure valueはcode pointer、environment pointer、environment destructorの組である。destructorはcapture型を知る生成function
であり、generic reference-count runtimeはenvironment layoutを解釈しない。

call以外へ流出しないlocal closureはdescriptorをmaterializeせず、environment structをstack上に置いて外側の
bindingをborrowする。単純alias chainとclosure本体のself referenceを合わせて調べ、全referenceがcallee位置に限られる
場合だけこの表現を使う。self closureを別functionのargumentなどの値として使う場合を含め、それ以外はreference count付き
heap environmentへfallbackする。stack environmentはretainもdestroyもしない。direct-use planは元programから再導出し、
creator、alias、top-level、callee以外の使用を含む集合のexact matchをdebug buildのemission前に検査する。

これらのplanとcontrol region/frame planはmodule-private constructorだけから作り、fieldを外部stageへ公開しない。`is_valid`による
全再導出はdebug assertionとfocused mutation testに置き、release compilerでは同じ解析を二重実行しない。release時のstage
contractはconstructorがauthoritative inputだけからclosedなplanを返すことであり、validatorは別のruntime authorityではない。

## programとhost境界

top-level initializerの一時値は各initializerの終了時にdestroyし、保存したtop-level値は`main`のreturn後に逆順でdestroyする。
argument descriptor列のruntime allocationもsource-level `main`のreturn後に解放する。

extern parameterはcall中だけborrowされる。managed resultの各fieldはownership shareを一つmalへtransferしなければならない。
Symbol resultはruntime-owned `MalSymbolAdmission`をfinishして作る。malはextern resultをowned valueとして受け取り、通常の
binding cleanupへ接続する。finish前にreturnする経路ではadapterがadmissionをdropする。
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

過去の測定baselineは[managed Engram性能記録](../history/performance/managed-engrams.md)に置く。
