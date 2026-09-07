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
| `source` | file bytes、UTF-8 admission、file identity、require graph、byte span、位置計算 |
| `lexer` | 文字列からtokenへのadmissionとlexical error |
| `parser` / `ast` | token列からsource-oriented ASTへのsyntax admission |
| `resolve` | name identity、scope、lexical captureの推論 |
| `types` / `check` | canonical typeとtyped AST、type ruleのvalidation |
| `core` / `anf` / `closure` | desugaring、evaluation order、closure representation |
| `c_emit` | typed lowered programからC translation unitとheaderへの変換 |
| `pipeline` | admitted済みin-memory source graphに対するcompiler stageの構成とstructured outcomeの返却 |
| `editor` | resolved identity、source上のdeclaration/reference、checked typeをeditor queryへ構成 |
| `driver` | source file、require path、temporary path、C compiler process、C build inputのownership |
| `diagnostic` | stage errorを利用者向け表現としてrenderする共通機構 |

predefined scopeの名前とreserved identityは`resolve/predefined`の一つの宣言から生成する。resolver、type checker、
editorはそれぞれ別の一覧を持たず、このmappingを参照する。source declaration用のidentityはreserved identityの
最大値から採番し、primitive追加時に名前表、ID、手動の個数定数を同期しない。

後段が前段のraw inputを再解釈してはならない。未検証入力とadmit済み出力を、optional fieldやflagを持つ
一つの型で兼用しない。許される操作が異なるsemantic stateには別の型を使う。

raw bytes、path、OS error、process status、C toolchain argumentは`source`、CLI、`driver`の境界で止める。
core passへ渡す前に`SourceFile`、`Diagnostic`、またはtyped compiler outcomeへ変換する。

source identityとspanのようにpipeline全体で同じ意味を持つ概念だけを明示的に横断させる。診断のための
spanを保持しても、後段がsource textの意味を独自に解析する理由にはならない。

## 構成経路

public use caseごとに必要なstageだけを構成する。後段を通すこと自体をvalidationの代用にしない。

| Use case | Path | Outcome |
|---|---|---|
| `check`、diagnostic | `source -> lexer -> parser -> resolve -> check` | checked programまたはstructured diagnostic |
| editor semantic query | frontendのresolved programとchecked program `-> editor` | source identityに基づくsemantic index |
| `format` | `source -> lossless lexer -> parser -> formatter` | commentとliteral spellingを保持したsource text |
| `emit-header`、`emit-host` | frontend `-> core::ProgramInterface -> c_emit` | checked host interfaceだけから生成したC headerまたはadapter stub |
| `emit-c` | frontend `-> core -> anf -> closure -> c_emit` | 対になるC translation unitとheader |
| `build` | `emit-c` path `-> driver -> C compiler/linker` | executableまたはexternal-boundary error |

`ProgramInterface`はchecked programからcore境界で一度だけ抽出する。type alias、external type、external operationの
source-level metadataを持ち、ANFとclosure conversionは内容を変更しない。host interfaceだけを生成する経路は
value bindingをlowerせず、このmetadataを直接`c_emit`へ渡す。C translation unitを生成する経路では同じ
`ProgramInterface`をlowered executable bodyと一緒に運ぶ。

`pipeline`はin-memory source graphから上記stageを構成し、filesystemやprocessを扱わない。`driver`はrequire pathを解決して
source graphへadmitし、生成物のpath、temporary directory、C compiler processを所有する。`cli`はargumentを
use caseへ写し、`main`はstdioとprocess exit statusだけを接続する。

## 現在のmodule境界

大きいstageは、stage間の新しい表現を増やさず、stage内部のpolicyで分割する。

