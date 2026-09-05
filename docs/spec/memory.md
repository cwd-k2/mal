# memory primitive

Status: Current v0.5 profile

## `Ptr`

`Ptr`は型なしのmutable data addressである。値はcopyableであり、複製しても指すstorageのlifetimeを
延長しない。pointer literal、null、equality、integerとの変換はない。mal programは`Ptr`を`extern`から
受け取るか、`offset`の結果として得る。

`Ptr`はlength、allocation identity、ownershipを保持しない。各`extern` contractは、返すpointerが指す
live region、読み書きの可否、lifetime、およびstorageを無効にするoperationを定める。同じpointerのaliasは
同じstorageを観測する。allocationとdeallocationはpredefined primitiveではない。

## primitive

v0.5の最小operation集合は次である。

```text
offset      :: (Ptr, UInt64) -> Ptr
loadInt64   :: Ptr -> Int64
storeInt64  :: (Ptr, Int64) -> Unit
loadUInt8   :: Ptr -> UInt8
storeUInt8  :: (Ptr, UInt8) -> Unit
```

これらはpredefined scopeにあるdirect-call-only primitiveであり、first-class function valueとして参照できない。
引数は通常のcallと同じく左から右へ一度ずつ評価する。

`offset(pointer, bytes)`は同じlive region内で`bytes`だけ後方のaddressを返す。region末尾の直後を指す値は
作れるがload/storeには使えない。targetのaddress計算で`bytes`を表現できなければtrapする。regionの外へ
移動するoffsetはcontract違反である。

load/storeは指定scalarの全byteを対象とし、alignmentを要求しない。`storeInt64`の後に同じaddressから
`loadInt64`すると、間に同じbytesへのwriteがなければ元の値を得る。`UInt8`にも同じ規則を適用する。
異なるscalar operationで同じbytesを観測した場合のbyte orderとrepresentationはbackend host ABIが定める。

必要byte数がlive regionに収まらない、read不可のregionをloadする、write不可のregionをstoreする、または
lifetime終了後にaccessするprogramはcontract違反であり、trapを含む特定の結果を保証しない。boundsを
deterministically検査するには、programがlengthを別のscalarとして保持し、access前に検査する。

product、sum、String、external opaque type、function、`Ptr`自体を直接load/storeするprimitiveはない。
aggregateはscalar fieldを個別に読み、既存のconstructorでmal valueとして組み立てる。

## minimality

この機能はcollection、allocator、bounds policyを追加せず、indexed storageに共通するmechanismだけを提供する。
採択理由と競技programによる評価は[D022](../design/decisions.md#d022-型なしptrをmemory-primitiveのbaselineとする)に記録する。
