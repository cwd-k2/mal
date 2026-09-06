# 関連調査

Status: Informative

2026-09-04 時点。mal の機能を増やすためではなく、小さく保つ際に隠れやすい実装・仕様コストを確認した。

## 単純型付き lambda calculus

[Programming Language Foundations in Agda: Properties](https://plfa.github.io/Properties/) は、well-typed closed term が value であるか step できるという progress と、step 後も型が保たれる preservation を、単純型付き lambda calculus に対して機械検証している。[Products and Sums](https://plfa.github.io/More/) では product/sum の introduction、elimination、case の規則が示される。

mal への含意:

- product/sum 自体は小さく明瞭な型規則を持てる。
- exhaustive case は単なる利便性ではなく、stuck state を避ける中心規則になる。
- `extern` と trap は「必ず value または step」という純粋な progress の外側に、明示した結果として加えるべきである。

## 整数と trap

[WebAssembly Core numeric semantics](https://webassembly.github.io/spec/core/exec/numerics.html) は固定幅 integer を bit pattern と signed interpretation に分けて定義する。[execution conventions](https://webassembly.github.io/spec/core/exec/conventions.html) は trap を通常の value とは別の計算結果として扱う。

mal への含意:

- signed/unsigned の representation と interpretation を分ければ、二の補数 wrap を backend 非依存に定義しやすい。
- division、conversion、shift の edge case は「target の挙動に従う」ではなく個別に列挙する必要がある。

[LLVM の UB/poison 解説](https://llvm.org/docs/UndefinedBehavior.html) では `nsw` 付き演算の overflow が poison を作る。これは、mal の wrap semantics を LLVM/C backend の signed operation へ無条件に写せないことの確認材料になる。

## 浮動小数点

[IEEE 754-2019](https://standards.ieee.org/ieee/754/6210/)はbinary format、基本演算、比較、変換、rounding、exception handlingを規定する。ただし規格名を挙げるだけでは、言語が公開するrounding modeやexception flag、NaN payloadの保証までは決まらない。

[WebAssembly Core numeric semantics](https://webassembly.github.io/spec/core/exec/numerics.html)はIEEE 754演算をround-to-nearest, ties-to-evenへ固定し、directed rounding、exception flag、quiet/signaling NaNの観測を持たない。NaN payloadのpropagationも要求しない。

[C23 working draft §6.3.1.4](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n3096.pdf)では、standard floating typeからintegerへの変換で表現不能な場合はundefined behaviorとなり、standard integerからfloatingへの非exactな変換結果にもimplementation-definedな選択が残る。また`FLT_EVAL_METHOD`により中間評価のrangeとprecisionが型より広くなり得る。

malへの含意:

- Cのcastやexpressionをそのままmalの意味とせず、target capabilityの検査と必要なconversion guardを入れる。
- fast-math、演算の再結合、暗黙のFMA、中間精度による結果差を許さない。
- 動的なfloating environmentを持たず、WebAssemblyに近い固定profileにするとruntime surfaceを増やさずに意味を閉じられる。

## closure

[Efficient and Safe-for-Space Closure Conversion](https://doi.org/10.1145/345099.345125) は closure conversion が runtime representation を決める compiler の重要な段階であることを示す。[Selective Lambda Lifting](https://arxiv.org/abs/1910.11717) は lambda lifting と closure allocation の trade-off を扱う。

[C++ working draftのlambda capture規則](https://eel.is/c%2B%2Bdraft/expr.prim.lambda.capture)はcapture listでcopy/reference/default captureを明示し、[closure type規則](https://eel.is/c%2B%2Bdraft/expr.prim.lambda)はlambdaごとに固有の無名class typeを与える。malはこのうち「保持する名前をsourceに示す」という性質だけを採り、reference/default captureとlambda固有のsource-level typeは採用しない。

mal への含意:

- 「escape しなければ capture 可」は escape analysis と implementation-dependent acceptance を持ち込むため、言語規則にはしない。
- v0.4 はlexical closureを採用するが、captureする名前はsourceのcapture listで明示する。reference compilerの回収戦略はprogram-lifetime arenaに限定する。
- binding が immutable でも environment の配置は必要だが、mutable cell の共有規則は不要になる。

## ABI、Engram、resource

[WebAssembly Component Model Canonical ABI](https://github.com/webassembly/component-model/blob/main/design/mvp/CanonicalABI.md) は scalar 以外の値を component 境界で渡すために、layout、allocation、post-return など多くの規則を必要とする。[Component Model overview](https://component-model.bytecodealliance.org/advanced/canonical-abi.html) も、string や composite type には wire representation と ownership rule が必要だと説明する。

mal への含意:

- `extern print :: Engram -> Unit` という型だけでは相互運用仕様は完成しない。
- pointer を source language から隠しても、buffer の ownership と lifetime は消えない。
- opaque resource を unrestricted value とするなら、resource safety を保証しないことを明記する必要がある。
- v0.4のEngramはextern return時にmal-ownedなprogram-lifetime storageへcopyし、mutable bytesはexternal opaque bufferへ分離する。これによりhostへprogram-lifetime bufferを要求しない。

## 調査からの結論

mal の差別化は「理論上もっとも少ない primitive」ではなく、次の境界を短く、完全に説明できることに置くのがよい。

1. 純粋な typed core
2. trap を含む決定的な scalar semantics
3. immutable lexical closure と単純な program-lifetime environment
4. trusted な extern/ABI contract

とくに 4 を仕様外として無言で残すと、言語表面だけが小さく、実際の system は利用者ごとの暗黙仕様へ分裂する。
