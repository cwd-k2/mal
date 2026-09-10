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
| `core` / `anf` / `closure` / `control` | desugaring、evaluation order、closure representation、applicationの明示的control遷移 |
| `execution` | closure-converted programを保持し、closure target、tail fusion、continuation graph、recursive region、call mode、semantic frameをbackend非依存の実行計画として構成 |
| `c_emit` | typed lowered programからC translation unitとheaderへの変換 |
| `pipeline` | admitted済みin-memory source graphに対するcompiler stageの構成とstructured outcomeの返却 |
| `editor` | resolved identity、source上のdeclaration/referenceと型注釈の表示、checked canonical typeをeditor queryへ構成 |
| `driver` | source file、require path、temporary path、C compiler process、C build inputのownership |
| `driver/toolchain` | pinned Clangからhost target tripleとdata layoutを取得し、LLVM/C artifactを同じtargetへcompile |
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
| `parser/expression/forms` | product、call、conversion、sum injection |
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
| `control` | closure-converted blockからcallを含まないstate、terminator、resume frameのlive valueを構成 |
| `execution/closure` | closure creatorとaliasを追跡し、静的に既知のapplication targetを構成 |
| `execution/application` | application siteごとのcaller、known target、型互換なpossible user-function targetを構成 |
| `execution/tail` | direct self tailとpureなknown tail forwarderをcaller continuationと同じ遷移へfusion |
| `execution/continuation` | possible application graphからfusion済みtail edgeを除いたcontinuation edgeを構成 |
| `execution/region` | residual continuation graphのrecursive SCC partitionとregion内site・target所属を構成 |
| `execution/call` | recursive regionからapplicationごとのdirect、self-tail、dispatch判定を導出し、native call graphを非循環化 |
| `execution/frame` | region内non-tail suspension siteからtyped frame、suspensionをまたぐclosure lifetime、arena需要、constructor cardinality、frameが運ぶenvironment ownerを導出 |
| `execution/ownership` | 型がmanaged ownerを含むかの分類と、stage間で保持するatom identityに基づくpath-sensitiveなlast-use、transfer可否を構成 |
| `backend/c` | 移行中のC body oracle、public C header、host stubを各pipeline use caseへ公開 |
| `backend/abi` | LLVM moduleとC shimが共有するinternal pointer/out-pointer bridgeを一つのplanから構成 |
| `backend/llvm` | admission済みexecution planをtarget tripleとdata layoutを持つLLVM moduleおよびC shimへ変換。現在はcaptureを持たないunmanaged valueと単独の`Symbol`、direct call、self-tail edge、unmanaged direct-self continuation frame、数値scalar・`Ptr`のextern callをadmit。managed aggregate、managed frame、`Symbol` externは未admit |
| `backend/llvm/body/types` | LLVM内のunmanaged value type、target pointer size、size、alignment、structural representationを構成 |
| `backend/llvm/body/aggregate` | productとsumのLLVM value構築、case dispatch、payload抽出を構成 |
| `backend/llvm/body/scalar` | 整数・浮動小数点型のLLVM幅、alignment、signedness、literal、instruction選択を構成 |
| `backend/llvm/body/memory` | `Ptr`のbyte offsetと、unalignedな数値scalar・pointer load/storeをLLVM memory operationへ変換 |
| `backend/artifact` | LLVM module、C shim、public headerをsuffix推論なしに型で区別 |
| `backend/runtime` | checked-in C11 runtime sourceをartifact種別とfile名付きで選択 |
| `runtime/c11/core.c` | program非依存のtrap terminalを実装 |
| `runtime/c11/control.c` | frameの型やresume targetを解釈せず、control byte storageのcapacity、growth、releaseを実装 |
| `runtime/c11/symbol.c` | LLVM artifact用のreference-counted flat `Symbol`、観測、連結、外部byte copyを実装。rope表現への置換は同じruntime責務内に留める |
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
| `c_emit/body/control` | typed control frame宣言とcontrol emissionのmodule境界 |
| `c_emit/body/control/local` | direct selfだけからなるcontrol regionのC definition構成 |
| `c_emit/body/control/common` | 複数entryまたはindirect edgeを持つcontrol regionのentry、wrapper、region別C activationの構成 |
| `c_emit/body/control/common/terminator` | region machine内のstate terminator、dispatch、resume、returnの構成 |
| `c_emit/body/control/common/terminator/returning` | typed resultのroot返却とframe resume、active environment ownerのrelease |
| `c_emit/body/control/ownership` | control local slotとframe間のmanaged owner copy、move、cleanupの構成 |
| `c_emit/body/control/support` | reachable state、local slot、activation-local control stackとcached arenaの変換、control operation変換の補助構成 |
| `c_emit/body/name` | lowered identityから衝突しないC identifierへのmapping |
| `c_emit/body/analysis/symbol_at_cursor` | 全self-tail edgeで保持されるSymbol parameterと、そのactivation内だけでcursorを再利用できるbyte access siteの計画 |
| `c_emit/body/analysis/owned_call` | last-use argumentを受け取るowned direct entryのcall graph上の需要計画 |
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
| `c_emit/runtime/core` | runtime contextと各runtime responsibilityの構成順序 |
| `c_emit/runtime/core/allocation` | allocation header、reference count、implementation resource failureの構成 |
| `c_emit/runtime/core/control` | constructor cardinalityに応じたcontrol stack helperの需要選択とheterogeneous aligned byte stackのgrowth operationを構成 |
| `c_emit/runtime/core/control/homogeneous` | homogeneous fixed-width stackのgrowth operationを構成 |
| `c_emit/runtime/core/symbol` | Symbol lifetime、host byte copy、materializationの構成 |
| `c_emit/runtime/symbol` | Symbol byte traversal、comparison、byte access、concatenationの構成 |
| `c_emit/runtime/symbol/leaf_cursor` | allocation-freeなrope leaf順走査とbounded pending pathの構成 |
| `c_emit/runtime/symbol/leaf_cursor/index` | byte indexへのseek、連続accessの検出、非局所accessの通常traversal fallbackの構成 |
| `c_emit/runtime/symbol/concatenate` | borrowed/consuming concat、unique flat buffer拡張、balanced rope構築の構成 |
| `c_emit/runtime/numeric/conversion` | checked numeric conversionとarithmetic trap helperの構成 |

