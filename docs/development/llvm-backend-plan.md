# LLVM backend実装計画

Status: Accepted work plan

この文書は[実行backendの責務境界](../design/execution-backend.md)を現在のC backendから実装するstage、module構成、検証gateを
定める。現在のmodule責務は[compilerの責務境界](../implementation/responsibilities.md)、test commandは
[test policy](testing.md)を正とする。

## 完了形

```text
compiler/
├── runtime/c11/
│   ├── runtime.h
│   ├── core.c
│   ├── control.c
│   ├── memory.c
│   └── symbol/
│       ├── core.c
│       ├── leaf_cursor.c
│       └── concatenate.c
└── src/
    ├── execution/
    │   ├── ast.rs
    │   ├── application.rs
    │   ├── continuation.rs
    │   ├── region.rs
    │   ├── frame.rs
    │   ├── closure.rs
    │   ├── ownership.rs
    │   └── mod.rs
    ├── backend/
    │   ├── abi/
    │   ├── llvm/
    │   │   ├── body/
    │   │   ├── syntax/
    │   │   ├── types.rs
    │   │   └── mod.rs
    │   ├── c/
    │   │   ├── interface/
    │   │   ├── shim/
    │   │   ├── syntax/
    │   │   └── mod.rs
    │   ├── artifact.rs
    │   ├── runtime.rs
    │   └── mod.rs
    ├── pipeline.rs
    └── driver/
        ├── toolchain.rs
        └── mod.rs
```

directoryは実装する責務が生じた時点でだけ作り、空のmoduleを先行作成しない。file分割は実際のowned behaviorに従い、上図を
一度にscaffoldする完了条件にはしない。

## Stageとauthority

| Area | Authority |
|---|---|
| `control` | callを含まないstate、terminator、state liveness |
| `execution` | `ProgramInterface`を変更せず保持し、application target、tail fusion、continuation graph、recursive region、edge mode、semantic frame、owner transferを構成 |
| `backend::abi` | LLVMとCが共有するinternal bridge signature、type、attribute、layout identity |
| `backend::llvm` | execution planからLLVM type、function、basic block、instructionへのrefinement |
| `backend::c::interface` | `ProgramInterface`からpublic C headerとhost stubへの変換 |
| `backend::c::shim` | process entry、extern bridge、public/internal value変換 |
| `runtime/c11` | program非依存のstorage、reference count、control growth、memory、Symbol、rope operation |
| `backend::runtime` | checked-in C11 runtime sourceの選択とcompiler binaryへの同梱 |
| `backend` | 互いに対応するLLVM module、C shim、header、runtime inputを一つのartifact setへ構成 |
| `pipeline` | frontend、lowering、execution plan、backend generationのin-memory構成 |
| `driver::toolchain` | pinned Clang/LLVMのtarget admission、process argument、status、diagnostic |
| `driver` | source graph、temporary artifact、requireされたC input、compile、link、最終出力 |

`execution`にはbackend間で一致すべき正しさの判断だけを置く。LLVM block配置、`alloca`、linkage、instruction attribute、runtime helper
選択は`backend::llvm`に置く。Symbol cursorなど特定表現だけの最適化は、それがsemantic lifetimeを変えない限りbackend-local planに
残す。

## API境界

`pipeline::emit_c`を新構成の中心にしない。pipelineはtarget非依存の実行計画までを構成し、artifact生成はbackendへ渡す。

```text
pipeline::lower_execution(graph) -> execution::Program
backend::generate(program, target) -> backend::Artifacts
backend::emit_header(interface) -> CHeader
backend::emit_host(interface, header_name) -> CSource
```

`backend::Artifacts`は少なくともLLVM module、generated C shim、public header、必要なruntime inputを型で区別して持つ。driverは
文字列のfile suffixからartifact種別を再推論しない。program生成では`execution::Program`が保持するinterfaceだけを使い、同じ
interfaceを別引数で受け取らない。header-only pathはvalue bindingをlowerせず、引き続き`ProgramInterface`から直接生成する。

