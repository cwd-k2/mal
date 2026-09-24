# mal文書index

この文書は、知りたい内容からauthorityへ到達するためのindexである。言語の紹介とbuild例はrepository rootの
[`README.md`](../README.md)に置き、ここでは規則や手順を重複させない。

言語仕様と`malc`はいずれも **v0.6** である。実装のconformance条件は
[conformance matrix](development/conformance.md)、compilerの現在の構成と責務は`docs/implementation/`で管理する。

## 言語を読む順序

初めて読む場合は次の順を推奨する。

1. [最小性の方針](design/minimality.md)
2. [値、解釈、control](design/value-interpretation-and-control.md)
3. [EngramとExternのauthority](design/authority.md)
4. [言語の範囲](spec/scope.md)
5. [型](spec/types.md)
6. [EngramとExtern](spec/engrams.md)
7. [parametric polymorphism](spec/generics.md)
8. [Symbol](spec/symbols.md)
9. [AddressとBuffer](spec/memory.md)
10. [式と binding](spec/expressions.md)
11. [literalとoperator](spec/operators.md)
12. [result boundaryとcompletion](spec/control.md)
13. [実行意味論](spec/execution.md)
14. [`extern` 境界](spec/extern.md)
15. [C host ABI](spec/c-host-abi.md)
16. [プログラム構造](spec/programs.md)
17. [字句・文法](spec/grammar.md)

## 目的別の入口

| 目的 | 最初に読む文書 | 次に参照するauthority |
|---|---|---|
| 値、application、continuationの設計軸を理解する | [値、解釈、control](design/value-interpretation-and-control.md) | [実行意味論](spec/execution.md)、[result boundaryとcompletion](spec/control.md) |
| `malc`を使う | [`malc`利用contract](development/compiler-usage.md) | [C host ABI](spec/c-host-abi.md) |
| formatterを使う | [formatting policy](development/formatting.md) | [grammar](spec/grammar.md) |
| editorを設定する | [editor tooling](development/editor-tooling.md) | [test方針](development/testing.md) |
| compilerを変更する | [compilerの責務境界](implementation/responsibilities.md) | [implementation notes](implementation/compiler.md)、[Engram ownership](implementation/ownership.md)、[test方針](development/testing.md) |
| execution backendを変更する | [実行backendの責務境界](implementation/execution-backend.md) | [生成物例](implementation/llvm-backend-artifacts.md)、[LLVM backend調査](research/llvm-backend.md) |
| `Address`、`Buffer`、C host copyを使う | [AddressとBuffer](spec/memory.md) | [C host ABI](spec/c-host-abi.md)、[authority](design/authority.md) |
| table、tree、graphなどのdata modelを設計する | [表現と関係を分ける](design/representation-and-relations.md) | [`Buffer`](spec/memory.md) |
| application control loweringを変更する | [application control lowering](implementation/application-control-lowering.md) | [compilerの責務境界](implementation/responsibilities.md)、[Engram ownership](implementation/ownership.md) |
| primitive `loop`を検討する | [first-class primitive `loop`の導入計画](proposals/primitive-loop.md) | [反復とdomain step](design/value-interpretation-and-control.md#反復controlとdomain-stepを分ける)、[application control lowering](implementation/application-control-lowering.md) |
| result boundaryを使う | [result boundaryとcompletion](spec/control.md) | [式とbinding](spec/expressions.md)、[採択理由](history/decisions/D051.md) |
| parametric polymorphismを使う | [parametric polymorphism](spec/generics.md) | [型](spec/types.md)、[external memory](spec/memory.md) |
| C host adapterを書く | [C host interface例](development/c-host-interface-examples.md) | [C host ABI](spec/c-host-abi.md)、[EngramとExtern](spec/engrams.md) |
| 仕様とtestを対応させる | [conformance matrix](development/conformance.md) | [`spec/`](spec/) |
| 設計理由を調べる | [設計決定履歴](history/decisions/) | [最小性](design/minimality.md)、[authority](design/authority.md) |
| 性能を評価する | [性能調査toolと作業領域](development/performance-investigation.md) | [generated program最適化policy](development/generated-program-optimization.md)、[性能測定履歴](history/performance/)、[test方針](development/testing.md) |
| 外部事例を調べる | [関連調査](research/prior-art.md) | link先の一次資料 |

## 文書の役割

| 場所 | 役割 |
|---|---|
| `spec/` | 利用者と実装者が従う規範的仕様 |
| `design/` | 現在の設計policyと、複数の規範領域を横断する判断軸 |
| `implementation/` | `malc`の現在の責務、構成、実行backend |
| `development/` | repositoryを変更・検証する現在の手順とpolicy |
| `proposals/` | 未採択の設計素案と、判断前に解決する論点。現在のruleではない |
| `research/` | 外部仕様・先行事例から得た根拠 |
| `history/` | 過去の設計判断、退役事項、条件付き測定記録 |

仕様と実装文書が衝突した場合は`spec/`を優先する。仕様で意図的に未指定とする挙動は該当する規範文書に直接記載する。
退役した名称、構文、ABI、意味論、完了済み作業、測定結果は`history/`だけに残す。各文書は過去との差分ではなく、現在のruleと
behaviorを直接記述する。

## 文書構造

各文書には一つの安定した責務を持たせる。自然な責務境界がある場合、hand-written documentは200行以下を
目安にし、500行を超える前にauthorityまたは読者が調べる目的で分割する。分割後はindexまたはowner文書から
到達できるようにし、同じruleを複数の文書へ複製しない。

行数を満たすための番号付き断片や、独立して意味を持たないpageは作らない。mechanical fixture、測定結果、
一箇所で全体をreviewする必要があるcanonical schemaはこの目安の対象外とする。
