# BufferとHost

Status: Exploratory support document

この文書は、memory操作の語彙をIxPool、Buffer、`Host<A>`がどう分け持つかと、BufferとHost<A>のoperationを管理する。
区分は[Pool primitive](pool.md#区分)、Bufferの各operationの意味とpreconditionは[AddressとBuffer](../../../spec/memory.md)を
正とする。

## 語彙の分担


memory操作は、抽象する単位で四つの語彙に分かれ、IxPool、Buffer、`Host<A>`が分け持つ。

| 語彙 | 抽象する単位 | 操作 | 持つ型 |
|---|---|---|---|
| slot | 一つのcoordinateのplace | `peek`、`swap`、`grow`、Meta | IxPool |
| sequence | 一つのLiveなrun `[0, count)` | `make`、`new`、`get`、`put`、`#`、growth policy | Buffer |
| run | 連続した要素範囲の一括の転送 | `fill`、`copy` | Buffer |
| host境界 | hostとの値の交換 | `admit`、`observe`、ImPoolの範囲との変換、`Symbol`の構築 | `Host<A>` |

IxPoolはslotを抽象し、hostともrunとも関わらない。Bufferは一つのLiveなrunを抽象し、runの転送を持つ。IxPoolはslotを空ける
語彙を、Bufferは占有状態を気にせずrunを扱う語彙を持ち、互いに相手の持たない語彙を補う。

IxPoolがrunの語彙を持たないのは、runを本当に必要とするcontainerがBufferだけだからである。Map、木のLiveな集合は区間に
ならず、Dequeが成長時に必要とするのは値を写す`copy`ではなく移す操作である（[container](../containers/overview.md#runの語彙)）。

host境界は、IxPoolにもBufferにも置かず、不変な値の型`Host<A>`に置く。[authority](../../../design/authority.md#境界では動詞を選ぶ)
が分類するadmissionとobservationはどちらも値の操作であり、可変なidentityをhost境界へ出さずに済む。`Host<A>`は全要素が値を
持つ密な列なので、Vacantを含む範囲をhostとどう交換するかという問題も型の上で起きない。IxPoolはAddressの権限を持たず、
BufferはIxPoolと`Host<A>`の上に全operationを書ける。

## Buffer

Bufferは言語の組み込み型ではなく、IxPoolの上のpreludeのopaque型であり、全operationを[Buffer実装](../containers/buffer.md)の
参照実装で定める。型、意味、preconditionは現行の[AddressとBuffer](../../../spec/memory.md)のままであり、本案は変えない。

| operation | 区分 | 参照実装 |
|---|---|---|
| `from<A>(address, offset, length)` | 定数倍の周辺 | `admit`した`Host<A>`の要素をIxPoolへ置く |
| `buffer.into(address, offset, length)` | 派生 | Bufferを`freeze`した値の範囲から`Host<A>`を作って`observe`する |
| `*buffer`（`Buffer<UInt8>`から`Symbol`） | 派生 | byte列の`freeze`。`symbol(host(freeze(buffer), 0, count))`であり、Liveなrun `[0, count)`だけを値にする |
| `*symbol`（`Symbol`から`Buffer<UInt8>`） | 定数倍の周辺 | byte列の`thaw`。`Symbol`のbyte列を`thaw`し、Metaを長さにしたBufferを返す。`Symbol`をImPoolへ写す部分は`symbol # index`のloop |
| `fill`、`copy` | 派生 | slot操作のloop |

`*buffer`と`*symbol`は、identityを共有する側と値の側の間の`freeze`と`thaw`をbyte列に特化し、Bufferのinvariantに合わせて
Liveなrunへ射影したものである。[Symbol conversion](../../../spec/memory.md#symbol-conversion)が定める「以後のBuffer変更はresultを変更しない」と
「Symbolは変更されない」は、`freeze`と`thaw`の後の書き込みがもう一方から観測されないことと一致する。

Bufferは組み込みlibraryとして提供し、runtimeは参照実装と同じ結果になる一括処理で実装してよい。現行runtimeと同じ費用は、
この実装の自由で保つ。`*`はbyte IxPoolのstorageを`Symbol`と共有してよく、書き込みはcopy-on-writeにする
（[representation](../runtime/contract.md#runtime-representation)）。

containerが現行runtimeと同じoverflow trapをmalで起こすには、[primitive `trap`案](../../primitive-trap.md)の
`trap :: Symbol -> []`を使う。Pool案はこの採択に依存する。

## Host

`Host<A>`は、`Representable`な`A`の要素が`[0, n)`に密に並ぶ、canonical layoutの不変な値である。hostとの交換はこの型だけが
持つ。`Host<A>`はhostの値を写した不変な値であり、Poolの値の側に属する。意味の上では、全slotがLiveな`ImPool<Unit, A>`を
`Representable`な要素とcanonical layoutに限ったものに当たる。

```mal
Host<A>

admit<A> :: (Address, USize, USize) -> Host<A>;
observe<A> :: (Host<A>, Address, USize) -> Unit;
host<Meta, A> :: (ImPool<Meta, A>, USize, USize) -> Host<A>;
symbol :: Host<UInt8> -> Symbol;
```

`#host`は要素数を、`host # index`は要素を返す。

Addressは加減算も比較も持たないため、それ単独では位置を表さず、host storageというExternの所有するcarrierのoriginに当たる。
位置はoffsetが表し、`(address, offset)`はPoolとcoordinateの組と同じ形を取る。`admit(address, offset, length)`と
`host(value, offset, length)`はどちらもcarrierの範囲を写した`Host<A>`を作り、`observe(host, address, offset)`は`Host<A>`を
host storageの範囲へ書く。host storageとImPoolの違いは、所有がExternかEngramか、host storageが可変か、大きさと各位置の状態を
malが観測できるかにある。IxPoolの範囲は、`freeze`して値にしてから`host`へ渡す。

| operation | 区分 | 意味 | precondition |
|---|---|---|---|
| `admit(address, offset, length)` | 意味論の核 | host storageの`[offset, offset + length)`をcopyした値を返す | 対象rangeがreadable、初期化済みで、各要素がvalid canonical representationを持つ |
| `observe(host, address, offset)` | 意味論の核 | 全要素をhost storageの`[offset, offset + #host)`へcopyする | 対象rangeが`#host`要素分writable |
| `host(value, offset, length)` | 意味論の核 | ImPoolの`[offset, offset + length)`の値を持つ`Host<A>`を返す | 範囲が`n`以内で全slotがLive |
| `#host`、`host # index` | 意味論の核 | 要素数と要素を読む | `index < #host` |
| `symbol(host)` | 意味論の核 | 同じbyte列の`Symbol`を返す | なし |

`Host<A>`からImPoolやIxPoolへの変換は`host # index`と`swap`のloopで書けるため、定数倍の周辺である。`admit`と`observe`のhost側の条件、
offsetとlengthの加算やallocation sizeを表現できない場合のtrapは、現行の[C host copy boundary](../../../spec/memory.md#c-host-copy-boundary)
と同じである。`Symbol`は意味の上では`Host<UInt8>`と同じ密で不変なbyte列であり、`symbol`は表現を`Symbol`の専用の形へ移す。

