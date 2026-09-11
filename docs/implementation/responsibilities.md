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
| `execution` | closure-converted programを保持し、semantic application factsと明示的に選択されたoptimization decisionから、continuation graph、recursive region、call mode、semantic frameをbackend非依存の実行計画として構成 |
| `backend/c` | `ProgramInterface`からpublic C headerとhost stubへの変換 |
| `pipeline` | admitted済みin-memory source graphに対するcompiler stageの構成とstructured outcomeの返却 |
| `editor` | resolved identity、source上のdeclaration/referenceと型注釈の表示、checked canonical typeをeditor queryへ構成 |
| `driver` | source file、require path、temporary path、C compiler process、C build input、およびbuild optimization profileの選択 |
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
| `format` | `source -> lossless lexer -> parser -> formatter` | commentとliteral spellingを保持したsource text |
| `emit-header`、`emit-host` | frontend `-> core::ProgramInterface -> backend/c` | checked host interfaceだけから生成したC headerまたはadapter stub |
| `build` | frontend `-> execution -> LLVM module + C shim/runtime -> pinned Clang` | executableまたはexternal-boundary error |

`ProgramInterface`はchecked programからcore境界で一度だけ抽出する。type alias、external type、external operationの
source-level metadataを持ち、ANFとclosure conversionは内容を変更しない。host interfaceだけを生成する経路は
value bindingをlowerせず、このmetadataを直接`backend/c`へ渡す。`build`では同じ`ProgramInterface`をLLVM executable bodyと
C shimの共通ABI planへ渡す。

`pipeline`はin-memory source graphから上記stageを構成し、filesystemやprocessを扱わない。`driver`はrequire pathを解決して
source graphへadmitし、生成物のpath、temporary directory、C compiler processを所有する。`cli`はargumentを
use caseへ写し、`main`はstdioとprocess exit statusだけを接続する。

## 現在のmodule境界

大きいstageは、stage間の新しい表現を増やさず、stage内部のpolicyで分割する。

