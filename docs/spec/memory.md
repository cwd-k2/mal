# `Buffer`

Status: Accepted v0.7

この文書はmal-owned mutable sequenceである`Buffer<T>`、element lifecycle、range operationを定める。surface syntaxは
[字句と文法](grammar.md)、extern Cからruntime carrierを扱う規則は[C ABI](c-host-abi.md)を正とする。

## Storable

Buffer elementは、Buffer storageがcarrierのlifetimeを保持し、read、replace、relocation、place終了を完結できる型である。
compilerは閉じた`Storable(A)` judgmentを持つ。

```text
Storable(Unit)
Storable(numeric scalar)
Storable(ByteSize)
Storable(USize)
Storable(Symbol)
Storable(external opaque type)
Storable((A...))       if all Storable(A)
Storable([A...])       if the sum has at least two variants and all Storable(A)
Storable(Buffer<A>)    if Storable(A)
```

functionとempty sumはstorableでない。external opaque valueはpointer-widthのtrivialなcarrierとして保存し、Bufferへの格納や
Bufferからの取得はhost referentのlifetimeを延長せず、closeなどのresource operationも暗黙に行わない。transparent aliasは
展開後に判定し、file-local opaque typeはhidden representationから判定する。nested Bufferも同じ規則を再帰的に
満たさなければならない。`Buffer<A>`は`Storable(A)`の場合だけwell-formedである。

Buffer elementのread、append、replace、fill、copyはcarrierについてshallowである。要素がBuffer handleなら、返した値と格納された
値は同じ内側identityを指し、そのidentityへの変更を共有観測する。外側Bufferのreplaceは格納したhandleだけを置き換え、内側Bufferを
deep copyしない。Bufferは格納した`Symbol`またはBuffer handleのresponsibilityを、外側Bufferが到達不能になるか要素が上書きされる
まで保持する。

functionを除く理由はmutable identityではない。closure environmentの型にはcapture edgeが現れず、同じBufferをcaptureしたclosureを
そのBufferへ格納するとreference-counted ownership cycleを型から検出できない。Buffer handleの入れ子は、表現に寄与するrecursive
typeがなくfile-local opaque representationの再帰も拒否されるため、このhidden back-edgeを導入しない。

## Runtime element representation

Buffer elementはspecialization後のruntime carrier layoutで保存する。primitive widthとalignment、product field offset、sumのtagと
payload、strideはtarget layoutからcompilerが決める。productはsource order、sumはactive tagとpayloadを保持する。padding、inactive
payload、runtime owner fieldを含む正確なlayoutはsource semanticsではなく、同じartifactのgenerated C headerとLLVM moduleが共有する
runtime ABIである。

layout、stride、offset、allocation sizeをtarget object sizeで表現できない型はartifact生成時に拒否する。extern Cはgenerated headerと
`mal.h`からruntime carrierを直接扱えるが、別artifact、file、network、永続storageのformatとしてこのlayoutを使えない。

## Buffer value

`Buffer<A>`はmal-ownedなmutable有限要素列である。値はbuffer identityへの共有参照としてcopyされ、どのaliasから行った変更も
同じBufferを指す全aliasから観測できる。参照が到達不能になった後のstorage回収はbackendとruntimeが行い、source-levelの`free`、
retain、releaseは存在しない。

```text
make<A>(USize)                     -> Buffer<A>
#Buffer<A>                         -> USize
Buffer<A>.new(A)                   -> USize
Buffer<A>.get(USize)               -> A
Buffer<A>.put(USize, A)            -> Unit
Buffer<A>.fill(USize, USize, A)    -> Unit
Buffer<A>.copy(USize, Buffer<A>, USize, USize) -> Unit
```

要素index、count、capacity、lengthは`USize`で表す。domainで座標の役割を名前に残す場合は、利用者がtransparent aliasを定義できる。
aliasはnominal identityを作らず、範囲、carrier identity、domain invariantを証明しない。

`make<A>(capacity)`はcount 0のBufferを返す。capacityは初期allocationの要求であり、論理countではない。後続の`new`はcapacityを
超えてgrowthできる。`new`は末尾へ追加し、その安定した0-based indexを返す。`get`と`put`は現在のindexを読み書きする。
期待resultが`Buffer<A>`なら`make(capacity)`から`A`を推論できる。期待型がなければ明示形を使う。receiver-firstでない`new`、
`get`、`put`も同じpredefined operationである。

`buffer.fill(offset, length, value)`は半開区間`[offset, offset + length)`の全要素へ`value`を代入する。
`buffer.copy(destinationOffset, source, sourceOffset, length)`は`source`の半開区間をreceiverのdestination rangeへ代入する。
どちらも既存要素を上書きし、destination rangeが現在の末尾を越える場合はcountをrange末尾まで延ばす。開始offsetは現在のcount以下で
なければならず、未初期化の穴は作らない。長さ0のrangeもこのoffset条件に従う。sourceとdestinationが同じBufferを指しrangeが
重なる場合、operation開始時点のsource rangeをcopyした結果になる。

receiver-firstでない形は`fill(buffer, offset, length, value)`と
`copy(destination, destinationOffset, source, sourceOffset, length)`である。

operationのoperandはsource順に一度だけ評価する。count、capacity、range末尾、stride、allocation byte数をtargetで表現できない場合と
allocationに失敗した場合はtrapする。stride 0でもcountとrange末尾のoverflowはtrapする。

Bufferをfunction parameter、result、aggregate field、closure capture、通常のgeneric argument、別のBufferのelementに置ける。
`get`は格納されたcarrierを返し、`put`、`fill`、`copy`が上書きしたcarrierは以後そのplaceから到達できない。handle elementでは
carrierの複製が同じreferentへのauthorityを保存し、referentを複製しない。

## Symbol conversion

```text
*Buffer<UInt8> -> Symbol
*Symbol        -> Buffer<UInt8>
```

`*buffer`は変換時点のbytesを持つimmutableなSymbol snapshotを返す。以後のBuffer変更はresultを変更しない。`*symbol`は同じbytesで
初期化した変更可能なBufferを返し、Symbolは変更されない。実装はcopy-on-writeでstorageを共有してよいが、source-levelのaliasingと
immutabilityを変えてはならない。operandはconsumeされず、変換後も利用できる。

## 未検査precondition

| Operation | Precondition |
|---|---|
| `buffer.get(index)`、`buffer.put(index, value)` | `index < #buffer` |
| `buffer.fill(offset, length, value)` | `offset <= #buffer` |
| `destination.copy(destinationOffset, source, sourceOffset, length)` | `destinationOffset <= #destination`かつ`sourceOffset + length <= #source` |

precondition違反時の結果は保証しない。extern Cがruntime carrier、count、owner、element responsibilityを破壊した後のBuffer operationも
同様に結果を保証しない。
