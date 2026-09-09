# application control lowering設計

Status: Current implementation design

この文書はclosure-converted ANFから明示的なcontrol machineを導出し、portable Cへrefineする規則と検証境界を管理する。
性能上の必要性と採用gateは[generated program最適化計画](generated-program-optimization.md)、source semanticsは
[実行意味論](../spec/execution.md)、現在のcompiler構成は[implementation notes](../implementation/compiler.md)を正とする。
理論上の背景は[関連調査](../research/prior-art.md#controlとcontinuation)に置く。

## 対象とauthority

対象は再帰patternではなく、Mal functionへのすべてのapplicationである。control loweringはapplication位置、continuation
frame、enter、resume、returnの意味を所有する。C backendはframe layout、storage、dispatchとmanaged valueの
copy、transfer、destroyを所有する。source-level function type、closure、`Ptr`、Engramへcontinuation表現を公開しない。

v0.5のexternはMal function valueを運ばず、hostからMal closureをcallbackしない。primitive、memory、extern operationは現在の
block内で同期的に完了し、`Call`だけが別のMal functionへcontrolを移す。この境界が変わる場合は最適化ではなくcore semanticsの
変更として扱う。

## 実装状態

control IR、backward liveness、possible application graph、tail fusion後のresidual continuation graphとrecursive SCC partition、
typed frame layout、continuation constructor数に応じたframe region別のgrowable control storageは実装済みである。C emitterは
direct self non-tail recursionを一つのC activation内のcontrol machineにし、live valueをtyped frameへ保存してreturn時にresumeする。
`Symbol`とそれを含むproduct・sumは[ownership規約](../implementation/ownership.md#control-frame)どおりcopyしてframe ownerを作り、
activation cleanup後、resume時にlocal slotへmoveする。suspendで現在のactivationを終える際はlive ownerをframeへmoveして元slotを
zero化し、不要なretain/releaseを発生させない。
direct self tail callは`goto`へfusionし、acyclicなknown callは通常のtyped C callを保つ。

function valueを含むlocal stateもcontrol machineへ移し、suspensionをまたぐclosure environmentはheap ownerとしてframeに保存する。
managed captureはenvironmentの型別destructorまで含めてretain/releaseする。引数を分解して直ちに`function(value)`を返すpureな
known forwarderへself closureを渡すtail edgeは、forwarderを省略してdirect self tailの`goto`へfusionする。

first-class functionを介してcall graph cycleを閉じるedgeは所属するcontrol regionの共通machineで実行する。共通machineを必要とするregion集合は
siteごとのedge modeから一度だけ導出する。callee descriptorのcode identityから
有限なuser-function targetを選び、environment ownerとtyped argumentをtarget entryへmoveする。非tail edgeではcallerのlive valueと
environment ownerをtyped frameへ保存し、tail edgeではframeを増やさない。cycleを閉じないindirect callとuser function以外のtargetは
typed C callを保つ。名前のforward referenceによる相互再帰は現在のsource languageが受理しないため、この最適化の完了条件には
含めない。frameを持つregionのarenaは`MalContext`へpointerとcapacityだけをcacheし、実行中のtopとcurrent frameはmachine localに
保持する。tail遷移だけのregionはarenaを生成しない。

## 抽象machine

source側のconfigurationをexpression、environment、evaluation contextの組、control IR側をprogram point、live value、
continuation stackの組として対応させる。

```text
Source  = <expression, environment, K>
Control = <program-point, values, S>

K ::= Halt | Context_site(live-values, K)
S ::= Empty | Frame_site(live-values, S)
```

ANF blockに次のbindingとsuffixがあるとする。

```text
x := Call(callee, argument)
rest
```

`rest`から参照され、call前に定義済みのvalueを`live(rest)`とする。loweringは`live(rest)`を`Frame_site`へ移し、calleeの
codeとenvironment、argumentを次のentry stateにする。calleeのreturnはtop frameが指定する型でresultを受け取り、`x`へbindし、
frameをpopして`rest`のprogram pointへ移る。

calleeがfirst-class valueでも、runtimeのclosure descriptorからcodeとenvironmentを選べばよい。frame constructorを決めるのは
callerのapplication位置なので、points-to解析やcallee候補の列挙は変換の前提ではない。有限なprogramではapplication位置が有限で
あり、frame constructorも有限になる。

## tail application

tail applicationではcall resultへ追加するcontextがなく、calleeへ渡すcontinuationはcallerが受け取ったcontinuationと同一になる。

```text
non-tail application: push Frame_site(..., K); enter callee
tail application:                         enter callee with K
```

したがってtail recursionは独立したcontrol mechanismではなく、frameを増やさないapplicationの特殊例である。現在のdirect self
tail callの`goto`はこの遷移をC statementへfusionしたbackend specializationとして統合し、別のsemantic authorityにしない。

## Cへのrefinement

各call siteではresultとlive valueの型が既知なので、全Mal valueを共通boxへ変換しない。frameごとにtyped payloadとresult受取位置を
生成し、異なるlayoutのframeをruntime-owned control stackへ置く。control stackはMal programから観測できない。

生成Cは次の条件を満たす。

- Mal application handlerは次のhandlerをC callしたまま実行せず、dispatcherへ戻るか同一dispatcher内でjumpする。
- tail applicationはcontrol stackを増やさない。
- frameのsize、alignment、offset計算はoverflowとCのundefined behaviorを生まない。
- stack storageの移動後に古いframe pointerを保持しない。managed payloadのC object lifetimeも保存する。
- resultを次状態へ移してから、通常returnで不要になるframe fieldをdestroyする。
- trap時は現在の意味論どおり一般的なstack unwindを行わない。
- externとruntime helperのC callは同期的に戻り、Mal call depthに比例するC stackを作らない。

複数のframe constructorを持つregionは、`max_align_t`境界へ丸めたframeを置く連続growable byte storageを使う。control contextは
capacity、次の空きoffset、top frameのoffsetを持つ。pushはsize加算とalignment丸めのoverflowを検査してから必要ならstorageをgrowし、
grow後にoffsetからframe pointerを再取得する。popも保存したoffsetから直前のtopと空きoffsetを復元する。frame headerはresume stateと
直前frameのoffsetを持ち、payloadはcall siteごとに異なるtyped structとする。

frame constructorが一つだけのregionは、そのframe型の固定幅slot列を使う。`top`とcapacityはslot数を表し、resume stateは
constructorから静的に決まるため、frame header、alignment丸め、直前frame offset、runtime tag dispatchを持たない。storageのbaseは
allocatorが全object typeに必要なalignmentを満たし、`sizeof(frame)`間隔が後続slotのalignmentを保つ。どちらの表現でもhandler間、
growを伴い得るoperation間、dispatcherへのreturnをまたいでframe pointerを保持しない。

dispatchはgenerated
program内で一意なstate tagを使う。function closureのcode identityはentry stateへ対応し、entry payloadへenvironmentとargumentを
移してからdispatcherへcontrolを返す。C entry point、top-level initializerなどMal外部のcallerだけがdispatcherを開始してresultを
受け取る。

control storageはEngramまたはclosure environmentではなく、Mal programから到達不能なimplementation storageである。sizeが
`size_t`で表現不能な場合とallocation failureは`mal_trap`へ写像せず、理由を示して`abort()`するimplementation resource failureと
する。trapを捕捉できない現在のprofileではprocessの異常終了という観測は同じだが、言語上のallocation ruleとは分類を混同しない。

## backend specialization

一般control IRをsemantic authorityとし、C表現は証明できる範囲で戻す。すべてのapplicationをcontrol IRへlowerすることは、すべての
edgeをdispatcherで実行することを意味しない。既存のdirect call、inline化、direct self tail callのC表現は次の条件内で維持する。

| edge | 許される表現 |
|---|---|
| direct self tail | 同一stateの`goto` |
| pureなknown tail forwarderへ渡すdirect self | forwarderを省略した同一entryへの`goto` |
| その他のtail | frameを増やさないdispatch |
| C-call edge集合がacyclicなdirect call | 通常のC call |
| recursive SCC内またはtarget不明のcall | explicit control stack |

direct-call解析とpoints-to解析は正しさの条件ではなく、dispatchとframe操作を除去するための最適化である。通常のC callへ戻すedge
集合についてはcycleがないことを検証し、Mal call depthに対するC stack使用量をboundedに保つ。

初版は静的なdirect-call graphで同じrecursive SCCに属する通常edgeをexplicit化し、SCC間のedgeとrecursive SCCから外れるhelper
callを通常callにする。非末尾のdirect self edgeはそれ単独でrecursive SCCになるためexplicit controlを使う。dispatcher実行中に
calleeがsuspendし得る通常C callを残したままcycleを一部だけdispatchへ変えると、suspend前のC frameを保持した再入によりstackが
増え得る。このためSCC内の部分direct化は、control transferをdirect callerまでunwindする専用calling conventionなしには行わない。

静的にcall depth上限を証明するspecialization、suspendを伝播するdirect convention、一定段数だけC callするbounded batchingは
将来の候補だが、bounded C stackと実測上の利益を独立に示してから追加する。初版の判定はprofileなしに結果が変わらない規則とする。

## 解析authorityとminimality

C表現の判定は、同じ事実を後段が再推論しない一方向の導出にする。

```text
control IR + closure use
  -> possible application graph
  -> fused tail transition
  -> residual continuation graph
  -> recursive control regions
  -> siteごとのedge mode
  -> suspension frameとclosure lifetime
  -> constructor cardinality別のframe表現
  -> region emissionとarena需要
```

possible application graphは各Mal function applicationについてcaller、有限なuser-function target集合、known targetか
first-class targetかを一度だけ所有する。target集合は型互換性とclosure-use解析から保守的に求め、direct C call、dispatch、
frame、storageの都合を混ぜない。top-level initializerはrecursive SCCのnodeではないが、同じsite target情報を使ってedge modeを
決める。

direct self tailとpure forwarder fusionは、Mal applicationのpossible targetを変えず、C continuationを作らない`goto`遷移を構成する。
residual continuation graphはpossible application graphからこのfused siteを除いたedgeだけを所有する。これはprofile依存の選択ではなく、
tail applicationがcaller continuationをそのまま渡すことに基づく表現上の同値変換である。

recursive control regionはresidual continuation graphのrecursive SCCをauthorityとする。同一regionを閉じるtargetだけをregion dispatch対象とし、
region外targetはcondensation graph上の通常C callへfallbackできる。siteのedge modeはregion所属から導出し、edgeごとの到達性探索で
cycleを再判定しない。fusion済みedgeがpossible target情報から消えることはないが、実行時のC call、dispatcher、region identityを
要求する根拠にはしない。

suspensionはnon-tail applicationが同一region targetへcontrolを渡す場合だけ発生する。frame layout、resume live owner、environment
owner、local closureのheap fallbackはこのsuspension site集合からだけ導出する。resumeがenvironmentを参照しても、direct-selfだけの
local machineでは同じactivationのenvironmentが全遷移で維持されるためframeへ複製しない。複数entryまたはindirect edgeを持つ共通machineだけが
active environment ownerをframeへmoveする。callee選択のために`Dispatch`を使うこと自体は
suspensionを意味せず、region外へ通常C callするだけのindirect siteはactivation-local lifetimeを延長しない。`Symbol`を含むmanaged
valueでもsource-level lifetime authorityは変わらず、frameが必要な場合にだけownerの一時的な保存場所がlocal slotからframeへ移る。

regionの存在、共通machineの必要性、arenaの必要性、arena表現も分離する。複数entryまたはindirect region edgeがあれば共通machineを使うが、
tail遷移だけのregionはcontinuationを保存しないためarenaを持たない。arenaはframeを持つregionだけに割り当て、`MalContext`がcacheする
pointerとcapacityを、各invocationだけが持つ`top`とcurrent frameから型として分ける。一つのsuspension siteだけを持つregionは
homogeneous、複数siteを持つregionはheterogeneousとframe authorityが分類し、emitterやruntimeがsite集合を再走査しない。

実装と検証は次の依存順を保つ。

1. possible application graphを独立した解析結果にし、target列挙を一箇所へ集約する。
2. fusion可能なtail siteと渡すargumentを一度だけ構成し、残存continuation graphからregionを導出する。
3. edge modeをregionから導出し、direct C-call graphがcondensation DAGに含まれることを検査する。
4. frameとclosure lifetimeをsuspension siteから導出し、共通machine判定の複製を除く。
5. constructor cardinalityからhomogeneousまたはheterogeneousなstack表現を導出する。
6. common machine需要とresume livenessから、frameがenvironment ownerを運ぶかを導出する。
7. cached arenaとactivation stackを別のC型にし、frameを持たないregionのarenaを生成しない。
8. 各段階でfocusedな構造・lifetime testと全compiler testを通し、性能値は意味論・minimalityを満たした結果の回帰監視にだけ使う。

planのconstructorとfieldはbackend module内に閉じる。全再導出validatorはdebug assertionとfocused mutation testでconstructorの
exactnessを検査し、release compilerでは重複解析を行わない。validatorを常時実行する必要が生じるのは、planを外部入力から
復元する境界を追加した場合であり、その時点でcompiler-stage admission contractとして設計し直す。

## control region

可能call graphの各recursive SCCを一つのregionとする。同じregionを閉じるdirectまたはfirst-class edgeはregion内dispatch、
異なるregionへのedgeはcondensation graph上の非循環な通常C callとする。runtime callee候補が同一regionとregion外の両方を含む場合は、
code identityで前者だけをdispatchし、後者を通常callへfallbackする。direct self recursionは要素数1のregionであり、indirect cycleと
異なる規則を持たない。

各region invocationは次の状態を持つ。

```text
Region = <program-point, values, storage, representation-state>
Frame  = <resume-constructor, typed-live-values, environment-owner?>
```

frame列とsource evaluation contextの対応には`R`を使う。frameを持つregionごとに
cached arenaを持ち、invocation中の`top`と、heterogeneous表現だけが使う`current-frame`はregion machineのC local stateにする。arenaのpointerとcapacityだけを
`MalContext`へ戻して次のinvocationで再利用する。region内edgeはC callしないため同じarenaへ再入せず、region間callは別arenaを使う。externから
Mal closureをcallbackできない現在のhost contractもこの非再入性の前提である。

constructorが複数ならframeはcall siteごとの可変size typed payloadとし、最大variant幅のunion slotへ一律に広げない。headerの
resume constructorとprevious offsetがframe列を復元する。constructorが一つなら同じtyped payloadだけを固定幅で並べ、`top`をslot indexとして
既知resumeへ直接移る。pushのfast pathは`top <= capacity`不変条件の下でcompile-time frame幅を使い、growth時だけcapacityとbyte sizeの
overflowを検査する。local machineのenvironmentはactivation residentとし、共通machineのactive environmentだけをframeへ保存する。
pop、managed ownerのmove、environment destructor、tail edgeでframeを増やさない規則は両表現で共通とする。

C stack上に同時に存在するregion activationはcondensation graphのpath長でboundされ、Mal recursion depthには比例しない。
local direct-self machineと複数functionを扱うcommon machineは生成moduleを分けるが、
同じregion storage規約とframe規約に従う。構造検査はC-call graphの非循環性、region内dispatch targetの閉包、frame ownerの一意性を
対象とする。性能上の採用gateと過去の比較結果は[generated program最適化計画](generated-program-optimization.md)と
[性能測定履歴](../history/performance/generated-c.md#control-region-refinement)に置く。

## 正しさ

各control stateのframe列を元のANF evaluation contextへ戻す対応`R`を定める。sourceのstepに対してcontrol machineが有限stepで
対応し、result、divergence、trap、左から右の評価順、extern traceを保存することを変換規則ごとに確認する。managed valueでは
各transitionの前後でownerが一つだけ存在することも対応関係へ含める。

網羅性はfixture一覧ではなく、closure-converted IRの全`Operation`に対するexhaustiveな変換で保証する。`Case`と
`PrimitiveBranch`は選んだsub-blockへ現在のcontinuationを渡し、変換後のIRに未処理の`Call`を残さない。operation variantの追加時は
Rustのexhaustive matchがcontrol loweringの更新を要求する構造にする。

## control IR

`closure`と`c_emit`の間にcontrol lowering stageを置く。各top-level initializerとlifted functionはentry stateを持ち、stateは
callを含まないbinding列と一つのterminatorからなる。

```text
Terminator ::= Return(value)
             | Goto(target)
             | Jump(target, value)
             | Call(callee, argument, resume)
             | TailCall(callee, argument)
             | Case(scrutinee, arm-targets)
             | PrimitiveBranch(operator, operands, otherwise-target, then-target)
```

`Goto`はvalueを渡さないstate分割、`Jump`は同じMal activation内のcase arm resultをjoin stateへ渡す局所遷移であり、どちらも
control frameを増やさない。stateのoptionalなinput patternが`Jump`のvalue、case payload、またはnon-tail `Call`のresultを受け取る。
function bodyの最終resultをそのまま返すapplicationだけを`TailCall`とし、`Case`と`PrimitiveBranch`のarmへreturn destinationを
渡すことでbranch内のtail positionも保存する。

lowering後にstate graphのbackward livenessを解き、各stateへentry時に必要なcaller-local bindingを型とspan付きで記録する。
`Call`のframe payloadはresume stateのlive-inと一致する。top-level bindingはprogram storageから再取得できるためframeへ複製しない。
`EnvironmentField`またはenvironmentを伴う`SelfClosure`がstate以降で必要ならstateの`needs-environment`を立てる。C backendはこの
明示情報だけからframe payloadとstate遷移時のcleanupを構成し、独自にclosure IRのsuffixを再解析しない。

## 保証と計測の境界

| 分類 | 確認する性質 |
|---|---|
| 構造検査 | 未処理`Call`なし、tail edgeでpushなし、C-call edge集合がacyclic、region内non-tail siteとframe集合の一致、frame fieldとresume live-inの一致、constructor cardinalityとstack表現の一致、crossing closure集合とarena集合の一致 |
| semantic test | result、evaluation order、extern trace、managed lifetime、trapの一致 |
| generated C検査 | Mal call depthに対するC stack bound、defined alignmentとsize計算、期待するfusion |
| 実測 | wall-clock、instruction count、branch、cache、code size、resident memory |

実時間の非劣化は数学的には保証しない。dispatcherやframe操作が通常callのoptimizationを妨げる可能性があるため、focused recursion
fixtureだけでなく既存corpusを同じtoolchainと測定手順で再計測する。意味論上の一般化と、採用するC表現の性能判断を分離する。

新しいsurface syntaxが既存coreへ完全にdesugarされる限り、このmachineの対象は増えない。host callback、exception、async、
continuation captureなど新しいcontrol effectをcoreへ加える場合だけ、machine state、仕様、検証を同時に拡張する。