| Module | Internal responsibility |
|---|---|
| `parser/expression` | Pratt loop、prefix dispatch、operator precedence |
| `parser/expression/forms` | product、call、extern call、conversion、sum injection |
| `parser/expression/lambda` | parameterとlambda bodyの構成 |
| `parser/expression/control` | `if`、`case`、expression blockの構成 |
| `resolve` | source file内の宣言順序、resolved itemの構成、lambda identity |
| `resolve/files` | require先のpublic name導入、file-private name、program item順序 |
| `resolve/scope` | declaration identity、name lookup、scope stack、重複検査 |
| `resolve/expression` | expression、transitive capture、lambda-local ownershipの解決 |
| `check` | program順序、value environment、checked itemの構成 |
| `check/types` | alias collection、cycle検査、canonical type expansionと表示 |
| `check/interface` | extern transport検査とsource-level alias metadata |
| `check/initializer` | top-level closed-value admission |
| `check/float` | decimal float literalからIEEE 754 binary interchange formatへの正確なrounding |
| `check/float/big_uint` | decimal float roundingだけが使うdependency-freeの非負多倍長整数演算 |
| `formatter/layout` | block compactnessとtop-level groupの事前計算 |
| `formatter/control` | block positionとRHSにある`if`、`case`の事前分類 |
| `formatter/token` | 一般tokenのspacingとsource上の明示的なline breakの保持 |
| `formatter/token/control` | `if`、`case`、block delimiterの出力state遷移 |
| `core/interface` | checked programからhost-visible metadataだけを抽出 |
| `c_emit/syntax` | C translation unit、declaration、expression、statement、definition、preprocessor構文のRust内DSL。構文nodeは最終renderまで保持する |
| `c_emit/syntax/name`、`c_emit/syntax/literal` | identifier、numeric token、string literalなどC terminalへのadmissionとescaping |
| `c_emit/syntax/*/render` | 対応する構文nodeのprecedence、indent、line break、token spelling |
| `c_emit/types::TypeRegistry` | translation unit全体のstructural representation identityとC typeへのmapping |
| `c_emit/types/collect` | lowered programから必要なstructural representationを収集する走査 |
| `c_emit/types/lifetime` | managed typeの分類、internal aggregate copy/destroy、extern前のSymbol materializationの構成 |
| `c_emit/types::HostTypes` | externから到達できるhost-visible typeの分類とheader/source宣言の構成 |
| `c_emit/types/host/product`、`c_emit/types/host/sum` | host-visible aggregateのconstructor、observer、checked projectionの構成 |
| `c_emit/types/host/lifetime` | host-visible managed carrierのclone/take/drop operationの構成 |
| `c_emit/body` | lowered function bodyからC definition群を構成するstateとdispatch |
| `c_emit/body/name` | lowered identityから衝突しないC identifierへのmapping |
| `c_emit/body/ownership` | closure-converted IR上のpath-sensitiveなlast-use解析とtransfer可否の計画 |
| `c_emit/body/closure_use` | local closureのalias追跡、callee-only判定、stack environment候補の計画 |
| `c_emit/body/call` | direct call、tail call、flattened product argumentの解析 |
| `c_emit/body/function` | closure environment、indirect/direct function definitionの構成 |
| `c_emit/body/entry` | program initializerとentry pointの構成 |
| `c_emit/body/entry/arguments` | process argumentからMal entry argumentへのmarshalling |
| `c_emit/body/expression` | operationからC expressionへのdispatch |
| `c_emit/body/expression/primitive` | numeric、comparison、Symbol primitiveのC semantics |
| `c_emit/body/expression/atom` | typed atomのC representation |
| `c_emit/body/statement` | binding operationからstatement emissionへのdispatch |
| `c_emit/body/statement/control` | branch、case、direct tail recursionのcontrol flow |
| `c_emit/body/statement/result` | result bindingとclosure environmentのmaterialization |
| `c_emit/header/prefix` | generated headerのinclude guard、portability macro、runtime ABI prefix |
| `c_emit/runtime/core` | allocation、reference count、Symbol admission、flat/rope lifetimeとmaterialization、host-visible Symbol lifecycle operationの構成 |
| `c_emit/runtime/symbol` | Symbol comparison、byte access、concatenation operationの構成 |
| `c_emit/runtime/symbol/concatenate` | borrowed/consuming concat、unique flat buffer拡張、balanced rope構築の構成 |
| `c_emit/runtime/numeric/conversion` | checked numeric conversionとarithmetic trap helperの構成 |

## Code structure

各moduleには一つの安定した責務を持たせる。自然な責務境界がある場合、hand-written code fileは
200行以下を目安にする。500行を超える前にowned behaviorまたは語彙で分割する。

行数を満たすための番号付きfileや恣意的な断片は作らない。generated file、lock file、mechanical fixture、
一箇所でcontractをreviewする必要があるcanonical schemaはこの目安の対象外とする。

文書の構造と行数基準は[documentation index](../README.md#文書構造)が定める。

## Semantic portability

言語semanticsをhost RustやCの偶発的挙動へ依存させない。特にevaluation order、integer overflow、trap、
floating-point conversion、generated ABIは、該当stageが明示的に保証する。
