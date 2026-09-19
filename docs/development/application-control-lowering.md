# application control lowering設計

Status: Current implementation design

この文書はclosure-converted ANFからbackend-independentなapplication control planを導出する規則を管理する。source semanticsは
[実行意味論](../spec/execution.md)、compiler内の責務は[compilerの責務境界](../implementation/responsibilities.md)、LLVMでの具体化は
[実行backendの責務境界](../design/execution-backend.md)を正とする。

## authority

`control`はMal function callをstate terminatorへ分離し、call結果をaliasとjoinだけでfunction resultへ転送するidentity continuationを
tail callへ正規化してから、stateごとのbackward livenessと`needs_environment`を構成する。`Unit`では値が一つしかないため、
call結果のidentityによらず、`Unit`のatom bindingとjoinだけを通って`Unit`を返すcontinuationも同じ正規形にする。
function内のcontrol graphは再帰edgeを含まず、lowererはsuccessorをpredecessorより先に構成する。この順序により
livenessは一回のbackward dataflow passで確定し、applicationによる再帰は後続のexecution planだけが扱う。
`execution`はこの表現とclosure-use情報から次を一方向に導出する。

```text
control IR + closure use
  -> possible application graph
  -> optional execution optimization decision
  -> residual continuation graph
  -> recursive control region
  -> siteごとのcall mode
  -> parameter destination
  -> suspension frame + return/frame relation
```

possible application graphは各applicationのcaller、known target、および型互換な有限のinternal function target集合を所有する。
`execution/optimization`はpossible application graphを変更せず、構文からtarget identityを追跡できるsiteまたは型互換target集合が
一要素のsiteに対するdirect call、direct self-tail fusion、pureなknown tail forwarder fusionのdecisionだけを構成する。空のoptimization setはdecisionを一つも作らず、すべてのadmitted programをgenericな
dispatchとrecursive regionで実行できるbaselineである。tail fusionはcallerのcontinuationをそのまま渡すedgeだけを除き、possible
target情報自体は保持する。
residual graphのrecursive SCCをcontrol regionとし、region内edgeだけが明示的なstate遷移になる。

call modeは次の四つである。

- `Direct`: condensation graph上の非循環なnative call。
- `DirectRegion`: 静的に既知の同一recursive region内targetへのstate遷移。
- `DirectSelfTail`: parameter slotを更新して同じentry stateへ戻るframeなしの遷移。
- `Dispatch`: first-class calleeのcode identityで有限targetを選ぶ。region内targetならstate遷移、region外targetならindirect native callになる。

function parameterのcontrol bindingは`execution/parameter`がcall mode共通の`Bind(slot)`または`Discard`へ変換する。backendは
parameter patternを再解釈せず、このdestinationをtarget固有のowner operationとstorageへ変換する。

## continuation frame

同じregion内のnon-tail callだけがcallerをsuspendする。frameはresume state、resume時に必要なlive value、および共通regionで必要な
active closure environment ownerを保持する。semantic field集合はresume stateのlive-inと一致し、backendがclosure IR suffixを再走査して
増減してはならない。全自己再帰edgeが同じparameter bindingを保持するというexecution decisionがあり、そのfieldがownership上borrowの
場合だけ、LLVMのphysical layoutはfieldをslotに常駐させてframeへのstoreとresume loadを省ける。top-level valueはconstant planから再取得
できるためframeへ保存しない。`execution/frame/resume`は同じcontrol
machineに属するreturn siteとframe tagだけを組にし、result型とresume input型が一致する組を`Resume`、それ以外を`Unreachable`とする。

`execution/frame/replacement`はfunctionとtop-levelの入口を通常到達、frameのresume stateをそのframeの退役後到達として
`Unreachable | Available(frame) | Unavailable`のforward must-dataflowを構成する。通常到達または異なる退役frameが合流した時点で
退役容量のidentityを失い、次のframeを作るcallで伝播を止める。
全到達pathが同じ退役frameから来るsuspension siteだけが、その容量を再利用できる。これはsourceの再帰形や深さではなくcontrol
machineのstorage residency factであり、planはtarget layoutを扱わない。LLVM backendは新frameのsizeが退役frame以下の場合だけ
`reserve_frame`を省き、途中のnative callがstorageを再配置し得るため現在のstorage pointerを再取得して同じtop位置へ書く。frame
startのalignmentはtarget上の全runtime value alignmentの最大値と4-byte tag metadata契約の大きい方とし、tagged、untaggedを問わず
frame sizeをこの値へ丸める。そのため
初期topと各frame末尾が同じalignment invariantを保ち、退役frameのstartは後続frameの全fieldに有効なbaseとなる。

