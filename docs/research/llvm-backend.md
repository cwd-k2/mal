# LLVM execution backend調査

Status: Informative

2026-09-10時点。この文書はLLVM IRが所有できる最適化と、frontendに残るstorage、ABI、targetの責務を確認する。
malで採択した境界は[実行backendの責務境界](../design/execution-backend.md)を正とする。

## stackとtail call

[LLVM Language Referenceの`alloca`](https://llvm.org/docs/LangRef.html#alloca-instruction)は、確保領域を現在実行中のfunctionの
stack frameに属し、そのfunctionがreturnすると解放されるものとして定義する。`alloca`はlocal storageの配置を表すが、
non-tail recursionが保存する未完了のevaluation contextをbounded storageへ変換しない。

同じ文書の[`musttail`](https://llvm.org/docs/LangRef.html#call-instruction)は、recursive call graph cycleでもunboundedなstack
growthを起こさないことを保証する。一方、call直後の`ret`、一致するcalling conventionとprototype、ABIに影響するattributeの一致などを
要求する。このためtail applicationには使えるが、callee resultへ処理を加えるnon-tail applicationの代替にはならない。

## coroutine

[LLVM coroutine documentation](https://llvm.org/docs/Coroutines.html)では、suspendをまたいでliveな値をcoroutine frameへspillし、
必要なら動的に確保する。caller内の`alloca`へframe allocationをelideできる場合もあるが、recursiveまたはmutually recursiveな
functionでは通常このelisionを適用できない。同文書はcoroutine intrinsicのLLVM release間compatibilityを保証しない。

malはapplicationのreturn先を内部continuationとして持つが、外部からresumeまたはdestroyするcoroutine objectを言語に公開しない。
coroutine loweringを使ってもframe storageとallocation policyは消えないため、既存control IRの直接loweringより小さくならない。

## 最適化しやすいIR

[Performance Tips for Frontend Authors](https://llvm.org/docs/Frontend/PerformanceTips.html)は、moduleにtarget tripleとdata layoutを
含め、可能な限りprivateなlinkageを使うことを勧める。また高いin-degreeを持つbasic blockはregister allocationに不利になり得るとし、
function localを`alloca`で表す場合はentry block先頭へ置くことを勧める。SROAとMem2Regが除去できるlocalはSSAへ昇格できる。

malへの含意:

- generated function、frame type、helperは外部公開が必要なもの以外をmodule内へ閉じる。
- known state transitionは直接branchにし、動的callee選択だけをdispatcherへ集める。
- explicit continuation arenaと、SSAへ昇格可能なactivation-local slotを同じstorageとして扱わない。
- 既存generated CをClangでLLVM IRへ変換し、direct backendのIR shapeと比較する。

## C ABIとtarget

[LLVM FAQ](https://llvm.org/docs/FAQ.html#can-i-compile-c-or-c-code-to-platform-independent-llvm-bitcode)は、LLVM IRがCより低水準で、
platform C ABIへ適合するIRはfrontendがtargetごとに生成する必要があると説明する。LangRefの`ccc`はtarget C calling conventionを
選ぶが、source aggregateをどのLLVM parameter、coercion、`sret`、`byval`へlowerするかを自動決定するsource-level ABI layerではない。

LLVMのfrontend guidanceはClangとLLVM libraryを同じrevisionまたはreleaseで使うよう求める。moduleのtarget tripleとdata layout、
C runtimeをcompileするClang、IR optimizerとcode generatorを同じtoolchain selectionに束ねる必要がある。

malへの含意:

- host-visible aggregate ABIはgenerated C shimに閉じ、LLVMとの内部bridgeはpointerとout-pointerを基本にする。
- scalarを直接渡すbridgeは、C declarationとLLVM parameter attributeを一つのtarget-aware planから生成する。
- LLVM moduleとC artifactは同じtargetへcompileし、異なるdata layoutをlinkしない。
- textual LLVM IRをversion-independentなpublic interchange formatにしない。

## C runtimeとLTO

[Clang ThinLTO documentation](https://clang.llvm.org/docs/ThinLTO.html)では、C translation unitをLLVM bitcodeとして出力し、link時に
cross-module optimizationとfunction importingを行える。したがって汎用runtimeをC sourceに保つことは、runtime callを永久に
optimization barrierにすることを意味しない。

LTOの有無でobservable semantics、public ABI、resource failure、stack boundが変わってはならない。初期実装ではnative object間の
bridgeを正しさのbaselineにし、LTOは独立したperformance optionとして測定する。
