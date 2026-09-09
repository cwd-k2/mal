# 式と binding

Status: Current v0.5 profile

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
    \(a, b) { a + b };
```

lambdaは周辺から与えられる期待関数型に対して検査し、自身から関数型を推論しない。期待関数型がなければ
compile-time errorである。bindingのRHSに直接lambdaを書く場合は、bindingの型annotationが期待型になる。

期待関数型のparameter型が`Unit`ならlambdaは0 parameter、productでlambdaが複数parameterを持つなら要素数は
一致しなければならない。1 parameterのlambdaは期待parameter型全体を受け取るため、productも一つの名前へbindingできる。
各parameter名には対応する型を与え、bodyは期待関数型のresult型に対して検査する。resultがさらに関数型でbodyの
result expressionがlambdaなら、この規則を再帰的に適用する。parameterごとの型annotationとlambda自身の戻り型構文はない。

ラムダbodyから参照する外側のlocal bindingはby-valueでlexically captureされる。

```mal
makeAdder :: Int32 -> (Int32 -> Int32) := \(x) {
    \(y) { x + y };
};
```

top-level binding、predefined binding、compiler primitiveはenvironmentへcaptureせず直接参照する。nested lambdaだけが
さらに外側のlocal valueを参照する場合も、compilerが各lambda境界を通して値を転送する。

captureの時点とlifetimeは[実行意味論のclosure規則](execution.md#scope-と-closure)に従う。

lambda、`if` branch、`case` armのblockは、0個以上のbindingまたはexpression statementと、最後のresult expressionからなる。最後の`;`はoptionalであり、改行は構文に影響しない。result expressionのないblockとreturn statementはない。

```mal
log :: Symbol -> Unit := \(message) { extern print(message) };
```

## 関数適用

適用には必ず `()` を使う。

```mal
f()
f(x)
f(x, y)
makeFunction()(x)
```

`f()` は意味上 `f(())`、`f(a, b)` は `f((a, b))` へ lower できる。callee を先に評価し、続いて引数を左から右へ評価する。

[memory](memory.md#primitive)に列挙する`load`、`store`、`read`、`write`はpredefined functionであり、通常のfunctionと同じく
直接callするほか、値としてbindingしたり引数として渡したりできる。pointerのbyte offsetは`+`と`-`、Symbolの
lengthとbyte accessは[`#` operator](symbols.md#operator)で表す。

## 型で修飾したmemory primitive

canonical memory representationを持つ型`T`には`T.size`、`T.load`、`T.store`を定める。`T.size`は
storage幅をbyte数で表す`UInt64`のtarget constantであり、通常のfunction callではなくhost operationも
実行しない。`T.load`と`T.store`はfirst-class functionである。`Symbol.read`と`Symbol.write`は
canonical object representationではなくraw bytesをcopyするfirst-class functionである。

```mal
descriptorSize :: Unit -> UInt64 := \() {
    Ptr.size + UInt64.size + UInt8.size;
};

readUInt64 :: Ptr -> UInt64 := UInt64.load;
```

`.`の形は一般のmodule、member、methodを導入せず、memory仕様が列挙するpredefined primitiveに限定する。
transparent aliasは展開したcanonical typeによって利用できるprimitiveを決める。完全な型、署名、動作は
[memory primitive](memory.md)に定める。

`T.size`と型で修飾したfunctionはtop-levelのclosed valueに使える。上の加算はlambda body内の通常のexpressionであり、top-level initializerに
binary operationを追加しない。

## if

`if` は `Bool` に対する `case` の surface syntax であり、core term ではない。condition の括弧、`then`、`else` はすべて必須である。標準の表記ではconditionの後、`then`、`else`をそれぞれ別の行に置く。

```mal
absolute :: Int32 -> Int32 := \(x) {
    if (x < 0)
        then { -x }
        else { x };
};
```

