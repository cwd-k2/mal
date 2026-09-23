# 式と binding

Status: Accepted v0.6 profile

## binding

`:=` は immutable binding を作る。型 annotation は `::` で書く。

```mal
x := 10;
y :: Int32 := x + 20;
```

再代入はない。内側の scope では同じ名前を shadow できる。binding の scope は、その宣言の直後から現在の block の末尾までである。

直積は pattern で分解できる。

```mal
(name, length) := value;
(_, y) := point;
```

pattern は identifier、`_`、product pattern からなる。pattern 内で同じ identifier を二度 binding してはならない。

## ラムダ

```mal
add :: (Int32, Int32) -> Int32 :=
    (a, b) -> a + b;
```

lambdaは周辺から与えられる期待関数型に対して検査し、自身から関数型を推論しない。期待関数型がなければ
compile-time errorである。bindingのRHSに直接lambdaを書く場合は、bindingの型annotationが期待型になる。

期待関数型のparameter型が`Unit`ならlambdaは空pattern、そうでなければ括弧内のpatternをparameter型に対して検査する。
`(a, b)`はproduct parameterを分解し、`(value)`はparameter型全体を一つの名前へbindingする。`_`とnested product
patternもbindingと同じ意味を持つ。bodyは期待関数型のresult型に対して検査する。resultがさらに関数型でbodyの
result expressionがlambdaなら、この規則を再帰的に適用する。parameterごとの型annotationとlambda自身の戻り型構文はない。

ラムダbodyから参照する外側のlocal bindingはby-valueでlexically captureされる。

```mal
makeAdder :: Int32 -> (Int32 -> Int32) :=
    (x) -> (y) -> x + y;
```

top-level binding、predefined binding、compiler primitiveはenvironmentへcaptureせず直接参照する。nested lambdaだけが
さらに外側のlocal valueを参照する場合も、compilerが各lambda境界を通して値を転送する。

