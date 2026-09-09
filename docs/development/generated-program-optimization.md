# generated program最適化計画

Status: Current work plan

この文書はreference C backendが生成するprogramの未解決なruntime性能課題、着手順、完了条件を管理する。
測定値と解決済みの経緯は[generated C performance](../history/performance/generated-c.md)、managed valueの正しさは
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

activeなcompiler rewriteはない。application controlは
[application control lowering設計](application-control-lowering.md)のconstructor cardinality別region arenaを現在の表現とする。
新しい最適化は、authorityを分散させず、C stack boundを維持し、focused caseと既存corpusの両方で独立した改善を示す場合に限って
この計画へ追加する。過去のbaseline、比較結果、棄却したprototypeは
[generated C performance](../history/performance/generated-c.md#application-control-lowering実装前後)に置く。

queue化とmemoizationは評価順または計算量を変える別のalgorithmなので、このcost modelには含めない。

## 共通の完了条件

一つのcost modelごとにfocused generated-C test、native execution、通常のcompiler testを通す。managed lifetimeへ触れる変更は
[managed Engram性能記録](../history/performance/managed-engrams.md)の通常・sanitizer pressure suiteも通す。ABI変更はgenerated header、host helper、
repository内adapterを同時に更新する。

wall-clockでは同じinput、stdout、C compiler、optimization option、warmup/run数を揃え、5 ms未満のcaseを採否の主根拠にしない。改善が
focused caseだけに留まる場合は、複雑さとcode sizeに見合う独立した利用形状があるまで採用しない。
