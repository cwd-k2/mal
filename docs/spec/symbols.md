# `Symbol`

Status: Current v0.5 profile

## 値

`Symbol`はmalに組み込まれたimmutableな有限byte値である。source identifier、interned atom、
array、buffer、encoded textではない。Unicode character、code point、grapheme、normalizationの概念を持たず、
valid UTF-8も保証しない。

`Symbol`は[Engram](engrams.md)の一種である。値の意味、表現、到達可能性に応じたstorageの保持と回収はmalだけが
支配し、source programやhostは個別のstorage identityを観測しない。値のcopyは同じimmutable byte sequenceを
与えるが、descriptorやallocationの同一性は言語の意味に含まれない。

literalのbytesはprogram imageのstatic storageに置いてよい。連結、`extern` result、`loadSymbol`などから得る
runtime値は、host storageを参照する値ではなくmalへ受け入れられた新しい`Symbol`である。正確な境界規則は
[EngramとExtern](engrams.md#境界のoperation)に従う。

## literal

`"..."`は`Symbol`を表すliteral notationである。quoted contentsを実行時に構築するoperationではない。
mal sourceはUTF-8であり、raw source characterはUTF-8 bytesとしてliteralへ入るため、`"あ"`は3 bytesを持つ。

```mal
"hello"
""
"\x00\xff"
```

escapeは最低限`\\`、`\"`、`\n`、`\r`、`\t`、`\0`、`\xNN`を認める。`\xNN`はちょうど2桁の
hexadecimal digitで任意の1 byteを表す。

## operator

```mal
#value
value # index
left + right
```

`#value`はbyte lengthを`UInt64`で返す。`value # index`は`index < #value`をpreconditionとし、`UInt64`の
0-based indexにあるbyteを`UInt8`で返す。precondition違反時の実行結果は保証しない。binary `#`はnon-associativeである。

`left + right`は`#left + #right`が`UInt64`で表せることをpreconditionとし、両operandのbytesを順に連結した
新しい`Symbol`を返す。空`Symbol`は単位元である。precondition違反時の実行結果は保証しない。必要なstorage sizeを
targetで表現できない場合、またはstorageを確保できない場合はtrapする。実装は観測可能な結果を変えない限り
storageを共有または再利用してよい。

`==`と`!=`はbyte-wise equalityとし、orderingは定義しない。これらのoperationはfirst-class functionではない。

## mutable bytesとの分離

`Symbol`の内容は変更できない。mutableな外部storageは`Ptr`またはexternal opaque typeで表し、必要なbytesを
`loadSymbol`または`Symbol`を返す`extern`によって明示的にmalへ受け入れる。反対方向のcopyには`storeSymbol`
または`Symbol` parameterを持つ`extern`を使う。

反復回数がboundedでない入力をすべて`Symbol`へ変換すれば、実装が回収可能と判断するまでstorageを必要とする。
stream処理では再利用可能な`Ptr` regionへ入力し、保持すべき値だけを`Symbol`にする構成を選べる。
