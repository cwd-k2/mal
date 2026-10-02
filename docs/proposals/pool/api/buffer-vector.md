# BufferとVector

Status: Exploratory support document

この文書は、memory操作の語彙をIxPool、Buffer、Vectorがどう分け持つかと、BufferとVectorのoperationを管理する。区分は
[Pool primitive](pool.md#区分)、Bufferの各operationの意味とpreconditionは[AddressとBuffer](../../../spec/memory.md)、Vectorの参照実装は
[列のcontainer](../containers/sequences.md#vector)を正とする。

## 語彙の分担

memory操作は、抽象する単位で四つの語彙に分かれ、IxPool、Buffer、Vectorが分け持つ。

| 語彙 | 抽象する単位 | 操作 | 持つ型 |
|---|---|---|---|
| slot | 一つのcoordinateのplace | `peek`、`swap`、`grow`、Meta | IxPool |
| sequence | 一つのLiveなrun `[0, count)` | `make`、`new`、`get`、`put`、`#`、growth policy | Buffer |
| run | 連続した要素範囲の一括の転送 | `fill`、`copy` | Buffer |
| host境界 | hostとの値の交換 | `from`、`into`、`Symbol`の構築 | Vector |

IxPoolはslotを抽象し、hostともrunとも関わらない。Bufferは一つのLiveなrunを抽象し、runの転送を持つ。IxPoolはslotを空ける
語彙を、Bufferは占有状態を気にせずrunを扱う語彙を持ち、互いに相手の持たない語彙を補う。

IxPoolがrunの語彙を持たないのは、runを本当に必要とするcontainerがBufferだけだからである。Map、木のLiveな集合は区間に
ならず、Dequeが成長時に必要とするのは値を写す`copy`ではなく移す操作である（[container](../containers/overview.md#runの語彙)）。

host境界は、IxPoolにもBufferにも置かず、値の列であるVectorに置く。[authority](../../../design/authority.md#境界では動詞を選ぶ)
が分類するadmissionとobservationはどちらも値の操作であり、可変なidentityをhost境界へ出さずに済む。Vectorは`[0, length)`が
すべて値を持つ密な列なので、Vacantを含む範囲をhostとどう交換するかという問題も起きない。IxPoolはAddressの権限を持たず、
BufferはIxPoolとVectorの上に全operationを書ける。

## BufferとVectorの対

BufferとVectorは、一つのLiveなrunを、identityを共有する側と値の側でそれぞれ表す。

```mal
opaque Buffer<A> :: IxPool<USize, A>;    // Metaはcount、[0, count)がLive
opaque Vector<A> :: ImPool<USize, A>;    // Metaはlength、[0, length)がLive
```

二つのinvariantは同じ形なので、Bufferを`freeze`した値はそのままVectorであり、Vectorを`thaw`した値はそのままBufferである。
Bufferは組み立てるための可変な列、Vectorは確定した値の列であり、byte列ではこの対が`Buffer<UInt8>`と`Symbol`に当たる。
どちらも組み込みlibraryとして提供し、runtimeは参照実装と同じ結果になる一括処理で実装してよい。

次の表は、BufferとVectorのoperationを対で並べる。Vectorの読み出しと長さの名前は仮であり、更新と追加をBufferと同じ`put`、`new`へ
揃えるかは[primitiveの名前](../README.md#未決定事項)と合わせて決める。

| 役割 | Buffer | Vector |
|---|---|---|
| 作る | `make<A>(capacity)` | 未定 |
| 末尾に足す | `buffer.new(value)` | `vectorAppend(vector, value)` |
| 読む | `buffer.get(index)` | `vector.get(index)`（仮） |
| 書き換える | `buffer.put(index, value)` | `vectorSet(vector, index, value)` |
| 長さ | `#buffer` | `#vector`（仮） |
| 範囲を埋める | `buffer.fill(offset, count, value)` | なし |
| 範囲を写す | `buffer.copy(offset, source, sourceOffset, count)`（既存のBufferへ書く） | `slice(vector, offset, length)`（新しいVectorを作る） |
| hostから読む | `thaw(from(address, offset, length))` | `from<A>(address, offset, length)` |
| hostへ書く | `slice(freeze(buffer), offset, length).into(address, destination)` | `vector.into(address, offset)` |
| `Symbol`へ | `*buffer` | `symbol(vector)` |
| `Symbol`から | `*symbol` | なし |
| 相手への変換 | `freeze(buffer)` | `thaw(vector)` |

## Buffer

Bufferは言語の組み込み型ではなく、IxPoolの上のpreludeのopaque型であり、全operationを[Buffer実装](../containers/buffer.md)の
参照実装で定める。hostとの交換はVectorへ移し、Bufferは`from`と`into`を持たない。それ以外のoperationの型、意味、preconditionは
現行の[AddressとBuffer](../../../spec/memory.md)のままである。

| operation | 区分 | 参照実装 |
|---|---|---|
| `*buffer`（`Buffer<UInt8>`から`Symbol`） | 派生 | byte列の`freeze`。`symbol(freeze(buffer))`であり、Liveなrun `[0, count)`だけを値にする |
| `*symbol`（`Symbol`から`Buffer<UInt8>`） | 定数倍の周辺 | byte列の`thaw`。`Symbol`のbyte列をVectorへ写して`thaw`する。写す部分は`symbol # index`のloop |
| `fill`、`copy` | 派生 | slot操作のloop |

`*buffer`と`*symbol`は、BufferとVectorの間の`freeze`と`thaw`をbyte列に特化したものである。
[Symbol conversion](../../../spec/memory.md#symbol-conversion)が定める「以後のBuffer変更はresultを変更しない」と
「Symbolは変更されない」は、`freeze`と`thaw`の後の書き込みがもう一方から観測されないことと一致する。

hostとBufferの間の交換は、対応表のとおりVectorを経由し、現行仕様のBufferの`from`と`into`はこの形へ移る。`slice`はVectorの
参照実装の範囲の写しである。

現行runtimeと同じ費用は、組み込みlibraryとしての実装の自由で保つ。`*`は、Bufferがlast useならstorageを`Symbol`へ移せる
（[freezeとthaw](pool.md#freezeとthaw)）。

containerが現行runtimeと同じoverflow trapをmalで起こすには、[primitive `trap`案](../../primitive-trap.md)の
`trap :: Symbol -> []`を使う。Pool案はこの採択に依存する。

## Vector

Vectorは値の列であり、ImPoolの上のpreludeのopaque型である。読み出し、更新、連結などは参照実装で定め、hostとの交換と
`Symbol`の構築だけをruntimeのprimitiveとする。

```mal
from<A> :: (Address, USize, USize) -> Vector<A>;
into<A> :: (Vector<A>, Address, USize) -> Unit;
symbol :: Vector<UInt8> -> Symbol;
```

| operation | 区分 | 意味 | precondition |
|---|---|---|---|
| `from<A>(address, offset, length)` | 意味論の核 | host storageの`[offset, offset + length)`をcopyした、長さ`length`のVectorを返す | `Representable(A)`。対象rangeがreadable、初期化済みで、各要素がvalid canonical representationを持つ |
| `vector.into(address, offset)` | 意味論の核 | `[0, length)`の全要素をhost storageの`[offset, offset + length)`へcopyする | `Representable(A)`。対象rangeが`length`要素分writable |
| `symbol(vector)` | 意味論の核 | `[0, length)`と同じbyte列の`Symbol`を返す | なし |

runtimeは`Representable`な要素のVectorをcanonical layoutで連続に置き、`from`と`into`を一括copyで実装してよい。この配置は
sourceから観測できない。`from`と`into`のhost側の条件、offsetとlengthの加算やallocation sizeを表現できない場合のtrapは、
現行の[C host copy boundary](../../../spec/memory.md#c-host-copy-boundary)と同じである。`Symbol`は意味の上では`Vector<UInt8>`に
`+`、`/`、`%`、`==`などのtext操作を加えた密で不変なbyte列であり、`symbol`は表現を、storageを共有するsliceのview、static storageの
literal、占有tagのないbyte列という`Symbol`の専用の形へ移す。

Addressは加減算も比較も持たないため、それ単独では位置を表さず、host storageというExternの所有するcarrierのoriginに当たる。
位置はoffsetが表し、`(address, offset)`はPoolとcoordinateの組と同じ形を取る。`from`はhost storageの範囲を写したVectorを作り、
`into`はVectorをhost storageの範囲へ書く。現行のBufferの`into`がmal側の範囲を取るのに対し、Vectorの`into`はhost側の位置を取り、
mal側の範囲は`slice`で切り出す。host storageとImPoolの違いは、所有がExternかEngramか、host storageが可変か、
大きさと各位置の状態をmalが観測できるかにある。
