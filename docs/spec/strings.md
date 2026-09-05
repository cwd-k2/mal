# String

Status: Current v0.5 profile

## 値とstorage

`String`はimmutableな有限byte sequenceである。型名は[D017](../design/decisions.md#d017-immutable-byte-sequenceの型名はstringとする)に従う。名前からtext encodingを推論してはならず、Unicode character、code point、grapheme、normalizationの概念を持たず、valid UTF-8も保証しない。

String値はcopyableなdescriptorとして振る舞う。値を複製してもbytes自体を複製する必要はない。bytesはmal program終了まで有効で変更されず、source-levelの個別解放操作は存在しない。

String literalのbytesは静的storageに置いてよい。実行時に`extern`から得るStringはmal-ownedなprogram-lifetime storageへcopyする。[`extern`境界の規則](extern.md#stringのlifetime)に従う。

## literal

mal sourceはUTF-8である。raw source characterはUTF-8 bytesとしてliteralへ入るため、`"あ"`は3 bytesを持つ。

```mal
"hello"
"こんにちは"
""
"\x00\xff"
```

escapeは最低限`\\`、`\"`、`\n`、`\r`、`\t`、`\0`、`\xNN`を認める。`\xNN`はちょうど2桁のhexadecimal digitで任意の1 byteを表す。

## operator

Stringに組み込む観測operatorは次である。

```mal
#value
value # index
```

`#value`はbyte lengthを`UInt64`で返す。`value # index`は`UInt64`のindexにあるbyteを`UInt8`で返す。
いずれもString descriptorを観測する組み込みoperatorであり、function valueとしては存在しない。
`byteLength`と`byteAt`はpredefined nameではない。

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

StringはGoの`string`に似たimmutable viewを提供するが、v0.5はGoの`[]byte`に相当する組み込みarray/sliceを
持たない。mutable storageが必要なら`Ptr`とlength、または`ByteBuffer`などのexternal opaque typeを使う。

```mal
extern ByteBuffer;
extern bufferNew :: UInt64 -> ByteBuffer;
extern bufferWrite :: (ByteBuffer, UInt64, UInt8) -> Unit;
extern bufferToString :: ByteBuffer -> String;
extern bufferFree :: ByteBuffer -> Unit;
```

これはpredefined APIではない。`bufferToString`のresultにはextern return時のcopy規則を適用する。opaque handleのalias、bounds、allocation、freeの安全性はhost contractとprogramの責務である。

String concatenationのように新しいbytesを作る処理も、必要ならStringを返す`extern`として宣言する。そのresultは同じくmal-owned storageへcopyされる。
