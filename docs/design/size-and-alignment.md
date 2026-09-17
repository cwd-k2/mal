# `Address`、`ByteSize`、`Count`、layout、memory placement、`Packed`の試案

Status: Discussion draft (2026-09-17)

この文書はtarget依存のbyte量、値のmemory representation、alignmentを満たすplacement、およびhostが提供する
storageとの境界を一つの候補として整理する。現行の規範は[`types`](../spec/types.md)、
[`Symbol`](../spec/symbols.md)、[`memory primitive`](../spec/memory.md)、[`program`](../spec/programs.md)、
[`C host ABI`](../spec/c-host-abi.md)を正とし、この試案だけを根拠にsource、ABI、compilerを変更しない。
型parameterそのものの範囲は
[`parametric polymorphismと型index付きprimitiveの試案`](parametric-polymorphism.md)に置く。

## 目的

現行仕様はstorage sizeとaddress offsetを`UInt64`で表し、scalar load/storeを型ごとの名前付きfunctionとして提供する。
すべてのload/storeはalignmentを要求せず、productとsumにはcanonicalなexternal memory representationを定めない。
この試案は次の摩擦をまとめて扱う。

- targetのaddress計算に使うbyte量を固定幅の`UInt64`から分離する。
- size、alignment、padding、sum tagを型ごとに手計算せず、明示したlayout shapeへ閉じ込める。
- exactなaddressの解釈と、次のaligned addressへの移動を区別する。
- layoutを通常のruntime valueにせず、配置済みの`Cursor`と`Region`をgeneric codeへ渡す。
- allocation policyを言語が所有せず、host operationにはbyte量またはconcrete operation固有のcontractだけを公開する。

この代数自体はmemory safetyを提供しない。`Address`、`Cursor`、`Region`はbounds、initialization、permission、ownership、
lifetimeを型で証明せず、各operationに必要な条件はcallerまたはhost contractのpreconditionとして残る。primitiveは
preconditionをruntimeで検査せず、違反時の結果を保証しない。検査済みconstructorやより強い型から成るabstractionを上に作ることは
妨げない。ここで分離するのは操作の意味と単位である。

## 概念モデル

| 概念 | 保持する意味 | 保持しない意味 |
|---|---|---|
| `ByteSize` | targetがobject sizeとbyte offsetとして扱えるunsigned量 | pointer representation、要素数、alignment保証 |
| `Count` | target上の有限collectionの要素数とindexに使うdimensionlessなunsigned量 | byte量、pointer representation |
| `Address` | 外部storageのopaqueなlocation | numeric value、型、length、access permission、ownership、alignment保証 |
| layout shape | 一要素のrepresentation、size/stride、required alignmentを表す`@`または`#`の構文operand | source type、runtime payload、storage、lifetime、具体的なaddress |
| `Cursor<A>` | runtimeの`Address`と、canonicalな`A`に一意なstatic layout | bounds、initialization、permission、ownership、lifetime、型としてのalignment保証 |
| `Region<A>` | `Cursor<A>`へ`Count`を適用した有限個の要素location | initialization、permission、allocation identity、ownership、lifetime延長 |
| `Packed<A>` | mal-ownedなimmutable有限要素列と要素数 | external layout、contiguous storageの保証、mutable storage、host resourceのlifetime |

layout shapeは通常のvalueでも`Layout<A>`というsource typeでもない。`@`によるplacementまたは`#`によるsize queryの構文operandであり、
nameへbindingしたり、parameter、result、product field、sum payload、closure capture、extern argumentとして渡したりしない。
compilerはshapeをtarget固有のconstantへ解決し、ANF、LLVM IR、public host ABIへlayout descriptorを渡さない。

`Cursor<A>`と`Region<A>`はruntime carrierを持つ通常の型付きvalueである。layout shapeはcanonicalなsource typeへ解決し、
一つのcanonical `A`とtargetの組には一つのartifact-local layoutだけを認める。transparent aliasは展開後の型へ正規化する。
runtimeではCursorがAddress、RegionがAddressとCountを運べばよい。

