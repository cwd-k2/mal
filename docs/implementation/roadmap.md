# v0.4 implementation roadmap

Status: Current

この文書はreference compilerのactive milestone、実装順、各milestoneの完了条件を定める。言語とhost interfaceの規則は[`spec/`](../spec/)、stageの責務は[responsibilities](responsibilities.md)、検証方法は[test policy](../development/testing.md)をauthorityとする。

## 現在位置

| Gate | Status | Outcome |
|---|---|---|
| M0 | Complete | `Unit`、`Int32`、sum、Bool、closure、externをnative executableまで通す |
| S0 | Complete | M0をchecked-in exampleと再利用可能なfixture経路で安定させる |
| M1 | Complete | 固定幅integer familyとbyte literal |
| M2 | Complete | product、destructuring、opaque handle、aggregate ABI |
| M3 | Active | immutable byte `String`とextern copy contract |
| M4 | Planned | annotated self recursionとdirect tail-call lowering |
| M5 | Planned | strict `Float32`/`Float64` profile |
| R0 | Planned | v0.4 conformanceとrelease readiness |

一度にactiveにするgateは一つだけとする。active gateの完了条件を満たしてstatusを更新してから次へ進む。

## vertical sliceの進め方

各milestoneは次の順序で進める。途中のstageまで実装したsyntaxを利用可能扱いにせず、public CLIからnative実行できる単位で閉じる。

1. 対象範囲に関係する未決事項を解決し、必要ならdecisionと`spec/`を更新する。
2. lexer/parser、resolution/type checking、typed core/ANF、closure conversionのうち影響するstageを実装する。
3. C representation、runtime、generated header、driverを同じ変更範囲で完成させる。
4. focusedなpositive/negative/edge testを各所有stageへ置く。
5. checked-in exampleまたはnative fixtureを`malc build`でcompile、link、executeする。
6. `check`、`emit-c`、`build`の利用者向け失敗を、所有する境界からspanまたはprocess context付きで返す。
7. [test policy](../development/testing.md)の完了commandを通し、active documentationからstaleな記述を除く。

milestoneのcommitは、仕様決定、独立したcompiler stage、native/public boundaryのように単独で検証できる単位へ分ける。commit数自体は完了条件にしない。

## S0: M0 operational baseline

M0の機能追加は行わず、実際に使う入口を次のmilestone群の共通baselineにする。

### Scope

- `examples/m0/`にmal source、generated headerをincludeするhost C、Nushell向け実行手順を置く。
- integration testのtemporary directory、Clang実行、process結果検査を共通fixture supportへ集約する。
- checked-in exampleを`malc check`、`emit-c`、`build`のpublic commandで検査するsmoke testを置く。
- closure environment allocation failureをdeterministicに発生させ、`mal_trap`へ到達するtest seamをtest build内に限定して用意する。
- source、output、linker input、C compiler failureの代表的なdriver diagnosticを固定する。

### Done

- clean checkoutから`nix develop`へ入り、exampleの記載commandだけでhost-linked executableを実行できる。
- exampleとtestが別々のmal programやhost contractを複製していない。
- capture-free/capturing closure、左から右のextern作用、wrap、division trap、allocation failureがnative boundaryで観測される。
- generated C/header/executableをrepositoryへ追跡しない。

## M1: fixed-width integers and byte literal

### Specification gate

