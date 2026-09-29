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
6. [kindとtype constructor](spec/type-constructors.md)
7. [EngramとExtern](spec/engrams.md)
8. [parametric polymorphism](spec/generics.md)
9. [Symbol](spec/symbols.md)
10. [AddressとBuffer](spec/memory.md)
11. [式と binding](spec/expressions.md)
12. [literalとoperator](spec/operators.md)
13. [result boundaryとcompletion](spec/control.md)
14. [実行意味論](spec/execution.md)
15. [`extern` 境界](spec/extern.md)
16. [C host ABI](spec/c-host-abi.md)
17. [プログラム構造](spec/programs.md)
18. [字句・文法](spec/grammar.md)

## 目的別の入口

| 目的 | 最初に読む文書 | 次に参照するauthority |
|---|---|---|
| 値、application、continuationの設計軸を理解する | [値、解釈、control](design/value-interpretation-and-control.md) | [実行意味論](spec/execution.md)、[result boundaryとcompletion](spec/control.md) |
| `malc`を使う | [`malc`利用contract](development/compiler-usage.md) | [C host ABI](spec/c-host-abi.md) |
| formatterを使う | [formatting policy](development/formatting.md) | [grammar](spec/grammar.md) |
| editorを設定する | [editor tooling](development/editor-tooling.md) | [test方針](development/testing.md) |
| compilerを変更する | [compilerの責務境界](implementation/responsibilities.md) | [implementation notes](implementation/compiler.md)、[Engram ownership](implementation/ownership.md)、[test方針](development/testing.md) |
| execution backendを変更する | [実行backendの責務境界](implementation/execution-backend.md) | [生成物例](implementation/llvm-backend-artifacts.md)、[LLVM backend調査](research/llvm-backend.md) |
| generated C / LLVMの構築を変更する | [C / LLVM構文構築](implementation/backend-syntax-construction.md) | [内部DSL reference](implementation/backend-syntax-reference.md)、[compilerの責務境界](implementation/responsibilities.md)、[test方針](development/testing.md) |
| `Address`、`Buffer`、C host copyを使う | [AddressとBuffer](spec/memory.md) | [C host ABI](spec/c-host-abi.md)、[authority](design/authority.md) |
| table、tree、graphなどのdata modelを設計する | [表現と関係を分ける](design/representation-and-relations.md) | [`Buffer`](spec/memory.md)、[index構造](design/indexed-buffer-structures.md)と[更新案](proposals/indexed-buffer-tree-examples.md) |
| application control loweringを変更する | [application control lowering](implementation/application-control-lowering.md) | [compilerの責務境界](implementation/responsibilities.md)、[Engram ownership](implementation/ownership.md) |
| primitive `trap`を検討する | [first-class primitive `trap`の導入計画](proposals/primitive-trap.md) | [実行意味論](spec/execution.md#trap)、[C host ABI](spec/c-host-abi.md#failureとconcurrency) |
| 型引数推論と型別operationを使う | [operation family](spec/operation-families.md) | [parametric polymorphism](spec/generics.md) |
| higher-kindedな型parameterを使う | [kindとtype constructor](spec/type-constructors.md) | [parametric polymorphism](spec/generics.md)、[operation family](spec/operation-families.md) |
| file内だけでrepresentationを観察できる型を使う | [file-local opaque type](spec/types.md#file-local-opaque-type) | [operation family](spec/operation-families.md) |
| Engramの保持、破棄、storage lifecycleの内部基盤を検討する | [Engram lifecycle loweringの共通基盤](proposals/engram-lifecycle-foundation.md) | [Engram ownership](implementation/ownership.md)、[D055](history/decisions/active/D055.md)、[D083](history/decisions/active/D083.md) |
| Pool上へopaque containerを構成する案を検討する | [Poolとopaque型によるcontainer基盤](proposals/pool/README.md) | [Pool lifecycle contract](proposals/pool/lifecycle-contract.md)、[container例](proposals/pool/container-examples.md)、[COW例](proposals/pool/array-ownership.md) |
| lambda literalのredex扱いを単一continuationとcallee位置へ広げる案を検討する | [その場でapplicationされるlambda literalの拡張](proposals/immediate-lambda-redex.md) | [lambdaの中断とredex](design/value-interpretation-and-control.md#lambdaの中断とredex)、[直和の除去](spec/expressions.md#直和の除去)、[D072](history/decisions/active/D072.md) |
| result boundaryを使う | [result boundaryとcompletion](spec/control.md) | [式とbinding](spec/expressions.md)、[採択理由](history/decisions/active/D051.md) |
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
| `proposals/` | 未採択の設計素案と、部分採択後に残る拡張候補。採択済み部分のruleは`spec/`、理由は`history/`を正とする |
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