compilerはsourceに公開しない閉じた`Representable(A)` judgmentを持ち、layoutを持てる型を判定する。layout shapeから作った
`Cursor<A>`または`Region<A>`を受け取ることは、このjudgmentのもとで`A`をaccessするために必要なstatic informationを与えるが、
そのAddressに有効なrepresentationが実在することは証明しない。

最初のprofileでは`Unit`、全numeric scalar、`Address`、`ByteSize`、`Count`をrepresentableとする。全fieldがrepresentableなproductと、
全variantがrepresentableな二項以上のsumも再帰的にrepresentableとする。transparent aliasは展開後の型で判定するため、`Bool`は
`[Unit, Unit]`と同じrepresentableな型になる。function、external opaque type、`Packed<A>`、`Cursor<A>`、`Region<A>`、empty sumは
representableにしない。

`Address`はcopyableなopaque locationであり、numeric scalarではない。`address + bytes`と`address - bytes`はinteger値の
加減算ではなく、同じstorage capabilityからbyte単位のlocationを派生させるoperationである。address literal、null、equality、
`Address`同士の加減算、integerとの相互変換は提供しない。

## Layout shape

layout shapeは`@`または`#`の直後だけに置き、通常のtype expressionやruntime valueから区別する。

```text
unit
i8
u32
f64
address
bytesize
count
```

productとsumはsource typeと同じdelimiterの内側にlayout shapeを書き、flatなn項構造とnested構造をそのまま区別する。
`&`と`|`によるlayout compositionは設けない。

```text
(i8, u64, i32)          // (Int8, UInt64, Int32)
((i8, u64), i32)        // ((Int8, UInt64), Int32)
(i8, (u64, i32))        // (Int8, (UInt64, Int32))

[unit, i32, address]     // [Unit, Int32, Address]
[[unit, i32], address]   // [[Unit, Int32], Address]
[unit, [i32, address]]   // [Unit, [Int32, Address]]
```

一要素productと一要素sumはsource typeと同様に存在しない。empty sum shape `[]`は認めない。transparent alias `Alias`は
canonical typeへ展開し、その型がrepresentableならshapeとして使える。predefined alias `Bool`と`[unit, unit]`は同じcanonical
typeとlayoutを表し、Bool専用のmemory representationは設けない。

`#shape`は一要素のstrideを`ByteSize`で返すtarget constantである。

```mal
#i8
#(u64, i32)
#Bool
```

### primitiveと`Unit`

numeric scalar、`Address`、`ByteSize`、`Count`のstrideとrequired alignmentはtarget data layoutから決める。numeric scalarの
storage幅はその型のbit幅、`Address`はdefault address spaceのpointer storage幅、`ByteSize`と`Count`はpointer index幅を使う。
異なるscalar shapeで同じbytesを観測した場合のbyte orderとrepresentationは対応するbackend host ABIが定める。
Addressをloadする場合、storageが以前のAddress storeまたはhost contractによる有効なpointer representationを保持することを
preconditionとし、primitiveは検査しない。

`Unit`はstride 0、required alignment 1とする。Unitのloadとstoreはstorageをdereferenceせず、UnitをstoreしたCursorは同じlocationを
返す。`Region<Unit>`はstorageを消費せずに任意のCountを持てる。

### product

productはfieldをsource orderのまま配置する。先頭fieldのoffsetを0とし、後続fieldのoffsetは直前fieldの末尾からそのfieldのrequired
alignmentまで前方へ丸める。product全体のrequired alignmentは全fieldの最大値、strideは最後のfieldの末尾から全体alignmentまで
前方へ丸めた値とする。これにより内部paddingとtail paddingを含み、nested productはflattenせず各layoutを再帰的に適用する。

### sum

sumは0-basedのvariant indexを持つtag、padding、全variantで共有するpayload領域の順に配置する。tagにはvariant数を表せる最小の
builtin unsigned型を使う。

```text
2 .. 2^8 variants          UInt8
2^8 + 1 .. 2^16 variants  UInt16
2^16 + 1 .. 2^32 variants UInt32
2^32 + 1 .. 2^64 variants UInt64
```

