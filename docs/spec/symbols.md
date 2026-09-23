# `Symbol`

Status: Accepted v0.6 profile

## 値

`Symbol`はmalに組み込まれたimmutableな有限byte値である。source identifier、interned atom、
array、buffer、encoded textではない。Unicode character、code point、grapheme、normalizationの概念を持たず、
valid UTF-8も保証しない。

`Symbol`は[Engram](engrams.md)の一種である。値の意味、表現、到達可能性に応じたstorageの保持と回収はmalだけが
支配し、source programやhostは個別のstorage identityを観測しない。値のcopyは同じimmutable byte sequenceを
与えるが、descriptorやallocationの同一性は言語の意味に含まれない。

literalのbytesはprogram imageのstatic storageに置いてよい。連結とexternal bytesのadmissionで得るruntime storageは
host storageを参照せず、malが所有する。`Buffer<UInt8>`からの変換は変換時点のbytesをimmutable snapshotとして保持する。
host bytesはC host profileの`from<UInt8>`でBufferへcopyしてから`*`でSymbolにする。

## literal

`"..."`は`Symbol`を表すliteral notationである。quoted contentsを実行時に構築するoperationではない。
mal sourceはUTF-8であり、raw source characterはUTF-8 bytesとしてliteralへ入るため、`"あ"`は3 bytesを持つ。

```mal
"hello"
""
"\x00\xff"
```

受理するescapeのsource spellingは[grammar](grammar.md#文法概要)に定める。`\xNN`はちょうど2桁のhexadecimal digitで
任意の1 byteを表し、その他のescapeは対応する単一byteを表す。

## operator

```mal
#value
value # index
left + right
```

`#value`はbyte lengthを`USize`で返す。`value # index`は`index < #value`をpreconditionとし、`USize`の
0-based indexにあるbyteを`UInt8`で返す。precondition違反時の実行結果は保証しない。binary `#`はnon-associativeである。

`left + right`は数学的な`#left + #right`がUSizeとtarget allocation sizeで表せることをpreconditionとし、両operandのbytesを順に連結した
新しい`Symbol`を返す。空`Symbol`は単位元である。precondition違反時の実行結果は保証しない。必要なstorage sizeを
targetで表現できない場合、またはstorageを確保できない場合はtrapする。実装は観測可能な結果を変えない限り
storageを共有または再利用してよい。

`==`と`!=`はbyte-wise equalityとし、orderingは定義しない。これらのoperationはfirst-class functionではない。

length、byte access、equalityは既存のbyte sequenceを観測するoperationであり、新しいEngramを構成しない。したがって、
有効なoperandに対して内部表現だけを理由とするstorage allocationやallocation failureを追加してはならない。
`*symbol`によるBuffer変換と`*buffer`によるSymbol変換はoperandをconsumeしない。実装はcopy-on-writeでstorageを共有してよいが、
Bufferの変更をSymbolから観測できてはならない。
`Symbol`自体はextern境界を通らない。

byte accessはimmutableなbyte valueに対する位置指定のobservationである。反復的な更新または再利用可能なsequenceには
`Buffer<UInt8>`、host resource固有のaccessにはextern contractを使う。

## mutable bytesとの分離

`Symbol`の内容は変更できない。mutableなmal-owned bytesは`Buffer<UInt8>`、host-owned storageはAddressとextern contractで表す。
C host storageとのcopyは`from<UInt8>`と`buffer.into`だけが行う。
