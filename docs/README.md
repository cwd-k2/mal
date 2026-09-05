# mal documentation

このディレクトリは `mal language specification v0.4` を、参照しやすさと議論のしやすさを優先して再構成したものである。

現時点の status は **v0.4 release candidate 1**。規範項目は `docs/spec/`、残るrelease作業は
[implementation roadmap](implementation/roadmap.md)で管理する。

v0.4の要約は[release profile](releases/v0.4.md)にまとめる。

## 読む順序

初めて読む場合は次の順を推奨する。

1. [最小性の方針](design/minimality.md)
2. [言語の範囲](spec/scope.md)
3. [型](spec/types.md)
4. [String](spec/strings.md)
5. [式と binding](spec/expressions.md)
6. [実行意味論](spec/execution.md)
7. [`extern` 境界](spec/extern.md)
8. [C host ABI](spec/c-host-abi.md)
9. [プログラム構造](spec/programs.md)
10. [字句・文法](spec/grammar.md)

利用者向けのcompiler command、対応環境、toolchain、生成物は[reference compiler利用contract](development/compiler-usage.md)にまとめる。実装者はまず[v0.4 implementation roadmap](implementation/roadmap.md)、[compilerの責務境界](implementation/responsibilities.md)、[test方針](development/testing.md)を参照する。仕様とtestの対応は[conformance matrix](development/conformance.md)に集約する。M0の履歴は[M0 implementation record](implementation/m0.md)、各stageの背景は[compiler implementation notes](implementation/compiler.md)に残す。設計理由は[決定記録](design/decisions.md)、意図的な未指定事項は[未決事項](design/open-questions.md)を参照する。

## 文書の役割

| 場所 | 役割 |
|---|---|
| `spec/` | 利用者と実装者が従う規範的仕様 |
| `design/` | 採否を議論中の判断、理由、選択肢 |
| `implementation/` | compiler/backend の非規範的な実装案 |
| `development/` | repositoryを変更・検証する現在の手順とpolicy |
| `research/` | 外部仕様・先行事例から得た根拠 |

仕様と実装案が衝突した場合は `spec/` を優先する。ただし、[未決事項](design/open-questions.md) に載っている項目は確定仕様ではない。

## v0.4 の短い定義

mal は strict call-by-value の単純型付き関数型言語である。immutable binding、関数、直積、直和、固定幅 scalar、immutable byte string を持ち、外部世界との作用は `extern` call に限定する。

reference compiler `malc` はRustで実装し、最初のbackendはCを生成する。extern implementationはgenerated headerに対するC adapterとして用意し、link時に解決する。

メモリ、I/O、ファイル、ネットワーク、時刻、乱数、thread は言語の値・作用として組み込まず、host 側の責務とする。