`2^64`を超えるvariantを持つsumは拒否する。payloadのrequired alignmentは全variantの最大値とし、payload offsetはtagの末尾から
そのalignmentまで前方へ丸める。payload extentは全variantのstrideの最大値、sum全体のrequired alignmentはtagと全variantの
最大値、sumのstrideはpayload末尾から全体alignmentまで前方へ丸めた値とする。loadするtagがvariant数未満であることは
callerのpreconditionであり、primitiveは検査しない。

### paddingと非選択payload

storeはproduct field、sum tag、選択したpayloadだけを書き、内部padding、tail padding、非選択payloadの残存bytesを変更しなくてよい。
loadはpaddingと非選択payloadを読まず、paddingを含むbytewise equalityも提供しない。

layoutはopaqueなtarget layoutであり、同じcompiled artifactと対応host adapterの範囲で有効とする。file、network、
永続storageには別の明示的codecを使い、このlayoutをstable wire formatとして扱わない。public C ABIのby-value aggregate
representationとも同一視せず、必要な変換はgenerated adapterが所有する。

## `ByteSize`と`Count`

`ByteSize`をdefault address spaceでobject sizeとbyte offsetを表すtarget依存のunsigned numeric type、`Count`を有限collectionの
要素数とindexを表すtarget依存のunsigned numeric typeとする。両者のruntime representationは同じでもsource typeを分ける。

```text
source type           ByteSize                    Count
literal suffix        bytes                       count
example               10bytes                     10count
public C ABI type     mal_ByteSize_t               mal_Count_t
C representation     size_t                       size_t
LLVM representation  pointer index幅のinteger     pointer index幅のinteger
```

`ByteSize`と`Count`はliteral、同じ型同士の加減算と比較、明示的numeric conversionの対象にする。`Count`にはdimensionlessな
unsigned integerとして乗除算とremainderも認める。address offsetと各process argumentのbyte lengthには`ByteSize`を使い、
Region、Packed、Symbolの要素数とindex、およびprocess argument countには`Count`を使う。

```mal
p + count * #u64
p - count * #u64
p + 8 * #u64
```

`Count * ByteSize`と`ByteSize * Count`はbyte extentを返すdimension付きの閉じたprimitiveとし、`ByteSize * ByteSize`は認めない。
これにより`8 * #u64`のsuffixなしliteralは周辺型から`Count`に一意に決まる。addressの派生が同じlive region内または末尾の
直後に収まることと、byte extentを表す乗算がoverflowしないことはpreconditionであり、このmemory primitiveは検査しない。

### postfix numeric conversion

layoutの`@`とは別に、numeric valueへの`.i8`、`.u32`、`.f64`などを明示的numeric conversionの候補とする。
これは一般のfield accessやmember lookupではなく、仕様が列挙するnumeric destinationだけを持つ閉じたpostfix familyである。

```mal
n := readCount();
byte := n.u8;
offset := (base + delta).bytes;
ratio := value.f64;
```

通常のnumeric literalには既存の型suffixを使う。

```mal
100u8
1.5f32
```

literalにconversionを適用すること自体は禁じない。integer conversionがmodulo規則を持つ場合、範囲外literalとの違いを
明示できるためである。

```mal
300u8          // range error
(300i64).u8    // modulo conversion
```

`.bytes`と`.count`を`ByteSize`、`Count`へのconversion spellingにするかは未決である。この試案を採択する場合は、現行の
`T(value)`と`value[T]`へ第三の表記を追加せず、postfix conversionへ置き換える範囲を同時に決める。

## Placement

placementとaccessの型関係は次になる。`Shape`は`@`または`#`の直後に置くlayout shapeを表す。

```text
Address + ByteSize          -> Address
Address - ByteSize          -> Address
!Cursor<A>                  -> Address
!Region<A>                  -> Address

Address@Shape               -> Cursor<A>
alignForward(Cursor<A>)      -> Cursor<A>
Cursor<A>@Count             -> Region<A>

Cursor<A> <- A              -> Cursor<A>
<-Cursor<A>                 -> A

<-Region<A>                 -> Packed<A>
Region<A> <- Packed<A>      -> Region<A>
Region<A> / Count           -> Region<A>
Region<A> % Count           -> Region<A>
Packed<A> / Count           -> Packed<A>
Packed<A> % Count           -> Packed<A>
#Region<A>                  -> Count
#Packed<A>                  -> Count
Packed<A> # Count           -> A
```

