# generated program最適化計画

Status: Current work plan

この文書はreference C backendが生成するprogramの未解決なruntime性能課題、着手順、完了条件を管理する。
測定値と解決済みの経緯は[generated C performance](performance.md)、managed valueの正しさは
[Engram ownership](../implementation/ownership.md)、言語とhostのauthorityは
[EngramとExtern](../spec/engrams.md)を正とする。

## 制約

- operandとeffectの評価順を保持する。
- source preconditionを満たすprogramへ新しいfailureを加えない。
- `Symbol`とclosureのidentity、到達可能性、lifetime authorityをmalに残す。
- `Ptr`のregion、permission、alias、alignmentをbackendが推測しない。
- first-class function callとextern ABIに必要なgeneric representationを、direct callだけの測定から削除しない。
- narrow integer representationは、全operationの値域とwrap semanticsを証明できる場合だけ使う。
- wall-clock比較の両binaryは同じC compiler identityとoptionでbuildし、ambient `CC`を継承しない。
- pair比較はroundごとに先行順を反転し、runnerではなくHyperfineのprocess時間を使う。
- 最適化後のLLVM IR、deterministic counter、wall-clockの少なくとも二つで変更理由を確認する。

## 現在の課題

| 優先度 | 軸 | 観測したcost | Fixture |
|---:|---|---|---|
| 1 | applicationのcontrol lowering | 軽いnode処理でのcall costとC stack深度 | Hanoi、線形unwind |

## 1. applicationのcontrol lowering

自然なHanoi再帰と、continuationを4個のscalar fieldからなる`Ptr` stackへdefunctionalizeしたmal版は、同じmove列のhashを返した。
後者は単一のself tail recursionからgenerated Cの`goto` loopになった。各moveをcheapなwrap演算にしたdepth 26では、自然版と
flat版の交互30回測定は114.94 msと77.89 ms、同一processの`cpu-clock` samplingは約60%と40%だった。非末尾call以外の
node処理が軽いとき、自然な記述に約1.48倍のcostが残る。

線形unwindでは最適化後assemblyにも非末尾の再帰`call`とframeごとの3個の8-byte pushが残った。8 MiBのC stackでdepth
250,000は完了したが275,000はsignal 11で終了し、明示stack版は1,000,000まで同じ結果を返した。この確認のwall-clockは
Nushell runnerを含む診断値なので、変換の採否には標準のHyperfine fixtureを別途要求する。

同じ変換をmal sourceで記述することは可能だが、frame offset、capacity、program counter、push/popと全live stateの持ち回しを
利用者が所有する。自然な非末尾再帰が中心構文である以上、これを通常のsource記法とは扱わない。

### 一般形

対象を特定の再帰構文へ限定しない。closure conversion後にも保たれるANF blockの各applicationについて、call後のsuffixと
そこから参照するlive valueを一種類のcontinuation frameにする。有限なprogramにはapplication位置が有限個しかないため、
first-classなcalleeを含めてもframe constructorの集合は有限になる。v0.5ではfunction valueがhost境界を越えず、hostから
mal closureをcallbackできないため、この変換はwhole-programで閉じる。

有限である必要があるのはcallee候補ではなく、caller側のapplication位置である。indirect callはruntimeのclosure descriptorから
codeとenvironmentを選ぶが、戻り先frameはcall siteから一意に決まるため、points-to解析や再帰targetの列挙を変換の前提にしない。
direct callの同定はdispatchを省けるbackend optimizationにだけ使う。

machine stateは実行位置、現在のvalue群、continuation stackからなる。`x := f(a); rest`は`rest`が必要とするvalueをframeへ
移し、`f(a)`へ遷移する。calleeのresultはtop frameをpopし、`x`へbindして`rest`を再開する。direct、indirect、recursive、
non-recursive applicationは同じ規則を使い、call graphの循環をsource構文とは独立に扱う。

