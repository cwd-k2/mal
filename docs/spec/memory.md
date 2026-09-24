# AddressとBuffer

Status: Accepted v0.6

この文書はhost-managed storageを指す`Address`、mal-owned mutable sequenceである`Buffer<T>`、C hostとのcopy境界を定める。
surface syntaxは[字句と文法](grammar.md)、public C representationは[C host ABI](c-host-abi.md)を正とする。

## Address

`Address`はhost-managed resourceを指すcopyableなopaque capabilityである。数値、null、要素型、extent、permission、ownership、
alignment、allocation identityをsource-levelでは持たない。複製してもreferentのlifetimeを延長しない。

mal codeは`Address`をdereference、変更、比較、加減算、integer変換できない。通常のoperationはAddressの向こう側にある表現を
知らず、Addressを解釈する能力はextern contractまたは後述するC host copy primitiveだけが与える。

`ByteSize`はhost contractがbyte量に使うtarget幅のunsigned量、`USize`は有限collectionの要素数、index、capacityに使う
target幅のunsigned量である。両者は別のsource typeで、literal suffixは`bytes`と`usize`である。

## Representable

compilerは閉じた`Representable(A)` judgmentを持つ。

```text
Representable(Unit)
Representable(numeric scalar)
Representable(Address)
Representable(ByteSize)
Representable(USize)
Representable((A...))       if all Representable(A)
Representable([A...])       if the sum has at least two variants and all Representable(A)
```

`Symbol`、function、external opaque type、`Buffer<A>`、empty sumはrepresentableでない。transparent aliasは展開後に判定する。
`Buffer<A>`は`Representable(A)`の場合だけwell-formedである。

RepresentableはC host copy boundaryでcanonical representationを持ち、Buffer storageへ値を格納できることを表す。
Address referentが実際にそのrepresentationを持つことや、access可能であることは証明しない。

## Canonical layout

numeric scalar、`Address`、`ByteSize`、`USize`のstrideとrequired alignmentはtarget data layoutから決める。numeric scalarの
storage幅はbit幅、Addressはdefault address spaceのpointer representation幅、ByteSizeとUSizeはpointer index幅を使う。
byte orderとscalar representationはbackend host ABIが定める。

`Unit`はstride 0、required alignment 1である。canonical layoutのstrideが0になる型はstorageをdereferenceせず、Bufferと
C host copy primitiveはlogical countだけを扱う。Unitだけからなるproductとtransparent aliasにも同じ規則を適用する。

productはfieldをsource orderに配置する。先頭offsetは0、後続offsetは直前fieldの末尾からそのfieldのrequired alignmentまで
前方へ丸める。全体alignmentは全fieldの最大値、strideは最後のfieldの末尾から全体alignmentまで前方へ丸める。
nested productはflattenしない。

sumは0-based variant indexのtag、padding、全variantで共有するpayload領域の順に配置する。tagはvariant数を表せる最小の
`UInt8`、`UInt16`、`UInt32`、`UInt64`を使い、`2^64`を超えるvariantを拒否する。payload offsetはtag末尾から全variantの
最大alignmentまで前方へ丸め、payload extentは全variant strideの最大値とする。sum strideはpayload末尾からsum全体の
alignmentまで前方へ丸める。

