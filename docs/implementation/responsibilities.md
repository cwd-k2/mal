# Compiler の責務境界

Status: Current implementation policy

この文書はcompiler codeの分類、各stageのownership、表現の変換境界を定める。pipelineの構成は
[compiler implementation notes](compiler.md)、言語の挙動は[`spec/`](../spec/)をauthorityとする。

## 分類

codeの配置はpackage、依存library、interfaceの有無ではなく、扱う語彙と実装するpolicyで決める。

| 分類 | 責務 |
|---|---|
| External boundary | filesystem、CLI、process、C toolchainなど外部表現との入出力、検証、変換 |
| Compiler stage | 直前の表現を受け入れ、自stageで検証済みの表現を生成 |
| Language policy | `spec/`が定める名前、型、評価、loweringの規則を実行 |
| Foundation | source identity、span、diagnosticなど複数stageが同じ意味で共有する概念 |
| Bootstrap | 入力選択、責務の構成、最終出力の配送 |

pure functionも、そのfunctionが扱う語彙とpolicyを所有するstageへ置く。purityやtransport非依存性だけを
理由にfoundationへ移さない。interfaceで包んでも外部の型、失敗、lifecycleをcontractに露出するなら
境界を作ったことにはならない。

## Stageのownership

各stageは直前の表現をadmitして次の表現へ変換する。入力では直前stageの語彙を使ってよいが、成功時の
出力は自stageで検証済みの型にする。

| Area | Ownership |
|---|---|
| `main` | argument sourceとI/Oの接続、exit statusの配送 |
| `cli` | command grammar、利用エラー、use caseの選択 |
| `source` | file bytes、UTF-8 admission、file identity、byte span、位置計算 |
| `lexer` | 文字列からtokenへのadmissionとlexical error |
| `parser` / `ast` | token列からsource-oriented ASTへのsyntax admission |
| `resolve` | name identity、scope、capture listのvalidation |
| `types` / `check` | canonical typeとtyped AST、type ruleのvalidation |
| `core` / `anf` / `closure` | desugaring、evaluation order、closure representation |
| `c_emit` | typed lowered programからC translation unitとheaderへの変換 |
| `driver` | source file、temporary path、C compiler process、linker inputのownership |
| `diagnostic` | stage errorを利用者向け表現としてrenderする共通機構 |

後段が前段のraw inputを再解釈してはならない。未検証入力とadmit済み出力を、optional fieldやflagを持つ
一つの型で兼用しない。許される操作が異なるsemantic stateには別の型を使う。

raw bytes、path、OS error、process status、C toolchain argumentは`source`、CLI、`driver`の境界で止める。
core passへ渡す前に`SourceFile`、`Diagnostic`、またはtyped compiler outcomeへ変換する。

source identityとspanのようにpipeline全体で同じ意味を持つ概念だけを明示的に横断させる。診断のための
spanを保持しても、後段がsource textの意味を独自に解析する理由にはならない。

## Source structure

各moduleには一つの安定した責務を持たせる。自然な責務境界がある場合、hand-written source fileは
200行以下を目安にする。500行を超える前にowned behaviorまたは語彙で分割する。

行数を満たすための番号付きfileや恣意的な断片は作らない。generated file、lock file、mechanical fixture、
一箇所でcontractをreviewする必要があるcanonical schemaはこの目安の対象外とする。

## Semantic portability

言語semanticsをhost RustやCの偶発的挙動へ依存させない。特にevaluation order、integer overflow、trap、
floating-point conversion、generated ABIは、該当stageが明示的に保証する。
