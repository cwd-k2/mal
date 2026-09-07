# memory primitive

Status: Current v0.5 profile

## `Ptr`

`Ptr`は型なしのmutable data addressである。値はcopyableであり、複製しても指すstorageのlifetimeを
延長しない。pointer literal、null、equality、integerとの変換はない。mal programは`Ptr`を`extern`から
受け取るか、pointerに対する`+`または`-`の結果として得る。

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

この値はmemory上のcanonical表現だけを測る。pointerに対する`+`と`-`の右operandの単位もbyteであるため、
field offsetは`@T`の和として記述できる。

v0.5では`Unit`、`Symbol`、product、sum、external opaque type、functionにcanonical memory表現を定めず、これらへの
`@`をcompile-time errorとする。特にproductとsumはC ABI上の表現を持っていても、そのpaddingやbackend内部の
layoutをsource-level memory contractにはしない。

## primitive

v0.5のoperation集合はbyte offset、全numeric scalarと`Ptr`のobject representation、および`Symbol`のbyte copyである。

```text
+           :: (Ptr, UInt64) -> Ptr
-           :: (Ptr, UInt64) -> Ptr
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
loadSymbol   :: (Ptr, UInt64) -> Symbol
storeSymbol  :: (Ptr, Symbol) -> Unit
```

pointerに対する`+`と`-`はbinary operatorである。load/storeはpredefined scopeにあるfirst-class functionであり、
binding、引数、resultとして扱える。直接callとfunction valueを介したcallは同じmemory operationを行う。
operand、callee、引数は通常のoperatorとcallの規則どおり左から右へ一度ずつ評価する。

これらはExtern-owned storageに関わるが、program固有の`extern` operationではない。言語がcanonical representationとの
変換を一度だけ定めるprimitiveであり、allocation、region、permission、lifetimeのpolicyは`Ptr`を提供する
`extern` contractに残す。productやsumなどcanonical memory representationを持たない型のcodecは、このprimitiveを
組み合わせたmal関数として記述するか、host固有の意味が必要な場合に型固有の`extern` contractとして定義する。
配置の判断規則は[authority policy](../design/authority.md#policyとmechanismを分ける)に定める。

`pointer + bytes`はaddressを`bytes`だけ大きい側へ、`pointer - bytes`は小さい側へ移動する。resultは同じlive region内、
またはregion末尾の直後でなければならない。末尾の直後を指す値は作れるがload/storeには使えない。targetの
address計算で`bytes`を表現できなければtrapする。regionの外へ移動するoffsetはcontract違反である。

load/storeは指定型の全byteを対象とし、alignmentを要求しない。`storeInt64`の後に同じaddressから
`loadInt64`すると、間に同じbytesへのwriteがなければ元の値を得る。他のnumeric scalarにも同じ規則を適用する。
異なるscalar operationで同じbytesを観測した場合のbyte orderとrepresentationはbackend host ABIが定める。

`storePtr`はdata addressのobject representationをstorageへcopyし、`loadPtr`はそれを`Ptr`として復元する。
`storePtr`の後に同じaddressから`loadPtr`すると、間に同じbytesへのwriteがなければ同じstorageを指す値を得る。
pointerの格納に必要なbyte数はtarget ABIが定め、格納されたpointerを複製しても指すstorageのlifetimeは延長しない。
`storePtr`またはhostが有効な`MalType_Ptr`として書いたものではないbytesを`loadPtr`するprogramはcontract違反である。

`loadSymbol(pointer, length)`は指定した外部regionの`length` bytesをcopyし、新しいmal-controlled `Symbol`を返す。
`length == 0`ではpointerをdereferenceしない。lengthをtarget allocation sizeで表現できない場合やallocation failureは
trapする。`storeSymbol(pointer, value)`は`value`の全bytesを外部regionへcopyし、descriptorやownershipは書き出さない。
したがって`Symbol`にはcanonical memory表現も`@Symbol`もない。

同じaddressへ`storeSymbol`した後、そのbyte lengthを指定して`loadSymbol`すれば、間にwriteがない限り同じbyte
sequenceを持つ別の`Symbol`を得る。このround tripはidentityやlifetimeの移動ではなく、二回のbyte copyである。

必要byte数がlive regionに収まらない、read不可のregionをloadする、write不可のregionをstoreする、または
lifetime終了後にaccessするprogramはcontract違反であり、trapを含む特定の結果を保証しない。boundsを
deterministically検査するには、programがlengthを別のscalarとして保持し、access前に検査する。

product、sum、external opaque type、functionを直接load/storeするprimitiveはない。
aggregateは対応するnumeric scalarまたは`Ptr` fieldを個別に読み、既存のconstructorでmal valueとして組み立てる。
host contractはexternal opaque typeに固有の保存・復元`extern`を別途提供できるが、それはpredefined memory
表現を追加しない。したがってその型への`@`は引き続きerrorであり、保存表現、復元したhandleの有効性、resource
lifetimeはhost contractの責務である。authorityの一般則は[EngramとExtern](engrams.md#authority)に定める。

## minimality

この機能はcollection、allocator、bounds policyを追加せず、indexed storage、pointer graph、Symbol fieldに共通するmechanismだけを提供する。
採択理由とlocal algorithm corpusによる評価は[D022](../design/decisions/D022.md)に記録する。
