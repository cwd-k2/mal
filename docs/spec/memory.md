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

`T.size`は、型`T`の値をこの文書のload/store表現でmemoryへ置くために必要なbyte数を`UInt64`で返す。
host callを伴わないtarget constantであり、transparent aliasは展開して測る。

| `T` | `T.size` |
|---|---:|
| `Int8`, `UInt8` | 1 |
| `Int16`, `UInt16` | 2 |
| `Int32`, `UInt32`, `Float32` | 4 |
| `Int64`, `UInt64`, `Float64` | 8 |
| `Ptr` | target ABIの`MalType_Ptr` object representationのbyte数 |

この値はmemory上のcanonical表現だけを測る。pointerに対する`+`と`-`の右operandの単位もbyteであるため、
field offsetは`T.size`の和として記述できる。

v0.5では`Unit`、`Symbol`、product、sum、external opaque type、functionにcanonical memory表現を定めず、これらへの
`.size`をcompile-time errorとする。特にproductとsumはC ABI上の表現を持っていても、そのpaddingやbackend内部の
layoutをsource-level memory contractにはしない。

## primitive

v0.5のoperation集合はbyte offset、全numeric scalarと`Ptr`のobject representation、および`Symbol`のbyte copyである。

```text
+            :: (Ptr, UInt64) -> Ptr
-            :: (Ptr, UInt64) -> Ptr
T.size       :: UInt64
T.load       :: Ptr -> T
T.store      :: (Ptr, T) -> Unit
Symbol.read  :: (Ptr, UInt64) -> Symbol
Symbol.write :: (Ptr, Symbol) -> Unit
```

`T.size`、`T.load`、`T.store`の`T`には`Int8`、`Int16`、`Int32`、`Int64`、`UInt8`、`UInt16`、
`UInt32`、`UInt64`、`Float32`、`Float64`、`Ptr`を認める。transparent aliasで修飾した場合は、その
canonical typeがこの集合に含まれるかで判定する。`Symbol`はcanonical memory representationを持たないため
`.size`、`.load`、`.store`を持たず、raw byte copyの`.read`と`.write`だけを持つ。

pointerに対する`+`と`-`はbinary operatorである。load/store/read/writeは型で修飾したpredefined first-class functionであり、
binding、引数、resultとして扱える。直接callとfunction valueを介したcallは同じmemory operationを行う。
operand、callee、引数は通常のoperatorとcallの規則どおり左から右へ一度ずつ評価する。

これらはExtern-owned storageに関わるが、program固有の`extern` operationではない。言語がcanonical representationとの
変換を一度だけ定めるprimitiveであり、allocation、region、permission、lifetimeのpolicyは`Ptr`を提供する
`extern` contractに残す。productやsumなどcanonical memory representationを持たない型のcodecは、このprimitiveを
組み合わせたmal関数として記述するか、host固有の意味が必要な場合に型固有の`extern` contractとして定義する。
配置の判断規則は[authority policy](../design/authority.md#policyとmechanismを分ける)に定める。

`pointer + bytes`はaddressを`bytes`だけ大きい側へ、`pointer - bytes`は小さい側へ移動する。targetのaddress計算で
`bytes`を表現でき、resultが同じlive region内またはregion末尾の直後になることをpreconditionとする。末尾の直後を
指す値は作れるがload/storeには使えない。precondition違反は`Ptr`を供給したhost contractへの違反であり、特定の
実行結果を保証しない。

load/storeは指定型の全byteを対象とし、alignmentを要求しない。`Int64.store`の後に同じaddressから
`Int64.load`すると、間に同じbytesへのwriteがなければ元の値を得る。他のnumeric scalarにも同じ規則を適用する。
異なるscalar operationで同じbytesを観測した場合のbyte orderとrepresentationはbackend host ABIが定める。

`Ptr.store`はdata addressのobject representationをstorageへcopyし、`Ptr.load`はそれを`Ptr`として復元する。
`Ptr.store`の後に同じaddressから`Ptr.load`すると、間に同じbytesへのwriteがなければ同じstorageを指す値を得る。
pointerの格納に必要なbyte数はtarget ABIが定め、格納されたpointerを複製しても指すstorageのlifetimeは延長しない。
`Ptr.store`またはhostが有効な`MalType_Ptr`として書いたものではないbytesを`Ptr.load`するprogramはcontract違反である。

`Symbol.read(pointer, length)`は指定した外部regionの`length` bytesをcopyし、新しいmal-controlled `Symbol`を返す。
`length == 0`ではpointerをdereferenceしない。lengthをtarget allocation sizeで表現できない場合やallocation failureは
trapする。`Symbol.write(pointer, value)`は`value`の全bytesを外部regionへcopyし、descriptorやownershipは書き出さない。
このobservationはmal-owned storageを新しく構成せず、`Symbol`の内部表現を理由とするallocation failureを追加しない。
したがって`Symbol`にはcanonical memory表現も`Symbol.size`もない。

同じaddressへ`Symbol.write`した後、そのbyte lengthを指定して`Symbol.read`すれば、間にwriteがない限り同じbyte
sequenceを持つ別の`Symbol`を得る。このround tripはidentityやlifetimeの移動ではなく、二回のbyte copyである。

必要byte数がlive regionに収まらない、read不可のregionをloadする、write不可のregionをstoreする、または
lifetime終了後にaccessするprogramはcontract違反であり、trapを含む特定の結果を保証しない。boundsを
deterministically検査するには、programがlengthを別のscalarとして保持し、access前に検査する。

product、sum、external opaque type、functionを直接load/storeするprimitiveはない。
aggregateは対応するnumeric scalarまたは`Ptr` fieldを個別に読み、既存のconstructorでmal valueとして組み立てる。
host contractはexternal opaque typeに固有の保存・復元`extern`を別途提供できるが、それはpredefined memory
表現を追加しない。したがってその型への`.size`は引き続きerrorであり、保存表現、復元したhandleの有効性、resource
lifetimeはhost contractの責務である。authorityの一般則は[EngramとExtern](engrams.md#authority)に定める。

## minimality

この機能はcollection、allocator、bounds policyを追加せず、indexed storage、pointer graph、Symbol fieldに共通するmechanismだけを提供する。
機構の採択理由とlocal algorithm corpusによる評価は[D022](../history/decisions/D022.md)、source syntaxの理由は
[D037](../history/decisions/D037.md)に記録する。
