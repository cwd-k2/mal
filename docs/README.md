# mal documentation index

この文書は、知りたい内容からauthorityへ到達するためのindexである。言語の紹介とbuild例はrepository rootの
[`README.md`](../README.md)に置き、ここでは規則や手順を重複させない。

現時点のstatusは **v0.5 development**。規範項目は`docs/spec/`、compilerの現在の構成と責務は
`docs/implementation/`で管理する。

## 言語を読む順序

初めて読む場合は次の順を推奨する。

1. [最小性の方針](design/minimality.md)
2. [EngramとExternのauthority](design/authority.md)
3. [言語の範囲](spec/scope.md)
4. [型](spec/types.md)
5. [EngramとExtern](spec/engrams.md)
6. [Symbol](spec/symbols.md)
7. [memory primitive](spec/memory.md)
8. [式と binding](spec/expressions.md)
9. [result boundaryとcompletion](spec/control.md)
10. [実行意味論](spec/execution.md)
11. [`extern` 境界](spec/extern.md)
12. [C host ABI](spec/c-host-abi.md)
13. [プログラム構造](spec/programs.md)
14. [字句・文法](spec/grammar.md)

## 目的別の入口

| 目的 | 最初に読む文書 | 次に参照するauthority |
|---|---|---|
| `malc`を使う | [reference compiler利用contract](development/compiler-usage.md) | [C host ABI](spec/c-host-abi.md) |
| formatterを使う | [formatting policy](development/formatting.md) | [grammar](spec/grammar.md) |
| editorを設定する | [editor tooling](development/editor-tooling.md) | [test方針](development/testing.md) |
| compilerを変更する | [compilerの責務境界](implementation/responsibilities.md) | [implementation notes](implementation/compiler.md)、[Engram ownership](implementation/ownership.md)、[test方針](development/testing.md) |
| execution backendを変更する | [実行backendの責務境界](design/execution-backend.md) | [生成物例](development/llvm-backend-artifacts.md)、[LLVM backend調査](research/llvm-backend.md) |
| `Address`、target size、layout、memory placementの試案を確認する | [`Address`、target size、layout、placementの試案](design/size-and-alignment.md) | [generic memory surface syntax](design/memory-syntax.md)、[`Region`と`Packed`によるmemory transferの試案](design/region-and-packed.md) |
| `Region`、`Packed`、partial I/Oの試案を確認する | [`Region`と`Packed`によるmemory transferの試案](design/region-and-packed.md) | [`Address`、target size、layout、placementの試案](design/size-and-alignment.md)、[authority](design/authority.md) |
| generic memory operatorとconversion構文の試案を確認する | [generic memory surface syntax](design/memory-syntax.md) | [grammar](spec/grammar.md)、[式とbinding](spec/expressions.md) |
| application control loweringを変更する | [application control lowering](development/application-control-lowering.md) | [compilerの責務境界](implementation/responsibilities.md)、[Engram ownership](implementation/ownership.md) |
| result boundaryを使う | [result boundaryとcompletion](spec/control.md) | [式とbinding](spec/expressions.md)、[採択理由](history/decisions/D051.md) |
| parametric polymorphismと型index付きprimitiveの試案を確認する | [parametric polymorphismと型index付きprimitiveの試案](design/parametric-polymorphism.md) | [最小性](design/minimality.md)、[`Address`、target size、layout、placementの試案](design/size-and-alignment.md) |
| C host adapterを書く | [C host interface例](development/c-host-interface-examples.md) | [C host ABI](spec/c-host-abi.md)、[EngramとExtern](spec/engrams.md) |
| 仕様とtestを対応させる | [conformance matrix](development/conformance.md) | [`spec/`](spec/) |
| 設計理由を調べる | [設計決定履歴](history/decisions/) | [最小性](design/minimality.md)、[authority](design/authority.md) |
| 性能を評価する | [性能測定履歴](history/performance/) | [generated program最適化policy](development/generated-program-optimization.md)、[test方針](development/testing.md) |
| 外部事例を調べる | [関連調査](research/prior-art.md) | link先の一次資料 |

## 文書の役割

| 場所 | 役割 |
|---|---|
| `spec/` | 利用者と実装者が従う規範的仕様 |
| `design/` | 現在の設計policy、採択済み判断の理由、およびstatusを明示した試験設計 |
| `implementation/` | compiler/backend の現在の責務と構成 |
| `development/` | repositoryを変更・検証する現在の手順とpolicy |
| `research/` | 外部仕様・先行事例から得た根拠 |
| `history/` | 過去の設計判断、退役事項、条件付き測定記録 |

仕様と実装文書が衝突した場合は`spec/`を優先する。仕様で意図的に未指定とする挙動は該当する規範文書に直接記載する。
退役した名称、構文、ABI、意味論、完了済み作業、測定結果は`history/`だけに残す。spec、implementation、development、test、
example、source commentは過去との差分ではなく、現在のruleとbehaviorを直接記述する。

## 文書構造

各文書には一つの安定した責務を持たせる。自然な責務境界がある場合、hand-written documentは200行以下を
目安にし、500行を超える前にauthorityまたは読者が調べる目的で分割する。分割後はindexまたはowner文書から
到達できるようにし、同じruleを複数の文書へ複製しない。

行数を満たすための番号付き断片や、独立して意味を持たないpageは作らない。mechanical fixture、測定結果、
一箇所で全体をreviewする必要があるcanonical schemaはこの目安の対象外とする。

## v0.5 の短い定義

- malはstrict call-by-valueの単純型付き関数型言語であり、immutable binding、関数、lexical result block、直積、直和、固定幅scalar、
  immutable byte値`Symbol`、型なし`Ptr`によるmemory accessを持つ。
- mal内部で意味とlifetime authorityを持つ値をEngramと総称し、外部resourceへのcapabilityから区別する。
- 外部世界との作用はexternal operationのapplicationと明示的なmemory accessに限定する。allocation、deallocation、I/O、
  ファイル、ネットワーク、時刻、乱数、threadはhost側の責務とする。
- reference compiler `malc`はRustで実装し、executionをLLVM module、process entryとhost bridgeをC11 shim、program非依存の
  機構をC11 runtimeへ変換する。
- extern implementationはgenerated headerに対するC adapterとして用意し、link時に解決する。
- v0.5 development profileが現在生成するC host ABI versionは`0x000600`であり、source language versionとは独立に番号を持つ。