public CLIの`emit-c`は現在のC oracleが存在する間だけ維持する。LLVM backendへのcutover時に単一C translation unitというcontractを
終了し、inspection用LLVM IRと複数artifactを一つのcommandへ混ぜるかは別のCLI decisionとして決める。`build`、`emit-header`、
`emit-host`の利用者向け目的は維持する。

## Clang/LLVMとC11

reference compilerのtarget toolchainはpinned Clang/LLVMだけとし、GCCその他のC compilerとの互換分岐を新設しない。runtime、shim、
host adapterはC11としてClangでcompileし、LLVM moduleも同じtoolchainのtarget tripleとdata layoutを使う。linkerはClang driverから
起動する。

移行時にambient `CC`によるcompiler差し替えを廃止し、`__GNUC__` fallbackと任意C compiler向けdiagnosticを削除する。Clang extensionを
使う場合はC11だけでは表せない責務に限定し、public headerで必要なextensionとversionを明記する。runtime Cのbitcode化とLTOは
同じClangで行うoptional optimizationとし、正しさの前提にしない。

## 実装順

1. 現在のC backend outputとnative behaviorをoracle fixtureとして固定する。
2. `control`をpipelineで明示的に構成し、`c_emit/body/analysis`からbackend-independentな解析を`execution`へ移す。C outputを変えない。（完了）
3. `c_emit`のpublic interface、C syntax、runtime生成、body生成のAPIを分け、移行中のbody emitterをlegacy oracleとして隔離する。
4. internal bridge ABI planを導入し、同じplanからC declarationとLLVM signature、parameter attributeを生成してClangでlink検査する。（pointer/out-pointer root bridgeは完了）
5. scalarだけのfunction body、branch、direct call、tail edgeをLLVM IRへlowerし、C shimからrootを呼ぶ。（整数・浮動小数点型と`Ptr`は完了）
6. direct-self non-tail region、homogeneous frame、managed ownerのないresumeをLLVMへ移す。（unmanagedな数値scalar frameは完了）
7. heterogeneous frame、managed owner、closure environment、indirect recursive regionを順に移す。（数値型、`Unit`、`Bool`、product、sumからなるunmanaged direct-self frameと複数constructorは完了）
8. generated runtime logicをchecked-in C11 sourceへ移し、control、allocation、Symbol、ropeの順にRust C emitterから削除する。（LLVM artifact用core・control runtimeは完了。C oracle側の削除と他runtimeは未完了）
9. extern、entry、terminal returnをgenerated C shimへ分離し、全host ABI fixtureを新artifact setで通す。（数値scalar・`Ptr` externと`Unit -> Int32` entryは完了）
10. `build`をClang/LLVM artifact pipelineへ切り替え、ambient `CC`とGCC compatibilityを削除する。
11. C body oracleとの差分検査と性能gateを通した後、legacy body emitterと`emit-c` contractを退役する。

各stepは使われないfuture moduleを作らず、移したauthorityの旧constructorと再解析を同じchangeで削除する。

## Verification gate

- LLVM IRはselected Clangでparse、verify、optimize、object化する。
- C shimとruntimeはC11 warningをerrorにして同じClangでcompileする。
- C oracleとresult、evaluation order、extern trace、trap、managed lifetimeを差分検査する。
- recursive fixtureでnative stack使用量がMal recursion depthに比例しないことを検査する。
- frame growth後のpointer再取得、ownerの一意性、terminal returnをfocused testで検査する。
- LTOなしをcorrectness baselineとし、LTOありでも同じobservable resultになることを検査する。
- 既存generated CをClangが生成した最適化後IRとdirect LLVM IRを比較し、instruction、branch、allocation counter、wall-clockのうち
  少なくとも二つで移行理由を確認する。
