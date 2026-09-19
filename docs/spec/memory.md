# external memory

Status: Accepted v0.6 profile

この文書はexternal storageのaddress、target依存量、canonical memory representation、typed view、access、未検査preconditionを
定める。有限regionとmal-owned sequenceのtransferは[`Region`と`Packed`](packed.md)、surface grammarは
[字句と文法](grammar.md)を正とする。

## 基本型

`Address`はordinary byte-addressable external storageのlocationを運ぶcopyableなcapabilityである。numeric value、null、
要素型、extent、permission、ownership、alignment保証を持たず、複製してもreferentのlifetimeを延長しない。
`Address + ByteSize`は同じstorage capabilityから前方のbyte位置を派生させる。Addressの減算、Address同士の演算、equality、
literal、integerとの変換はない。

`ByteSize`はtargetがobject sizeとbyte offsetに使うunsigned量、`USize`は有限collectionの要素数とindexに使うunsigned量である。
両型はdefault address spaceのpointer index幅を持つ別のsource typeであり、literal suffixは`bytes`と`usize`である。

```mal
extent :: ByteSize := 64bytes;
length :: USize := 8usize;
```

同じ型同士の加減算と比較、明示的numeric conversionを両型に認める。`USize`には乗除算とremainderも認める。
`USize * ByteSize`と`ByteSize * USize`は`ByteSize`、`ByteSize * ByteSize`はerrorである。加減乗算はtarget幅でwrapする。
divisionとremainderはdivisorがzeroでないことをpreconditionとする。memory extentとして使う数学的な積はoverflowしてはならない。

`Region<A>`はAddress、canonicalな`A`に一意なstatic layout、USize個のlocationを運ぶ。storageのinitialization、permission、
allocation identity、ownership、lifetimeを取得しない。

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

function、external opaque type、`Region<A>`、`Packed<A>`、`Buffer<A>`、empty sumはrepresentableでない。
transparent aliasは展開後に判定する。`Region<A>`、`Packed<A>`、`Buffer<A>`は`Representable(A)`の場合だけwell-formedである。
このjudgmentはstorageにvalidなrepresentationが実在することを証明しない。

## Layout shape

layout shapeは`#`によるstride queryだけが受け取るcompile-time構文operandである。binding、parameter、
result、field、capture、extern argumentとして運ばず、compilerがtarget constantへ解決する。shapeは次のclosed spellingからなる。

```text
unit
i8 i16 i32 i64
u8 u16 u32 u64
f32 f64
address bytesize usize
bool
```

`bool`はpredefined `Bool`のcanonical type `[Unit, Unit]`を表す唯一のshape aliasである。user-defined aliasと型identifierは
shapeに現れない。productとsumはsource typeと同じdelimiterを使い、flatなn項構造とnested構造を区別する。

```text
(i8, u64, i32)
((i8, u64), i32)
[unit, i32, address]
[unit, [i32, address]]
```

一要素product、一要素sum、empty sum shapeはない。`#shape`は一要素のstrideを`ByteSize`で返すtarget constantである。

## Canonical layout

numeric scalar、`Address`、`ByteSize`、`USize`のstrideとrequired alignmentはtarget data layoutから決める。numeric scalarの
storage幅はbit幅、Addressはdefault address spaceのpointer storage幅、ByteSizeとUSizeはpointer index幅を使う。byte orderと
scalar representationはbackend host ABIが定める。

`Unit`はstride 0、required alignment 1である。canonical layoutのstrideが0になる型はstorageをdereferenceせず、
Region、Packed、Bufferではlogical countだけを持つ。Unitだけからなるproductとtransparent aliasにも同じ規則を適用する。

productはfieldをsource orderに配置する。先頭offsetは0、後続offsetは直前fieldの末尾からそのfieldのrequired alignmentまで
前方へ丸める。全体alignmentは全fieldの最大値、strideは最後のfieldの末尾から全体alignmentまで前方へ丸める。
nested productはflattenしない。

