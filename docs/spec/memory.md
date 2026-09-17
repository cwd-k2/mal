# external memory

Status: Accepted v0.6 profile

この文書はexternal storageのaddress、target依存量、canonical memory representation、placement、access、未検査preconditionを
定める。有限regionとmal-owned sequenceのtransferは[`Region`と`Packed`](packed.md)、surface grammarは
[字句と文法](grammar.md)を正とする。

## 基本型

`Address`はordinary byte-addressable external storageのlocationを運ぶcopyableなcapabilityである。numeric value、null、
要素型、extent、permission、ownership、alignment保証を持たず、複製してもreferentのlifetimeを延長しない。
`Address + ByteSize`と`Address - ByteSize`は同じstorage capabilityからbyte位置を派生させる。Address同士の演算、equality、
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

`Cursor<A>`はAddressとcanonicalな`A`に一意なstatic layoutを運ぶ。`Region<A>`は同じlayoutを持つUSize個のlocationを運ぶ。
どちらもstorageのinitialization、permission、allocation identity、ownership、lifetimeを取得しない。

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

function、external opaque type、`Cursor<A>`、`Region<A>`、`Packed<A>`、empty sumはrepresentableでない。
transparent aliasは展開後に判定する。`Cursor<A>`、`Region<A>`、`Packed<A>`は`Representable(A)`の場合だけwell-formedである。
このjudgmentはstorageにvalidなrepresentationが実在することを証明しない。

## Layout shape

layout shapeは`@`によるplacementまたは`#`によるstride queryだけが受け取るcompile-time構文operandである。binding、parameter、
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

`Unit`はstride 0、required alignment 1である。load/storeはstorageをdereferenceせず、次Cursorは同じlocationになる。
`Region<Unit>`はstorageを消費せず任意のUSizeを持てる。

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

## Placementとaccess

```text
Address + ByteSize          -> Address
Address - ByteSize          -> Address
Address@Shape               -> Cursor<A>
Cursor<A>@USize             -> Region<A>
?Cursor<A>                  -> Address
?Region<A>                  -> Address
Cursor<A>!                  -> Cursor<A>
Region<A>!                  -> Region<A>
Cursor<A> <- A              -> Cursor<A>
<-Cursor<A>                 -> (A, Cursor<A>)
```

`Address@Shape`はlocationを動かさず、shapeに対応するcanonical `A`のCursorを作る。別shapeへ切り替える場合は`?cursor`で
Addressへ戻す。`Cursor<A>@USize`は現在locationからstrideを繰り返すRegionを作る。Cursorと一要素Regionは別の型である。

loadは現在位置の値とstrideだけ進んだCursorのproductを返し、storeも同じ次Cursorを返す。どちらもexternal storageを
consumeせず、referentのlifetimeを変更しない。store結果を使わない場合は通常のexpression statementとして捨てられ、
`_ := cursor <- value;`と明示する必要はない。結果破棄の一般則は[expression statement](expressions.md#expression-statement)に定める。

```mal
(value, next) := <-address@u64;
(<-address@u64)[(value, next) -> use(value, next)]

end := address@u8
    <- first
    <- second;
```

exact Cursor accessはunaligned accessを認める。backendは保証されたalignmentがなければalignment 1のload/storeまたは同等の
byte accessへlowerする。postfix `!`は現在位置から`A`のrequired alignmentを満たす最初のlocationへのalign-upである。
RegionではUSizeを保存し、USize 0でもlocationをalign-upする。exact placementは全backendのbaseline、`!`はpointer provenanceを
保って実装できるintegral-pointer targetだけのcapabilityとし、未対応targetは`!`を使うartifactをsource diagnosticで拒否する。
alignmentの数値queryはない。

## 未検査precondition

callerまたはAddressを提供したhost contractは次の条件を満たす。

| Operation | Precondition |
|---|---|
| `Address +/- ByteSize` | 数学的offsetがoverflowせず、resultが同じlive region内または末尾の直後にある |
| `Address@Shape` | なし |
| `Cursor@USize` | `USize * stride(A)`がoverflowせず、全locationが同じlive region内にある。USize 0またはstride 0では先頭が末尾の直後でもよい |
| Cursor load | 現在の一要素がreadable、初期化済みでvalid representationを持つ |
| Cursor store | 現在の一要素がwritableである |
| load/store result | 次locationが同じlive region内または末尾の直後にある |
| `Cursor<A>!` | skipするpaddingと一要素分のextentが同じlive regionに収まる |
| `Region<A>!` | skipするpaddingとUSize要素分のextentが同じlive regionに収まる。USize 0ではpaddingだけを対象とする |

primitiveはbounds、permission、initialization、lifetime、extent、overflow、Address representation、sum tagを検査しない。
precondition違反時の特定の結果を保証しない。実装が内部corruptionを避けるためにtrapしても、そのtrapはimplementation detailである。
preconditionを満たしたoperationがmal-owned storageを必要とし、allocationに失敗した場合はtrapする。

末尾の直後を指すCursorは保持、Addressへの投影、USize 0のRegion形成に使える。通常のload/storeには使えない。
Unit accessと`Region<Unit>`はpermission、initialization、storage extentを要素へ要求しない。

## Target contract

backendはsource memory layout専用のtarget layout planを作り、runtime valueの内部layoutを再利用しない。default address spaceの
pointer representation幅、pointer index幅、primitive ABI alignmentをtarget data layoutから別々に取得する。layout計算をtargetの
object sizeで表現できない型はartifact生成時に拒否する。reference C backendのmappingは[C host ABI](c-host-abi.md)に定める。
