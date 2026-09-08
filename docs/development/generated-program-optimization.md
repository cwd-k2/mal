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

| 優先度 | 軸 | 観測したcost | Corpus |
|---:|---|---|---:|
| 1 | scalar memoryとcontrol flow | branch-heavy heapの`Ptr` accessとstate update cost | 043 |
| 2 | record storageとcontrol flow | same-width化後にも残るheap、DP、subset scanのcost | 011、063 |

この順序は絶対時間と差の大きさに加え、現行authorityの範囲内で不要な処理を除ける見込みで決める。新しいprofileで順位を
変える場合は、同じ意図を各言語で自然に記述した比較fixtureを使い、source上の記法差をcompiler差として扱わない。

## 1. scalar memoryとcontrol flow

043では、最適化後IRでsource helperとaggregate callはentry bodyへ統合されている。表面的なC function数を
理由にbody統合やcloneを追加せず、次のcostをlocal variantとoptimization remarkで一つずつ分離する。

- alias可能な複数`Ptr`
- tail-lowered loopのbranchとstate update

typed/aligned accessへ置き換えた診断variantと、host allocation実装を見せるLTO variantはどちらも改善しなかったため、
alignmentとcross-translation-unitのallocation visibilityは現在の主原因候補から外す。direct Cの`-fwrapv` variantも
通常buildと同等だったため、wrap semanticsの値域証明は優先しない。popped distanceをdirection loopへ追加parameterとして
渡すvariantも元のmal版と同等で、最適化後IRでは既存loadがloop invariantになっていたため採用しない。

一つのcost modelはheap固有のbranch topologyとstate updateの責務に閉じる。既存contractから導けるhelper統合や
control-flow簡約だけをbackend変更候補にする。
改善にnon-alias、alignment、region分離、狭いinteger rangeが必要なら、その保証を選ぶauthorityをlanguageまたは
extern contractの仕様課題として先に記録する。

## 2. record storageとcontrol flow

011と063の元の比率差は、そのままではcompilerの責務だけを測っていなかった。011をsame-widthに揃えたvariantは改善せず、
063をsame-widthかつportable popcountへ揃えたvariantは1.45倍から約1.27倍へ縮んだ。したがって011ではwidthを原因候補から外し、
063では元の差の一部をfixture差として除外する。

063に残っていたsigned right shiftのmask合成は、integer lowering全体でClangが`ashr`へ認識できるportable表現へ変更した。
独立したshift fixtureは改善したが063全体は変わらなかったため、残差をshift loweringへ帰属させない。

次は最適化後IRで、011のflat 3-field jobとC structのrecord move、heap上のDPとC local storage、および063のnested scanを
別々に分類する。sourceで自然に異なるcontrol flowを、backendの局所rewriteでC sourceへ似せない。複数の独立fixtureに共通して
残る表現costだけをbackend設計候補にする。

011単体ではLTOが改善したが、全corpusでは中央値に効果がなく059を退行させたため、一律LTOは採用しない。現行extern contractは
allocationのfreshnessや呼び出し間のnon-aliasを保証しない。host実装がたまたま`malloc`を使うfixtureから属性を逆輸入せず、
その保証が必要ならlanguage/host contractの独立した要求を先に置く。

`popcount`の差だけを隠すbackend specializationや新しいprimitiveは導入しない。言語surfaceへbit-count operationを加える場合は、
Typical90 fixtureではなく独立した言語要求とhost contractを先に必要とする。

## 共通の完了条件

一つのcost modelごとにfocused generated-C test、native execution、通常のcompiler testを通す。managed lifetimeへ触れる変更は
[managed Engram性能](ownership-performance.md)の通常・sanitizer pressure suiteも通す。ABI変更はgenerated header、host helper、
repository内adapterを同時に更新する。

wall-clockでは同じinput、stdout、C compiler、optimization option、warmup/run数を揃え、5 ms未満のcaseを採否の主根拠にしない。改善が
focused caseだけに留まる場合は、複雑さとcode sizeに見合う独立した利用形状があるまで採用しない。