sumは0-based variant indexのtag、padding、全variantで共有するpayload領域の順に配置する。tagはvariant数を表せる最小の
`UInt8`、`UInt16`、`UInt32`、`UInt64`を使い、`2^64`を超えるvariantを拒否する。payload alignmentは全variantの最大値、
payload offsetはtag末尾からそのalignmentまで丸める。payload extentは全variant strideの最大値、sum alignmentはtagと
全variantの最大値、sum strideはpayload末尾から全体alignmentまで丸める。

storeはproduct field、sum tag、選択payloadだけを書き、paddingと非選択payloadを変更しなくてよい。loadはそれらを読まない。
このlayoutは同じartifactと対応adapterの範囲だけで有効であり、mal runtime representation、public C aggregate carrier、
file、network、永続storageのformatではない。
reference C hostがこのlayoutを読み書きする場合は、public carrierをcastせず、[C host ABIのnamed alias helper](c-host-abi.md#canonical-memory-access)を使う。

## Address derivationとtyped view

```text
Address + ByteSize          -> Address
view<A>(Address, USize, USize, Region<A> -> R) -> R
Region<A>.get(USize)        -> A
Region<A>.put(USize, A)     -> Unit
```

`view`の二つのUSizeはelement単位の`offsetStart`と`offsetEnd`であり、半開区間`[offsetStart, offsetEnd)`を表す。
引数を通常のapplication順で一度ずつ評価した後にRegionを形成し、callbackを一度適用する。strideがnonzeroなら先頭は
`address + offsetStart * stride(A)`、zeroならAddressを派生または観測しない。どちらもRegionのlengthは
`offsetEnd - offsetStart`である。`view`はallocationせず、callback resultをそのまま返す。ここで`R`はintrinsicがcallbackから
決めるresult型を表し、Region、Buffer、またはこれらを再帰的に含んではならない。

Regionはsource-level constructorを持たず、`view`のcallback parameterとして導入する。Region parameterとその派生値には
[`Region`と`Packed`](packed.md#scoped-authority)のlexical escape制約を適用する。

`get`はindex位置の値をadmitし、`put`はindex位置へ値をobserveする。external storageをconsumeせず、referentのlifetimeを変更しない。
receiver-first表記は`get(region, index)`、`put(region, index, value)`と同じpredefined operation identityを指す。

```mal
address.view<Int64>(0usize, count, (region) -> {
    first := region.get(0usize);
    region.put(1usize, first);
});
```

Region accessは1より強いalignmentを仮定しない。backendはalignment 1のload/storeまたは同等のunaligned-safe accessへlowerする。
実Addressのalignmentを利用するfast pathはoptimizationとしてよいが、source-levelのalign-up、alignment assertion、数値queryはない。

## 未検査precondition

callerまたはAddressを提供したhost contractは次の条件を満たす。

| Operation | Precondition |
|---|---|
| `Address + ByteSize` | 数学的offsetがoverflowせず、resultが同じlive region内または末尾の直後にある |
| `view<A>(address, start, end, ...)` | `start <= end`である。strideがnonzeroならbyte offsetがoverflowせず、半開区間が同じlive region内にある。empty区間では先頭が末尾直後でもよい |
| `region.get(index)` | `index < #region`である。strideがnonzeroならoffsetがoverflowせず、locationがreadable、初期化済みでvalid representationを持つ |
| `region.put(index, value)` | `index < #region`である。strideがnonzeroならoffsetがoverflowせず、locationがwritableである |

primitiveはbounds、permission、initialization、lifetime、extent、overflow、Address representation、sum tagを検査しない。
precondition違反時の特定の結果を保証しない。実装が内部corruptionを避けるためにtrapしても、そのtrapはimplementation detailである。
preconditionを満たしたoperationがmal-owned storageを必要とし、allocationに失敗した場合はtrapする。

zero-stride elementはAddress referent、permission、initialization、storage extentを要求しない。`get`はbounds内で型の唯一の値を返し、
`put`はbounds内でstorageを変更しない。

## Target contract

backendはsource memory layout専用のtarget layout planを作り、runtime valueの内部layoutを再利用しない。default address spaceの
pointer representation幅、pointer index幅、primitive ABI alignmentをtarget data layoutから別々に取得する。layout計算をtargetの
object sizeで表現できない型はartifact生成時に拒否する。reference C backendのmappingは[C host ABI](c-host-abi.md)に定める。
