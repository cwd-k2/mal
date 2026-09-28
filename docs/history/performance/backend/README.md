# backend性能測定履歴

Status: Historical records

このdirectoryは実行backendの測定記録を実装世代ごとに分ける。現在の実装や未解決課題を示す場所ではない。
現在の採用条件は[generated program最適化policy](../../../development/generated-program-optimization.md)、責務境界は
[実行backend](../../../implementation/execution-backend.md)、直近のprogram全体の比較は
[example corpus](../examples.md)を正とする。

| 実装世代 | 記録 | 現在の位置づけ |
|---|---|---|
| 退役したgenerated C backend | [generated C](generated-c.md) | LLVM移行前のbaseline、control lowering、C表現の調査記録 |
| 現行LLVM backendへ至る測定 | [LLVM](llvm.md) | 採択済み・棄却済みoptimizationの時系列記録 |

両文書の「現在」「現行」は測定当時を指す。現在のruleへ読み替えず、判断を再利用するときは日付、commit、toolchain、
後続の測定を確認する。