`Address@Shape`はlocationを動かさずにshapeを適用し、canonicalな`A`に一意なlayoutを持つCursorを作る。Cursorへ別のshapeを
適用してはならない。異なるlayoutへ切り替えるprogramはprefix `!`で現在位置のAddressへ明示的に戻してから、新しいshapeを適用する。

```mal
cursor := address@u8;
sameCursor := cursor@u64;       // error
sameCursor := cursor <- @u64;   // error
nextCursor := (!cursor)@u64;
```

prefix `!`はCursorからlayout、Regionからlayoutとcountをsource-level valueとして捨て、同じlocationのAddressを返す
forgetful projectionである。

```text
!(address@i32)        == address
!(address@i32@count)  == address
```

`alignForward`は採択するsource spellingではなく、alignment移動operatorの意味を表すmetanotationである。実際のoperatorには
locationが前方へ動き得ることが分かる、`@`とは別のspellingを選ぶ。

`alignForward(cursor)`は現在位置から`A`のrequired alignmentを満たす最初のlocationまで進め、同じcanonical layoutのCursorを返す。
これはalignment assertionではなく実際のalign-upである。すでにalignedならlocationを変えない。新しいstorage、permission、
ownership、lifetimeは作らず、skipするpaddingと後続のaccessに必要なextentが元のlive regionへ収まることをpreconditionとする。
alignment移動後にcount 0のRegionを作る場合も、先にlocationをalign-upする。

```mal
cursor := address@u64;
region := address@u64@count;
alignedCursor := alignForward(address@u64);       // metanotation
alignedRegion := alignForward(address@u64)@count; // metanotation
```

Cursorと一要素Regionは同一視しない。

```mal
address@i8          // Cursor<Int8>
address@i8@1count   // Region<Int8>
```

alignment移動はAddressやlayout shapeには直接適用できず、shapeが確定したCursorにだけ適用できる。

```mal
alignForward(address) // error; metanotation
```

`@` suffixはapplicationより弱く、`<-`より強く結合する候補とする。countに複合式を置く場合は括弧で境界を明示する。

```mal
region := p@(u64, i32)@count;
dynamic := p@u8@(requested - consumed);
```

## Cursor accessとalignment

`Cursor<A>`はcanonicalな`A`に一意なstatic layoutを持つため、genericなload/storeが型`A`だけからrepresentationを探索する必要はない。
loadは現在位置を進めず、storeは同じlayoutのstrideだけ進んだCursorを返す。

```mal
value := <-(p@u64);
next := p@u64 <- value;

alignedValue := <-alignForward(p@u64);        // metanotation
alignedNext := alignForward(p@u64) <- value;  // metanotation
```

同じCursorからのstore chainは`A`だけを受け取り、各stepで同じstrideだけ進む。heterogeneousな値を一単位として扱う場合は
aggregate shapeを使う。逐次codecとして異なるlayoutへ切り替える場合は、各境界でAddressへ戻す。

```mal
end := p@u8
    <- first
    <- second
    <- third;

entryEnd := p@(u8, u64) <- (header, payload);

afterHeader := p@u8 <- header;
afterVersion := (!afterHeader)@i32 <- version;
end := (!afterVersion)@address <- payloadAddress;
```

すべてのCursor accessはunaligned accessを認め、alignmentを意味上のpreconditionにしない。backendはalignment保証がなければ
alignment 1のload/storeまたは同等のbyte accessへlowerする。compilerは最適化のために各expressionが保証する最小alignmentを
factとして追跡してよい。alignment移動のresultでは`A`のrequired alignmentを使え、storeはstrideがrequired alignmentの倍数で
あるためこのfactを保存できる。joinまたはcall boundaryでfactを保存できなければalignment 1へ弱める。これは性能だけに影響し、
programの意味を変えない。過大なLLVM alignmentを指定してはならない。

alignment移動の実装はopaque pointer capabilityとtargetのprovenance規則に依存する。最初のprofileでは、default address spaceの
Addressを整数化してprovenanceを保ったままalign-upできるtargetだけにこのoperatorを提供する。non-integral pointerなど、この操作を
定義できないtargetではalignment移動operatorを提供しないが、unaligned Cursor accessは引き続き利用できる。

