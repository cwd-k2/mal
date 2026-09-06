# Compiler implementation notes

Status: Current non-normative overview

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
  -> closure conversion
  -> C emitter
  -> host C compiler
```

reference compiler は Rust で実装する。compiler 自身を mal で書く必要はなく、mal の minimalism を実装言語へそのまま要求しない。

stageごとのownershipは[compilerの責務境界](responsibilities.md)に置く。実装の変更履歴はGitを正とし、
この文書には現在のpipelineとlowering方針だけを記載する。

実装はRust standard libraryを中心に構成する。外部crateは、標準libraryだけで実装する場合より明確に単純になるものを
必要に応じて追加し、特定のparser frameworkやcompiler frameworkを前提にしない。

lexer は handwritten、parser は recursive descent と Pratt parsing を組み合わせる。C compiler の起動、temporary file、diagnostic、target 設定は core compiler logic から分離する。

## environments

型検査の主な環境は次で足りる。

```text
aliases : TypeIdentifier -> Type
values  : ValueIdentifier -> Type
externs : ValueIdentifier -> FunctionType
```

predefined環境には`Bool :: [Unit, Unit]`、`false :: Bool`、`true :: Bool`を入れ、top-level duplicate declarationを
拒否する。aliasはcycleを検出して展開し、構造的に比較する。parameterは明示型、bindingはRHSから推論できる。
overload resolutionはoperatorとoperand typeの組で閉じる。

name resolverは各value bindingにtop-levelまたは所属lambdaのidentityを記録する。capture listの各名前が外側のlocal valueへ解決されることを確認し、environment fieldをlist順に作る。bodyから別lambda所属のlocal bindingへの参照を見つけた場合、その名前がcapture listになければerrorとする。

compilerはfree-variable setからcaptureを補完しない。nested lambdaのcapture listで使われる名前も現在のlambda内の参照として検査するため、lambda境界ごとの明示的な受け渡しが必要になる。top-level/predefined bindingはenvironment fieldにしない。

integer/float literal は最初から `Int64`/`Float64` に固定せず、期待型を受け取れる literal node として検査する。期待型がなければ default を適用する。

decimal float literalはhost parserやC compilerのdecimal conversionへ意味を委ねず、数学的な十進値から目的のbinary32/binary64 bit patternへties-to-evenで正しく丸める。C emitterはそのbit patternを失わず再現できる表現を出力する。finite rangeをoverflowするliteralは診断する。

lexer は byte literal を token 化するときに escape を decodeし、exactly one byteであることを検査する。AST以降では値と`UInt8`型を持つinteger literalとして扱ってよい。

numeric separatorは各radixの有効なdigitに挟まれたsingle underscoreだけを受理する。検証後にunderscoreを除去してからinteger valueの計算またはdecimal floatのcorrect roundingを行う。

extern declarationの型検査ではaliasを展開し、parameter/resultの全subtypeを再帰的に走査する。function型が現れた場合はextern-safe制約により拒否する。

typed coreで確定したtype alias、external type、external operationからなる`ProgramInterface`は、ANFとclosure
conversionで意味も表現も変えず共有する。各loweringは実行表現だけを変換し、host interfaceを複製または再解釈しない。

## desugaring と ANF

複数parameter/argumentはproduct parameter/application、0 parameter/argumentは`Unit`へlowerする。blockの末尾式はbody resultへ、sequential bindingはnested letまたはlambda applicationへ落とせる。

surface `if`、`!`、`&&`、`||`、Bool equality は、operand を一度だけ左から右へ評価する `case` と temporary binding へ
desugarする。直ちにbranchとして消費する数値・Symbol comparisonはtyped core以降で専用のprimitive branchとして保持し、
C backendでBool valueをmaterializeしない。値として必要なcomparison resultと構造的な`[Unit, Unit]`はC backendで0/1の
`uint8_t`へ写像する。

Symbol operatorの`#value`と`value # index`は型検査後にそれぞれSymbol lengthとbounds-checked byte accessの
専用core operationへlowerする。`Symbol + Symbol`はleft、rightの順に一度ずつ評価するbinary primitiveとして保持し、
C backendでprogram-lifetime storageを確保してbytesを連結する。いずれもpredefined value lookupや通常のfunction callは経由しない。

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

extern symbol、generated header、linker input、runtime contextのcontractは[C host ABI](../spec/c-host-abi.md)に従う。