generated programのoptimizationは既存stageの責務を越えて新しい意味論を作らない。managed borrowとtail stateは
`c_emit/body/analysis`と`c_emit/body`、`Symbol`の連続表現は`c_emit/runtime`、host value descriptorとterminal returnは
`c_emit/header`が所有する。着手順と計測gateは
[generated program最適化計画](../development/generated-program-optimization.md)を正とする。

## Code structure

各moduleには一つの安定した責務を持たせる。自然な責務境界がある場合、hand-written code fileは
200行以下を目安にする。500行を超える前にowned behaviorまたは語彙で分割する。

子moduleを持つmoduleは同名directoryの`mod.rs`をrootとし、ownerと子のsourceを同じdirectory treeへ置く。
子を持たないmoduleは親directory直下の単一`.rs` fileに置く。integration testのcrate rootなどtoolingが配置を
規定するfileはその規則を優先する。

行数を満たすための番号付きfileや恣意的な断片は作らない。generated file、lock file、mechanical fixture、
一箇所でcontractをreviewする必要があるcanonical schemaはこの目安の対象外とする。

文書の構造と行数基準は[documentation index](../README.md#文書構造)が定める。

## Semantic portability

言語semanticsをhost RustやCの偶発的挙動へ依存させない。特にevaluation order、integer overflow、trap、
floating-point conversion、generated ABIは、該当stageが明示的に保証する。