condition は `Bool`、すなわち構造的に `[Unit, Unit]` と等しい型でなければならない。両 branch の結果型は同一でなければならない。branch 内の binding はその branch にだけ scope を持ち、最後の式が branch の値になる。

上の形は次へ desugar する。condition は一度だけ、branch より先に評価する。

```mal
case (x < 0)
    [0](_) { x }
    [1](_) { -x }
```

`then` は `Bool` の index 1、`else` は index 0 に対応する。`else if` 専用構文はなく、必要なら `else` block の結果に別の `if` を置く。

## case

```mal
getOrZero :: MaybeInt32 -> Int32 :=
    \(value) {
        case (value)
            [0](_) { 0 }
            [1](x) { x };
    };
```

scrutinee の括弧は必須であり、直和型でなければならない。arm の pattern は該当 index の項型に対して検査する。arm は exhaustive、index は重複なし、全 block の結果型は同一でなければならない。

arm の block は `if` の branch と同じ expression block であり、0 個以上の binding または expression statement と最後の結果式からなる。pattern binding と block 内の binding は arm ごとの同じ scope に属し、その arm の外から参照できない。

## literal

```mal
123
-123
0xff
0b101010
123i32
255u8
1.5f32
2.0f64
1_000
0xff_ffu32
1_000.25f64
```

負号は literal token の一部ではなく unary `-` として扱う。整数の型suffixは`i8`、`i16`、`i32`、`i64`、
`u8`、`u16`、`u32`、`u64`とする。suffix のない整数は周辺型から決め、決まらなければ `Int64`。
浮動小数は周辺型から決め、決まらなければ `Float64` とする。

decimal float literalは数学的な十進値から目的型へround-to-nearest, ties-to-evenで正しく丸める。有限範囲をoverflowするliteralはcompile-time errorとする。underflowは通常の演算と同じくsubnormalまたは符号付きzeroへ丸め得る。v0.5はinfinity、NaN、hexadecimal floatのliteralを持たない。

decimal pointを使う形は整数部と小数部の両方を必須とする。`e`または`E`による10進exponentと
optionalな符号を認める。decimal point、exponent、`f32`/`f64` suffixのいずれかがあるliteralを
float literalとする。完全な形は[grammar](grammar.md#numeric-separator)に定める。

numeric separatorの`_`は各digit sequenceのdigit間だけに置け、値と型に影響しない。完全な規則は[字句仕様](grammar.md#numeric-separator)に定める。

Symbol literal は最低限 `\\`、`\"`、`\n`、`\r`、`\t`、`\0`、`\xNN` を認める。byte列としての意味とstorageは[Symbol](symbols.md#literal)に定める。

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
pointer:  Ptr + UInt64, Ptr - UInt64
Symbol:   Symbol + Symbol, == !=
```

数値比較は predefined `Bool` を返す。異なる数値型を暗黙変換しない。

`Symbol + Symbol`はbyte sequenceを連結して`Symbol`を返す。完全な規則は
[Symbol](symbols.md#operator)に定める。

Bool operator は core primitive ではなく `case` へ desugar する。特に `&&` と `||` は左 operand を一度だけ先に評価し、必要な場合だけ右 operand を評価する。

```mal
a && b
```

```mal
case (a)
    [0](_) { false }
    [1](_) { b }
```

```mal
a || b
```

```mal
case (a)
    [0](_) { b }
    [1](_) { true }
```

`!`、Bool の `==` と `!=` も同様に、一つまたは二つの exhaustive `case` へ desugarできる。Bool equality の両 operand は通常の operator と同じく、case 分岐より前に左から右へ必ず評価する。これらの演算子は評価を省略または重複させてはならない。

組み込み数値型を conversion form として使える。

```mal
y := Int32(x);
z := Float64(y);
```

これは数値変換であり、bit reinterpretationではない。

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
extern log("done");
```

これは概念上 `_ := extern log("done");` と同じである。