- integer conversionは[D013](../design/decisions.md#d013-整数型間の変換はdestination-widthでmoduloとする)、shift countは
  [D014](../design/decisions.md#d014-shift-countはleft-operandと同じ型とする)に従う。
- byte literalは[D006](../design/decisions.md#d006-byte-literal-は-b--uint8-とする)、numeric separatorは
  [D011](../design/decisions.md#d011-numeric-separatorを認める)と[grammar](../spec/grammar.md#numeric-separator)に従う。

### Scope

- `Int8`、`Int16`、`Int32`、`Int64`、`UInt8`、`UInt16`、`UInt32`、`UInt64`。
- suffix付き/なしinteger literal、`Int64` default、全境界値、`b'…' :: UInt8`。
- arithmetic、comparison、bitwise、shift、確定したinteger conversion。
- widthごとのwrapとdivision/remainder/shift trap。
- C scalar mappingと全integer scalarのextern ABI。

### Done

- literal、operator、conversionごとに型検査と境界値のpositive/negative testがある。
- signed演算はどのwidthでもC undefined behaviorやimplementation-defined bit conversionへ依存しない。
- byte escape全種とmalformed byte literalをlexer boundaryで検査する。
- `examples/m1/`のinteger/byte programがpublic `malc build`経路でhost結果とtrapを再現する。

## M2: products, opaque handles, and aggregate ABI

### Specification gate

- opaque resource safetyは[D015](../design/decisions.md#d015-opaque-valueはcopyable-handleとする)、extern ABIは
  [D016](../design/decisions.md#d016-externはmal-c-abiとadapterを介する)に従う。

### Scope

- product type/expression/pattern、nested destructuring、wildcard、pattern内duplicate検査。
- 複数parameter/argumentのproduct loweringと左から右の一回評価。
- external opaque typeと一machine-word handle representation。
- C host ABIが定めるtop-level product flattening、nested product/sum、aggregate result。
- functionを再帰的に含むextern signatureのrejectionをaggregate実装後も維持する。

### Done

- productを各stageで直接検査し、pattern bindingのscopeとspanを保持する。
- duplicate member typeを持つsumとnested aggregateがtag/field orderを失わない。
- generated headerだけを見てhost adapterを実装でき、Clangでcompile/link/executeできる。
- `examples/m2/`でopaque handleを含むaggregateをhostと往復する。

## M3: immutable byte String

### Specification gate

- immutable byte sequenceの型名は[D017](../design/decisions.md#d017-immutable-byte-sequenceの型名はstringとする)に従う。
- host buffer取得・copy完了・解放のadapter contractに未確定部分があれば[extern contract](../spec/extern.md)と[C host ABI](../spec/c-host-abi.md)を先に更新する。

### Scope

- String literalと全escape、UTF-8 source characterからbyte列への変換。
- `byteLength`、`byteAt`、byte-wise `==`/`!=`、bounds trap。
- static literal storageとruntime String用program-lifetime storage。
- `MalString` parameter borrow、host result copy、allocation/length failure。

### Done

- embedded NUL、非ASCII、`\xNN`、空String、invalid escapeを境界testで固定する。
- host scratch bufferをreturn後に変更・解放してもmal Stringが変化しない。
- String captureがscope外へescapeしてもbytesとdescriptorがprogram終了まで有効である。
- `examples/m3/`がhostから受け取ったbytesを検査し、結果をhostへ返す。

## M4: self recursion and tail calls

### Specification gate

- [Q5 top-level initialization](../design/open-questions.md#q5-top-level-initialization)のsource-orderと自己参照例外を確定する。

### Scope

- 型annotationを持ち、RHSが直接lambdaであるbindingだけのself reference。
- forward referenceとmutual recursionのrejection。
- recursive closure/functionのloweringとC declaration order。
- direct tail recursionのloop lowering。non-tail recursionの意味は通常のcallとして保持する。

### Done

- local/top-levelの許可されたself recursionと、不正なunannotated/non-lambda/mutual recursionを検査する。
- direct tail recursive fixtureがC call stackの深さに比例せず実行できる。
- recursive call周辺のextern argumentとeffect順序がsource orderどおりである。
- `examples/m4/`に大きな入力を処理するtail-recursive programを置く。

## M5: strict floating point

### Specification gate

- [D009](../design/decisions.md#d009-floatは-ieee-754-2019-の固定profileとする)とtarget C toolchainの対応を照合し、保証できないtargetを拒否する条件を決める。
- integer/float conversionとdecimal literalの規則が`spec/`内で閉じていることを確認する。

### Scope

- `Float32`/`Float64` literalのdecimalからbinaryへのties-to-even変換。
- arithmetic、comparison、signed zero、infinity、NaN、subnormal。
- integer/floatと`Float32`/`Float64`間conversion、必要なruntime guard。
- C emitterのstrict-FP設定とscalar extern ABI。

### Done

- halfway、最大有限値、subnormal/zero境界、overflow literalをbit patternで検査する。
- 各primitiveがoperand precisionで個別にroundされ、reassociationやimplicit FMAを許さない。
- float-to-integerのNaN/infinity/範囲外がcast前にtrapする。
- `examples/m5/`とnative conformance testがtarget capabilityを明示して実行される。

## R0: v0.4 release gate

### Scope

- v0.4の全機能をspec節からpositive/negative/edge/native testへ対応付けるconformance matrix。
- `check`、`emit-c`、`build`、generated header、trap、toolchain failureの利用者向けcontract確認。
- supported target/toolchain、`CC`、shared library loader、generated artifact policyの文書化。
- 未決事項のうちv0.4の実装挙動に影響するものをdecisionまたは明示的な未指定事項へ分類する。
- release build、version、example一式をclean checkoutで再検証する。

### Done

- [`scope`](../spec/scope.md)に含まれる機能が未実装syntaxとして残っていない。
- `spec/`、generated ABI、compiler behavior、exampleに既知の矛盾がない。
- 全milestoneのpublic exampleとconformance suiteがpinned environmentで成功する。
- v0.4で意図的に提供しない機能がactive planning textではなくscopeとして記述されている。
