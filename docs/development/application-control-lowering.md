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

control IR、backward liveness、direct-call cycle判定、typed frame layout、growable control storageは実装済みである。C emitterは
direct self non-tail recursionを一つのC activation内のcontrol machineにし、live valueをtyped frameへ保存してreturn時にresumeする。
`Symbol`とそれを含むproduct・sumは[ownership規約](../implementation/ownership.md#control-frame)どおりcopyしてframe ownerを作り、
activation cleanup後、resume時にlocal slotへmoveする。copy-onlyの正しさを先に成立させており、last-use transferは未適用である。
direct self tail callは従来どおり`goto`へfusionし、acyclicなknown callは通常のtyped C callを保つ。

function valueを含むlocal stateもcontrol machineへ移し、suspensionをまたぐclosure environmentはheap ownerとしてframeに保存する。
managed captureはenvironmentの型別destructorまで含めてretain/releaseする。引数を分解して直ちに`function(value)`を返すpureな
known forwarderへself closureを渡すtail edgeは、forwarderを省略してdirect self tailの`goto`へfusionする。その他のindirect callと
first-class functionを介したcall cycleは現時点では従来のC emissionを使う。次の適用段階ではclosureのcodeとenvironmentをentry
stateへ渡す共通dispatchを導入し、sanitizerを含むlifetime testを同じgateとして維持する。名前のforward referenceによる相互再帰は
現在のsource languageが受理しないため、この最適化の完了条件には含めない。

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

最初のreference表現は`max_align_t`境界へ丸めたframeを置く連続growable byte storageとする。control contextはcapacity、次の
空きoffset、top frameのoffsetを持つ。pushはsize加算とalignment丸めのoverflowを検査してから必要ならstorageをgrowし、grow後に
offsetからframe pointerを再取得する。handler間、growを伴い得るoperation間、dispatcherへのreturnをまたいでframe pointerを
保持しない。popも保存したoffsetから直前のtopと空きoffsetを復元する。

frame headerはresume stateと直前frameのoffsetを持ち、payloadはcall siteごとに異なるtyped structとする。dispatchはgenerated
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
| 構造検査 | 未処理`Call`なし、tail edgeでpushなし、C-call edge集合がacyclic、frame fieldがlive valueと一致 |
| semantic test | result、evaluation order、extern trace、managed lifetime、trapの一致 |
| generated C検査 | Mal call depthに対するC stack bound、defined alignmentとsize計算、期待するfusion |
| 実測 | wall-clock、instruction count、branch、cache、code size、resident memory |

実時間の非劣化は数学的には保証しない。dispatcherやframe操作が通常callのoptimizationを妨げる可能性があるため、focused recursion
fixtureだけでなく既存corpusを同じtoolchainと測定手順で再計測する。意味論上の一般化と、採用するC表現の性能判断を分離する。

新しいsurface syntaxが既存coreへ完全にdesugarされる限り、このmachineの対象は増えない。host callback、exception、async、
continuation captureなど新しいcontrol effectをcoreへ加える場合だけ、machine state、仕様、検証を同時に拡張する。
