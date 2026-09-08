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
- 最適化後のLLVM IR、deterministic counter、wall-clockの少なくとも二つで変更理由を確認する。

## 現在の課題

| 優先度 | 軸 | 観測したcost | Corpus |
|---:|---|---|---:|
| 1 | recursive reduction | 非tail再帰の結果を結合するhelper callが残る | 016 |
| 2 | aggregate call topology | 複数scalarからなる探索stateをdirect call間で受け渡す | 032 |
| 3 | scalar memory contract | `Ptr` access、wrap/trap、helper control flowの複合cost | 005 |

この順序は絶対時間と差の大きさに加え、現行authorityの範囲内で不要な処理を除ける見込みで決める。新しいprofileで順位を
変える場合は、同じ意図を各言語で自然に記述した比較fixtureを使い、source上の記法差をcompiler差として扱わない。

## 1. recursive reduction

016では内側の非tail再帰だけを再現するfocused fixtureを置き、最適化後IRのcall、code size、実行時間を固定する。次の二案を
小さい方から比較する。

1. direct-onlyかつ単一hot call siteのfunctionに限定し、callerの評価順を保つbudget付きbody統合を行う。
2. operationのeffectと結合方法を証明できるreductionだけをaccumulator loopへ変換する。

一律の`always_inline`、無制限のclone、特定corpusの関数形状を名前や定数で認識する処理は導入しない。body統合でcallが消え、
code sizeを悪化させず、corpus全体の5 ms以上の幾何平均を退行させないことを完了条件とする。body統合で解決しない場合だけ、
reduction解析に必要なIR上のeffectと結合則を設計する。

## 2. aggregate call topology

032では最適化後IRで、recursive call境界のregister、stack spill、aggregate構築を分けて計測する。不要なaggregateが残る場合は
既知direct callだけを対象に、callerとcalleeで共有するleaf signatureへscalar replacementする。明示的に値として使うproduct、
first-class function、leaf数上限を超えるentryはgeneric representationへfallbackする。

aggregateが最適化後に残らない場合は、表面的なC struct数を理由に変更しない。call boundaryが支配的なら、1と同じbody統合の
適用範囲を広げる前にcode-size budgetとclone数上限を決める。

## 3. scalar memory contract

005では次を一つずつ変えたlocal variantと最適化remarkで寄与を分離する。

- unaligned load/store
- alias可能な複数`Ptr`
- wrapping arithmeticとtrapを保つhelper
- tail-lowered nested loopのcontrol flow

既存contractから導けるhelper統合やcontrol-flow簡約だけをbackend変更候補にする。改善にnon-alias、alignment、region分離、狭い
integer rangeが必要なら、先に誰がその事実を選び保証するかをlanguageまたはextern contractの仕様課題として記録し、C attributeを
先行して付けない。

## 共通の完了条件

一つのcost modelごとにfocused generated-C test、native execution、通常のcompiler testを通す。managed lifetimeへ触れる変更は
[managed Engram性能](ownership-performance.md)の通常・sanitizer pressure suiteも通す。ABI変更はgenerated header、host helper、
repository内adapterを同時に更新する。

wall-clockでは同じinput、stdout、optimization option、warmup/run数を揃え、5 ms未満のcaseを採否の主根拠にしない。改善が
focused caseだけに留まる場合は、複雑さとcode sizeに見合う独立した利用形状があるまで採用しない。