storeはproduct field、sum tag、選択payloadだけを書き、paddingと非選択payloadを変更しなくてよい。loadはそれらを読まない。
このlayoutは同じartifactと対応adapterの間だけで有効であり、mal runtime representation、public C aggregate carrier、file、
network、永続storageのformatではない。C hostがこのlayoutを読む場合はpublic carrierをcastせず、
[C host ABIのnamed alias helper](c-host-abi.md#canonical-memory-access)を使う。

## Canonical representationとtarget contract

backendはcanonical memory専用のtarget layout planを作り、runtime valueの内部layoutを再利用しない。default address spaceの
pointer representation幅、pointer index幅、primitive ABI alignmentをtarget data layoutから別々に取得する。layout、stride、
offset、allocation sizeをtargetのobject sizeで表現できない型はartifact生成時に拒否する。C hostのmappingは
[C host ABI](c-host-abi.md)に定める。

## Buffer

`Buffer<A>`はmal-ownedなmutable有限要素列である。値はbuffer identityへの共有参照としてcopyされ、どのaliasから行った変更も
同じBufferを指す全aliasから観測できる。参照が到達不能になった後のstorage回収はbackendとruntimeが行い、source-levelの
`free`、retain、releaseは存在しない。

```text
make<A>(USize)                     -> Buffer<A>
#Buffer<A>                         -> USize
Buffer<A>.new(A)                   -> USize
Buffer<A>.get(USize)               -> A
Buffer<A>.put(USize, A)            -> Unit
Buffer<A>.fill(USize, USize, A)    -> Unit
Buffer<A>.copy(USize, Buffer<A>, USize, USize) -> Unit
```

`make<A>(capacity)`はcount 0のBufferを返す。capacityは初期allocationの要求であり、論理countではない。後続の`new`はcapacityを
超えてgrowthできる。`new`は末尾へ追加し、その安定した0-based indexを返す。`get`と`put`は現在のindexを読み書きする。
receiver-firstでない`new(buffer, value)`、`get(buffer, index)`、`put(buffer, index, value)`も同じpredefined operationである。

`buffer.fill(offset, length, value)`は半開区間`[offset, offset + length)`の全要素へ`value`を代入する。
`buffer.copy(destinationOffset, source, sourceOffset, length)`は`source`の半開区間
`[sourceOffset, sourceOffset + length)`をreceiverの`[destinationOffset, destinationOffset + length)`へ代入する。
どちらも既存要素を上書きし、destination rangeが現在の末尾を越える場合はcountをrange末尾まで延ばす。開始offsetは現在の
count以下でなければならず、未初期化の穴は作らない。長さ0のrangeもこのoffset条件に従う。`copy`でsourceとdestinationが
同じBufferを指しrangeが重なる場合、operation開始時点のsource rangeをcopyした結果になる。

receiver-firstでない形は`fill(buffer, offset, length, value)`と
`copy(destination, destinationOffset, source, sourceOffset, length)`である。

operationのoperandはsource順に一度だけ評価する。count、capacity、range末尾、stride、allocation byte数をtargetで表現できない場合と
allocationに失敗した場合はtrapする。stride 0でもcountとrange末尾のoverflowはtrapする。

Bufferをfunction parameter、result、aggregate field、closure capture、通常のgeneric argumentに置ける。Buffer elementだけは
`Representable`に閉じるため、Buffer storageから別のmanaged ownerへのedgeは生じない。

## Symbol conversion

```text
*Buffer<UInt8> -> Symbol
*Symbol        -> Buffer<UInt8>
```

`*buffer`は変換時点のbytesを持つimmutableなSymbol snapshotを返す。以後のBuffer変更はresultを変更しない。
`*symbol`は同じbytesで初期化した変更可能なBufferを返し、Symbolは変更されない。実装はcopy-on-writeでstorageを共有してよいが、
source-levelのaliasingとimmutabilityを変えてはならない。operandはconsumeされず、変換後も利用できる。

## C host copy boundary

次のpredefined generic operationはC hostとのcopyを行う。offsetとlengthは`A`の要素単位であり、byte単位ではない。

```text
from<A>(Address, USize, USize)             -> Buffer<A>
Buffer<A>.into(Address, USize, USize)      -> Unit
```

`from<A>(address, offset, length)`はhost storageの半開区間`[offset, offset + length)`をsource順にcopyし、countが`length`の
新しいBufferを返す。`buffer.into(address, offset, length)`はBufferの同じ半開区間をhost storageの先頭へcopyする。
`into`はBufferを変更またはconsumeしない。receiver-firstでない形は`into(buffer, address, offset, length)`である。

copyにはcanonical representationを使う。numeric scalar、Address、ByteSize、USizeの幅とalignmentはtarget ABI、
productはsource順のfieldとpadding、sumはvariant tagとactive payloadを使う。public C aggregate carrier自体のlayoutとは独立であり、
同じrepresentationをhost codeが扱う場合はgenerated canonical memory helperを使う。

stride 0の型はstorageをdereferenceせず、logical countだけをcopyする。offsetとlengthの加算、byte offset、allocation sizeがtargetで
表現できない場合と、mal-owned allocationに失敗した場合はtrapする。

## 未検査precondition

| Operation | Precondition |
|---|---|
| `buffer.get(index)`、`buffer.put(index, value)` | `index < #buffer` |
| `buffer.fill(offset, length, value)` | `offset <= #buffer` |
| `destination.copy(destinationOffset, source, sourceOffset, length)` | `destinationOffset <= #destination`かつ`sourceOffset + length <= #source` |
| `from<A>(address, offset, length)` | 対象rangeが同じlive storage内にあり、readable、初期化済みで、各要素がvalid canonical representationを持つ |
| `buffer.into(address, offset, length)` | `offset + length <= #buffer`で、destinationが`length`要素分writableである |

host storageのextent、permission、initialization、lifetime、overlap、Address representationはhost contractが所有する。
primitiveはこれらを検査せず、違反時の結果を保証しない。zero-stride elementはhost storageのreadability、writability、extentを要求しない。
Address要素をBufferへcopyしても、そのreferentのlifetimeは延長しない。
