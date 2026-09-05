# implementation roadmap

Status: Current

この文書はreference compilerのgateと完了条件を定める。言語とhost interfaceは[`spec/`](../spec/)、
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
| R0 | Complete | v0.4 conformanceとrelease readiness |
| M6 | Complete | v0.5の型なし`Ptr`と最小scalar memory primitive |

## M6: memory primitive

indexed storageの共通mechanismをhostの用途別opaque operationからmalへ戻す。

### 完了済み

- predefined `Ptr`、byte単位の`offset`、`Int64`/`UInt8` load/storeを全stageへ実装した。
- `Ptr`をextern-safeとし、generated C headerへ`MalPtr`を公開した。
- unaligned accessを含むfocused type/ABI/native testとchecked-in M6 exampleを追加した。
- Typical 90 023のprofile列挙、CSR構築、row transition、集計をmalへ移し、公式sampleと24×24 caseを検証した。

### Done

- allocation、bounds、collection policyを言語へ追加せず、algorithm上のindexed storageをmalから操作できる。
- memory semantics、host contract、C ABI、invalid accessの境界が`spec/`に記述されている。
- frontend focused testとgenerated Cのnative testが成功する。

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
- [対応toolchainと生成物の利用contract](../development/compiler-usage.md)を公開した。
- packageとrelease文書を`0.4.0-rc.1`へ揃えた。
- clean checkoutのpinned environmentでformat、Clippy、全test、M0〜M5 example、release buildを再検証した。

### Done

- [`scope`](../spec/scope.md)に含まれる全機能がconformance matrixで検証先を持つ。
- `spec/`、generated ABI、compiler behavior、exampleに既知の矛盾がない。
- 全public exampleとconformance suiteがclean checkoutのpinned environmentで成功する。
- v0.4で提供しない機能と意図的な未指定事項が`spec/`に記述されている。
- activeな計画文書に完了済み作業や古い暫定規則が残っていない。
