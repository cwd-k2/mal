# mal documentation

このディレクトリは `mal language specification v0.5` を、参照しやすさと議論しやすさを優先して構成したものである。

現時点のstatusは **v0.5 development**。規範項目は`docs/spec/`、実装gateは
[implementation roadmap](implementation/roadmap.md)で管理する。

完了済みv0.4 candidateの要約は[release profile](releases/v0.4.md)にまとめる。

## 読む順序

初めて読む場合は次の順を推奨する。

1. [最小性の方針](design/minimality.md)
2. [言語の範囲](spec/scope.md)
3. [型](spec/types.md)
4. [String](spec/strings.md)
5. [memory primitive](spec/memory.md)
6. [式と binding](spec/expressions.md)
7. [実行意味論](spec/execution.md)
8. [`extern` 境界](spec/extern.md)
9. [C host ABI](spec/c-host-abi.md)
10. [プログラム構造](spec/programs.md)
11. [字句・文法](spec/grammar.md)

利用者向けのcompiler command、対応環境、toolchain、生成物は
[reference compiler利用contract](development/compiler-usage.md)にまとめる。実装者はまず
[implementation roadmap](implementation/roadmap.md)、
[compilerの責務境界](implementation/responsibilities.md)、[test方針](development/testing.md)を参照する。
M7の測定根拠と引き継ぎ順は[generated C performance評価](development/performance.md)、仕様とtestの対応は
[conformance matrix](development/conformance.md)に集約する。M0の履歴は
[M0 implementation record](implementation/m0.md)、各stageの背景は
[compiler implementation notes](implementation/compiler.md)に残す。設計理由は
[決定記録](design/decisions.md)、意図的な未指定事項は
[未決事項](design/open-questions.md)を参照する。

## 文書の役割

| 場所 | 役割 |
|---|---|
| `spec/` | 利用者と実装者が従う規範的仕様 |
| `design/` | 採否を議論中の判断、理由、選択肢 |
| `implementation/` | compiler/backend の非規範的な実装案 |
| `development/` | repositoryを変更・検証する現在の手順とpolicy |
| `research/` | 外部仕様・先行事例から得た根拠 |

仕様と実装案が衝突した場合は `spec/` を優先する。ただし、[未決事項](design/open-questions.md) に載っている項目は確定仕様ではない。

## v0.5 の短い定義

malはstrict call-by-valueの単純型付き関数型言語である。immutable binding、関数、直積、直和、固定幅scalar、
immutable byte string、型なし`Ptr`によるscalar memory accessを持つ。外部世界との作用は`extern` callと
明示的なmemory storeに限定する。

reference compiler `malc` はRustで実装し、最初のbackendはCを生成する。extern implementationはgenerated headerに対するC adapterとして用意し、link時に解決する。

allocation、deallocation、I/O、ファイル、ネットワーク、時刻、乱数、threadはhost側の責務とする。