tail edgeはframeをpushしない。region外callはnative stackを使ってよいが、region condensation graphが非循環なのでMal recursion depthに
比例したnative recursionを作らない。region内non-tail recursionはprogram固有のtyped frameをgenericなgrowable byte storageへ積む。
storageのcapacity、growth、overflow、releaseはC runtime、tag、layout、owner transfer、resume targetはLLVM IRが所有する。

## 共通region machine

複数entryまたはindirect recursive edgeを持つregionでは、各公開entry型に対応するwrapper bodyが同じregion state集合を実行する。
callee closureからcode pointerとenvironmentを取り出し、application graphが列挙したtargetだけへdispatchする。target parameterへargumentを
移してから、同じregionのtargetならentry stateへbranchし、region外のtargetなら非循環なnative callとして呼ぶ。application graphの
列挙外にあるcode identityだけが到達不能なcompiler invariant違反である。

一つのregionは異なるparameter型とresult型のfunctionを含み得る。dispatchで選ばれたtargetのparameter型はapplication graphが保証する。
parameter destinationが`Discard`ならentry slotはなく、遷移が受け取ったmanaged argumentをreleaseしてからbodyへ進む。共通machineが
各return siteと各frame tagの組合せを列挙するとき、frame planが`Unreachable`とした組だけをtarget IRの`unreachable`にする。

non-tail遷移ではcaller live valueをframeへ移し、必要ならcaller environment ownerもframeへ移す。callee return時はframe tagからresume
stateを選び、fieldとenvironmentをlocal slotへ戻してresultをresume inputへ移す。program entryが一つのcontrol topをinternal callへ渡し、
各recursive region invocationはentry時のtopをbaseとして保持する。topがそのbaseへ戻ったらresultをnative callerへ返すため、外側regionの
frameを保持したまま別regionを呼んでも同じarena上で互いのframeを解釈しない。

## ownerと失敗

managed valueをframeへ保存するときはframeが独立したownership shareを持つ。local cleanup後もcallee argumentとenvironmentがliveである
順序を保ち、resumeまたはterminal returnで各shareを一度だけreleaseする。詳細は
[managed value ownership](../implementation/ownership.md)を正とする。

control storageはMal programから到達不能なimplementation storageである。sizeがtarget整数で表現不能な場合とallocation failureは
言語上のeffectではなくimplementation resource failureとしてprocessを異常終了させる。

## 検証

executionの各materialized planはauthorityから期待集合を再構成する`is_valid`を持ち、debug buildとfocused testでapplication target、
選択済みoptimization decision、region、call mode、parameter destination、frame payload、return/frame relationの完全性を検査する。

各execution optimization techniqueは自身の適用条件だけを所有する。集約planは有効なtechnique集合を入力として再構成でき、後段は
technique identityではなく選択されたcontinuation elisionとdirect targetだけを読む。techniqueを追加するために後段のplan型やbackend
contractの変更が必要なら、単なるoptimizationではなくexecution authorityの変更として扱う。

- 全application siteにcall modeがあり、direct native call graphがacyclicである。
- 空のoptimization setと各techniqueの単独有効化で同じresult、effect order、trap、owner lifetime、bounded native stackを保持する。
- region内non-tail siteとframe集合、frame fieldとresume live-inが一致し、退役frame容量の再利用元が全到達pathで一意である。
- tail edgeがframeを増やさず、深いself recursionとfirst-class cycleでnative stack使用量がdepthに比例しない。
- heterogeneous frame、managed field、environment owner、複数target dispatchを実行testで確認する。
- heterogeneous result型を持つ共通region、wildcard parameterへのmanaged argument、同じ構造型を持つ異なる役割のfunctionを確認する。
- frame storage growth後にpointerを再取得し、result、evaluation order、extern trace、trap、managed lifetimeを保持する。

host callback、exception、asyncなどMal functionのcontinuationを新たに外部へ運ぶ機能を追加する場合は、このmachineのauthority変更として
先に設計する。
