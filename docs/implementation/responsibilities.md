# Compiler の責務境界

Status: Current v0.6 implementation policy

この文書はcompiler codeの分類、各stageのownership、表現の変換境界を定める。pipelineの構成は
[compiler implementation notes](compiler.md)、言語の挙動は[`spec/`](../spec/)をauthorityとする。

実装上のminimalityはcode量の最小化ではない。意味を所有するauthorityを一箇所に置き、後続stageがその事実を
順方向に保存・消費でき、表現の形から意味を逆推論しないことを指す。局所的な短さより、原則から素直に導けることと
stage間の摩擦の少なさを優先する。

## 分類

codeの配置は依存library、interfaceの有無ではなく、扱う語彙と実装するpolicyで決める。crateの分け方は次節に示す。

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

## crate構成

crateの境界は、その層だけを必要とする別々の利用者がいるかで引く。外部依存を持つのは`mal-lsp`だけであり、ほかのcrateは標準libraryだけで構成する。

| crate | 所有するstage | 依存 | 利用者 |
|---|---|---|---|
| `mal-syntax` | `source`、`diagnostic`、`lexer`、`ast`、`parser`、source graphの読み込み | なし | 全crate |
| `mal-fmt` | formatter、`mal-fmt` command | `mal-syntax` | 利用者、`mal-lsp` |
| `mal-frontend` | `resolve`、`check`（specialization含む）、`editor`、`analysis` | `mal-syntax` | `mal-backend`、`mal-compiler`、`mal-lsp` |
| `mal-backend` | `core`、`anf`、`closure`、`control`、`execution`、`backend`、C11 runtime、`pipeline` | `mal-syntax`、`mal-frontend` | `mal-compiler` |
| `mal-compiler` | `cli`、`driver`（source graphの配置、file出力、Clang/LLD process）、`malc` command | 上記すべて | 利用者 |
| `mal-lsp` | LSP server | `mal-syntax`、`mal-frontend`、`mal-fmt` | editor |

`mal-backend`はfilesystemもprocessも扱わず、生成物を文字列として返す。`malc`だけがfileを書き、Clangを起動する。frontendより下のcrateは
backendとClangを知らないので、formatterとlanguage serverはcompilerの依存を引かない。

## Stageのownership

各stageは直前の表現をadmitして次の表現へ変換する。入力では直前stageの語彙を使ってよいが、成功時の
出力は自stageで検証済みの型にする。

| Area | Ownership |
|---|---|
| `main` | argument sourceとI/Oの接続、exit statusの配送 |
| `cli` | command grammar、利用エラー、use caseの選択 |
| `source` | file bytes、UTF-8 admission、file identity、byte span、位置計算 |
| `graph`、`requirement` | source graphのfilesystem読み込み（open documentのoverlayを含む）、require pathの相対解決と補完候補、requirementのcycle検出 |
| `lexer` | 文字列からtokenへのadmissionとlexical error |
| `parser` / `ast` | token列からsource-oriented ASTへのsyntax admission |
| `formatter` | lossless lexerとparserの結果から、commentとliteral spellingを保持したcanonical source textを構成 |
| `resolve` | name identity、scope、lexical captureの推論 |
| `types` / `check` | canonical typeとtyped AST、type ruleのvalidation、entry bindingのidentityとadmitted parameter form |
| `core` / `anf` / `closure` / `control` | desugaring、evaluation order、closure representation、applicationの明示的control遷移 |
| `execution` | closure-converted programを保持し、semantic application factsと明示的に選択されたoptimization decisionから、continuation graph、recursive region、call mode、semantic frame、managed responsibility factをbackend非依存の実行計画として構成 |
| `backend/c` | `ProgramInterface`からpublic C headerとhost stubへの変換 |
| `backend/llvm` | admitted execution planからLLVM moduleとC shimへの変換 |
| `backend/abi` | LLVM moduleとC shimが共有するinternal bridgeのABI planを一つ構成 |
| `backend/source_layout` | runtime value layoutと独立に、canonical memoryのstride、alignment、offsetをtarget data layoutから構成 |
| `runtime/c11` | program非依存のC11 mechanism。allocation、reference count、control storage、Symbol operation |
| `analysis`、`pipeline` | admitted済みin-memory source graphに対するcompiler stageの構成とstructured outcomeの返却 |
| `editor` | current tokenから作るsyntax indexと、resolved identity・source上のdeclaration/reference・checked typeから作るsemantic indexをeditor queryへ構成 |
| `driver` | 生成物のpath、temporary path、C compiler process、C build input、および`--optimization`の選択 |
| `driver/toolchain` | pinned Clangからhost target tripleとdata layoutを取得し、LLVM/C artifactを同じtargetへcompile |
| `driver/toolchain/optimization` | semantic correctness optionから独立したbaselineまたはproductionのClang optimization argumentを構成 |
| `diagnostic` | stage errorを利用者向け表現としてrenderする共通機構 |

