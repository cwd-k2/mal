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
- 最適化後のLLVM IR、deterministic counter、wall-clockの少なくとも二つで変更理由を確認する。

## 現在の課題

| 優先度 | 軸 | 観測したcost | Corpus |
|---:|---|---|---:|
| 1 | scalar memoryとcontrol flow | branch-heavy heapの`Ptr` accessとstate update cost | 043 |
| 2 | comparison fixture admission | narrow storage/counterとintrinsicをbackend costから分離する | 011、063 |

この順序は絶対時間と差の大きさに加え、現行authorityの範囲内で不要な処理を除ける見込みで決める。新しいprofileで順位を
変える場合は、同じ意図を各言語で自然に記述した比較fixtureを使い、source上の記法差をcompiler差として扱わない。

## 1. scalar memoryとcontrol flow

043では、最適化後IRでsource helperとaggregate callはentry bodyへ統合されている。表面的なC function数を
理由にbody統合やcloneを追加せず、次のcostをlocal variantとoptimization remarkで一つずつ分離する。

- alias可能な複数`Ptr`
- tail-lowered loopのbranchとstate update

typed/aligned accessへ置き換えた診断variantと、host allocation実装を見せるLTO variantはどちらも改善しなかったため、
alignmentとcross-translation-unitのallocation visibilityは現在の主原因候補から外す。direct Cの`-fwrapv` variantも
通常buildと同等だったため、wrap semanticsの値域証明は優先しない。

一つのcost modelはheap固有のbranch topologyとstate updateの責務に閉じる。既存contractから導けるhelper統合や
control-flow簡約だけをbackend変更候補にする。
改善にnon-alias、alignment、region分離、狭いinteger rangeが必要なら、その保証を選ぶauthorityをlanguageまたは
extern contractの仕様課題として先に記録する。

## 2. comparison fixture admission

011と063の比率差は、現在のままではcompilerの責務だけを測っていない。direct Cのindexやstorageはmalの`Int64`より
狭く、063はC compiler固有の`__builtin_popcount`を使う。各言語で同じ意図を自然に表すfixtureとして、次を別々に測る。

- same-widthのscalarとstorageに揃えたvariant
- 各言語で利用可能なoperationだけで同じalgorithmを書くvariant

差が残る場合にのみ最適化後IRでoperationを分類する。狭いC型やintrinsicが理由なら、それを隠すbackend specializationや
新しいprimitiveは導入せず、言語surfaceと比較fixtureの差として記録する。

## 共通の完了条件

一つのcost modelごとにfocused generated-C test、native execution、通常のcompiler testを通す。managed lifetimeへ触れる変更は
[managed Engram性能](ownership-performance.md)の通常・sanitizer pressure suiteも通す。ABI変更はgenerated header、host helper、
repository内adapterを同時に更新する。

wall-clockでは同じinput、stdout、C compiler、optimization option、warmup/run数を揃え、5 ms未満のcaseを採否の主根拠にしない。改善が
focused caseだけに留まる場合は、複雑さとcode sizeに見合う独立した利用形状があるまで採用しない。
