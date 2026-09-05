# v0.4 implementation roadmap

Status: Current

この文書はreference compilerのactive gateと残作業を定める。言語とhost interfaceは[`spec/`](../spec/)、
stageの責務は[responsibilities](responsibilities.md)、検証方法は[test policy](../development/testing.md)を正とする。

## 現在位置

| Gate | Status | Outcome |
|---|---|---|
| M0 | Complete | `Unit`、`Int32`、sum、Bool、closure、externの最初のnative vertical slice |
| S0 | Complete | checked-in exampleと再利用可能なnative fixture |
| M1 | Complete | 固定幅integer、短縮numeric suffix、byte literal |
| M2 | Complete | product、destructuring、opaque handle、aggregate ABI |
| M3 | Complete | immutable byte `String`とextern copy contract |
| M4 | Complete | annotated self recursionとdirect tail-call lowering |
| M5 | Complete | strict `Float32` / `Float64` profile |
| R0 | Active | v0.4 conformanceとrelease readiness |

完了済みgateの詳細はtest、example、decision、およびGit履歴に残す。M0当時の境界だけは
[M0 implementation record](m0.md)に要約する。

## R0: v0.4 release gate

言語機能の追加は行わず、仕様・実装・公開境界が同じv0.4 profileを表すことを確認する。

### 完了済み

- 全implementation milestoneをpublic `check`、`emit-c`、`build`経路まで実装した。
- numeric suffixをintegerの`i8`〜`u64`とfloatの`f32` / `f64`へ統一した。
- v0.4の実装挙動に影響する未決事項をdecisionと`spec/`へ反映した。
- M0〜M5のchecked-in exampleをnative driver testから実行している。
- [`spec/`とtestのconformance matrix](../development/conformance.md)を作成した。
- public CLI、generated header、trap、toolchain failureのcontractをdriver testで監査した。

### 次セッションの作業順

1. supported target/toolchain、`CC`、shared library loader、generated artifact policyを利用者向け文書へ集約する。
2. package versionとrelease文書をv0.4 release candidateとして揃える。
3. clean checkoutのpinned environmentでrelease build、全test、M0〜M5 exampleを再検証する。

### Done

- [`scope`](../spec/scope.md)に含まれる全機能がconformance matrixで検証先を持つ。
- `spec/`、generated ABI、compiler behavior、exampleに既知の矛盾がない。
- 全public exampleとconformance suiteがclean checkoutのpinned environmentで成功する。
- v0.4で提供しない機能と意図的な未指定事項が`spec/`に記述されている。
- activeな計画文書に完了済み作業や古い暫定規則が残っていない。
