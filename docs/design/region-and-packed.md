# `Region`と`Packed`によるmemory transferの試案

Status: Discussion draft (2026-09-17)

この文書は同じlayoutを持つexternal location列`Region<A>`と、mal-ownedなimmutable sequence `Packed<A>`の間のtransfer、
partial I/O、host ABIとの境界を定める。shape、stride、placement、alignment、未検査memory accessは
[`Address`、target size、layout、placementの試案](size-and-alignment.md)を正とする。現行の規範は
[`Symbol`](../spec/symbols.md)、[`memory primitive`](../spec/memory.md)、[`extern`](../spec/extern.md)、
[`C host ABI`](../spec/c-host-abi.md)であり、この試案だけを根拠にsource、ABI、compilerを変更しない。

## Authority

`Region<A>`はAddressとCountを持つexternal location列であり、storageのinitialization、permission、allocation identity、ownership、
lifetimeを取得しない。read capacity、write capacity、初期化済み範囲のどれであるかは、それを構成したprogramまたはhost contractが
定める。

`Packed<A>`はmal-ownedなflat immutable有限要素列とCountである。external layout、mutable storage、external resourceのlifetimeは
持たない。runtimeはbacking bufferを必要な間保持し、最後のviewを失った後に解放する。`A`が`Address`を含む場合も、Packed bufferの
lifetimeだけをmalが支配し、各Addressのreferentのlifetimeは延長しない。

両型は`Representable(A)`の場合だけwell-formedである。この条件によりPacked elementはmal-managed ownerを含まず、要素ごとの
retain/releaseを必要としない。

## Operation

```text
<-Region<A>                 -> Packed<A>
Region<A> <- Packed<A>      -> Region<A>
Region<A> / Count           -> Region<A>
Region<A> % Count           -> Region<A>
Packed<A> / Count           -> Packed<A>
Packed<A> % Count           -> Packed<A>
#Region<A>                  -> Count
#Packed<A>                  -> Count
Packed<A> # Count           -> A
*Packed<UInt8>              -> Symbol
*Symbol                     -> Packed<UInt8>
```

これらを含むexpression全体の結合順序とformatter規則は
[`generic memory surface syntax`](memory-syntax.md)を正とする。

`#region`はlocation数だけを返し、readability、writability、initializationを示さない。`#packed`は常に読み出せる実在要素数を返す。

`<-region`は全要素をindex順にadmitし、新しいPackedを返す。preconditionを満たした後のallocation failureはtrapする。
`region <- packed`は先頭からobserveし、書いた範囲の直後から始まるsuffix Regionを返す。zero-count Regionのadmissionはempty
Packed、zero-count Packedのstoreは元のRegionを返し、storageをdereferenceしない。`Packed<Unit>`はbacking element storageを
持たずCountだけで表現してよい。

`value / count`はcount要素のprefix、`value % count`はその直後のremainderを返す。Regionはstorageをdereferenceせずviewだけを
分ける。Packedは同じbuffer owner、offset、countを持つslice viewを返す。operandは通常のexpressionと同じ順で一度だけ評価する。

```mal
packed := <-region;
written := outputRegion / #packed;
available := outputRegion <- packed;
```

## Symbol

`Symbol`は`Packed<UInt8>`のaliasにせず、既存のmanaged byte valueとhost ABIを維持する。prefix `*`はこの二型の間だけのclosed
conversion familyであり、pointer dereferenceや任意のgeneric conversionではない。

`*packed`は同じbytesのSymbol、`*symbol`は同じbytesのflat `Packed<UInt8>`を返す。実装はcopyまたはowner共有を選べる。
Symbolが非flatなら`*symbol`はflat bufferをmaterializeする。operandとresultはどちらも変換後に有効であり、必要なallocationの
failureはtrapする。external storageのzero-copy Packed viewは最初のprofileへ入れない。

## Host allocationとpartial I/O

external storageのallocation、failure、ownership、deallocationはpredefined primitiveにしない。layout shapeをhost ABIへ渡さず、
host operationは`Address`、`ByteSize`、`Count`、またはoperation固有のconcrete contractを受け取る。

