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
lifetimeを型で証明せず、各operationに必要な条件はcallerまたはhost contractのpreconditionとして残る。実装がそれらをruntimeで
検査することも、この代数を低水準primitiveとして隠し、検査済みconstructorやより強い型から成るabstractionを上に作ることも
妨げない。ここで分離するのは操作の意味と単位である。

## 概念モデル

| 概念 | 保持する意味 | 保持しない意味 |
|---|---|---|
| `ByteSize` | targetがobject sizeとbyte offsetとして扱えるunsigned量 | pointer representation、要素数、alignment保証 |
| `Count` | target上の有限collectionの要素数とindexに使うdimensionlessなunsigned量 | byte量、pointer representation |
| `Address` | 外部storageのopaqueなlocation | numeric value、型、length、access permission、ownership、alignment保証 |
| layout shape | 一要素のrepresentation、size/stride、required alignmentを表す`@`または`#`の構文operand | source type、runtime payload、storage、lifetime、具体的なaddress |
| `Cursor<A>` | runtimeの`Address`とstaticなlayout shape | bounds、initialization、permission、ownership、lifetime、型としてのalignment保証 |
| `Region<A>` | `Cursor<A>`へ`Count`を適用した有限個の要素location | initialization、permission、allocation identity、ownership、lifetime延長 |
| `Packed<A>` | mal-ownedなimmutable有限要素列と要素数 | external layout、contiguous storageの保証、mutable storage、host resourceのlifetime |

layout shapeは通常のvalueでも`Layout<A>`というsource typeでもない。`@`によるplacementまたは`#`によるsize queryの構文operandであり、
nameへbindingしたり、parameter、result、product field、sum payload、closure capture、extern argumentとして渡したりしない。
compilerはshapeをtarget固有のconstantへ解決し、ANF、LLVM IR、public host ABIへlayout descriptorを渡さない。

`Cursor<A>`と`Region<A>`はruntime carrierを持つ通常の型付きvalueである。layout shapeは型検査後のspecializationに残る
static identityであり、runtimeではCursorがAddress、RegionがAddressとCountを運べばよい。最初のprofileでは一つの`A`に
一つのartifact-local layoutだけを認める。

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

一要素productと一要素sumはsource typeと同様に存在しない。`unit` shapeのsizeとalignment、empty sum shape `[]`を認めるか、
transparent aliasをshapeの`Alias`で展開できるかは採択前に固定する。

`#shape`は一要素のstrideを`ByteSize`で返すtarget constantである。

```mal
#i8
#(u64, i32)
```

product layoutはsource orderのfield offset、内部padding、tail padding、全体alignmentを決める。sum layoutはtag representation、
payload offset、最大payload extent、tail padding、全体alignmentを決める。正確なtag、invalid representation、paddingの
admissionとobservationは採択前に固定する。

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
直後に収まることと、byte extentを表す乗算がoverflowしないことは現時点ではprecondition候補である。

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
Cursor<A>@align             -> Cursor<A>
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

`Address@Shape`はlocationを動かさずにshapeを適用し、固定したlayout identityを持つCursorを作る。Cursorへ別のshapeを
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

`Cursor<A>@align`は現在位置からshapeのrequired alignmentを満たす最初のlocationまで進め、同じlayout identityのCursorを返す。
これはalignment assertionではなく実際のalign-upである。すでにalignedならlocationを変えない。新しいstorage、permission、
ownership、lifetimeは作らず、skipするpaddingと後続のaccessに必要なextentが元のlive regionへ収まることをpreconditionとする。
suffixは左から右へ適用するため、`address@u64@align@0count`もalign-upした後にcount 0のRegionを作る。

```mal
cursor := address@u64;
region := address@u64@count;
alignedCursor := address@u64@align;
alignedRegion := address@u64@align@count;
```

Cursorと一要素Regionは同一視しない。

```mal
address@i8          // Cursor<Int8>
address@i8@1count   // Region<Int8>
```

`@align`はAddressやlayout shapeには直接適用できず、shapeが確定したCursorにだけ適用できる。

```mal
address@align     // error
```

