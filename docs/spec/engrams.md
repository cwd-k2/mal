# Engram

Status: Current v0.5 profile

## 値とstorage

`Engram`はmalに組み込まれたimmutableな有限byte値である。array、buffer、encoded textではない。
名前は「内部へ書き込まれたもの」を表し、[D017](../design/decisions.md#d017-immutable-byte値の型名はengramとする)に従う。
Unicode character、code point、grapheme、normalizationの概念を持たず、valid UTF-8も保証しない。

Engram値はcopyableなdescriptorとして振る舞う。値を複製してもbytes自体を複製する必要はない。bytesはmal program終了まで有効で変更されず、source-levelの個別解放操作は存在しない。

Engram literalのbytesはprogram imageにあらかじめ含まれ、静的storageに置いてよい。host側の一時byte
bufferはEngramではない。`extern`のresultとしてmal-ownedなprogram-lifetime storageへcopyされた時点で、
新しいEngramになる。[`extern`境界の規則](extern.md#engramのlifetime)に従う。

## literal

`"..."`はEngramを表すliteral notationである。numeric literalがnumeric valueを表すのと同様に、
quoted contentsを実行時に構築するoperationではない。

mal sourceはUTF-8である。raw source characterはUTF-8 bytesとしてliteralへ入るため、`"あ"`は3 bytesを持つ。

```mal
"hello"
"こんにちは"
""
"\x00\xff"
```

escapeは最低限`\\`、`\"`、`\n`、`\r`、`\t`、`\0`、`\xNN`を認める。`\xNN`はちょうど2桁のhexadecimal digitで任意の1 byteを表す。

## operator

Engramに組み込む観測operatorは次である。

```mal
#value
value # index
```

`#value`はbyte lengthを`UInt64`で返す。`value # index`は`UInt64`のindexにあるbyteを`UInt8`で返す。
いずれもEngram descriptorを観測する組み込みoperatorであり、function valueとしては存在しない。

indexは0-basedで、範囲外の`#` accessはtrapする。binary `#`はnon-associativeである。

`==`と`!=`はbyte-wise equalityとする。orderingとconcatenationは定義しない。

```mal
"a" == "a"
"a" != "b"
```

次はcompile-time errorである。

```mal
"a" < "b"
"a" + "b"
```

## mutable bytesとの分離

Engramは内容を観測できるが変更できない。v0.5は組み込みのarray/sliceを持たない。内容を加工するには
`Ptr`とlength、または`ByteBuffer`などのexternal opaque typeが表すmutable host storageへcopyし、加工後の
bytesを別のEngramとしてmalへcopyする。

```mal
extern ByteBuffer;
extern bufferNew :: UInt64 -> ByteBuffer;
extern bufferWrite :: (ByteBuffer, UInt64, UInt8) -> Unit;
extern bufferToEngram :: ByteBuffer -> Engram;
extern bufferFree :: ByteBuffer -> Unit;
```

これはpredefined APIではない。`bufferToEngram`のresultにはextern return時のcopy規則を適用する。opaque handleのalias、bounds、allocation、freeの安全性はhost contractとprogramの責務である。

concatenationのように新しいbytesを作る処理も、必要ならEngramを返す`extern`として宣言する。そのresultは同じくmal-owned storageへcopyされる。