predefined scopeの名前とreserved identityは`resolve/predefined`の一つの宣言から生成する。resolver、type checker、
editorはそれぞれ別の一覧を持たず、このmappingを参照する。source declaration用のidentityはreserved identityの
最大値から採番し、primitive追加時に名前表、ID、手動の個数定数を同期しない。

後段が前段のraw inputを再解釈してはならない。未検証入力とadmit済み出力を、optional fieldやflagを持つ
一つの型で兼用しない。許される操作が異なるsemantic stateには別の型を使う。

stage constructorは成功時に自身のinvariantを満たす型だけを返す。authorityから結果集合を再構成する構造validatorは、
外部入力のadmissionではなくcompiler内部の整合性検査である。通常testとdebug buildでは`debug_assert!`で実行し、release buildの
恒常costにはしない。利用者入力により失敗し得る規則はvalidatorへ委ねず、所有stageがstructured diagnosticとして常に検査する。

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
| `mal-fmt` | `source -> lossless lexer -> parser -> formatter` | canonical source textまたはstructured diagnostic |
| `emit header`、`emit host` | frontend `-> core::ProgramInterface -> backend/c` | checked host interfaceだけから生成したC headerまたはadapter stub |
| `build` | frontend `-> execution -> LLVM module + C shim/runtime -> pinned Clang` | executableまたはexternal-boundary error |
| `emit atcoder` | `build`と同じ生成入力 `-> pinned Clang/LLD -> assembly carrier` | mal sourceをcommentに保持した単一C++ sourceまたはexternal-boundary error |

`ProgramInterface`はchecked programからcore境界で一度だけ抽出する。type alias、external type、external operationの
source-level metadataを持ち、ANFとclosure conversionは内容を変更しない。host interfaceだけを生成する経路は
value bindingをlowerせず、このmetadataを直接`backend/c`へ渡す。`build`では同じ`ProgramInterface`をLLVM executable bodyと
C shimの共通ABI planへ渡す。

`analysis`（`mal-frontend`）と`pipeline`（`mal-backend`）はin-memory source graphから上記stageを構成し、filesystemやprocessを扱わない。
`driver`は`mal-syntax`が読み込んだsource graphを受け取り、生成物のpath、temporary directory、C compiler processを所有する。`cli`はargumentを
use caseへ写し、`main`はstdioとprocess exit statusだけを接続する。

## generics、memory、host境界の変換

genericsとexternal memoryも既存stageのadmission責務に従う。

