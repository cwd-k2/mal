# 設計決定記録

仕様上の判断と、その理由を記録する。後から変更する場合も古い理由を消さず、status と後継 decision を記載する。

## D001. v0.4 では local capture を禁止する

- Status: Superseded by D003
- Date: 2026-09-04
- Scope: mal v0.4

この decision は履歴として残している。現在の仕様には適用せず、後継の [D003](#d003-v04-は-lexical-closure-を持つ) に従う。

### 決定

ラムダは、外側のラムダの parameter または外側の local binding を参照できない。参照した program は compile-time error になる。

ラムダ自身の parameter、ラムダ body 内の先行 binding、top-level value binding、compiler primitive は参照できる。external operation は `extern symbol(...)` の形で呼び出せる。

capture しない関数は first-class value であり、引数として渡したり戻り値として返したりできる。

### 理由

capture 付き関数は、一般に code と environment の組を必要とする。関数が定義 scope の外へ出る場合は environment の lifetime と storage を定めなければならない。

mal v0.4 は GC、ownership、borrow、allocator を持たない。capture を禁止すると、environment allocation、closure conversion、escape analysis、および backend ごとの closure ABI を仕様と初期 compiler から外せる。

### 却下した選択肢

non-escaping lambda だけに capture を許す案は採用しない。escape 判定が必要になり、compiler によって同じ source の受理可否が変わり得るためである。

capture 付き lambda を完全に実装する案も v0.4 では採用しない。将来導入する場合は、environment representation、lifetime、allocation、function ABI を一つの機能として設計する。

### 影響

partial application、local state を覚える callback、関数を生成する `makeAdder` のような pattern は直接書けない。必要な environment は通常の引数または専用 product として明示的に渡す。

この書き換えは環境型ごとに行う必要がある。v0.4 は polymorphism と existential type を持たないため、任意の environment を持つ closure の汎用 encoding は提供しない。

## D002. reference compiler は Rust で実装する

- Status: Accepted
- Date: 2026-09-04
- Scope: reference compiler

### 決定

最初の reference compiler は Rust で実装する。handwritten lexer、recursive-descent parser、Pratt expression parser、typed intermediate representation、C emitter という構成を基本とする。

初期実装は Rust standard library を中心とし、外部 crate の採用は個別に判断する。parser generator や大規模な compiler framework は前提にしない。

### 理由

Rust は固定幅整数、明示的な data representation、algebraic data type、arena と ID を用いる compiler 内部表現を直接記述できる。単一 native executable にしやすく、C ABI や追加backendとの接続にも適している。

mal の言語としての最小性は、compiler の実装言語まで最小であることを要求しない。reference implementation では変更容易性だけでなく、型検査器と lowering の保守性を優先する。

### 位置づけ

これは mal program の意味論を定める言語仕様ではなく、reference implementation の選択である。他言語で互換 compiler を実装することを妨げない。

## D003. v0.4 は lexical closure を持つ

- Status: Accepted
- Date: 2026-09-04
- Scope: mal v0.4 and reference compiler
- Supersedes: D001
- Refined by: D007

### 決定

ラムダは lexical scope の local value を capture できる。ラムダ式の評価は、code と immutable environment からなる closure を生成する。closure は定義 scope の外へ escape してよい。

言語意味論は environment の配置と回収方式を観察可能にしない。reference compiler は capture を持つ environment を program-lifetime arena に配置し、v0.4 では個別に回収しない。allocation failure は trap とする。

### 理由

lexical capture は単純型付きラムダ計算の通常の意味に沿う。capture を禁止すると runtime は小さくなるが、name resolution に mal 固有の制限が加わり、higher-order function、partial application、関数を生成する処理が不自然になる。

binding が immutable なので、environment は定義時の値を保持すればよく、mutable cell の共有、capture-by-reference、更新順序を定義する必要がない。

program-lifetime arena は GC、reference counting、source-level ownership を必要としない。長時間実行中に closure を生成し続ける program では memory を回収できないが、v0.4 ではこの制約を受け入れる。

### 最適化

compiler は意味を保存する限り、capture 除去、lambda lifting、stack allocation、direct call 化を行ってよい。最適化の成否によって source program の受理可否を変えてはならない。

### extern との関係

external opaque value の binding は immutable でも、handle の指す resource が immutable または有効であるとは限らない。closure に capture された handle の lifetime safety は v0.4 では保証せず、extern implementation と program の責務に置く。

## D004. 直和型を `[A, B, C]` と書く

- Status: Accepted
- Date: 2026-09-04
- Scope: mal v0.4

### 決定

順序付き n 項の直和型は `[A, B, C]` と書く。項は二つ以上必要で、`[]` と `[A]` は不正かつ予約済みの構文とする。

```mal
MaybeInt32 :: [Unit, Int32];

none := MaybeInt32[0](());
some := MaybeInt32[1](42);
```

n-ary sum は primitive であり、nested sum と同一視しない。

```mal
Flat :: [A, B, C];
Nested :: [A, [B, C]];
```

### 理由

型の項順 `[A, B, C]`、injection の `T[0](value)`、case arm の `[0](pattern)` が同じ index notation で対応する。

array を組み込み構文として持たないため、角括弧を直和のために使用できる。また `|` は expression の bitwise OR だけになり、型文脈との使い分けと `->` に対する precedence 規則が不要になる。

### 影響

将来 array literal または array type を組み込み機能として追加する場合、`[]` は利用できない。v0.4 は array を library/extern storage 上に実装する方針なので、この制約を受け入れる。

## D005. `Bool` と `if` を直和と `case` から導出する

- Status: Accepted
- Date: 2026-09-04
- Scope: mal v0.4

### 決定

`Bool` は primitive type ではなく、predefined transparent alias と immutable binding である。

```mal
Bool :: [Unit, Unit];
false :: Bool := Bool[0](());
true :: Bool := Bool[1](());
```

surface conditional は次の構文とする。

```mal
if (condition)
    then {
        whenTrue
    }
    else {
        whenFalse
    }
```

これは condition を一度評価する `case` へ desugar する。`then` は index 1、`else` は index 0 に対応する。condition の括弧、`then`、`else` は必須とする。

`&&`、`||`、`!`、Bool equality も `case` へ desugar する。`&&` と `||` は short-circuit を維持する。数値およびStringの比較primitiveは同じBool表現を返す。

### 理由

Boolは二択の直和として既存の型とtermだけで表現できる。`if`をcoreに残さず、exhaustive `case`へ意味を一本化できる。

一方、すべての二分岐をindex付きcaseで書くと意図が読みにくいため、surfaceには`if`を残す。これはcoreの意味論を増やさない。

### 字句上の扱い

`Bool`、`false`、`true`は専用literal tokenではなく、predefined scopeにある通常のidentifierとして扱う。ただしtop-levelで同名を再定義してはならない。`if`、`then`、`else`、`case`はkeywordである。

## D006. byte literal は `b'…' :: UInt8` とする

- Status: Superseded by D025
- Date: 2026-09-04
- Scope: mal v0.4

### 決定

`Byte`型と`Char`型は追加しない。single byteは既存の`UInt8`で表し、読みやすさのため次のsurface literalを持つ。

```mal
b'a'
b'\n'
b'\xff'
```

byte literalは常に`UInt8`型であり、同じ値の明示型付きinteger literalへdesugarする。decode後にちょうど1 byteでなければcompile-time errorとする。

raw characterはprintable ASCIIからsingle quoteとbackslashを除いたものに限定する。escapeは`\\`、`\'`、`\n`、`\r`、`\t`、`\0`、`\xNN`を認める。

### 理由

malの`String`はUnicode stringではなくimmutable byte sequenceで、byte accessも`UInt8`を返す。`Char`はUnicode scalar、code point、graphemeなどの未提供概念を期待させる。

`Byte :: UInt8`はtransparent aliasとして新しい性質を与えない。一方、protocol parserなどで`0x0au8`の代わりに`b'\n'`と書けるsurface sugarには明確な可読性上の価値がある。

## D007. capture listを明示する

- Status: Accepted
- Date: 2026-09-04
- Scope: mal v0.4
- Refines: D003

### 決定

lambdaはparameter listの前にoptionalなcapture listを持つ。

```mal
makeAdder := \(x :: Int32) {
    \<x>(y :: Int32) {
        x + y;
    };
};
```

capture listを省略したlambdaはcapture-freeである。compilerがbodyのfree variableからcaptureを暗黙に追加してはならない。

captureはすべてby-value。listは1個以上の外側local value identifierを持ち、空list、duplicate、parameterとの同名、scope外の名前、top-level/predefined名の明示captureを禁止する。unlistedの外側local valueをbodyから参照した場合はcompile-time errorとする。

nested lambdaが複数のlambda境界を越えて値を使う場合、各境界のcapture listで明示的に受け渡す。

### 記号

意味論でclosureはしばしばcodeとenvironmentのpair `⟨λx.e, ρ⟩` として表される。ASCIIの`<...>`をenvironmentの列挙に対応させる。

`[]`は直和型とvariant indexに使用済みである。`<`と`>`は式中では比較演算子だが、backslash直後の構文位置ではcapture listとして一意にparseできる。

### 理由

capture listはclosureが保持する値とallocation sizeに影響する依存をsource上に示す。利用者がcompilerのfree-variable推論やimplicit capture規則を調べずに済み、意図しないcaptureを防げる。

malのbindingはimmutableなので、C++のreference capture、default capture、mutable closure、init captureは必要ない。またparametric polymorphismを持たず、すべてのlambdaを同じ構造的function typeとして扱うため、lambdaごとの匿名型をsourceへ導入しない。

capture listはenvironmentの内容を明示するが、配置と回収方法は変更しない。D003のprogram-lifetime arena方針に従う。

## D008. minimalismには利用者のcontrolと調査面積を含める

- Status: Accepted
- Date: 2026-09-04
- Scope: language and ecosystem design

### 決定

malのminimalismはprimitive数、compiler規模、runtime規模だけでは評価しない。利用者がprogramの挙動・cost・外部依存を理解するために調べる必要がある仕様とAPIの面積も含めて評価する。

言語はmechanismを提供し、allocation strategy、I/O、resource policyなどを可能な限り利用者またはhostのcontrol下に置く。外部policyは少数の明示的な`extern`またはruntime contractとして境界を示す。

### 判定基準

- sourceからdependency、evaluation order、保持されるstateを追えるか。
- implicit allocation、conversion、retain/release、effectがあるなら短く完全に列挙できるか。
- standard APIの便利さと引き換えに、名前・選択肢・規約の探索を要求していないか。
- 機能を削った結果、危険な慣習やbackendごとの未記述contractへ責務を移していないか。
- 利用者がpolicyを交換するために、新たな型systemや大規模frameworkを理解する必要がないか。

### surface sugar

surface sugarは一律にminimalismへ反するものではない。既存coreへの局所的でeffect-preservingな変換を説明でき、意図を明瞭にする場合は採用できる。`if`とbyte literalはこの基準で採用する。

### memory management

implicit reference countingはruntime codeが小さくても、retain/releaseの挿入位置とcost modelを隠すため採用しない。source-level manual freeもaliasとlifetimeの負担を未記述のまま利用者へ移すなら最小とはみなさない。

closure environmentとruntime生成Stringは例外としてdocumentedなprogram-lifetime storageを使用する。将来回収が必要になった場合は、implicit RCを既定にする前に、利用者が選択できる明示的arena/regionまたは交換可能な小さなruntime contractを検討する。

## D009. Floatは IEEE 754-2019 の固定profileとする

- Status: Accepted
- Date: 2026-09-04
- Scope: mal v0.4 and reference compiler

### 決定

`Float32`と`Float64`は、それぞれIEEE 754-2019のbinary32とbinary64である。normal、subnormal、正負のzero、正負のinfinity、NaNを持つ。

演算と変換のrounding modeは常にround-to-nearest, ties-to-evenとする。利用者がrounding modeを変更する機能、floating-point exception flagを観測・変更する機能、浮動小数点例外をtrapへ変える機能は持たない。

各primitive演算はoperand型の精度で個別に丸める。compilerは式の再結合、より広い精度での中間値保持、または暗黙のfused multiply-addにより結果を変えてはならない。subnormalをflush-to-zeroしてはならない。

floatのzero除算、overflow、invalid operationはtrapしない。IEEE 754に従ってinfinityまたはNaNを生成する。`+0.0 == -0.0`はtrueであり、大小比較でも等しい。NaNとの`==`はfalse、`!=`はtrue、`< <= > >=`はすべてfalseとする。

quiet NaNとsignaling NaNの違い、およびfloating-point exceptionはmalから観測できない。primitive演算によるNaNのsignとpayload、および変換時のpayload伝播は未指定とする。malはbit reinterpretationを持たず、NaN payloadの同一性を保証しない。

### literalと変換

decimal float literalは数学的な十進値として読み、contextまたはsuffixで決まる型へties-to-evenで正しく丸める。型が決まらなければ`Float64`とする。有限範囲をoverflowするliteralはcompile-time errorとする。v0.4はinfinity、NaN、hexadecimal floatのsource literalを持たない。

`Float32`から`Float64`への変換は正確である。`Float64`から`Float32`へはties-to-evenで丸める。整数からfloatへもties-to-evenで丸める。floatから整数へは小数部をzero方向へ捨て、NaN、infinity、または切り捨て後の値が目的型の範囲外ならtrapする。

整数型同士のconversion規則はこのdecisionに含めず、[D013](#d013-整数型間の変換はdestination-widthでmoduloとする)に定める。

### 理由

IEEE 754という名前だけではrounding mode、exception handling、NaN payload、conversion失敗、演算融合が決まらない。WebAssemblyと同様に固定roundingとnon-stop executionへ限定すると、動的なfloating environmentを言語とruntimeへ追加せず、backend間で比較できる意味を定義できる。

NaN payloadをsource semanticsに含めると、演算ごとのpropagationとbackend差を規定する必要がある。bit reinterpretationを持たないv0.4ではpayloadを抽象化する方が小さい。

## D010. `String`は mal-ownedなprogram-lifetime bytesとする

- Status: Accepted
- Date: 2026-09-04
- Scope: mal v0.4, extern contract, and reference compiler

### 決定

`String`はimmutableな有限byte sequenceである。String値はcopyableなdescriptorとして振る舞い、そのbytesはmal program終了まで有効で変更されない。source-levelの個別解放操作はない。

String literalのbytesは静的storageに置いてよい。実行時に新しくmalへ入るStringのbytesはmal-owned storageへcopyする。reference compilerはこれをprogram-lifetime arenaへ配置し、個別に回収しない。allocation sizeを表現できない場合とallocation failureはtrapする。

`extern`境界では次を固定する。

- malからhostへ渡すStringはcall中だけborrowされる。hostはreturn後にpointerを保持してはならない。
- hostからStringを返すsource-level operationは、callが完了する前にbytesをmal-owned storageへcopyした結果を返す。
- host側bufferの具体的な取得、copy後の解放、calling conventionはbackend adapter contractが定める。hostの後続変更や解放が、返されたmal Stringへ影響してはならない。

compilerは観測可能な意味を変えず、hostに期限切れ参照を残さないと証明できる場合にstorage allocationやcopyを省略してよい。

### mutable bytesとの分離

mutable byte arrayまたはbufferは組み込み型にしない。必要なprogramは`ByteBuffer`などのexternal opaque typeと、用途に応じた明示的な`extern` operationを宣言する。bufferからStringを返すoperationにも上記のcopy規則が適用される。

`ByteBuffer`という名前、operation集合、allocation/free policyはpredefined APIではない。opaque handleは通常のmal値としてcopyableなため、そのresource safetyは従来どおりhost contractとprogramの責務である。

### 理由

Goの`string`と`[]byte`はimmutabilityとmutabilityを分離するが、backing storageのlifetime自体はGCが支える。GCもownership typeもないmalでは、二つのsurface typeを追加するだけではlifetimeは決まらない。

externから返すbufferをhostがprogram終了まで保持する規則は、すべてのhost APIへ長いlifetimeを要求する。境界で必ずcopyすれば、Stringのlifetimeをclosure environmentと同じprogram-lifetime modelへ閉じ、hostが提供する一時bufferのpolicyから切り離せる。長時間programではstorageを回収できない制約をv0.4では受け入れる。

## D011. numeric separatorを認める

- Status: Accepted
- Date: 2026-09-04
- Scope: mal v0.4

### 決定

数値literalの各digit sequenceでは、二つの有効なdigitの間に一つのunderscore `_`をseparatorとして書ける。separatorは値と型に影響せず、lexerが検証後に除去する。

```mal
1_000
0xff_ffu32
0b1010_0001
1_000.25f64
```

underscoreはdigit sequenceの先頭・末尾、連続位置、radix prefix直後、小数点の直前・直後、型suffixの直前には置けない。

```text
_1       invalid
1_       invalid
1__000   invalid
0x_ff    invalid
1_.0     invalid
1._0     invalid
1_f32 invalid
```

### 理由

長い整数、bit mask、protocol constantの桁構造を明示できる。digit間だけという局所規則ならidentifierやwildcardとの曖昧性を増やさず、literal semanticsにも新しい値を追加しない。

## D012. 初期extern implementationはC adapterをlinkする

- Status: Accepted; refined by D016
- Date: 2026-09-04
- Scope: v0.4 reference compiler

### 決定

reference compilerはprogramが要求するextern symbolのC headerを生成する。利用者はそのheaderに対するC implementationまたは既存libraryへのadapterを用意し、生成Cと同じtarget ABIでlinkする。

C source、object、static archive、shared objectをlinker inputにできる。shared objectはplatform linker/loaderでprocess開始時に解決し、mal runtimeは`dlopen`、symbol discovery、plugin lifecycleを提供しない。

extern declarationを任意の既存C function declarationと同一視しない。symbol prefix、runtime context、scalar/String/opaque/aggregate mappingはprogram固有のgenerated headerと[C host ABI profile](../spec/c-host-abi.md)が定める。

### 理由

C backendを使う以上、同じtoolchainでcompileする小さなC adapterは最短のhost boundaryになる。C header parser、dynamic FFI、runtime loaderをcompilerへ組み込まず、既存library固有のownershipやerror policyをadapter内に明示できる。

## D013. 整数型間の変換はdestination widthでmoduloとする

- Status: Accepted
- Date: 2026-09-04
- Scope: mal v0.4

### 決定

整数型から整数型への明示的な変換は、source値が表す数学的整数をdestination型のbit widthでmodulo変換する。
destination widthを`n`、source値を`x`とすると、まず`r = x mod 2^n`を`0 <= r < 2^n`となる剰余として求める。

- destinationがunsignedなら結果は`r`である。
- destinationがsignedなら、`r < 2^(n-1)`では`r`、それ以外では`r - 2^n`である。

この変換はwidening、narrowing、signed/unsignedの全組み合わせに適用し、範囲外でもtrapまたはcompile-time errorにしない。
source式は一度だけ評価する。これはsourceの数学的な値から定義する数値変換であり、host representationのcastや
bit reinterpretationの偶発的な挙動には依存しない。

```mal
UInt8(-1i8)    // 255u8
Int8(255u16)   // -1i8
UInt16(-1i8)   // 65535u16
Int16(255u8)   // 255i16
```

### 理由

整数演算が各widthでwrapする言語において、変換もdestination widthの剰余としてtotalに定義すると、runtime failureを
追加せず全組み合わせを一つの規則で扱える。host Cの範囲外castはimplementation-definedになり得るため、backendは
unsigned arithmetic、明示的なbit copy、または同値な操作でこの結果を構成する。

範囲検査してtrapする案は、値が収まることを要求する別のoperationとしては有用だが、v0.4の唯一のconversion formには
採用しない。constantだけをcompile-time errorにする案は、同じ値がconstantかruntime valueかで意味が変わるため採用しない。

## D014. shift countはleft operandと同じ型とする

- Status: Accepted
- Date: 2026-09-04
- Scope: mal v0.4

### 決定

`<<`と`>>`のright operandはleft operandと同じ整数型でなければならず、結果もその型を持つ。暗黙変換は行わない。
left operandのbit widthを`n`、right operandの数学的な値を`count`とすると、`count < 0`または`count >= n`ならtrapする。
この検査はsigned/unsignedの両方に適用する。

有効な`count`に対する`x << count`は、`x * 2^count`をleft operandのwidthでwrapしたbit patternを持つ。
unsignedの`x >> count`はlogical shift、signedの`x >> count`はsign bitを複製するarithmetic shiftとする。
backendはCの範囲外shift、negative signed valueのleft shift、negative signed valueのimplementation-definedなright shiftへ
この意味を依存させてはならない。

### 理由

両operandを同じ型にすると、他のinteger binary operatorと同じ型検査規則を維持でき、count専用の型や暗黙変換を
追加せずに済む。negative countをunsigned interpretationとして説明するのではなく数学的な範囲検査として定義することで、
signed representationやhost shiftの挙動から独立する。

`UInt64`固定のcountはnegative valueを型で除外できる一方、すべてのshiftだけに特別なliteral contextと変換を要求するため
採用しない。型が正しくても値域は実行時にしか決まらないので、範囲外countは一律にtrapとする。

## D015. opaque valueはcopyable handleとする

- Status: Accepted
- Date: 2026-09-05
- Scope: mal v0.4 and extern contract

### 決定

external opaque typeの値は、通常のmal値と同様にbinding、copy、discardできる。値のcopyやscope終了に伴う暗黙の
retain、release、close、freeは行わない。reference compilerのC ABIでは一machine wordのhandleとして表す。

handleが指すresourceの有効期間、一意性、close/free protocol、zero bit patternの意味は個々のhost contractとmal
programの責務とする。v0.4はopaque resourceに対するlinear type、ownership、borrow、drop hookを持たない。

### 理由

opaque valueだけに暗黙のresource lifetimeを与えるには、copyの意味、closure capture、sum/productへの格納、discard、
host failureを横断するownership規則が必要になる。単純型付きのcopyable valueとして扱えば、言語runtimeへ特定のresource
policyを埋め込まず、必要なprotocolをtyped external operationとして明示できる。

one-wordに収まらないhost stateはhost側でboxする。正しく型付けされたmal programでも、期限切れhandleや二重closeを
防ぐことは保証しない。

## D016. externはmal C ABIとadapterを介する

- Status: Accepted
- Date: 2026-09-05
- Scope: mal v0.4 extern contract and reference compiler
- Refines: D012

### 決定

sourceの`extern` declarationはmal型を持つexternal operationを宣言し、任意のC function declarationを直接記述しない。
reference compilerはprogram固有のheaderに固定したmal C representationとsymbolを生成し、host implementationまたは
adapterを同じheaderに対してcompile/linkする。generated headerをfield order、tag、paddingを含むprogram ABIのauthorityとする。

top-level product parameterは直下の要素をsource orderでC parameterへflattenする。nested productとsumはgenerated aggregate
型、aggregate resultはgenerated型のby-value resultとする。opaque valueはD015のone-word wrapperで表す。これらを既存C
libraryのstructやcalling conventionと暗黙に同一視しない。

v0.4ではparameterまたはresultにfunction型を直接・再帰的に含むextern declarationを拒否する。callback calling convention、
hostによるclosure保持、hostから返るclosureのallocationは定義しない。この制約はmal内部のfirst-class closureへ影響しない。

### 理由

mal型とC型を直接同一視すると、aggregate layout、target calling convention、String lifetime、opaque resource policyがsource
declarationから判別できない。小さなadapter境界とgenerated headerへ集約すれば、C parserやdynamic FFIをcompilerへ追加せず、
target toolchainが実際に使用するABIとhost固有contractを明示できる。

## D017. immutable byte sequenceの型名は`String`とする

- Status: Accepted
- Date: 2026-09-05
- Scope: mal v0.4
- Refines: D010

### 決定

immutableな有限byte sequenceの組み込み型名は`String`とする。この名称はtext encodingやUnicodeの保証を伴わない。
値は任意のbyte列を保持でき、valid UTF-8、Unicode scalar、code point、grapheme、normalizationのinvariantを持たない。

source上のraw characterはsource encodingであるUTF-8のbytesとしてliteralへ入り、`\xNN` escapeは任意の1 byteを表す。
byte列としてのoperationとlifetimeは[String仕様](../spec/strings.md)に従う。

### 理由

既存のsyntax、型一覧、extern例との連続性を保ち、immutableな値であることをmutable buffer型と区別するため`String`を維持する。
UTF-8を保証しない点は型名だけでは伝わらないため、型とliteralのauthorityで明記する。

`Bytes`への改名はencoding上の誤解を減らせる一方、値の意味や安全性を変えず、既存文書とprogramを一斉に変更する移行コストが
生じるため採用しない。mutable byte storageは引き続きexternal opaque typeで表し、`String`へmutable semanticsを追加しない。

## D018. top-level initializationは作用のないclosed valueに限定する

- Status: Accepted
- Date: 2026-09-05
- Scope: mal v0.4

### 決定

top-level value bindingはsource orderでscopeに入る。RHSはliteral、product、sum injection、integer conversion、および
lambdaからなる作用のないclosed expressionに限定し、`extern` callと他のtop-level valueへの参照は認めない。
predefined constantの`false`と`true`はclosed valueとして参照できる。

通常のsequential bindingに対する唯一の例外として、単一のvalue name pattern、型annotation、直接のlambda RHSを持つ
bindingは、そのlambda body内から自分自身を参照できる。product pattern、annotationのないbinding、lambdaを別の式で
包んだRHSには例外を適用しない。forward referenceとmutual recursionは認めない。local bindingにも同じ自己参照例外を適用する。

### 理由

top-levelのeffectとfile間初期化順を導入せず、現在のstatic valueとclosureの生成だけでprogram initializationを閉じられる。
自己再帰の対象を構文的に限定することで、一般的なrecursive value、初期化中のcycle、暗黙のfixed-point semanticsを追加せずに
反復に必要な関数再帰を提供できる。

## D019. decimal float syntaxとC target profileを固定する

- Status: Accepted
- Date: 2026-09-05
- Scope: mal v0.4 and reference compiler
- Refines: D009

### decimal literal

decimal float literalは次のいずれかの形とする。

```text
DEC_DIGITS "." DEC_DIGITS EXPONENT? FLOAT_SUFFIX?
DEC_DIGITS EXPONENT FLOAT_SUFFIX?
DEC_DIGITS FLOAT_SUFFIX
```

`EXPONENT`は`e`または`E`、optionalな`+`または`-`、1桁以上のdecimal digit sequenceからなる。
`FLOAT_SUFFIX`は`f32`または`f64`である。数値separatorは整数部、小数部、exponentの各digit sequence内でのみ認める。

`.5`、`1.`、hexadecimal floatは認めない。decimal point、exponent、float suffixのいずれもないliteralは
integer literalである。exponentを含むliteralはsuffixがなくてもfloatである。

### C target profile

reference compilerのC backendは、`float`がbinary32、`double`がbinary64であり、両方がsubnormalを保持し、
`FLT_EVAL_METHOD == 0`のtargetでのみFloat機能を提供する。generated Cは`<float.h>`の定数と`sizeof`を
compile-timeに検査し、条件を満たさないtargetを拒否する。

generated Cはtoolchainが対応する場合に`#pragma STDC FENV_ACCESS ON`と`#pragma STDC FP_CONTRACT OFF`を指定する。
`malc build`はfast-math、reassociation、FP contraction、型より広い中間精度を無効にするtoolchain optionを付ける。
`emit-c`の利用者が別途compileする場合も、pragmaまたは対応するtoolchain optionで同じprofileを保つ必要がある。

malから呼ぶC adapterはround-to-nearest, ties-to-evenの浮動小数点environmentを保持し、flush-to-zeroや
denormals-are-zeroを有効にしてreturnしてはならない。これはmal値のABIと同様にadapter contractの一部とする。

### 理由

exponentがないとbinary64の最小subnormalや最大値を現実的な長さで書けない。整数部と小数部を必須にする
decimal point形はlexerの境界を明確に保ち、integer literalとの分類を後段の型contextに依存させない。

Cの`float`と`double`のwidthだけでは、subnormal、excess precision、contraction、rounding modeは保証されない。
backend、build driver、adapterが所有する条件を分けて明示し、保証できないtargetで別の意味になることを避ける。

## D020. numeric literalの型suffixは短縮名とする

- Status: Accepted
- Date: 2026-09-05
- Scope: mal v0.4

### 決定

integer literalの型suffixは`i8`、`i16`、`i32`、`i64`、`u8`、`u16`、`u32`、`u64`とする。
float literalの型suffixは`f32`、`f64`とする。型名をそのまま付ける`Int32`、`UInt8`、`Float64`形式は
suffixとして認めない。suffixのないinteger literalのdefaultは`Int64`、float literalのdefaultは`Float64`のままとする。

### 理由

literalでは値と型指定の境界が明瞭であり、固定幅を小文字の短い表記へ揃えることで頻出する定数を簡潔に書ける。
未releaseのv0.4内の変更なので、旧形式の互換syntaxや専用のmigration diagnosticは設けない。

## D021. v0.4のlexical detailとtrap mappingを固定する

- Status: Accepted
- Date: 2026-09-05
- Scope: mal v0.4 and reference compiler

### 決定

sourceはUTF-8とし、identifierとkeywordはASCIIで認識する。空白はASCII space、tab、CR、LF、commentは
`//`から行末までのline commentだけを認める。block comment、Unicode identifier、trailing commaは認めない。

lambda、`if` branch、`case` armのblockは最後にresult expressionを必須とする。その直後の`;`はoptionalであり、改行はsyntaxに影響しない。return statementは設けず、`return`は通常のvalue identifierとして扱う。

言語上のtrapは捕捉不能な異常終了とする。C backendのruntimeは理由をstderrへ出力して`abort()`し、hostからの
回復不能なcontract violationにもgenerated headerの`mal_trap`を使う。portableなprocess exit codeは規定しない。

### 理由

最小構文をrelease profileとして固定し、block間でresult規則を統一して字句や終了方法が実装の偶然に見える状態を解消する。
trapを通常のreturnや固定exit codeへ写像せず、埋め込み先が異常終了として確実に観測できるcontractを保つ。

## D022. 型なし`Ptr`をmemory primitiveのbaselineとする

- Status: Accepted; refined by D024
- Date: 2026-09-05
- Scope: mal v0.5 and reference compiler

### 決定

要素型を持たないcopyableなdata address型`Ptr`を追加する。operation集合はbyte単位の`offset`と、全fixed-width
integerおよび`Float32`/`Float64`の型別load/storeとする。operationはpredefinedかつdirect-call-onlyである。
型ごとの名前を使うことでoverloadやexpected type依存の型規則を追加せず、numeric scalar間の任意の例外も作らない。

`Ptr`はextern-safeとし、mal program内のpointerは`extern` resultまたは`offset`から得る。allocation、
deallocation、length、bounds、ownershipは組み込まず、programとhost contractが所有する。null、equality、
integer conversion、`Ptr<T>`、pointerおよびaggregateのload/storeは追加しない。

scalar accessはalignmentを要求せず、reference C backendは`memcpy`相当でlowerする。pointerが指すlive region、
permission、lifetimeに違反したaccessはhost contract違反であり、deterministicなtrapを保証しない。offsetがtargetの
address計算で表現できない場合はtrapする。

### 根拠

localのalgorithm corpusをv0.4で実装した結果、collection-orientedなworkloadでは用途別opaque operationが増え、
storageを使うalgorithm上の処理までC adapterへ移った。これはsurfaceを小さく
保つ代わりにtrusted host APIと利用者の調査面積を増やしていた。

representativeなindexed workloadを`offset`、`Int64`/`UInt8` accessだけで再実装すると、hostの責務をI/Oとallocationへ
限定し、data構築、transition、集計をmalへ戻せた。behavior caseとmaximum-order caseを完走したため、型なしscalar accessで
当初の境界問題を解消できることを確認した。

組み込み`Array<T>`はlength、index、alias、allocation、bounds、resize、viewのpolicyを同時に持ち込み、
`Ptr<T>`はaddressable type、aggregate layout、cast規則を追加する。scalar operationの合成で必要なprogramを
記述できる間は、この追加costを負わない。

## D023. `case`は括弧付きscrutineeとarm blockを持つ

- Status: Accepted
- Date: 2026-09-05
- Scope: mal v0.5 and reference compiler

### 決定

`case`は括弧で囲んだscrutineeの後に、一つ以上のindex付きarmを並べる。
各armのpattern直後にはexpression blockを置き、`=>`とarm固有の`;`は使用しない。

```mal
case (value)
    [0](_) {
        fallback
    }
    [1](item) {
        normalized := normalize(item);
        normalized
    }
```

armのpattern bindingとblock内のbindingはarmごとの同じscopeに属する。blockの最後の式がarmの値になる。

### 理由

括弧がscrutineeの境界を、各blockがarmの処理と値の境界をそれぞれ表す。これは`if (condition)`の後に
`then`と`else`のexpression blockを並べる構造と対応し、case全体を囲むbraceやpatternと結果の間の
追加separatorを不要にする。arm内でも`if` branchと同じbinding、statement、末尾式を使用できる。

## D024. `Ptr`をmemoryへload/storeできる

- Status: Accepted
- Date: 2026-09-05
- Scope: mal v0.5 and reference compiler
- Refines: D022

### 決定

predefinedなdirect-call-only primitiveとして`loadPtr :: Ptr -> Ptr`と
`storePtr :: (Ptr, Ptr) -> Unit`を追加する。pointerは整数へ変換せず、target ABIのdata address object
representationとしてalignmentを要求せずcopyする。格納されたpointerの複製はstorageのlifetimeを延長しない。

product、sum、String、external opaque type、functionのaggregate accessは引き続き追加しない。pointerを含む
node layoutのsize、allocation、deallocation、bounds、lifetimeはprogram固有のextern contractが所有する。

### 理由

D022のnumeric scalar accessだけでもarena内のindexやrelative offsetを使うdata structureは記述できるが、
pointer graphではedgeの保存と復元がhost operationへ流出する。`Ptr` accessを同じ明示的なmemory mechanismへ
加えると、hostの責務をstorage提供とlifetimeへ限定したまま、list、tree、graphのlink操作をmalへ戻せる。
pointerを`UInt64`として扱わないため、pointer幅、integer conversion、null、equalityをsource semanticsへ追加しない。

## D025. byte literalはsingle quoteだけで書く

- Status: Accepted
- Date: 2026-09-05
- Scope: mal v0.5 and reference compiler
- Supersedes: D006

### 決定

byte literalは`'a'`、`'\n'`、`'\xff'`のようにsingle quoteだけで書き、常に`UInt8`型を持つ。
raw character、escape、exactly one byteの規則はD006から変更しない。
したがって非ASCII source characterを含む`'あ'`はcompile-time errorである。

### 理由

malは`Char`型を持たず、single-quoted literalをbyte以外の意味に使わないため、`b` prefixは構文上も
型選択上も曖昧性を解消していなかった。literalの唯一の型をsyntaxに重ねて書かず、短いspellingへ一本化する。
Unicode characterを将来追加する場合は、byte literalの意味を変更せず別のsyntaxとして設計する。

## D026. String descriptorをmemoryへload/storeできる

- Status: Accepted
- Date: 2026-09-05
- Scope: mal v0.5 and reference compiler
- Refines: D010, D022, D024

### 決定

predefinedなdirect-call-only primitiveとして`loadString :: Ptr -> String`と
`storeString :: (Ptr, String) -> Unit`を追加する。operationはString descriptorだけをalignmentを要求せずcopyし、
参照先のbytesはcopyしない。String bytesのprogram-lifetimeとimmutabilityは変更しない。

memory上のdescriptorは、`storePtr`と同じpointer表現、その直後の`storeUInt64`と同じlength表現の順で
paddingなしに配置する。必要byte数はtarget ABIのpointer格納byte数と8の和であり、Cの`MalString` structに
含まれ得るpaddingには依存しない。

### 理由

String fieldを持つmemory上のnodeをdescriptor primitiveなしで構築すると、program-lifetimeですでに安定している
bytesを別領域へcopyし、pointerとlengthへ分解して管理する処理が必要になる。しかしStringのdata pointerは
immutabilityを保つためsource-levelに公開しておらず、復元にもhost operationが必要になる。

Stringは一般のproductと異なり、組み込みのdescriptor表現とprogram-lifetime invariantを持つ。そのdescriptorを
scalarや`Ptr`と同じmemory mechanismでround-trip可能にすれば、bytesのownershipを変えずにString fieldの操作を
malへ戻せる。storage layoutをpointerとfixed-width lengthの連結として定めることで、target依存のstruct paddingを
programのoffset計算へ持ち込まない。

## D027. `@T`でmemory storage幅を表す

- Status: Accepted
- Date: 2026-09-05
- Scope: mal v0.5 and reference compiler
- Refines: D022, D024, D026

### 決定

`@T :: UInt64`を、型`T`のcanonical memory storage表現が占めるbyte数を返すtarget constantとして追加する。
`@Ptr`はtarget ABIのpointer格納幅、`@String`は`@Ptr + 8`である。numeric scalarにも対応し、transparent
aliasは展開する。`Unit`、product、sum、external opaque type、functionはv0.5では拒否する。

`@`は値に対する通常のunary operatorでも、型をfirst-class valueへ変えるsyntaxでもない。後続のtypeを
storage幅へ写す専用の構文であり、host `extern`を必要としない。

### 理由

`offset`がbyte単位である以上、target依存のpointer幅を含むlayoutをsourceだけで記述するには、型からbyte幅を
得る手段が必要になる。bare type nameを数値として扱うとtype/value namespaceの境界が不明瞭になり、通常の
functionとしての`sizeof(T)`は型を値引数に見せる。専用sigilはlayout queryであることを短く明示する。

`<T>`も同じ短さを持つが、`<`と`>`は比較演算子およびlambda capture listですでに使用している。`@T`なら
それらのtoken列との境界を増やさず、field offsetの式でも`@String + @UInt8`と読める。

productとsumはbackend ABI上のC struct sizeを公開せず、canonical memory表現と対応するload/store戦略を
別途決定してから対象へ加える。これにより現在のbackend layoutを将来のsource contractとして固定しない。

## D028. Stringのlengthとbyte accessを`#` operatorで表す

- Status: Accepted
- Date: 2026-09-05
- Scope: mal v0.5 and reference compiler

### 決定

Stringのbyte lengthを`#value :: UInt64`、0-based byte accessを
`value # index :: UInt8`として表す。binary `#`のindexは`UInt64`で、範囲外はtrapし、operatorは
non-associativeとする。

### 理由

これらは通常のfunction valueではなく、すべてのString valueに常在する基本的な観測である。numeric scalarの
算術や比較と同じく型付きoperatorとして表し、値が元から持つprimitive operationとambient nameの境界を
明確にする。

unary `#`によるString byte lengthにはLuaなどの前例がある。binary `#`を同じoperator familyのbyte accessへ
割り当てることで、将来の汎用container indexingを暗示する`[]`を導入せず、StringがUnicode characterではなく
immutable byte sequenceである現在の意味を保つ。