| Module | Internal responsibility |
|---|---|
| `parser/expression` | Pratt loop、prefix dispatch、operator precedence |
| `parser/expression/forms` | product、双方向application、conversion、sum injection constructor |
| `parser/expression/lambda` | parameterとlambda bodyの構成 |
| `parser/expression/control` | `if`とexpression blockの構成 |
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
| `formatter/control` | block positionとRHSにある`if`の事前分類 |
| `formatter/token` | 一般tokenのspacingとsource上の明示的なline breakの保持 |
| `formatter/token/control` | `if`とblock delimiterの出力state遷移 |
| `core/interface` | checked programからhost-visible metadataだけを抽出 |
| `control` | closure-converted blockからcallを含まないstate、terminator、resume frameのlive valueを構成 |
| `control/liveness` | stateごとのlocal valueとclosure environmentのbackward livenessを構成 |
| `execution/closure` | closure creatorとaliasを追跡し、静的に既知のapplication targetを構成 |
| `execution/application` | application siteごとのcaller、known target、型互換なpossible internal function targetを構成 |
| `execution/optimization` | 空集合でも成立するexecution baselineに対し、有効化された個別techniqueのprogram固有decisionを構成し、競合しない一つのplanへ集約 |
| `execution/optimization/self_tail` | direct self tailをcaller continuationと同じ遷移へfusionできるsiteを判定 |
| `execution/optimization/tail_forwarder` | pureなknown tail forwarderをcaller continuationと同じ遷移へfusionできるsiteとargumentを判定 |
| `execution/optimization/direct_call` | semantic application factからknown targetをdirect call decisionへ選択 |
| `execution/continuation` | possible application graphから選択済みcontinuation elisionを除いたcontinuation edgeを構成 |
| `execution/region` | residual continuation graphのrecursive SCC partitionとregion内site・target所属を構成 |
| `execution/call` | recursive regionからapplicationごとのdirect、self-tail、dispatch判定を導出し、native call graphを非循環化 |
| `execution/parameter` | function parameterのcontrol bindingをcall mode共通の`Bind`または`Discard` destinationへ変換 |
| `execution/frame` | region内non-tail suspension siteからtyped frame、live value、およびframeが運ぶenvironment ownerを導出 |
| `execution/frame/resume` | 同じcontrol machineに属するreturn siteとframeについて、resume可能または到達不能な組合せを導出 |
| `execution/ownership` | 型がmanaged ownerを含むかをbackend間で共通に分類 |
| `backend/c` | public C headerとhost stubを`ProgramInterface`から構成 |
| `backend/abi` | LLVM moduleとC shimが共有するinternal pointer/out-pointer bridgeを一つのplanから構成 |
| `backend/llvm` | admission済みexecution planをtarget tripleとdata layoutを持つLLVM moduleおよびC shimへ変換。managed captureを持つfirst-class function、managed productとsum、direct・indirect call、self-tail edge、recursive regionのtyped continuation frame、transportableなextern callをadmit |
| `backend/llvm/host_bridge` | extern parameterとresultについて、LLVM value storageとpublic C host valueの間の再帰的marshallingを構成 |
| `backend/llvm/shim` | process argument descriptorの構築とinternal root bridgeを呼ぶC11 entry pointを構成 |
| `backend/llvm/body/types` | LLVM内のvalue type、target pointer size、size、alignment、structural representationを構成 |
| `backend/llvm/body/plan` | root、reachable state、slot、およびcheckerがadmitしたclosed top-level valueのtarget-specific LLVM constant planを構成 |
| `backend/llvm/body/aggregate` | productとsumのLLVM value構築、case dispatch、payload抽出を構成 |
| `backend/llvm/body/value` | local slot、product field、function境界にあるmanaged ownerの再帰的なretain、transfer、releaseを構成 |
| `backend/llvm/body/ownership` | control CFGのbackward livenessからbinding後にresponsibilityを失うdead ownerを導出 |
| `backend/llvm/optimization` | 空集合でも成立するLLVM loweringに対し、有効化されたtarget固有techniqueのemission decisionを構成 |
| `backend/llvm/optimization/symbol_concat` | dead owner factから`Symbol` concatへmoveしてよいoperandを選択し、storage再利用可能なruntime operationを指示 |
| `backend/llvm/body/frame` | direct-selfおよび共通recursive regionのcontinuation frame layout、code-pointer dispatch、suspend/resume時のlive ownerとactive environmentのtransferを構成 |
| `backend/llvm/body/scalar` | 整数・浮動小数点型のLLVM幅、alignment、signedness、literal、instruction選択を構成 |
| `backend/llvm/body/memory` | `Ptr`のbyte offsetと、unalignedな数値scalar・pointer load/storeをLLVM memory operationへ変換 |
| `backend/artifact` | LLVM module、C shim、public headerをsuffix推論なしに型で区別 |
| `backend/runtime` | checked-in C11 runtime sourceをartifact種別とfile名付きで選択 |
| `runtime/c11/core.c` | program非依存のtrap terminalを実装 |
| `runtime/c11/control.c` | frameの型やresume targetを解釈せず、control byte storageのcapacity、growth、releaseを実装 |
| `runtime/c11/symbol.c` | LLVM artifact用のreference-counted flat/rope `Symbol` storage、allocation-freeな観測と外部byte copy、平衡連結、一意なflat storageの再利用、C host境界での遅延materializationを実装 |
| `runtime/c11/symbol_internal.h` | C runtime内のprivate `Symbol` carrier、LLVM static leafと共有するheader layout、cursor stateを宣言 |
| `backend/c/syntax` | public headerとhost stubが実際に使うC declaration、expression、statement、preprocessor構文だけを型付きnodeとして保持しrender |
| `backend/c/syntax/name`、`backend/c/syntax/literal` | identifier、numeric token、string literalなどC terminalへのadmissionとescaping |
| `backend/c/syntax/*/render` | 対応する構文nodeのprecedence、indent、line break、token spelling |
| `backend/c/types::TypeRegistry` | host interface全体のstructural representation identityとC typeへのmapping |
| `backend/c/types/collect` | `ProgramInterface`からhost-visibleなstructural representationを収集する走査 |
| `backend/c/types::HostTypes` | externから到達できるhost-visible typeの分類とheader/source宣言の構成 |
| `backend/c/types/host` | host-visible aggregateのconstructor、observer、checked projection、およびmanaged carrier operationの構成 |
| `backend/c/types/host/declaration` | host adapter内部表現とpublic C headerに必要なhost-visible type declarationを構成 |
| `backend/c/header/prefix` | generated headerのinclude guard、portability macro、runtime ABI prefix |

generated programのoptimizationは既存stageの責務を越えて新しい意味論を作らない。program固有のcontrolとowner操作は
`backend/llvm`、`Symbol`のstorage表現は`runtime/c11/symbol.c`、host value descriptorとterminal returnは`backend/c/header`が所有する。着手順と計測gateは
[generated program最適化policy](../development/generated-program-optimization.md)を正とする。

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