```mal
extent := elementCount * #u64;
raw := hostAllocateBytes(extent);
region := raw@u64@elementCount;
```

alignmentを保証しないallocatorへexact extentだけを要求してから`!`を適用してはならない。余剰storageとdeallocation用の元Addressを
必要とするaligned allocationはconcrete host operationへ閉じ込める。

partial inputではhostがcapacity以下のCountと、そのprefixを初期化したというpostconditionを返す。mal wrapperは同じRegionを
prefixとremainderへ分け、保持する範囲だけをadmitする。

```mal
scratch := inputAddress@u8@capacity;
readCount := hostReadRaw(?scratch, #scratch);
valid := scratch / readCount;
unused := scratch % readCount;
saved :: Symbol := *(<-valid);
```

outputも同じ分割を使う。host writeはresultがinput Count以下であり、先頭result要素を消費したというpostconditionを持つ。

```mal
pending :: Packed<UInt8> := *saved;
scratch := outputAddress@u8@capacity;
transferCount := chooseTransferCount(#pending, #scratch);
current := pending / transferCount;
nextPending := pending % transferCount;
written := scratch / transferCount;
available := scratch <- current;
sentCount := hostWriteRaw(?written, #written);
retryCurrent := current % sentCount;
```

`retryCurrent`を`nextPending`より先に処理することで、未消費要素の順序を保つ。

`0count`をEOF、would-block、空入力、zero progressのどれとするかはoperation固有のcontractが定める。languageはretry、loop、
rollbackを行わない。progressとfailureを同時に返すoperationはhost-mappableなconcrete productまたはsumにCountとstatusを明示する。

## 未検査precondition

| Operation | Precondition |
|---|---|
| `Region / Count`、`Region % Count` | `count <= #region`かつprefix extentがoverflowしない |
| `Packed / Count`、`Packed % Count` | `count <= #packed` |
| `Packed # Count` | `index < #packed` |
| `<-Region<A>` | 全locationがreadable、初期化済み、valid representationで、allocation sizeがtargetで表現可能 |
| `Region<A> <- Packed<A>` | `#packed <= #region`で対象prefixがwritable |
| host partial input | result Countがcapacity以下で、そのprefixが初期化済み |
| host partial output | result Countがinput以下で、そのprefixがconsumed済み |

primitiveはこれらを検査せず、違反時の結果を保証しない。host postcondition違反もtrusted host contractへの違反である。実装が
防御的にtrapしてもsource-levelの保証にはしない。`Region<Unit>`と`Packed<Unit>`はelement storageを持たないため、readability、
writability、initialization、storage extentのpreconditionを要素へ要求しない。

## Host ABI

compilerは閉じた`HostMappable(A)` judgmentを持つ。最初のprofileでは`Unit`、`Bool`、numeric scalar、`Address`、`ByteSize`、
`Count`、`Symbol`、external opaque typeをbase caseとし、全要素がhost-mappableなproductとsumへ再帰適用する。transparent aliasは
展開後の型で判定し、source spellingはgenerated C名として保存してよい。

function、`Cursor<A>`、`Region<A>`、`Packed<A>`は、型argumentがconcreteでもhost-mappableにしない。generic aliasはconcreteな
type argumentを代入して完全に展開した後に判定し、built-in indexed typeが残れば拒否する。generic bindingとspecializationを
public C symbolやheader declarationへ出さない。

```mal
extern readRaw :: (Address, Count) -> Count;       // valid
extern readRegion :: Region<UInt8> -> Count;       // error
extern emitPacked :: Packed<UInt8> -> Unit;        // error
```

mal runtime representation、source memory layout、public C carrierは独立である。aggregateをhostへ渡すgenerated adapterはfield、tag、
active payloadを再帰的に変換し、C structをsource memory layoutとしてreinterpretしない。Addressが指すstorageのlayout、Count、
alignment、permission、lifetimeはoperation固有のhost contractに残す。

## 採択時に検証すること

- Region/Packedのzero-count、one-past、slice owner、Unit、Address elementを含むownership corpus
- 各preconditionのpositive/negative caseと、防御的trapをcontractにしないtest方針
- partial I/Oのinitialized/consumed prefixを表すhost contract例
