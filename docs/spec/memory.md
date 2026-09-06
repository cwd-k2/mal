# memory primitive

Status: Current v0.5 profile

## `Ptr`

`Ptr`は型なしのmutable data addressである。値はcopyableであり、複製しても指すstorageのlifetimeを
延長しない。pointer literal、null、equality、integerとの変換はない。mal programは`Ptr`を`extern`から
受け取るか、`offset`の結果として得る。

`Ptr`はlength、allocation identity、ownershipを保持しない。各`extern` contractは、返すpointerが指す
live region、読み書きの可否、lifetime、およびstorageを無効にするoperationを定める。同じpointerのaliasは
同じstorageを観測する。allocationとdeallocationはpredefined primitiveではない。

## storage 幅

`@T`は、型`T`の値をこの文書のload/store表現でmemoryへ置くために必要なbyte数を`UInt64`で返す。
host callを伴わないtarget constantであり、transparent aliasは展開して測る。

| `T` | `@T` |
|---|---:|
| `Int8`, `UInt8` | 1 |
| `Int16`, `UInt16` | 2 |
| `Int32`, `UInt32`, `Float32` | 4 |
| `Int64`, `UInt64`, `Float64` | 8 |
| `Ptr` | target ABIの`MalType_Ptr` object representationのbyte数 |
| `Engram` | `@Ptr + 8` |

この値はmemory上のcanonical表現だけを測り、Engramが参照するbytes、allocation metadata、C backend内部の
struct paddingは含めない。`offset`の単位もbyteであるため、field offsetは`@T`の和として記述できる。

v0.5では`Unit`、product、sum、external opaque type、functionにcanonical memory表現を定めず、これらへの
`@`をcompile-time errorとする。特にproductとsumはC ABI上の表現を持っていても、そのpaddingやbackend内部の
layoutをsource-level memory contractにはしない。

## primitive

v0.5のoperation集合はbyte offsetと、全numeric scalar、`Ptr`、およびEngram descriptorに対する型別load/storeである。

```text
offset      :: (Ptr, UInt64) -> Ptr
loadInt8    :: Ptr -> Int8
storeInt8   :: (Ptr, Int8) -> Unit
loadInt16   :: Ptr -> Int16
storeInt16  :: (Ptr, Int16) -> Unit
loadInt32   :: Ptr -> Int32
storeInt32  :: (Ptr, Int32) -> Unit
loadInt64   :: Ptr -> Int64
storeInt64  :: (Ptr, Int64) -> Unit
loadUInt8   :: Ptr -> UInt8
storeUInt8  :: (Ptr, UInt8) -> Unit
loadUInt16  :: Ptr -> UInt16
storeUInt16 :: (Ptr, UInt16) -> Unit
loadUInt32  :: Ptr -> UInt32
storeUInt32 :: (Ptr, UInt32) -> Unit
loadUInt64  :: Ptr -> UInt64
storeUInt64 :: (Ptr, UInt64) -> Unit
loadFloat32 :: Ptr -> Float32
storeFloat32 :: (Ptr, Float32) -> Unit
loadFloat64 :: Ptr -> Float64
storeFloat64 :: (Ptr, Float64) -> Unit
loadPtr      :: Ptr -> Ptr
storePtr     :: (Ptr, Ptr) -> Unit
loadEngram   :: Ptr -> Engram
storeEngram  :: (Ptr, Engram) -> Unit
```

これらはpredefined scopeにあるdirect-call-only primitiveであり、first-class function valueとして参照できない。
引数は通常のcallと同じく左から右へ一度ずつ評価する。

`offset(pointer, bytes)`は同じlive region内で`bytes`だけ後方のaddressを返す。region末尾の直後を指す値は
作れるがload/storeには使えない。targetのaddress計算で`bytes`を表現できなければtrapする。regionの外へ
移動するoffsetはcontract違反である。

load/storeは指定型の全byteを対象とし、alignmentを要求しない。`storeInt64`の後に同じaddressから
`loadInt64`すると、間に同じbytesへのwriteがなければ元の値を得る。他のnumeric scalarにも同じ規則を適用する。
異なるscalar operationで同じbytesを観測した場合のbyte orderとrepresentationはbackend host ABIが定める。

`storePtr`はdata addressのobject representationをstorageへcopyし、`loadPtr`はそれを`Ptr`として復元する。
`storePtr`の後に同じaddressから`loadPtr`すると、間に同じbytesへのwriteがなければ同じstorageを指す値を得る。
pointerの格納に必要なbyte数はtarget ABIが定め、格納されたpointerを複製しても指すstorageのlifetimeは延長しない。
`storePtr`またはhostが有効な`MalType_Ptr`として書いたものではないbytesを`loadPtr`するprogramはcontract違反である。

`storeEngram`はEngram descriptorをstorageへcopyし、Engramのbytes自体はcopyしない。storage表現は、
`storePtr`が用いるpointer表現、その直後の`storeUInt64`が用いるlength表現の順でpaddingなしに並べる。
したがって必要byte数はtarget ABIのpointer格納byte数に8を加えた値である。`storeEngram`の後に同じaddressから
`loadEngram`すると、間に同じbytesへのwriteがなければ同じbyte sequenceを持つEngramを得る。descriptorの
複製はEngram bytesのprogram-lifetimeを変更せず、bytesをmutableにしない。

`storeEngram`またはhostが既存の有効なmal Engramから上記storage表現で書いたものではないbytesを`loadEngram`する
programはcontract違反である。特にpointerはprogram終了まで有効で変更されないmal-ownedまたはliteralのEngram bytesを
指し、lengthはそのlive region内に収まらなければならない。

必要byte数がlive regionに収まらない、read不可のregionをloadする、write不可のregionをstoreする、または
lifetime終了後にaccessするprogramはcontract違反であり、trapを含む特定の結果を保証しない。boundsを
deterministically検査するには、programがlengthを別のscalarとして保持し、access前に検査する。

product、sum、external opaque type、functionを直接load/storeするprimitiveはない。
aggregateは対応するnumeric scalarまたは`Ptr` fieldを個別に読み、既存のconstructorでmal valueとして組み立てる。

## minimality

この機能はcollection、allocator、bounds policyを追加せず、indexed storage、pointer graph、Engram fieldに共通するmechanismだけを提供する。
採択理由とlocal algorithm corpusによる評価は[D022](../design/decisions.md#d022-型なしptrをmemory-primitiveのbaselineとする)に記録する。