`@` suffixはapplicationより弱く、`<-`より強く結合する候補とする。countに複合式を置く場合は括弧で境界を明示する。

```mal
region := p@(u64, i32)@count;
dynamic := p@u8@(requested - consumed);
```

## Cursor accessとalignment

`Cursor<A>`はstaticなlayout identityを持つため、genericなload/storeが型`A`だけからrepresentationを探索する必要はない。
loadは現在位置を進めず、storeは同じlayoutのstrideだけ進んだCursorを返す。

```mal
value := <-(p@u64);
next := p@u64 <- value;

alignedValue := <-(p@u64@align);
alignedNext := p@u64@align <- value;
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

compilerはsource typeとは別に、各Address、Cursor、Region expressionが保証する最小alignmentをfactとして追跡する候補である。
exact Cursorでは保証がなければalignment 1、`@align`のresultではshapeのrequired alignmentを使う。storeはstrideが
required alignmentの倍数であるためalignment factを保存する。joinまたはcall boundaryで
保証を保存できなければ保守的な値へ弱める。過大なLLVM alignmentを指定してはならない。

`@align`の実装はopaque pointer capabilityとtargetのprovenance規則に依存する。任意のAddressをportableにalign-upできないtargetでは
このsuffixを提供できない可能性があり、最初のprofileから外してhost contract由来のalignment evidenceだけを使う案も残す。

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

`/`と`%`はRegionとPackedを同じ境界でprefixとremainderへ分ける。`n = min(#value, count)`として、prefixのcountは`n`、
remainderのcountは`#value - n`になる。Regionではstorageをdereferenceせずviewだけを分け、Packedではobservableな要素列を保った
viewとしてよい。

`Packed<A>`自体にexternal layoutを暗黙に与えない。external storageへ戻すときは同じelement shapeからcapacity Regionを作る。

```mal
region := address@u8@count;
bytes := <-region;
byte := bytes # index;

output := outputAddress@u8@capacity;
current := bytes / #output;
pending := bytes % #output;
available := output <- current;
```

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

alignmentを保証しないallocatorへexact extentだけを要求してから`@align`を適用してはならない。一般には最大padding分の余剰storageと
deallocation用の元Addressが必要であり、alignmentを数値として公開しないprofileではconcreteなhost adapterへ閉じ込める。

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
current := pending / #scratch;
nextPending := pending % #scratch;
written := scratch / #current;
available := scratch <- current;
sentCount := hostWrite(written);
sent := written / sentCount;
retry := written % sentCount;
```

hostがcapacityを超えるcountを返した場合、zero progress、partial operation後のfailure、adapterによるruntime検査の範囲は未決である。

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

makeAlignedRegion<A> :: (Cursor<A>, Count) -> Region<A> :=
    (cursor, count) -> cursor@align@count;
```

CursorまたはRegionを作れること自体が`A`にartifact-local layoutがある証拠になる。specialization後にはconcreteなshapeが確定する。
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

- `#value`と`#shape`を区別するgrammar、transparent alias、`unit`、`[]`、layoutを持てる型の閉じた集合
- product field order、padding、tail paddingと、sum tag、payload、invalid representationの規則
- byte extentの乗算overflow、Region store overflow、Packed indexingをtrap、checked result、contract違反のどれにするか
- `ByteSize`と`Count`のarithmetic、literal range、target非依存`check`とのphase境界
- `.i8`などのpostfix conversionと現行`T(value)`、`value[T]`の移行範囲
- `Cursor`、`Region`、`Packed`をsource signatureとpublic C ABIへ書ける範囲
- RegionとPackedの`/`と`%`、empty value、evaluation order、cleanup、allocation failure
- `Symbol`を`Packed<UInt8>`へした場合のliteral、`+`、equality、`#`、indexing、C ABI名
- partial I/Oのzero progress、capacity超過、不正なconsumed count、途中failure
- `@align`を提供できるtargetと、alignment factをbinding、branch join、call boundaryで保存または弱める規則
- 現行`Ptr`から`Address`へのsource名とC ABI型名、および既存memory primitiveからの移行範囲