captureの時点とlifetimeは[実行意味論のclosure規則](execution.md#scope-と-closure)に従う。

lambdaの`->`の右辺は一つのexpressionである。複数のbindingやexpression statementが必要ならblock expressionを置く。
blockは0個以上のbody itemと最後のresult expressionからなり、最後の`;`はoptionalである。改行は構文に影響せず、
result expressionのないblockとreturn statementはない。

lambda bodyは通常の値で完了するほか、すべてのpathが`Abrupt`でもよい。完全な規則は
[result boundaryとcompletion](control.md#completion-judgment)に定める。lambdaのresult edgeへ名前を付ける専用構文はなく、
early resultや直和の構築にはbody全体をdirect result blockにできる。

```mal
log :: Symbol -> Unit := (message) -> print(message);
```

## direct block

blockは単独のexpressionとして置ける。body itemをsource orderで評価し、最後のexpressionがblockの値になる。block内の
bindingはblockの外へ出ず、外側のlocal bindingとresult binderは通常どおり参照できる。

```mal
value :: Int32 := {
    base :: Int32 := 40;
    base + 2
};
```

`{ body }`は`() -> { body }`の省略ではない。前者はその場で評価するblockであり、後者は`Unit` parameterを持つfunction
valueを作る。したがってbare blockは期待関数型から暗黙にlambdaへ変換されない。top-level initializerには置けない。

result continuationを導入するblockは[direct result block](control.md#direct-result-block)に定める。

## application

applicationはcontinuationを先に書く`f(a)`とvalueを先に書く`a[f]`のどちらでも表せる。
calleeがvalue nameであるapplicationは、receiver-first application（UFCS）として第一引数を先に書く
`a.f(...)`でも表せる。

```mal
f()
f(x)
f(x, y)
makeFunction()(x)
x[f]
x[f][g]
x.f()
x.f(y)
x.f(y).g()
```

`f(a)`と`a[f]`、`f()`と`()[f]`はそれぞれ同じapplicationである。`[f]`は`()[f]`のUnit valueを省略した形、
`f(a, b)`は`f((a, b))`である。`g(f(a))`、`g(a[f])`、`f(a)[g]`、`a[f][g]`も同じapplicationの列を表す。
表記に依存しない評価順は[評価戦略](execution.md#評価戦略)に定める。

receiver-first applicationの`a.f()`は`f(a)`、`a.f(b, c)`は`f(a, b, c)`と同じapplicationである。
generic calleeでも`a.f<T>(b)`は`f<T>(a, b)`と同じapplicationである。
`f`はreceiverの型から探索せず、source位置で通常のvalue nameとして解決してから既存のfunction application型規則を
適用する。receiver、残りの引数、calleeの順に評価する。

`.`、value name、parenthesized argument listは全体で一つのapplication suffixである。`a.f`はexpressionではなく、
field access、property、method value、bound functionを導入しない。

[AddressとBuffer](memory.md)のoperationは通常のexpressionとして評価する。`Symbol`と`Buffer`のlengthは`#`で表す。

external declarationが導入する名前も通常のfirst-class function valueである。参照や受け渡しではhost operationを
実行せず、applicationしたときだけ[`extern`境界](extern.md)を越える。

## Memory expression

`Address`の先はhost contractの領域であり、malは直接dereferenceやoffset計算を行わない。C host profileとの固定長copyでは、
indexed typeがcanonical representationを決める。

```mal
readUInt64 :: Address -> UInt64 :=
    (address) -> from<UInt64>(address, 0usize, 1usize).get(0usize);
```

型、評価、preconditionは[AddressとBuffer](memory.md)に定める。

## if

`if`は`Bool`に対するcontinuation applicationのsurface syntaxであり、core termではない。conditionの括弧、
`then`、`else`はすべて必須であり、各branchには一つのexpressionを置く。複数のbody itemやbranch-local bindingが
必要ならblock expressionを使う。標準の表記ではconditionの後、`then`、`else`をそれぞれ別の行に置く。

```mal
absolute :: Int32 -> Int32 := (x) ->
    if (x < 0)
    then -x
    else x;
```

condition は `Bool`、すなわち構造的に `[Unit, Unit]` と等しい型でなければならない。両 branch の結果型は同一でなければならない。branchに置いたblock内のbindingはそのblockにだけscopeを持つ。

上の形は次へdesugarする。conditionは一度だけ、選択したbranchより先に評価する。

```mal
(x < 0)[
    () -> x,
    () -> -x
]
```

`then` は `Bool` の index 1、`else` は index 0 に対応する。`else if` 専用構文はなく、必要なら `else` のexpressionに別の `if` を置く。

## 直和の除去

```mal
getOrZero :: MaybeInt32 -> Int32 :=
    (value) -> value[
        () -> 0,
        (x) -> x
    ];
```

二つ以上のcontinuationを持つ`value[f, g, ...]`は直和を除去する。
continuation数は直和の項数と一致し、位置`i`のcontinuationは第`i`項をparameterとするfunctionでなければならない。
すべてのresult型は同一である。scrutineeとcontinuationの評価は[評価戦略](execution.md#評価戦略)に定める。

作用と評価順を捨象した値の対応だけを見れば、`f : A -> X`と`g : B -> X`に対するcontinuation列は
copairing `[f, g] : [A, B] -> X`とみなせ、`value[f, g]`はそのcopairingの`value`へのapplicationと読める。
ここでの`[f, g]`は型の対応を説明するmetanotationであり、source expression、function value、またはfunctionのproductを
構築するものではない。sourceのcontinuation expressionを先に一つの値へまとめる書換えは、
[評価戦略](execution.md#評価戦略)が定める非選択continuationの遅延を保存しないため、一般には認めない。

一つのcontinuationを持つ`value[f]`はvalueの型にかかわらず通常のapplicationである。直和を一つのfunctionへ渡す場合も
この規則を使い、直和除去との違いはcontinuation数から一意に決まる。

continuationを持たない`value[]`は空直和のeliminationであり、[completion規則](control.md#empty)に従う。

## literal

```mal
123
-123
0xff
0b101010
123i32
255u8
64bytes
8usize
1.5f32
2.0f64
1_000
0xff_ffu32
1_000.25f64
```

負号は literal token の一部ではなく unary `-` として扱う。固定幅整数のsuffixは`i8`、`i16`、`i32`、`i64`、
`u8`、`u16`、`u32`、`u64`、target quantityのsuffixは`bytes`、`usize`とする。suffixのない整数は周辺型から決め、
決まらなければ`Int64`。
浮動小数は周辺型から決め、決まらなければ `Float64` とする。

decimal float literalは数学的な十進値から目的型へround-to-nearest, ties-to-evenで正しく丸める。有限範囲をoverflowするliteralはcompile-time errorとする。underflowは通常の演算と同じくsubnormalまたは符号付きzeroへ丸め得る。source literalとしてinfinity、NaN、hexadecimal floatを持たない。

decimal pointを使う形は整数部と小数部の両方を必須とする。`e`または`E`による10進exponentと
optionalな符号を認める。decimal point、exponent、`f32`/`f64` suffixのいずれかがあるliteralを
float literalとする。完全な形は[grammar](grammar.md#numeric-literal)に定める。

numeric separatorの`_`は各digit sequenceのdigit間だけに置け、値と型に影響しない。完全な規則は[字句仕様](grammar.md#numeric-literal)に定める。

Symbol literalのbyte列としての意味とstorageは[Symbol](symbols.md#literal)、受理するsource spellingは
[grammar](grammar.md#expression-form)に定める。

### byte literal

single quoteを使うbyte literalは常に`UInt8`型を持つ。prefixは付けない。

```mal
'a'
'\n'
'\x00'
'\xff'
'\''
'\\'
```

byte literal は decode 後にちょうど 1 byte でなければならない。raw character は 1 byte の printable ASCII に限定し、`'` と `\` は escape する。

認める escape は `\\`、`\'`、`\n`、`\r`、`\t`、`\0`、`\xNN` である。`NN` はちょうど2桁の hexadecimal digit とする。

```mal
''
'ab'
'あ'
```

はいずれも compile-time error である。byte literal は core では同じ値を持つ明示型付き UInt8 literal へ desugar する。

```text
'A'    => 65u8
'\xff' => 255u8
```

## primitive operator

同じ数値型同士に次を定義する。

```text
integer:  + - * / %  == != < <= > >=
float:    + - * /    == != < <= > >=
integer:  ~ & | ^ << >>
Bool:     ! && || == !=
target quantity: ByteSize、USizeに定めたclosed family
Symbol:   Symbol + Symbol, == !=
```

数値比較は predefined `Bool` を返す。異なる数値型を暗黙変換しない。

`Symbol + Symbol`はbyte sequenceを連結して`Symbol`を返す。完全な規則は
[Symbol](symbols.md#operator)に定める。

Bool operatorはcore primitiveではなくcontinuation applicationへdesugarする。特に`&&`と`||`は左operandを一度だけ
先に評価し、必要な場合だけ右operandを評価する。

```mal
a && b
```

```mal
a[
    () -> false,
    () -> b
]
```

```mal
a || b
```

```mal
a[
    () -> b,
    () -> true
]
```

`!`、Boolの`==`と`!=`も同様に、一つまたは二つのexhaustiveなcontinuation applicationへdesugarできる。
Bool equalityの両operandは通常のoperatorと同じく、分岐より前に左から右へ必ず評価する。これらの演算子は評価を
省略または重複させてはならない。

numeric conversionはclosed postfix suffixを使う。

```mal
y := x.i32;
z := y.f64;
offset := y.bytes;
```

`.i8`、`.i16`、`.i32`、`.i64`、`.u8`、`.u16`、`.u32`、`.u64`、`.f32`、`.f64`、`.bytes`、
`.usize`だけを認める。conversionはbit reinterpretationではない。型identifierをcalleeまたはpostfix argumentにする形式はない。

- `Float32`から`Float64`への変換は正確である。
- `Float64`から`Float32`へはround-to-nearest, ties-to-evenで丸める。
- integerからfloatへもround-to-nearest, ties-to-evenで丸める。
- floatからintegerへは、finiteであり、小数部をzero方向へ捨てた値が目的型の範囲内であることをpreconditionとする。
  小数部はzero方向へ捨てる。precondition違反時の実行結果は保証しない。

integer型からbit widthが`n`のinteger型への変換では、source値を数学的な整数`x`として`r = x mod 2^n`を
`0 <= r < 2^n`となるように求める。destinationがunsignedなら結果は`r`、signedなら`r < 2^(n-1)`のとき`r`、
それ以外では`r - 2^n`とする。widening、narrowing、signed/unsignedの全組み合わせでこの規則を使い、変換自体はtrapしない。
詳細な理由は[D013](../history/decisions/D013.md)に記録する。

product、一般の sum、function、opaque type に `==` は自動導出されない。`Bool` と構造的に同じ `[Unit, Unit]` は、上記の Bool equality の対象になる。

## expression statement

lambda body では式の結果を捨てられる。

```mal
log("done");
```

これは概念上 `_ := log("done");` と同じである。
