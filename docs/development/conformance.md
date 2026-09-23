# v0.6 conformance matrix

Status: Current v0.6 verification matrix

この文書は[`spec/`](../spec/)の規範を実装完了に必要なobservable evidenceへ対応させる。test layerとcommandは
[test policy](testing.md)を正とする。各行はfocusedなpositive、negative、edge caseと、必要なcross-boundary pathを要求する。
test function名やmodule配置は実装が所有し、この文書では固定しない。

## Frontendとsurface

| Authority | Focused evidence | Cross-boundary evidence |
|---|---|---|
| [grammar](../spec/grammar.md) | 全precedence levelの隣接、prefix/postfix/binary共有token、generic `<>`とcomparison/`>>`、formatter idempotence | parseからchecked programまでの代表的なgeneric memory source |
| [numeric conversion](../spec/expressions.md#primitive-operator) | 全closed suffix、rounding、modulo、float-to-integer precondition | conversionを含むLLVM artifactのcompile/execute |
| [types](../spec/types.md) | indexed type arity、alias expansion、recursive alias、type position以外のTYPE_IDENT rejection | editor hover/navigationとgenerated diagnostic |
| [program](../spec/programs.md) | generic top-level initializer、source order、entry signature、zero argument pointer | `(USize, Address)` process entryを実際のargvで実行 |

## Generics

| Authority | Focused evidence | Cross-boundary evidence |
|---|---|---|
| [declaration/application](../spec/generics.md#declarationとapplication) | duplicate parameter、arity mismatch、explicit application、first-classな単相value、generic extern rejection | required fileを跨ぐgeneric application |
| [requirements](../spec/generics.md#requirements) | signature内のnested indexed type、alias展開、requirement不足、既知の非representable型 | 型parameterを渡すgeneric間applicationとBufferを直接受け取るgeneric function |
| [specialization](../spec/generics.md#specialization) | canonical key共有、same-key recursion、polymorphic recursion rejection、65,536-node boundaryとspan | specialization後のprogramが既存ANF/ownership/backendだけで実行される |

## Memory layoutとaccess

| Authority | Focused evidence | Cross-boundary evidence |
|---|---|---|
| [Representable](../spec/memory.md#representable) | 全base、nested product/sum、Bool、empty sum、function、opaque、indexed type | representable aggregateのstore/load round-trip |
| [canonical layout](../spec/memory.md#canonical-layout) | primitive width/alignment、product padding/tail padding、sum tag/payload、nested shape、Unit stride 0 | target data layoutから作ったplanとLLVM/C adapterの一致 |
| [Buffer access](../spec/memory.md#buffer) | make/new/get/put、empty、growth、Unit、product/sum、alias越しのread-your-writes | managed lifetimeを含むcompiled artifact |
| [preconditions](../spec/memory.md#未検査precondition) | zero-count、zero-stride、rangeとhost lifetime contract | C host copy artifact |
| [target contract](../spec/memory.md#canonical-representationとtarget-contract) | pointer representation幅とindex幅の分離、unrepresentable layout、canonical alignment | reference targetとsynthetic data layout fixtures |

## Buffer、Symbol

| Authority | Focused evidence | Cross-boundary evidence |
|---|---|---|
| [C host copy](../spec/memory.md#c-host-copy-boundary) | offset、length、zero-count、Unit、Address element、operand一回評価 | host storageからBufferへcopyしhost storageへ戻す |
| [Symbol conversion](../spec/memory.md#symbol-conversion) | snapshot independence、変換元activation終了後のresult lifetime | mutation前後のSymbol/Buffer比較 |
| [managed construction](../spec/memory.md#buffer) | empty、growth、Unit、product/sum、read-after-new、alias越しのmutation、generic receiver application | Bufferをhelper・closure・再帰frameへ渡すnative fixture |
| [HostMappable](../spec/extern.md#host-mappable-type) | generic alias完全展開、Symbol/Buffer rejection、nested product/sum | Addressと長さだけを使うgenerated headerとC adapterをcompile/link/execute |

## ABI

| Authority | Focused evidence | Cross-boundary evidence |
|---|---|---|
| [C host ABI](../spec/c-host-abi.md) | ABI `0x000800`、`mal_Address_t`、size_t/index幅assertion、aggregate recursive mapping | generated header、LLVM module、C shim、runtimeを同じClang targetで実行 |
| [Engram/Extern](../spec/engrams.md) | admission、observation、capability transfer、invalid host representation | Addressと長さで借りたexternal bytesのadmissionとobservation |

## Completion gate

v0.6実装は、上表のfocused evidence、代表cross-boundary test、既存機能のregression testがすべて通り、
`nu scripts/check.nu`が成功した時点で完了する。防御的trapの存在をpositive contractとしてassertするtestは作らない。
