# literalとoperator

Status: Accepted v0.6

この文書はnumeric、byte、Symbolのliteral、primitive operator、numeric conversionを定める。application、binding、`if`、
直和の除去は[式とbinding](expressions.md)、concrete syntaxは[字句と文法](grammar.md)を正とする。

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
USize:    + - * / %  == != < <= > >=
ByteSize: + -        == != < <= > >=
Bool:     ! && || == !=
Symbol:   Symbol + Symbol, == !=
```

単項`-`は整数とfloatに使え、unsigned整数ではwrapする。`USize`と`ByteSize`はtarget幅のunsigned整数として整数と同じwrapと
divisorのpreconditionに従い、単項演算、bit演算、shiftを持たない。

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

