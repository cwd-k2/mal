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

一般変換、tail applicationとの関係、Cへのrefinementと機械的に確認する不変条件は
[application control lowering設計](application-control-lowering.md)を正とする。この計画ではcost modelと採用gateだけを管理する。

最初の実装単位は再帰patternではなく、この一般IRと変換規則である。検証fixtureはtail call、call後のscalar演算、複数の
non-tail call、caseをまたぐcall、managed valueを保持するframe、capturing closure、indirect call、深いunwindを含める。
generated CのMal call depthに対してC stack使用量がboundedであることを確認する。queue化とmemoizationは評価順または計算量を
変える別のalgorithmなので、このcost modelには含めない。

## 共通の完了条件

一つのcost modelごとにfocused generated-C test、native execution、通常のcompiler testを通す。managed lifetimeへ触れる変更は
[managed Engram性能](ownership-performance.md)の通常・sanitizer pressure suiteも通す。ABI変更はgenerated header、host helper、
repository内adapterを同時に更新する。

wall-clockでは同じinput、stdout、C compiler、optimization option、warmup/run数を揃え、5 ms未満のcaseを採否の主根拠にしない。改善が
focused caseだけに留まる場合は、複雑さとcode sizeに見合う独立した利用形状があるまで採用しない。
