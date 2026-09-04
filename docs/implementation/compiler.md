# Compiler implementation notes

Status: Non-normative draft

## pipeline

```text
source
  -> lexer
  -> recursive-descent / Pratt parser
  -> surface AST
  -> name resolution
  -> type checking
  -> desugaring to typed core
  -> ANF
  -> C emitter
  -> host C compiler
```

reference compiler は Rust で実装する。compiler 自身を mal で書く必要はなく、mal の minimalism を実装言語へそのまま要求しない。

最初のvertical sliceの作業順と受入条件は[M0 implementation plan](m0.md)に置く。

初期実装は Rust standard library を中心に構成する。外部 crate は、標準 library だけで実装する場合より明確に単純になるものを必要に応じて追加し、特定の parser framework や compiler framework を前提にしない。

lexer は handwritten、parser は recursive descent と Pratt parsing を組み合わせる。C compiler の起動、temporary file、diagnostic、target 設定は core compiler logic から分離する。

## environments

型検査の主な環境は次で足りる。

```text
aliases : TypeIdentifier -> Type
values  : ValueIdentifier -> Type
externs : ValueIdentifier -> FunctionType
```

初期環境には `Bool :: [Unit, Unit]`、`false :: Bool`、`true :: Bool` を入れ、top-level duplicate declaration を拒否する。alias は cycle を検出して展開し、構造的に比較する。parameter は明示型、binding は RHS から推論できる。overload resolution は operator と operand type の組で閉じる。

name resolverは各value bindingにtop-levelまたは所属lambdaのidentityを記録する。capture listの各名前が外側のlocal valueへ解決されることを確認し、environment fieldをlist順に作る。bodyから別lambda所属のlocal bindingへの参照を見つけた場合、その名前がcapture listになければerrorとする。

compilerはfree-variable setからcaptureを補完しない。nested lambdaのcapture listで使われる名前も現在のlambda内の参照として検査するため、lambda境界ごとの明示的な受け渡しが必要になる。top-level/predefined bindingはenvironment fieldにしない。

integer/float literal は最初から `Int64`/`Float64` に固定せず、期待型を受け取れる literal node として検査する。期待型がなければ default を適用する。

decimal float literalはhost parserやC compilerのdecimal conversionへ意味を委ねず、数学的な十進値から目的のbinary32/binary64 bit patternへties-to-evenで正しく丸める。C emitterはそのbit patternを失わず再現できる表現を出力する。finite rangeをoverflowするliteralは診断する。

lexer は byte literal を token 化するときに escape を decodeし、exactly one byteであることを検査する。AST以降では値と`UInt8`型を持つinteger literalとして扱ってよい。

numeric separatorは各radixの有効なdigitに挟まれたsingle underscoreだけを受理する。検証後にunderscoreを除去してからinteger valueの計算またはdecimal floatのcorrect roundingを行う。

extern declarationの型検査ではaliasを展開し、parameter/resultの全subtypeを再帰的に走査する。function型が現れた場合はv0.4の暫定extern-safe制約により拒否する。

## desugaring と ANF

複数 parameter/argument は product parameter/application、0 parameter/argument は `Unit` へ lower する。terminal `return` は body result へ、sequential binding は nested let または lambda application へ落とせる。

surface `if`、`!`、`&&`、`||`、Bool equality は、operand を一度だけ左から右へ評価する `case` と temporary binding へ desugar する。数値・String comparison の backend result は、C の truth valueをそのまま mal value とみなさず、tag 0/1 の `Bool` representation へ変換する。

```mal
f(g(x), h(y))
```

は評価順を保って概ね次になる。

```text
a := g(x)
b := h(y)
c := f(a, b)
c
```

## C backend

scalar は `<stdint.h>` の固定幅型へ写像する。signed `+ - *` は、対応する unsigned 型で演算して bit pattern を signed 型へ戻すなど、C の signed overflow に依存しない実装にする。

extern symbol、generated header、linker input、runtime contextの初期contractは[C host ABI](../spec/c-host-abi.md)に従う。

