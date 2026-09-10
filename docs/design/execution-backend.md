# 実行backendの責務境界

Status: Current implementation design

この文書はreference compilerが記述programを実行物へ変換するときのLLVM IR、C runtime、public C interfaceの
責務境界を定める。採択理由は[D041](../history/decisions/D041.md)、現在実装のmodule配置は
[compilerの責務境界](../implementation/responsibilities.md)、control変換は
[application control lowering](../development/application-control-lowering.md)、外部根拠は
[LLVM backend調査](../research/llvm-backend.md)を正とする。

## 原則

境界はdataとcontrolの構文上の違いではなく、program固有の実行計画と再利用可能なmechanismの違いで引く。

- LLVM IRは、解析済みMal programに固有の実行計画を所有する。
- C runtimeは、programによらないstorageと汎用operationを所有する。
- C shimとgenerated headerは、hostへ公開するC ABIを所有する。

reference compilerが使用するtarget toolchainはpinned Clang/LLVMに限定し、runtime、shim、host adapterのsource languageはC11とする。
GCCその他のC compilerとのsource compatibility、option compatibility、ABI compatibilityは設計条件にしない。

```text
closure + control
  -> execution plan
  -> backend
       -> generated LLVM module
       -> generated C shim and public header
       -> checked-in C11 runtime
  -> Clang/LLVM compile and link
```

LLVM IRからC statementへjumpする境界は作らない。同じMal functionのstateをC functionへ細分して呼ぶと、stateをまたぐSSA、
register lifetime、inlining、owner transferをcall境界で失う。このためcontrolだけでなく、control state内の軽量な計算を含む
generated function body全体を一つのLLVM optimization unitとして構成する。

## 責務

| Owner | Responsibility |
|---|---|
| Execution plan | application graph、tail fusion、recursive SCC、edge mode、resume liveness、frameが運ぶsemantic valueとowner |
| Generated LLVM IR | function body、basic block、call、branch、dispatch、program固有frame型、scalar演算、aggregate構築・分解、closure entry、typed cleanup |
| C runtime | allocation、reference count機構、control storage growth、Symbol flat storageと操作、fatal resource failure |
| Generated C shim | process entry、LLVM moduleのroot呼出し、extern call marshalling、terminal return、public valueと内部valueの変換 |
| Generated C header | host-visible type、operation definition macro、observer、constructor、public C ABI version |
| Driver | 同一targetと互換toolchainによるLLVM module、runtime C、shim C、requireされたC sourceのcompileとlink、明示された外部toolchain argumentとinspection artifactの配送 |

program固有のdata operationはdataを扱っていてもLLVM IRに属する。product fieldのprojection、sum tag branch、frame fieldへの
owner moveは実行計画の一部である。Symbol storageやreference count更新は別programでも同じmechanismなのでC runtimeに属する。
program固有のclosure environment destructorはfield型と順序を知るためLLVM IRに置き、generic allocation headerのreleaseは
C runtimeを呼ぶ。

## Cとの境界

public host interfaceは現在のC ABIを維持し、LLVM IRの型、calling convention、frame、closure carrierを公開しない。LLVMはCより
低水準であり、`ccc`を指定するだけではsource-level C aggregateのtarget ABI loweringをfrontendに代わって構成しない。このため
初期backendはhost-visible product、sum、SymbolをLLVM function signatureで直接受け渡さない。

LLVM moduleとgenerated C shim、C runtimeの内部bridgeは、`void` result、opaque pointer、input pointer、result out-pointerを
基本とする。fixed-width scalarをsignatureで直接渡す場合や共有record layoutが必要な場合は、一つのbackend ABI planからC
declarationとLLVM type、parameter attributeを生成し、双方でsignature、field、size、alignmentを再定義しない。bridge symbolは
public headerへ出さない。

driverはgenerated module、runtime C、C shim、利用者がrequireしたC sourceを同じLTO unitとしてcompile、linkする。LTOは
`mal_control_reserve_frame`のcapacity fast pathなど、責務境界に置いた小さいhelperのinliningとcross-module optimizationに使う。
正しさ、ABI一致、stack boundはLTOへ依存させず、LTOを無効にしても同じobservable resultを保つ。

## control storage

LLVM backendもnon-tail recursive regionには明示的なgrowable continuation storageを使う。LLVMのnative stackへ配置を委ねるのは、
一つのactivation内でlifetimeが閉じるlocalだけである。tail edgeはdirect branchまたは条件を満たす`musttail` callにできるが、
return後に使うlive valueを持つnon-tail edgeは深さに比例したstorageを必要とする。

frame constructor、typed payload、resume targetはprogram固有なのでLLVM IRが構成する。arena allocation、growth、capacity overflow、
cacheはC runtimeが構成する。managed valueをframeへmoveし復元する順序はexecution planが定め、LLVM backendがtyped operationへ
refineする。

既知のstate遷移はLLVM basic blockへの直接branchにし、runtime target選択が必要な箇所だけdispatchする。全遷移を一つのcentral
dispatcherへ戻すことや、多数のpredecessorを持つblockを無条件に作ることをbackendの正しさの条件にしない。

LLVM coroutine intrinsicは初期backendに使わない。malは外部resume、suspended coroutine identity、destroy operationを必要とせず、
recursive regionではcoroutine frame allocationのelisionも通常期待できない。既存control IRのframeとresumeを直接lowerする。

## targetとartifact

LLVM moduleはdriverが選んだtarget tripleとdata layoutを持ち、同じtargetへcompileしたC artifactとのみlinkする。LLVM textual IRや
bitcodeをtargetおよびLLVM versionから独立した配布形式とは扱わない。永続的なpublic artifactはC headerと最終objectまたは
executableであり、IR出力を公開する場合は使用toolchainとtargetに結びつくdiagnostic/development artifactとする。

artifactの具体像は[LLVM backend生成物例](../development/llvm-backend-artifacts.md)、採用後の最適化gateは
[generated program最適化policy](../development/generated-program-optimization.md)に置く。
