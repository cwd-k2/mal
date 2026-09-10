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
  -> application control lowering
  -> execution plan
  -> LLVM module + C shim/runtime
  -> pinned Clang
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
拒否する。aliasはcycleを検出して展開し、構造的に比較する。lambda parameterは期待関数型から決め、
lambda以外のbindingはRHSから型を推論できる。
overload resolutionはoperatorとoperand typeの組で閉じる。

name resolverは各value bindingにtop-levelまたは所属lambdaのidentityを記録する。bodyから別lambda所属のlocal bindingへの
参照を見つけると、現在のlambdaまでの各境界にcapture bindingを作り、内側closureの構築に必要な値を転送する。
capture順は最初のlexical参照順とする。top-level/predefined bindingはenvironment fieldにしない。

integer/float literal は最初から `Int64`/`Float64` に固定せず、期待型を受け取れる literal node として検査する。期待型がなければ default を適用する。

decimal float literalはhost parserやC compilerのdecimal conversionへ意味を委ねず、数学的な十進値から目的のbinary32/binary64 bit patternへties-to-evenで正しく丸める。LLVM backendはそのbit patternを16進定数として保持する。finite rangeをoverflowするliteralは診断する。

lexer は byte literal を token 化するときに escape を decodeし、exactly one byteであることを検査する。AST以降では値と`UInt8`型を持つinteger literalとして扱ってよい。

numeric separatorは各radixの有効なdigitに挟まれたsingle underscoreだけを受理する。検証後にunderscoreを除去してからinteger valueの計算またはdecimal floatのcorrect roundingを行う。

extern declarationの型検査ではaliasを展開し、parameter/resultの全subtypeを再帰的に走査する。function型が現れた場合は
extern transport制約により拒否する。

checked programからcore境界で、type alias、external type、external operationからなる`ProgramInterface`を抽出する。
`emit-header`と`emit-host`はvalue bindingをlowerせず、このinterfaceから生成する。C translation unitを作る経路では
同じinterfaceをcore programへ載せ、ANFとclosure conversionで意味も表現も変えず共有する。各loweringは実行表現だけを
変換し、host interfaceを複製または再解釈しない。

## desugaring と ANF

複数parameter/argumentはproduct parameter/application、0 parameter/argumentは`Unit`へlowerする。blockの末尾式はbody resultへ、sequential bindingはnested letまたはlambda applicationへ落とせる。

surface `if`、`!`、`&&`、`||`、Bool equality は、operand を一度だけ左から右へ評価する `case` と temporary binding へ
desugarする。直ちにbranchとして消費する数値・Symbol comparisonはtyped core以降で専用のprimitive branchとして保持し、
backendでBool valueをmaterializeしない。値として必要なcomparison resultと構造的な`[Unit, Unit]`はLLVM backendで
0/1の`i1`へ写像する。

Symbol operatorの`#value`と`value # index`は型検査後にそれぞれSymbol lengthとbyte accessの
専用core operationへlowerする。`Symbol + Symbol`はleft、rightの順に一度ずつ評価するbinary primitiveとして保持し、
LLVM backendからruntimeのmanaged storageを使ってbytesを連結する。いずれもpredefined value lookupや通常のfunction callは経由しない。

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

## LLVM backendとC境界

program固有の実行はLLVM IRへlowerする。scalarは仕様どおりのLLVM整数幅または`float`/`double`へ写像し、整数の
`+ - *`はwrap semanticsを保つ。浮動小数点演算にはfast-math flagを付けず、変換はsource-level preconditionと
ties-to-evenを満たすLLVM instructionを選ぶ。

productはLLVM struct、sumはtagと最大payloadを収めるstruct、Boolは`i1`で表現する。Symbol literalはLLVM moduleのstatic
leafを参照し、動的なSymbolはC11 runtimeのreference-counted flat/rope storageを使う。連結、比較、byte access、外部memoryとの
copyは汎用runtime operationへ委ね、連続byte viewはC host境界で要求された場合だけmaterializeする。targetで表現不能なallocation sizeと
allocation failureはmal trapへ写像する。

extern symbol、public header、C build input、runtime contextのcontractは[C host ABI](../spec/c-host-abi.md)に従う。
argument-aware entryではC shimが`argv[1]`以降を外部descriptor列へ置き、LLVM rootを`(UInt64, Ptr)`で呼ぶ。

type-qualified `size`は型検査でtransparent aliasを展開し、memory表現を持つ型だけをtyped IRへ残す。fixed-width scalarの
sizeは定数とし、`Ptr.size`はtarget data layoutから求める。`Symbol.size`は型検査で拒否する。

function valueはcode pointerとenvironment pointerの組へlowerする。captureを持つlambdaごとにimmutable environmentを生成し、
capture-free lambdaも同じmal function typeの共通calling conventionから呼べる表現を保つ。

call siteのcalleeがtop-level lambda、現在のself closure、またはidentityを追跡できるlocal closureならdirect entryへ進み、
runtime選択が必要なcalleeだけ共通closure entryからindirect callする。

managed valueはprogram固有の型を知るLLVM側がretain、transfer、releaseする。productとsumにはfield単位で再帰適用し、
closure environmentの最後のreleaseではcaptureを逆順に破棄する。tail edgeはLLVM basic block間の遷移にし、non-tailな
recursive regionはprogram固有のtyped frameをC runtimeのgrowable byte storageへ積む。frame payload、resume target、owner moveは
LLVM側だけが解釈する。詳細は[LLVM backendのownership](ownership.md)を正とする。

`Ptr`はLLVMの`ptr`へlowerし、`+`と`-`はbyte offsetとして扱う。数値scalarとpointerのload/storeは`align 1`のmemory operationを
使い、unaligned accessを許す。直接参照とfirst-class memory functionは同じoperationへ到達する。region、permission、lifetimeは
typed IRへ補わず、source-levelの[`memory` contract](../spec/memory.md)として保持する。

C representationの収集はhost interfaceだけを対象とする。`TypeRegistry`はextern signatureから到達できるstructural typeの
identityを所有し、`HostTypes`は公開型とexternal opaque type名を分類する。C shimとheaderは同じregistryを参照する。LLVM moduleと
C shimの間はopaque pointerとout-pointerを基本とするinternal ABIを使い、LLVM aggregate表現をpublic C ABIへ公開しない。
現在の仕様とtestの対応は[conformance matrix](../development/conformance.md)を正とし、この文書にはtest一覧を重複させない。