argument-aware entry pointではCの`argv[1]`以降のaddressとlengthを外部descriptor列へ置き、`(UInt64, Ptr)`として
source-level `main`を呼ぶ。Symbolへのcopyはsourceが`loadSymbol`を呼ぶ時点で行い、entry専用のcollection型は持たない。

product は compiler-generated struct、sum は tag と payload union、Symbol は概念上 pointer と length に lower できる。

```c
typedef struct {
    const uint8_t *data;
    uint64_t length;
} MalType_Symbol;
```

これは source language に pointer があることを意味しない。descriptorの複製はbytesを複製しない。aggregate ABI と lifetime は [`extern` contract](../spec/extern.md) に従う。

Symbol literalのdataは生成物のstatic storageへ置ける。host側byte bufferからSymbol resultを作るadapterは、source-level
extern callを完了する前にlengthを検査し、bytesをmal-owned arenaへcopyする。Symbol concatenationと`loadSymbol`の
resultも同じarenaへ置く。現在のreference runtimeはarenaをprogram終了時に一括解放するが、これは回収時期を
source semanticsへ固定しない実装上の選択である。lengthまたはallocation sizeのoverflowとallocation failureはmal trapへ写像する。

`loadSymbol`は外部regionから指定lengthのbytesをarenaへcopyし、`storeSymbol`はSymbol bytesを外部regionへcopyする。
`MalType_Symbol` descriptor自体をsource-level memoryへload/storeしない。

storage-size expressionは型検査でtransparent aliasを展開し、memory表現を持つ型だけをtyped IRへ残す。
C backendはfixed-width scalarを定数へ、`@Ptr`を`sizeof(MalType_Ptr)`へlowerする。これはgenerated Cのtargetで
評価する。`Symbol`にはsource-level memory表現がないため`@Symbol`を型検査で拒否する。

function value は概念上 code pointer と environment pointer の組へ lower する。capture を持つラムダごとに immutable environment struct と、environment pointer を追加引数として受け取る C function を生成する。capture-free lambda は environment を持たない表現へ最適化してよいが、同じ mal function type の値として呼べる共通の calling convention を保つ。

call siteのcalleeがimmutableなtop-level lambdaまたは現在のself closureと静的に分かる場合、C backendは
closureのfunction pointerを経由せず生成functionを直接callする。関数値として受け取ったcalleeとlocal closureは
共通calling conventionを使う。この区別はsourceから観測できず、既知関数の細粒度call costを減らす。

product値をproduct patternで分解するだけのbindingは、C backendでproduct全体の一時copyを作らず、元の値のfieldから
直接bindingを生成する。product parameterを持つ既知関数にはleaf fieldを個別に受けるdirect entryを生成し、共通closure
entryはaggregateを受けるthunkとして残す。direct entryのleaf数は16個までとし、それを超える場合はaggregate entryへ
fallbackする。product resultとfirst-class function callはtarget C ABIへ委ねる。

`Ptr`はC backendで`uint8_t *`をfieldに持つ`MalType_Ptr`へlowerする。pointerに対する`+`と`-`はbyte addressを移動し、targetの
`size_t`でoffsetを表現できない場合はtrapする。scalar load/storeはalignmentに依存しない`memcpy`相当の
runtime helperへlowerする。直接callはhelper operationへ直接lowerし、function valueとして参照された場合は同じ
operationを実行するcapture-free closure entryを生成する。region、permission、lifetimeはtyped IRに補わず、source-levelの
[`memory` contract](../spec/memory.md)として保持する。

reference runtime は closure environment とruntime Symbol bytes 用の program-lifetime storage を提供する。両者に個別の retain/release は生成しない。allocation failure は mal trap へ写像する。同じarenaを共有するかは実装上の選択である。

Float32/64を提供するtargetでは、binary32/binary64、subnormal、ties-to-evenの各要件をcompile-timeまたはtoolchain設定で確認する。C compilerのfast-math、式の再結合、implicit FMA contraction、型より広い中間精度によってmalの結果を変えてはならない。

floatからintegerへのC castは、NaN、infinity、範囲外を先に検査してmal trapへ分岐した後だけ実行する。
integerからfloat、およびFloat64からFloat32への変換も、C implementation任せでties-to-evenを保証できないtargetではhelperまたは
別のloweringを用いる。現在の仕様とtestの対応は[conformance matrix](../development/conformance.md)を正とし、この文書には
test一覧を重複させない。