| Boundary | Responsibility |
|---|---|
| lexer/parser | generic parameter/argument、postfix chain、共有tokenをsource-oriented ASTへ構成する。型やnameから構文を選ばない |
| resolve | generic bindingと型parameterへidentityを与え、concrete type argument付きvalue referenceを対応するbindingへ結ぶ |
| check | canonical generic type、arity、`Requirements(T)`、`Storable`、`Representable`、`HostMappable`、memory operatorの型を検査する。`from`と`buffer.into`はrepresentableな要素だけを受理する |
| specialization | checkerが確定したentry identityから到達するvalue bindingをsource順に選び、checked generic identityとcanonical concrete argumentをkeyにinstanceを共有して、単相checked programをcoreへ渡す |
| core以降 | open type parameter、requirement、layout dictionaryを受け取らず、concrete typeとprimitiveだけを扱う |
| backend source layout | runtime value layoutと独立した共有target layout planを作り、LLVM memory loweringとC canonical memory helperへ同じstrideとoffsetを供給する |
| execution ownership | `Buffer`をmanaged valueとして分類し、elementのAddress referentへownershipを拡張しない |
| LLVM Buffer element | 要素のstorage layoutを選び、`Symbol`を含む要素にはretainとreleaseのcallbackを生成してruntimeへ渡し、`get`と`put`のreference操作を出力する |
| runtime | managed Buffer storage、要素callbackによるreferenceの取得と解放、Unitのcount-only表現、Symbol snapshot copyを実装する |
| C interface | HostMappableな型だけをABI 0x000800とpublic headerへ写し、SymbolとBufferをpublic interfaceから拒否する |
| process shim | `argv + 1`をcopyして作ったargument Bufferを`Buffer<Symbol>` rootへ渡し、return後に解放する |

memory preconditionはcheckerやruntimeの防御機構へ移さない。backendはpreconditionを満たすinputの意味を実装し、内部corruptionを
避ける検査を置く場合もsource-level trapとして公開しない。target capability、型形成、host mappingのようにartifact生成前に
判定できる条件は、所有stageがstructured diagnosticとして拒否する。

## module構成

大きいstageは、stage間の新しい表現を増やさず、stage内部のpolicyでmoduleに分ける。moduleごとの責務は各directoryの
`README.md`とcodeを正とし、この文書にはmoduleの一覧を置かない。

- `mal-syntax`: [`parser`](../../crates/mal-syntax/src/parser/README.md)
- `mal-fmt`: [crate README](../../crates/mal-fmt/README.md)
- `mal-frontend`: [`resolve`](../../crates/mal-frontend/src/resolve/README.md)、[`check`](../../crates/mal-frontend/src/check/README.md)、[`editor`](../../crates/mal-frontend/src/editor/README.md)
- `mal-backend`: [`core`から`control`まで](../../crates/mal-backend/src/core/README.md)、[`execution`](../../crates/mal-backend/src/execution/README.md)、[`backend/llvm`](../../crates/mal-backend/src/backend/llvm/README.md)、[`backend/c`](../../crates/mal-backend/src/backend/c/README.md)、[`runtime/c11`](../../crates/mal-backend/runtime/c11/README.md)
- `mal-compiler`: [`driver`](../../crates/mal-compiler/src/driver/README.md)
- `mal-lsp`: [crate README](../../crates/mal-lsp/README.md)

generated programのoptimizationは既存stageの責務を越えて新しい意味論を作らない。program固有のowner successorは
`execution/ownership`、そのtyped LLVM operationは`backend/llvm`、共通byte ownerは`runtime/c11/bytes.c`、`Symbol` operation policyは
`runtime/c11/symbol.c`、HostMappableなhost valueとterminal returnは`backend/c/header`が所有する。着手順と計測gateは
[generated program最適化policy](../development/generated-program-optimization.md)を正とする。

## Code structure

各moduleには一つの安定した責務を持たせる。自然な責務境界がある場合、hand-written code fileは
200行以下を目安にする。500行を超える前にowned behaviorまたは語彙で分割する。crateの外から使わない項目は`pub(crate)`に
とどめ、`unreachable_pub` lintが余分な`pub`を検出する。

子moduleを持つmoduleは同名directoryの`mod.rs`をrootとし、ownerと子のsourceを同じdirectory treeへ置く。
子を持たないmoduleは親directory直下の単一`.rs` fileに置く。integration testのcrate rootなどtoolingが配置を
規定するfileはその規則を優先する。

行数を満たすための番号付きfileや恣意的な断片は作らない。generated file、lock file、mechanical fixture、
一箇所でcontractをreviewする必要があるcanonical schemaはこの目安の対象外とする。

文書の構造と行数基準は[documentation index](../README.md#文書構造)が定める。

## Semantic portability

言語semanticsをhost RustやCの偶発的挙動へ依存させない。特にevaluation order、integer overflow、trap、
floating-point conversion、generated ABIは、該当stageが明示的に保証する。