## RegionとPacked

`Region<A>`のcountは、そのregion valueが覆う要素locationの数である。capacity regionでは書き込み可能量を、partial read後の
regionでは初期化済みで読み出せる量を表し得るため、`#region`だけからinitializationやpermissionは分からない。それらはregionを
作ったoperationのcontractに属する。`#packed`はmal-ownedな列に実在する要素数であり、全要素を常に読み出せる。

`<-region`はregion全体をindex順にadmitして`Packed<A>`を作る。`region <- packed`は`#packed <= #region`をpreconditionとして
先頭からobserveし、書いた範囲の直後から始まるsuffix Regionを返す。zero-count regionのadmissionはempty packed、zero-count
packedのstoreは元と同じregionを返し、storageをdereferenceしない。

```mal
packed := <-region;
written := outputRegion / #packed;
available := outputRegion <- packed;

#written == #packed
#available == #outputRegion - #packed
```

`/`と`%`は`count <= #value`をpreconditionとして、RegionとPackedを同じ境界でprefixとremainderへ分ける。prefixのcountは
`count`、remainderのcountは`#value - count`になる。operatorは範囲を検査せず、超過時のsaturatingやclampingも行わない。
Regionではstorageをdereferenceせずviewだけを分け、Packedではobservableな要素列を保ったviewとしてよい。

Packed indexingの`index < #packed`と、Region storeの`#packed <= #region`もcallerのpreconditionであり、primitiveは検査しない。

`Packed<A>`自体にexternal layoutを暗黙に与えない。external storageへ戻すときは同じelement shapeからcapacity Regionを作る。

```mal
region := address@u8@count;
bytes := <-region;
byte := bytes # index;

output := outputAddress@u8@capacity;
transferCount := chooseTransferCount(#bytes, #output);
current := bytes / transferCount;
pending := bytes % transferCount;
available := output <- current;
```

この例の`chooseTransferCount`は`transferCount <= #bytes`かつ`transferCount <= #output`を満たすprogram側の処理を表し、
predefined operationではない。

`Symbol`は`Packed<UInt8>`のtransparent aliasとする候補を維持する。Packedは物理的に連続したstorageを意味せず、literal、flat leaf、
slice、Ropeなどをobservableなsequenceを変えない限り選べる。zero-copy external viewはowner-bearing viewまたはborrow規則が必要になるため、
最初のprofileへ入れない。

## Host allocationとpartial I/O

allocation、failure、ownership、deallocationはpredefined primitiveにしない。layout shapeはhost ABIへ渡さず、host operationは
`ByteSize`、`Count`、またはoperation固有のconcrete contractを受け取る。

```mal
extent := count * #u64;
raw := hostAllocateBytes(extent);
region := raw@u64@count;
```

alignmentを保証しないallocatorへexact extentだけを要求してからalignment移動を適用してはならない。一般には最大padding分の
余剰storageとdeallocation用の元Addressが必要であり、alignmentを数値として公開しないprofileではconcreteなhost adapterへ閉じ込める。

partial I/Oではcapacity Regionをhostへ渡し、返されたCountでinitialized prefixとunused suffixを分ける。

```mal
capacity := 1024count;
scratch := inputAddress@u8@capacity;
readCount := hostRead(scratch);
valid := scratch / readCount;
unused := scratch % readCount;
saved :: Symbol := <-valid;
```

output側も同じsplitを使う。

```mal
pending :: Packed<UInt8> := saved;
scratch := outputAddress@u8@capacity;
transferCount := chooseTransferCount(#pending, #scratch);
current := pending / transferCount;
nextPending := pending % transferCount;
written := scratch / transferCount;
available := scratch <- current;
sentCount := hostWrite(written);
sent := written / sentCount;
retry := written % sentCount;
```

hostが返すcountはcapacity以下でなければならず、これもhost contractのpreconditionとする。zero progressとpartial operation後の
failureをcallerがどう処理するかはoperation固有のcontractに残す。

## Genericsとの境界

generic codeは裸のAddressと型parameter`A`だけからlayoutを導けない。layout shapeは通常のvalueではないため、旧案の
`Layout<A>` parameterも設けない。callerが具体的なshapeからCursorまたはRegionを構成し、それをgeneric functionへ渡す。