call siteごとにresult型とlive valueの型が既知なので、全valueを共通のboxed unionへ変換する必要はない。各frameにtypedな
payloadとresult受取位置を持たせ、heterogeneousなframe列として表現できる。具体的な可変長layoutとdispatch方式はC backendで
比較するが、一般化をsource-level boxingや新しいsum型へ漏らさない。

tail callは別のcontrol mechanismではない。call後のcontinuationがcallerの現在のcontinuationそのものであり、新しいframeを
pushせずcalleeへ遷移できる一般形の特殊例である。現在のdirect self tail callから`goto`を生成する実装は、この規則を先に
specializeしたものと位置づける。control IR導入後は独立したtail-call authorityを残さず、同じ遷移を直接Cの`goto`へfusion
できる場合のbackend specializationへ移す。

### 責務境界

新しいcontrol IRをclosure conversionとC emitterの間に置く案を検証する。control loweringはapplication位置、frame constructor、
frameが保持するlive value、enter/return遷移を所有する。C backendはframeのC layout、storage growth、dispatch、valueごとの
copy/transfer/destroyを所有する。source-level function typeやclosure representationへcontinuation parameterを加えない。

control stackはmal programから観測できないruntime storageであり、`Ptr`やEngramとして公開しない。連続growable storageと
segmented storageの選択は、alignment、再配置中のmanaged value、allocation failure、最大resident sizeを測定してから決める。
実装上のresource exhaustionを言語上のEngram allocation trapへ無断で読み替えない。

一般control loweringをsemantic authorityとし、C call、direct jump、frame pushの選択はbackend optimizationとする。C callへ
戻す場合は、そのedge集合がMal call depthに比例してC stackを増やさないことを証明できる範囲に限る。再帰検出やcall graph解析は
一般変換の受理条件ではなく、生成Cからdispatchを除去するためにだけ用いる。

### 正しさと採用条件

各machine stateに対し、frame列を元のANF evaluation contextへ戻す対応を持たせる。enter、return、caseの各transitionが同じvalue、
同じ左から右の評価順、同じextern traceを保ち、frame内のmanaged valueにちょうど一つのownerがあることを確認する。trapは
stackをunwindしない現在の意味論を維持する。

最初の実装単位は再帰patternではなく、この一般IRと変換規則である。検証fixtureはtail call、call後のscalar演算、複数の
non-tail call、caseをまたぐcall、managed valueを保持するframe、capturing closure、indirect call、深いunwindを含める。
generated CのMal call depthに対してC stack使用量がboundedであることを確認する。queue化とmemoizationは評価順または計算量を
変える別のalgorithmなので、このcost modelには含めない。

網羅性はfixtureのpattern一覧ではなく、closure-converted IRの全`Operation`に対する構造的な変換で保証する。同期的なprimitive、
memory、extern operationは現在のblock内で完了し、`Call`だけが別のMal functionへcontrolを移す。`Case`と
`PrimitiveBranch`は選んだsub-blockへ現在のcontinuationを渡す。変換後のIRに未処理の`Call`を残さず、operation variant追加時は
exhaustive matchがcontrol loweringの更新を要求する構造にする。新しいsurface syntaxが既存coreへ完全にdesugarされる限り、
control loweringの対象は増えない。host callback、exception、継続のcaptureなど新しいcontrol effectをcoreへ加える場合だけ、
machine stateと仕様を同時に拡張する。

## 共通の完了条件

一つのcost modelごとにfocused generated-C test、native execution、通常のcompiler testを通す。managed lifetimeへ触れる変更は
[managed Engram性能](ownership-performance.md)の通常・sanitizer pressure suiteも通す。ABI変更はgenerated header、host helper、
repository内adapterを同時に更新する。

wall-clockでは同じinput、stdout、C compiler、optimization option、warmup/run数を揃え、5 ms未満のcaseを採否の主根拠にしない。改善が
focused caseだけに留まる場合は、複雑さとcode sizeに見合う独立した利用形状があるまで採用しない。