product は compiler-generated struct、sum は tag と payload union、String は概念上 pointer と length に lower できる。

```c
typedef struct {
    const uint8_t *data;
    uint64_t length;
} MalString;
```

これは source language に pointer があることを意味しない。descriptorの複製はbytesを複製しない。aggregate ABI と lifetime は [`extern` contract](../spec/extern.md) に従う。

String literalのdataは生成物のstatic storageへ置ける。hostからStringを受け取るadapterは、source-level extern callを完了する前にlengthを検査し、bytesをmal-ownedなprogram-lifetime arenaへcopyする。host bufferをMalStringへ直接保存してはならない。allocation size overflowとfailureはmal trapへ写像する。

function value は概念上 code pointer と environment pointer の組へ lower する。capture を持つラムダごとに immutable environment struct と、environment pointer を追加引数として受け取る C function を生成する。capture-free lambda は environment を持たない表現へ最適化してよいが、同じ mal function type の値として呼べる共通の calling convention を保つ。

reference runtime は closure environment とruntime String bytes 用の program-lifetime storage を提供する。両者に個別の retain/release は生成しない。allocation failure は mal trap へ写像する。同じarenaを共有するかは実装上の選択である。

Float32/64を提供するtargetでは、binary32/binary64、subnormal、ties-to-evenの各要件をcompile-timeまたはtoolchain設定で確認する。C compilerのfast-math、式の再結合、implicit FMA contraction、型より広い中間精度によってmalの結果を変えてはならない。

floatからintegerへのC castは、NaN、infinity、範囲外を先に検査してmal trapへ分岐した後だけ実行する。integerからfloat、およびFloat64からFloat32への変換も、C implementation任せでties-to-evenを保証できないtargetではhelperまたは別のloweringを用いる。

## QBE backend

仕様安定後は typed core/ANF から QBE IL を生成できる。QBE は scalar と aggregate を区別し、aggregate argument は ABI 上 pointer 経由になる場合があるため、source-level product と単純に同一視せず lowering layer を置く。

## milestone

1. M0: Unit、n-ary sum、injection、case、predefined Bool、Int32、literal、lexical closure、application、return、surface if、extern、main
2. M1: 全整数幅、byte literal、bit operation、trap tests
3. M2: product、product pattern
4. M3: String literal、byte primitives、extern lifetime tests
5. M4: self recursion、tail-call lowering
6. M5: Float32/64、literalのcorrect rounding、conversion guard、strict-FP tests

各 milestone は parser test、type error test、interpreter または core evaluator test、C backend execution test を同じ機能について揃える。

## 初期 conformance cases

- left-to-right evaluation を extern log で観測する
- signed add/multiply の wrap
- zero division、signed min / -1、overshift、`byteAt` bounds の trap
- sum index の範囲外、case の欠落・重複・arm type mismatch
- duplicate type を持つ sum の tag preservation
- byte literal の全escape、空・複数byte・非ASCII・不完全な`\xNN`のrejection
- numeric separatorのdecimal/hex/binary/floatでの受理と、先頭・末尾・連続・prefix/decimal point/suffix隣接のrejection
- `false`/`true` に対する `if` の branch 選択と condition の一回評価
- `&&`/`||` の short-circuit と Bool equality の eager left-to-right 評価
- capture 時点の値、nested capture、escaping closure、higher-order application
- unlisted/duplicate/out-of-scope/top-level captureのrejectionとlambda境界ごとのexplicit forwarding
- capture-free closure と capturing closure の共通 calling convention
- functionを直接またはproduct/sum内に含むextern declarationのrejection
- closure arena の allocation failure trap
- floatのsigned zero、infinity、NaN comparison、subnormal、各演算の型精度へのrounding
- decimal float literalの境界・ties-to-even・overflow rejection
- Float32/64間およびintegerとの変換、float-to-integerのNaN/infinity/範囲外trap
- extern String inputのcall中borrow、output copy、host scratch buffer変更後の独立性、allocation failure trap
- top-level forward reference と effectful initializer の rejection