```mal
readCursor<A> :: Cursor<A> -> A :=
    (cursor) -> <-cursor;

writeCursor<A> :: (Cursor<A>, A) -> Cursor<A> :=
    (cursor, value) -> cursor <- value;

makeRegion<A> :: (Cursor<A>, Count) -> Region<A> :=
    (cursor, count) -> cursor@count;
```

CursorまたはRegionを受け取ること自体が、`A`にartifact-local layoutがあるというstaticな条件を満たす。これは参照先storageの
有効性を示すものではない。specialization後にはconcreteなshapeが確定する。
複数representation、runtime layout descriptor、implicit dictionary、type reflectionは最初のprofileへ入れない。generic codeが
必要なextentはcallerが`Count * #shape`で計算するか、具体的なhost operationのcontractに閉じ込める。

## CとRustとの比較

| Concern | C | Rust | このmal試案 |
|---|---|---|---|
| byte量と要素数 | `size_t`を共用する | `usize`を共用する | `ByteSize`と`Count`をsourceで分ける |
| raw location | `void *` | raw pointer | numericに観測できない`Address` |
| finite external view | pointerとlength | slice | `Region<A>` |
| owned sequence | allocation固有 | `Vec<T>`など | immutableな`Packed<A>` |
| typed interpretation | `T *`への変換 | pointer cast | `Address@shape` |
| repeated placement | array typeまたはsize計算 | array、slice、allocator `Layout` | `Address@shape@Count`による`Region<A>` |
| aggregate representation | struct、union、enum | type layoutと`repr` | `Address@(...)`、`Address@[...]`によるartifact-local layout |
| unaligned access | `memcpy`など | `read_unaligned`、`write_unaligned` | exact placement由来のalignment 1 access |

Cのpointer conversion、alignment、allocation、`memcpy`の規則は
[`WG14 N1570`](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf)を参照する。Rustのtype layoutとraw pointer accessは
[`Rust Reference`](https://doc.rust-lang.org/reference/type-layout.html)、
[`std::alloc::Layout`](https://doc.rust-lang.org/std/alloc/struct.Layout.html)、
[`std::ptr`](https://doc.rust-lang.org/std/ptr/index.html)を参照する。この比較はCまたはRustのobject modelをmalへそのまま導入する
根拠ではなく、layout、placement、ownershipをどこで分離するかを確認するために使う。

## ABIとbackend

layout shapeはpublic ABIへ現れない。Cursor、Regionをconcrete extern signatureに認める場合、generated adapterが既知の
layout constantを使い、hostへ必要なpointer、Count、ByteSizeだけを公開する。openな型parameterはC headerへ公開しない。

LLVM backendはtarget data layoutからdefault address spaceのpointer representation幅、pointer index幅、各primitive representationの
ABI alignmentを別々に取得する。pointer representation幅とindex幅は一致するとは限らない。GEP operand、load/store alignment、
Cの`size_t`とLLVM module targetの整合をABI admissionで検証する。

現在のbackend内部にあるproduct、sum、Unit、Bool、closureのrepresentationはsource layoutではない。source layoutを採択する場合は、
target layout planとpublic C adapterの責務を明示し、既存internal representationを偶発的にsource contractへしない。

## 採択前に固定すること

- `#value`と`#shape`を区別するgrammar、およびtransparent alias shapeの解決規則
- `ByteSize`と`Count`のarithmetic、literal range、target非依存`check`とのphase境界
- `.i8`などのpostfix conversionと現行`T(value)`、`value[T]`の移行範囲
- `Cursor`、`Region`、`Packed`をsource signatureとpublic C ABIへ書ける範囲
- RegionとPackedの`/`と`%`、empty value、evaluation order、cleanup、allocation failure
- `Symbol`を`Packed<UInt8>`へした場合のliteral、`+`、equality、`#`、indexing、C ABI名
- partial I/Oのzero progressと途中failure
- alignment移動operatorのspelling、target profileのadmission検査、および最適化用alignment factの保存規則
- 現行`Ptr`から`Address`へのsource名とC ABI型名、および既存memory primitiveからの移行範囲
