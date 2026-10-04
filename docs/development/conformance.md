# v0.7 conformance matrix

Status: Current v0.7 verification matrix

この文書は[`spec/`](../spec/)の規範を実装完了に必要なobservable evidenceへ対応させる。test layerとcommandは
[test policy](testing.md)を正とする。各行はfocusedなpositive、negative、edge caseと、必要なcross-boundary pathを要求する。
test function名やmodule配置は実装が所有し、この文書では固定しない。

## Frontendとsurface

| Authority | Focused evidence | Cross-boundary evidence |
|---|---|---|
| [grammar](../spec/grammar.md) | 全precedence levelの隣接、prefix/postfix/binary共有token、generic `<>`とcomparison/`>>`、formatter idempotence | parseからchecked programまでの代表的なgeneric memory source |
| [numeric conversion](../spec/operators.md#primitive-operator) | 全closed suffix、rounding、modulo、float-to-integer precondition | conversionを含むLLVM artifactのcompile/execute |
| [type constructor](../spec/type-constructors.md) | kind polymorphism、partial/oversaturated application、kind mismatchとoccurs check、alias expansion、phantom generic alias、recursive alias、正規化とkind parameterのresource上限、type position以外のTYPE_IDENT rejection | 利用者定義aliasのsource spellingとphantom argumentを保つeditor hover/navigation、generated diagnostic |
| [program](../spec/programs.md) | generic top-level initializer、source order、entry signature、zero argument buffer | `Buffer<Symbol>` process entryを実際のargv（空argumentと非ASCII byteを含む）で実行 |

## Generics

| Authority | Focused evidence | Cross-boundary evidence |
|---|---|---|
| [declaration/application](../spec/generics.md#declarationとapplication) | duplicate parameter、kind/arity mismatch、constructor prefixの明示とrigid-only inference、phantom parameterの非constructor推論、operand・期待result・higher-order位置・lambda result・直和continuation payloadからの通常型推論、未確定と衝突、first-classな単相value、generic extern rejection | constructor termを含む推論形と明示形のspecialization key共有、required fileを跨ぐgeneric application |
| [requirements](../spec/generics.md#requirements) | signature内のnested `Buffer<A>`と`Buffer<F<A>>`、alias展開、requirement不足、既知の非storable型 | constructor parameterを渡すgeneric間applicationとBufferを直接受け取るgeneric function |
| [specialization](../spec/generics.md#specialization) | canonical key共有、same-key recursion、polymorphic recursion rejection、65,536-node boundaryとspan | specialization後のprogramが既存ANF/ownership/backendだけで実行される |
| [operation family](../spec/operation-families.md) | familyとexact/generic implementationの分類、closed constructor key、constructor pattern rejection、pattern overlap、全parameter束縛、減少、signature不一致、requirement伝播、missing implementation、opaque key | directly required familyへのimplementation、constructor keyまたはgeneric patternから選択したimplementationが既存backendだけで実行される |
| [file-local opaque type](../spec/types.md#file-local-opaque-type) | declaration identity、同一fileの構築と分解、別fileのrepresentation拒否、recursive representation | specializationでrepresentationへ消去したproduct/sumを既存backendとextern bridgeで実行する |

## Memory representationとaccess

| Authority | Focused evidence | Cross-boundary evidence |
|---|---|---|
| [Storable](../spec/memory.md#storable) | 全base、`Symbol`、nested product/sum、nested Buffer、empty sum、function、opaqueと、それらを含むaggregate | `Buffer<Symbol>`とnested Bufferの`new`・`get`・`put`・`fill`・`copy`を、成長、上書き、重なるcopy、handle alias、深い再帰の負荷を含めてAddressSanitizerで実行し、ソート、hash table、queue、木、closure、sum、early returnの各programをbaselineとproductionの両方で同じ検査にかける |
| [runtime element representation](../spec/memory.md#runtime-element-representation) | primitive width/alignment、product padding/tail padding、sum tag/payload、nested shape、Unit stride、managed element lifecycle | target data layoutから作ったLLVM layout、Buffer stride、generated C carrierの一致 |
| [Buffer access](../spec/memory.md#buffer) | make/new/get/put/fill/copy、empty、growth、Unit、product/sum、aliasとoverlap越しのread-your-writes、read-after-new、generic receiver application | managed lifetimeを含むcompiled artifact、Bufferをhelper・closure・再帰frameへ渡すnative fixture |
| [preconditions](../spec/memory.md#未検査precondition) | zero-count、zero-stride、range、growth後のpointer失効 | C hostが構成したBufferをmalと相互に変更するartifact |

## Buffer、Symbol

| Authority | Focused evidence | Cross-boundary evidence |
|---|---|---|
| [Symbol conversion](../spec/memory.md#symbol-conversion) | snapshot independence、変換元activation終了後のresult lifetime | mutation前後のSymbol/Buffer比較 |
| [Symbol operator](../spec/symbols.md#operator) | length、byte access、concatenation、`/`と`%`の端点・分割則・左結合range、型の拒否 | range viewを連結・比較するcompiled artifactとmanaged lifetime |
| [extern carrier](../spec/extern.md#admitted-type) | generic extern、functionを含む型、open typeの拒否、Symbol、Buffer、opaque representation、nested product/sumの受理 | managed carrierをCで構成、変更、share/dropし、malと往復するartifact |

## ABI

| Authority | Focused evidence | Cross-boundary evidence |
|---|---|---|
| [C host ABI](../spec/c-host-abi.md) | ABI `0x000a00`、共通`mal.h`、file header、target assertion、aggregate mapping、Symbol/Buffer helper、managed result move | generated header、LLVM module、C shim、runtimeを同じClang targetでcompile/link/execute |
| [Engram/Extern](../spec/engrams.md) | borrowed parameter、shared responsibility、result move、external opaque resource、invalid host representation | hostがmal-owned Bufferを生成し、malの変更をborrowで観測する |

## Specification cases

[`spec/`](../spec/)の言語規則のうち短いprogramで観測できるものは、`crates/mal-compiler/tests/spec/`のcaseで検査する。
各caseは、受理、特定の診断による拒否、実行結果のexit code、trapのいずれかを期待する。runtime carrier ABIはgenerated headerを使う
C adapterが実際に値を構成、観測して照合する。新しい言語規則にはcaseを一つ以上加える。

## Completion gate

v0.7実装は、上表のfocused evidence、代表cross-boundary test、既存機能のregression testがすべて通り、
`nu scripts/dev.nu check`が成功した時点で完了する。防御的trapの存在をpositive contractとしてassertするtestは作らない。
