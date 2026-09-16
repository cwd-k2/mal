# `Address`、`ByteSize`、`Count`、`Layout`、memory placement、`Packed`の試案

Status: Discussion draft (2026-09-16)

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

- targetのaddress計算に使う量を固定幅の`UInt64`から分離する。
- size、alignment、padding、sum tagを型ごとに手計算せず、opaqueなplacement ruleへ閉じ込める。
- exactなaddressの解釈と、次のaligned addressへの移動を区別する。
- memory representation ruleを明示的に渡したgeneric codeだけにparametricなload/storeを認める。
- allocation policyを言語が所有せず、host operationにはbyte量またはconcrete operation固有のcontractだけを公開する。

この代数自体はmemory safetyを提供する仕組みではない。`Address`、`Cursor`、`Region`はbounds、initialization、permission、ownership、
lifetimeを型で証明せず、各operationに必要な条件はcallerまたはhost contractのpreconditionとして残る。実装がそれらをruntimeで
検査することも、この代数を低水準primitiveとして隠し、検査済みconstructorやより強い型から成るsafe abstractionを上に作ることも
妨げない。ここで分離するのは操作の意味と単位であり、安全性の強制ではない。

## 概念モデル

この試案では次の概念を区別する。`Layout`、`Span`、`Cursor`、`Region`、`Packed`は内部表現をsourceから分解できない型index付きの
built-in opaque valueとする候補である。

| 概念 | 保持する意味 | 保持しない意味 |
|---|---|---|
| `ByteSize` | targetがobject sizeとbyte offsetとして扱えるunsigned量 | pointer representation、要素数、alignment保証 |
| `Count` | target上の有限collectionの要素数とindexに使うdimensionlessなunsigned量 | byte量、pointer representation |
| `Address` | 外部storageのopaqueなlocation | numeric value、型、length、access permission、ownership、alignment保証 |
| `Layout<A>` | `A`一個のcanonical representation、admission、observation、size/stride、required alignmentを表すcompile-time singleton | runtime payload、storage、lifetime、具体的なaddress |
| `Span<A>` | staticな`Layout<A>`、runtimeの要素数、全byte extent、base alignment | storage、ownership、host-visibleなlayout descriptor |
| `Cursor<A>` | 現在の`Address`と一つの`Layout<A>` | bounds、initialization、permission、ownership、lifetime、型としてのalignment保証 |
| `Region<A>` | `Address`へ`Span<A>`を適用した有限個の要素location | initialization、permission、allocation identity、ownership、lifetime延長 |
| `Packed<A>` | mal-ownedなimmutable有限要素列と要素数 | external layout、contiguous storageの保証、mutable storage、host resourceのlifetime |

`Address`はcopyableなopaque locationであり、numeric scalarではない。`address + bytes`と`address - bytes`はinteger値の
加減算ではなく、同じstorage capabilityからbyte単位のlocationを派生させるoperationである。address literal、null、equality、
`Address`同士の加減算、integerとの相互変換は提供しない。backendはこれをLLVM `ptr`やCの`void *`で運べるが、その表現を
sourceから観測できるとは限らない。

`Layout<A>`はsource上では通常のimmutable valueと同じsyntaxでbinding、parameter、resultに使える。ただし最初のprofileでは
一つの`A`にcanonicalなlayoutを一つだけ認めるstatic representation valueであり、runtime payloadとobservable identityを持たない。
compilerはlayout literalと`&`、`|`で構成したidentityをtarget固有のconstantへ解決し、specialization後のANF、LLVM IR、
public host ABIへlayout descriptorを渡さない。hostはlayoutを構成、分解、比較、保持できない。

`Cursor<A>`はstorageや所有者ではなく、現在位置へ`A`のrepresentation ruleを適用したmemory cursorである。書き込みは
同じlayoutのstrideだけ進んだ新しい`Cursor<A>`を返し、元のcursorとaddressを無効にしない。alignment factはcursorの
runtime fieldやsource typeではなく、cursor expressionについてcompilerが別に追跡する。raw `Address`には値のreadまたは
write規則を適用できず、先にlayoutを適用してcursorを作る必要がある。

`Region<A>`のcountは、そのregion valueが覆う要素locationの数である。capacityとして渡されたregionでは書き込み可能量を、
partial read後に返されたregionでは初期化済みで読み出せる量を表すため、`#region`だけからinitializationやpermissionは分からない。
それらはregionを作ったoperationのcontractに属する。これに対して`#packed`はmal-ownedな列に実在する要素数であり、packedの全要素は
常に読み出せる。したがって未初期化のcapacity regionはzero-filledとは限らず、そこからのloadはreadability preconditionを
満たさない。

alignmentは独立したsource valueを必須にせず、`Address`と`Layout`の関係として扱う。

```text
aligned(address, layout)
    iff addressが示すtarget locationがlayoutのrequired alignmentを満たす
```

`p <~ layout`、layout-awareなhost contract、およびalignedなcursorからのstride単位の移動は、この関係を
compilerが利用できる形で確立または保存する。`Cursor<A>`と`Region<A>`を作っても、元の`Address`を供給したcontractが
定めるregion、permission、lifetimeは増えない。

## Source notation

候補となるoperatorを役割ごとにまとめる。

| Notation | Result | Meaning |
|---|---|---|
| `#layout` | `ByteSize` | 一要素のbyte extent/strideを観測する |
| `layout * count` | `Span<A>` | 同じlayoutを`count`個並べる |
| `#span` | `ByteSize` | span全体のbyte extentを観測する |
| `left & right` | `Layout<(A, B)>` | product layoutを構成する |
| `left \| right` | `Layout<[A, B]>` | sum layoutを構成する |
| `address <- layout` | `Cursor<A>` | locationを動かさずlayoutを適用する |
| `cursor <- layout`、`region <- layout` | `Cursor<B>` | 現在位置またはregionのbaseへ別のlayoutを適用する |
| `address <- span`、`cursor <- span`、`region <- span` | `Region<A>` | placement位置を動かさず有限spanを適用する |
| `address <~ placement`、`cursor <~ placement`、`region <~ placement` | `Cursor<A>`または`Region<A>` | placement位置から次のrequired alignmentを満たす位置まで進めてlayoutまたはspanを適用する |
| `<-cursor` | `A` | cursorの現在位置から値を読む |
| `cursor <- value` | `Cursor<A>` | 値を書き、同じlayoutのstrideだけ進む |
| `<-region` | `Packed<A>` | region全体を先頭からadmitし、managedなimmutable sequenceを作る |
| `region <- packed` | `Region<A>` | packedをbaseからobserveし、直後から始まるremaining regionを返す |
| `value / count` | operandと同じfinite型 | 先頭から最大`count`要素のprefixを返す |
| `value % count` | operandと同じfinite型 | prefixを除いたremainderを返す |
| `#region` | `Count` | regionが表す要素数を観測する |
| `#packed` | `Count` | packedの要素数を観測する |
| `packed # index` | `A` | packedの要素を選ぶ |
| `address + bytes`、`address - bytes` | `Address` | byte単位でlocationを派生させる |

型の関係だけを取り出すと、memory placementとaccessの代数は次になる。ここでleft operandの`Cursor<B>`にある`B`は、
切替先の`A`と同じである必要はない。

```text
Layout<A> * Count         -> Span<A>

Address + ByteSize        -> Address
Address - ByteSize        -> Address

Address <- Layout<A>      -> Cursor<A>
Cursor<B> <- Layout<A>    -> Cursor<A>
Address <~ Layout<A>      -> Cursor<A>
Cursor<B> <~ Layout<A>    -> Cursor<A>

Address <- Span<A>        -> Region<A>
Cursor<B> <- Span<A>      -> Region<A>
Region<B> <- Span<A>      -> Region<A>
Address <~ Span<A>        -> Region<A>
Cursor<B> <~ Span<A>      -> Region<A>
Region<B> <~ Span<A>      -> Region<A>

Region<B> <- Layout<A>    -> Cursor<A>
Region<B> <~ Layout<A>    -> Cursor<A>

Cursor<A> <- A            -> Cursor<A>
<-Cursor<A>               -> A

<-Region<A>               -> Packed<A>
Region<A> <- Packed<A>    -> Region<A>
Region<A> / Count         -> Region<A>
Region<A> % Count         -> Region<A>
Packed<A> / Count         -> Packed<A>
Packed<A> % Count         -> Packed<A>
#Region<A>                -> Count
#Packed<A>                -> Count
Packed<A> # Count         -> A
```

placement operatorのleft operandは`Address`、`Cursor`、または`Region`であり、それぞれ次の位置を使う。

```text
placement(Address)   = address自身の位置
placement(Cursor<A>) = cursorの現在位置
placement(Region<A>) = regionのbase
```

`left <- placement`は現在位置へexact placementし、`left <~ placement`は現在位置からalign-upしてplacementする。
Regionをleft operandにする場合も終端ではなく常にbaseを使う。region storeは書き込み直後から始まるsuffix Regionを返すため、
そのresultのbaseが次のplacement位置になる。専用のend operationは設けない。

```mal
remaining := (address <- u8 * headerLength) <- headerPacked;
items := remaining <~ itemLayout * itemCount;
```

`items`は`headerPacked`を書き終えた位置からalign-upしたregionである。Regionからのplacementはbase locationを使うだけで、
新しいplacementがそのRegionへ収まることを型で保証しない。そのextentとpermissionは他のraw placementと同じくpreconditionである。

`Region<A>`のcontent operationは常にbaseから始める。`<-region`はregion全体をindex順にadmitしてmal-ownedな`Packed<A>`を作る。
storeは`#packed <= #region`をpreconditionとしてpackedの全要素を先頭からobserveし、書いた範囲の直後から始まるsuffix Regionを
返す。元のregionとpackedはimmutableなままであり、resultは新しいstorageを作らない。

```mal
packed := <-region;
written := outputRegion / #packed;
available := outputRegion <- packed;

#written == #packed
#available == #outputRegion - #packed
#written + #available == #outputRegion
written == outputRegion / #packed
available == outputRegion % #packed
```

`written`は元のbaseを持ち、`available`はその直後から始まる。store resultへ次のpackedを書けばremaining suffixを順に消費できる。

```mal
result := outputRegion <- firstPacked <- secondPacked <- thirdPacked;
```

各stepでは`#nextPacked <= #available`をpreconditionとする。zero-count regionのadmissionはempty packedを返し、zero-count packedの
storeは元のregionと同じbaseとcountを持つregionを返す。どちらもstorageをdereferenceしない。Countの減算はfitを比較した後にだけ行う。

`/`と`%`は`Region<A>`と`Packed<A>`を同じ境界で分ける。`value`をどちらかのfinite value、
`n = min(#value, count)`とすると次を満たす。

```text
#(value / count) == n
#(value % count) == #value - n
```

`value / count`は元のbaseまたは先頭要素から始まり、`value % count`はその直後から始まる。`count == 0count`ではempty prefixと
元のvalue、`count >= #value`では元のvalueとempty remainderを返す。どちらもcopyせずに表せるが、Packedのsource semanticsでは
元の要素列と同じ値を持つ独立したfinite valueとして扱う。Regionの`/`と`%`はstorageをdereferenceせずviewだけを分ける。
empty remainderのbaseは元のregionのone-past locationになり、そのregionをleft operandとする次のplacementはそこから始まる。

capacityを超えるpackedはstore前に同じ境界で分けられる。

```mal
current := pending / #region;
nextPending := pending % #region;
written := region / #current;
available := region <- current;
```

`Region<A>`から一要素のcursorを直接得る`#` operationは設けない。external regionの要素を個別に扱う場合は全体をpackedへ
admitして`packed # index`で選ぶか、元の`Address`からbyte offsetとlayoutを明示してcursorを作る。`packed # index`は
`index < #packed`をpreconditionとする。Regionにはbinary `#`を定義せず、個別のexternal element accessが必要なら元の`Address`から
byte offsetとlayoutを明示してcursorを作る。

`#`は既存の`#symbol`と`symbol # index`を、operand型ごとの閉じたprimitive familyへ拡張する。prefix形は、`Symbol`を含む
`Packed`と`Region`の要素数を`Count`、`Layout`と`Span`のbyte extentを`ByteSize`で観測する。binary形は`Packed`の
indexingに使い、引き続きnon-associativeとする。`Symbol`が`Packed<UInt8>`のaliasになるため、`#symbol`も`Count`を返す。
`*`はnumeric multiplicationに加えて、`Layout<A>`と`Count`から`Span<A>`を作る閉じたprimitiveとする。operand orderは
`Layout<A> * Count`だけを認め、`Count * Layout<A>`は提供しない。`^`は既存のinteger bit XORだけに使う。
`/`と`%`はnumeric division/remainderに加えて、`Region<A>`または`Packed<A>`と`Count`からprefix/remainderを作る閉じたprimitiveとする。

`<~`とbinary `<-`はmemory chain用の同じ最下位precedenceに置き、left-associativeとする候補を採る。既存の`*`と
binary `#`はそれらより強く結合するため、次は`p <- (u64 * count)`を表す。

```mal
p <- u64 * count
```

したがって通常の算術をstoreする場合も右辺を括弧で囲む必要はない。

```mal
end := p <- i32 <- a + 1 <- b * 2;
```

これは`(((p <- i32) <- (a + 1)) <- (b * 2))`として評価する。各operandは通常のoperator規則どおり左から右へ
一度ずつ評価する。prefix `<-cursor`と`<-region`はadmissionであり、binary chainとは別にoperandの位置を進めない。
前者は一つの`A`、後者はregion全体からmanagedな`Packed<A>`を作る。

`<<`は整数のbit shiftとして既に存在し、一定量のshiftと誤読できる。align-upが加える量はaddressごとに変わるため、
この試案ではplacementに`<~`を使い、`<<`をoverloadしない。

## `ByteSize`と`Count`

byte量を要素数と同じ型で表す`Size`という初期案は採らない。`ByteSize`をdefault address spaceでobject sizeと
byte offsetを表すtarget依存のunsigned numeric type、`Count`を有限collectionの要素数とindexを表すtarget依存の
unsigned numeric typeとする。両者のruntime representationは同じでもsource typeを分ける。

```text
source type           ByteSize                    Count
literal suffix        bytes                       count
example               10bytes                     10count
public C ABI type     mal_ByteSize_t               mal_Count_t
C representation     size_t                       size_t
LLVM representation  pointer index幅のinteger     pointer index幅のinteger
```

`ByteSize`と`Count`はliteral、同じ型同士の算術と比較、明示的conversionの対象にする。`10bytes`と`10count`がtargetの
範囲に収まらなければcompile-time errorとする。suffixのないliteralは`layout * 10`のように周辺型から`Count`へ決まる場合に
省略できる。`Count * ByteSize`と`ByteSize * Count`だけはbyte extentを返すdimension付きの閉じたprimitiveとする。

address offsetと各process argumentのbyte lengthには`ByteSize`を使う。spanとregionの要素数、`Packed`と`Symbol`のlengthとindex、
process argument countには`Count`を使う。Symbol全体のbyte extentが必要なら`#(u8 * #symbol)`で`ByteSize`へ変換する。
file format、network protocol、hashなど固定幅自体に意味がある値には
引き続き`UInt32`や`UInt64`を使う。CとRustは`size_t`または`usize`をbyte量と要素数の両方へ使うが、この試案では
source上の単位を区別し、ABI representationだけを共有する。

従来どおりbyte arithmeticを記述できる。

```mal
p + count * #u64
p - count * #u64
p + #(u64 * count)
```

`#(layout * count) == count * #layout`を満たす。addressの派生が同じlive region内または末尾の直後に収まることと、
`ByteSize`を返す乗算がoverflowしないことはpreconditionである。

## `Layout`と`Span`

primitive layoutは値として組み込み、型名からoperationを暗黙探索しない。

```text
u8  :: Layout<UInt8>
i32 :: Layout<Int32>
u64 :: Layout<UInt64>
```

product layoutとsum layoutを閉じたbuilt-in operatorで構成する。

```text
& :: (Layout<A>, Layout<B>) -> Layout<(A, B)>
| :: (Layout<A>, Layout<B>) -> Layout<[A, B]>
```

`u64 & i32`は両fieldのoffset、内部padding、tail padding、全体alignmentを決める。例えばsize 8/alignment 8の
`u64`とsize 4/alignment 4の`i32`なら、field offsetは0と8、raw endは12、strideは16、全体alignmentは8になる。

`u64 | i32`はtag representation、payload offset、最大payload extent、tail padding、全体alignmentを決める。
例えば1-byte tagと8-byte aligned payloadを使う単純な規則なら、tag offsetは0、payload offsetは8、strideは16となる。
tag値やinactive payloadを含む正確なadmissionとobservationは採択前に固定する。

layoutの表現には二つの水準があり、この試案は後者を候補とする。

| Model | Stability | Host access |
|---|---|---|
| canonical public layout | compiler versionを跨ぐ永続表現にできる | C structなどで直接解釈できる |
| opaque target layout | 同じcompiled artifactと対応host adapterの範囲で有効 | descriptorまたはgenerated helperを通じて解釈する |

opaque target layoutはcompilerにpaddingとtagの決定を任せる。file、network、永続storageには別の明示的codecを使い、
このlayoutを安定したwire formatとして扱わない。

`*`は一つのlayoutを有限のspanにする。

```text
* :: (Layout<A>, Count) -> Span<A>
#(layout * count) == count * #layout
```

`*`は`&`と`|`より強く結合する。構成したproductまたはsum layout全体を反復する場合は、`(u8 & u64) * count`のように
layout構成を括弧で囲む。

`Span<A>`はallocationそのものではない。layout、count、extent、base alignmentを持つplacement requestであり、
storageと結び付くのは`<-`、`<~`、またはそれを受け取るhost operationのcontractによる。

## `Packed`と`Symbol`

`Packed<A>`はmal-ownedなimmutable有限要素列であり、external storageのlayout、permission、lifetimeを保持しない。
`<-region`はregion全体を先頭からadmitし、元のexternal storageが失効しても保持できるpackedを作る。admissionはsource semanticsでは
external regionから独立した値を構成し、必要なstorage sizeを表現できない場合またはallocation failure時はEngram constructionと
同じ規則に従う。

```mal
region := address <- u8 * count;
bytes := <-region;
byte := bytes # index;
```

`Packed<A>`自体に`Layout<Packed<A>>`を暗黙に与えない。external storageへ戻すときは、要素の`Layout<A>`を持つcapacity
regionを作る。packedがcapacityを超える可能性があれば`/`と`%`でcurrent prefixとpending remainderに分け、currentを先頭へ書く。

```mal
output := outputAddress <- u8 * capacity;
current := bytes / #output;
pending := bytes % #output;
written := output / #current;
available := output <- current;
```

`Symbol`は`Packed<UInt8>`のtransparent aliasとする。

```mal
Symbol :: Packed<UInt8>;
```

```mal
symbol :: Symbol := <-(inputAddress <- u8 * count);

output := outputAddress <- u8 * #symbol;
written := output / #symbol;
available := output <- symbol;
```

`Packed`は物理的に連続したstorageを意味しない。Symbol literalはstatic leaf、`<-region`はcopyしたflat leaf、`/`と`%`はstorageを
共有するview、既存のSymbol連結はRope branchとして実装できる。実装はobservableなsequenceを変えない限りinline、flat leaf、
slice、Ropeなどを選べる。Regionへのstoreはleafを順に走査して直接copyでき、事前のflattenを要求しない。

`*Packed<A> -> Region<A>`のようなzero-copy viewは最初のprofileへ入れない。これをfirst-class valueにすると、
元のPackedをretainするowner-bearing viewまたはscoped borrowの規則が必要になり、連続pointerを要求する場合はRopeのflattenも
必要になり得る。managed valueのobservationはRegionへのstoreまたはconcreteなextern adapterのcall中borrowに閉じる。

## Exact placementとaligned placement

binary `<-`でlayoutまたはspanを適用するexact placementは現在位置を変更せず、その位置にplacementを適用する。`Address`、
`Cursor`、または`Region`へのlayout適用はCで`void *`を`T *`へ変換する操作に近いが、Cのeffective typeやprovenanceをそのまま
source semanticsへ取り込まない。Regionでは終端でなくbaseを現在位置とする。

```mal
cursor := p <- u64;
region := p <- u64 * count;
```

`<~`はlayoutのrequired alignmentを満たす最初のaddressまで前方へ進める。

```text
alignUp(address, layout)
    = address + paddingRequiredBy(address, layout)
```

```mal
cursor := p <~ u64;
region := p <~ u64 * count;
```

`p = 513`、`#u64 = 8bytes`、required alignmentが8 bytesなら、`p <- u64`は513を保ち、`p <~ u64`は520を指す。
unaligned accessは「実際にmisalignedである」という意味ではなく、alignmentをpreconditionとしてcode generatorへ渡さない
accessである。exact placementのresultが偶然alignedでも正しく動作する。

`<~`は新しいstorageやaccess permissionを作らない。spanを適用する場合、align-up後に`#span` bytesが元のlive regionへ収まり、
必要なpermissionを持つことをpreconditionとする。最大`alignment - 1` bytes進む可能性があるため、alignmentを保証しない
byte allocatorの結果へ適用するcallerは余剰storageとdeallocation用の元addressを管理する必要がある。

## Cursor accessとcompiler alignment fact

`Cursor<A>`はlayoutを保持するため、genericなload/storeが型`A`だけからrepresentationを探索する必要はない。

```mal
value := <-(p <- u64);
next := p <- u64 <- value;

alignedValue := <-(p <~ u64);
alignedNext := p <~ u64 <- value;
```

storeはcursorのlayoutを使い、同じlayoutのstrideだけ進んだ`Cursor<A>`を返す。同じlayoutの値はそのまま連続して書ける。

```mal
end := p
    <- u8
    <- first
    <- second
    <- third;
```

異なるlayoutへ切り替えるexact placementもaddressを進めない。`Cursor<A> <- Layout<B>`と`Cursor<A> <- A`は
right operandの型で区別する。最初のprofileでは`Layout`自体をmemory representationの対象に含めないため、二つのdomainは
重ならない。

alignment paddingを挟む場合、left-associativeなmemory chainで次のように書ける。

```mal
end := p
    <- u8
    <- tag
    <~ u64
    <- payload;
```

`p <- u8 <- tag`が返すcursorの現在位置を`<~ u64`がalign-upし、`Cursor<UInt64>`へ切り替えるため、最後のstoreでは
layoutを繰り返さない。paddingを入れずにlayoutだけを切り替える場合は`<~`の代わりに`<-`を使う。

compilerはsource typeとは別に、各cursor expressionが保証する最小alignmentをfactとして追跡する。

| Origin | Guaranteed alignment used for lowering |
|---|---:|
| `left <- layout` | 現在位置から保守的に導ける値。保証がなければ1 |
| `left <~ layout` | layoutのrequired alignment |
| alignedな`Region<A>`のbulk load/store | element layoutのrequired alignment |
| alignedな`Cursor<A> <- value` | 同じlayoutのrequired alignmentを保存 |
| guaranteeを保存できないjoinまたはcall boundary | 保守的な値へ弱める |

保証を持たないcursorはLLVMの`align 1` load/storeへ、保証を持つcursorはlayoutのrequired alignmentを指定した
load/storeへ変換できる。LLVMのalignment operandは性能hintではなく、過大申告するとundefined behaviorになるcontractである。
保証をsource typeとして区別する必要が実例から生じた場合に限り、`AlignedCursor<A>`などのrefinementを別途検討する。
exact placementで作ったregionのbulk accessはbaseの保守的なalignment factを使う。aligned placementで作ったregionでは、
layoutのstrideがrequired alignmentの倍数であるため、index順に処理するすべての要素でrequired alignmentを保存する。
`region / count`、`region % count`、region storeが返すsuffixも、元のregionから保守的に導けるalignment factだけを引き継ぐ。

## Host allocationとの境界

allocation、failure、ownership、deallocationはこの試案でもpredefined primitiveにしない。layoutはhost ABIへ渡さず、host operationは
`ByteSize`、`Count`、またはoperation固有のconcrete contractを受け取り、そのresultへmal側でplacementを適用する。
次の例はhost operationの成功resultから`raw`を取り出した後だけを示す。

```mal
span := u64 * count;
raw := hostAllocateBytes(#span);
region := raw <- span;
```

このregionはalignmentを仮定せずaccessできる。別のhost operationは、signatureにlayoutを渡さず、名前とconcrete contractで
`UInt64` spanに必要なsizeとbase alignmentを保証できる。

```mal
span := u64 * count;
raw := hostAllocateU64(count);
region := raw <~ span;
```

後者のhost contractは、成功時の`Address`が`u64 * count`のbase alignmentを満たし、少なくとも`#span` bytesのlive regionを指すと
定める。`<~`はそのcontractが守られていればaddressを動かさず、compiler alignment factを明示的に確立する。
failureをnull `Address`で表さず、sumまたはoperation固有のexternal opaque valueを使う現行規則は維持する。

alignmentを保証しないallocatorへ正確なbyte数だけ要求してから`<~`を適用してはならない。一般にはpayload extentに
`alignment - 1`を加え、派生addressとは別に元addressをdeallocationまで保持する必要がある。opaque layoutからalignmentを
数値として公開しないprofileでは、この処理をconcreteなhost adapterへ閉じ込める。

## Packed I/Oのstress example

固定capacityのscratch storageへhostがpartial readし、受け取ったbytesを`Symbol`として保持した後、別のhostへpartial writeを
行う例を考える。input側ではcapacity regionの一部だけが初期化されるため、host operationが返したcountを境界としてscratchを
読み出し可能なprefixと未使用suffixへ分ける。

```mal
capacity := 1024count;
scratch := inputAddress <- u8 * capacity;
readCount := hostRead(scratch);
valid := scratch / readCount;
unused := scratch % readCount;

#valid <= #scratch
#valid + #unused == #scratch
saved :: Symbol := <-valid;
```

`hostRead`のresult contractは`readCount <= #scratch`であり、`scratch / readCount`だけがinitializedかつreadableであると定める。
`<-scratch`はcapacity全体をadmitするため、未初期化のsuffixを含む可能性があり使えない。host bodyはlayoutを受け取らず、
generated adapterがconcreteなRegion carrierとhostのaddress/count resultを相互変換する。

output側では`Symbol`を`Packed<UInt8>`としてそのまま使い、scratch capacityを境界としてcurrent prefixとpending remainderに分ける。

```mal
pending :: Packed<UInt8> := saved;
scratch := outputAddress <- u8 * capacity;

current := pending / #scratch;
nextPending := pending % #scratch;
written := scratch / #current;
available := scratch <- current;

sentCount := hostWrite(written);
sent := written / sentCount;
retry := written % sentCount;
```

`current`は必ずscratchへ収まり、store resultの`available`は次のcopyに使えるsuffix capacityになる。`written`はhostへ渡す
initialized prefixである。`hostWrite`のresult contractを`sentCount <= #written`とし、同じ`/`と`%`で送信済みprefixと再送対象を
得る。専用のprefix、end、remainder result operationは要らない。

この例は次の未決事項を露出する。

- hostから返すcountが入力capacity以下であることを、trusted preconditionとadapterのruntime検査のどちらで扱うか。
- hostのbyte progress reportをadapterが`Count`へ変換して`/`と`%`の境界にする規則が必要になる。
- `sentCount == 0count`、hostがcapacityを超えるcountを返す場合、partial write後のfailureに対するpolicyが必要になる。
- scratch storageを再利用するには、`<-region`が作ったpackedだけがexternal lifetimeから独立し、host call終了後にregionを
  保持しないことをcontractにする必要がある。

このcorpusを採択判断に使い、RegionとPackedのfront splitには共通の`/`と`%`を使う。

## Layoutのcompile-time identity

layoutはmal sourceから見ると通常のimmutable valueであり、専用のbinding構文を追加しない。

```mal
word := u64;
entry := u8 & word;

place<A> :: (Address, Layout<A>) -> Cursor<A> :=
    (address, layout) -> address <- layout;
```

immutabilityだけではcompile-time constant性を導けないため、`Layout<A>`固有のsemantic ruleとして、一つの`A`に一つの
canonical singletonだけを認める。layoutを引数として受け取るgeneric bindingはconcrete `A`とlayout identityごとにspecializeし、
layout parameterをruntime calling conventionから消去する。layoutを返す式の評価にhost callなどのeffectが含まれる場合は、
effectを通常どおり実行し、singleton resultのruntime payloadだけを生成しない。

```text
source                         typed/runtime meaning
Layout<A> binding              static canonical identity
Layout<A> parameter            specialization input、runtime parameterなし
#layout                        target確定後のByteSize constant
layout * runtimeCount          static layoutとruntime Countを持つSpan<A>
Cursor<A>                      runtime Address + static layout identity
Region<A>                      runtime Address + Count + static layout identity
```

同じ`A`へpacked、endianness、versionなど複数のrepresentationを選ぶ機能は最初のprofileへ入れない。必要になった場合は
representation identityを型indexへ加えるか、file、network、永続storageと同様に明示的なcodecとして表す。これにより
required alignment、stride、field offsetは常にbackend constantとなり、LLVMへruntime alignmentを過大申告しない。

## CとRustとの比較

| Concern | C | Rust | このmal試案 |
|---|---|---|---|
| byte量と要素数 | `size_t`を`sizeof` result、byte数、要素数に共用する | `usize`を`size_of` result、slice length、indexに共用する | `ByteSize`と`Count`をsourceで分け、ABI representationだけを共有する |
| raw storage location | `void *`が近いがC object modelの規則を受ける | `*const u8`、`*mut u8`などのraw pointer | `Address`はnumeric value、型、length、permission、ownershipを持たないlocation |
| finite collection | borrowed arrayと別途length、またはowned allocation | sliceと`Vec<T>`などでviewとowned sequenceを分ける | `Region<A>`をexternal view、`Packed<A>`をmal-owned immutable sequenceとして分ける |
| typed interpretation | `T *`への変換。addressは変えない | `*const T`へのcast。addressは変えない | `p <- layout`がaddressを変えず`Cursor<A>`を作る |
| type layout | `sizeof`、`alignof`、field paddingとして型へ静的に付随し、first-class descriptorはない | type layoutにsize、alignment、field offsetがあり、別に`std::alloc::Layout`がある | compile-time singletonの`Layout<A>`がrepresentation、size、alignment、padding、tagを明示する |
| alignment evidence | typed pointer自体には保持せず、allocatorとprogramのcontractに置く | raw pointer自体には保持せず、referenceとoperationのsafety contractに置く | `<~`、host contract、alignedなregionのbulk accessからcompiler factとして導く |
| ordinary access | typed lvalue accessは型のalignmentを要求する | referenceと`ptr::read`/`write`はproper alignmentを要求する | exact placement由来ではalignmentを仮定せず、cursorのlayoutでaccessする |
| unaligned access | 一般のtyped primitiveはなく、portableなbyte copyには`memcpy`を使う | `read_unaligned`、`write_unaligned`がalignmentだけを緩和する | exact placementから作るcursorをalignment 1のload/storeへlowerする |
| align-up | castはaddressを動かさず、必要ならprogramまたはallocatorが別に計算する | pointer arithmeticまたはallocatorが担当する | `p <~ layout`が次の適合位置へ進む |
| allocation input | `malloc(size)`、`aligned_alloc(alignment, size)` | allocatorへ`Layout`を渡す | hostへ`ByteSize`、`Count`、またはconcrete operation固有の値だけを渡す |
| aggregate representation | struct/union/enumの規則とABIに現れる | `repr(Rust)`、`repr(C)`、`repr(packed)`などが制御する | `&`と`|`でopaque target layoutを構成し、stable wire formatとは分離する |

Cではcomplete object typeがaddressへのalignment requirementを持ち、alignmentは`size_t`値として表される。
`void *`から`T *`への変換はaddressをalign-upせず、resultが`T`に正しくalignedでなければundefined behaviorとなる。
`malloc`系が返すpointerのalignment contractと、alignmentとsizeを明示する`aligned_alloc`は
[`WG14 N1570`](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf)を参照する。`memcpy`はobjectを
`unsigned char`列としてcopyするため、Cでtyped unaligned dereferenceを避けるportableな境界になる。

Rustではtype layoutをsize、alignment、field offsetとして定義し、[`std::alloc::Layout`](https://doc.rust-lang.org/std/alloc/struct.Layout.html)
をallocatorへのfirst-class inputとして提供する。ただし`Layout`は`Layout<T>`ではなく、pointerとlayoutを結び付けた
`Cursor<T>`も標準で構成しない。raw pointerのvalidityはproper alignmentを含まず、通常のoperationとは別に
[`read_unaligned`](https://doc.rust-lang.org/std/ptr/fn.read_unaligned.html)と`write_unaligned`を提供する。
正確なtype layoutとraw pointer alignmentの規則は
[`Rust Reference`](https://doc.rust-lang.org/reference/type-layout.html)と
[`std::ptr`](https://doc.rust-lang.org/std/ptr/index.html)を参照する。

このmal試案はRustのallocator用`Layout`よりrepresentation ruleを強くし、CとRustが主にunsafeまたは外部contractへ
残す「addressへどのlayoutを適用したか」を`Cursor<A>`と`Region<A>`へ明示する。一方、bounds、initialization、permission、
ownership、lifetimeを証明するsafe referenceにはしない。必要なら、この低水準代数を直接公開しないlibraryまたはhost interfaceが
検査済みのsafe abstractionを別に構成する。

## ABIとbackend

`Layout<A>`はcompile-time singleton、`Span<A>`はstatic layoutとruntime countの組であり、どちらもlayout descriptorとして
public ABIへ渡さない。RegionとPackedの`/`と`%`はbaseまたはsequence storageとcountからprefixとremainderを導く。externはconcrete
specializationだけを公開し、generated adapterが既知のlayout constantを使って`ByteSize`やalignmentを計算する。hostにはoperationが
必要とする`ByteSize`、`Count`、pointer/count carrierなどだけを公開し、openな型parameterもC headerへ公開しない。

LLVM backendはtarget data layoutから次を別々に取得する。

- default address spaceのpointer representation幅
- default address spaceのpointer index幅
- 各primitive representationのABI alignment

pointer representation幅とindex幅は一致するとは限らない。GEPのinteger operand、load/storeのalignment、Cの`size_t`と
LLVM moduleのtargetが整合することをABI admissionで検証する。正確なLLVM contractは
[`Data Layout`](https://llvm.org/docs/LangRef.html#data-layout)、
[`getelementptr`](https://llvm.org/docs/LangRef.html#getelementptr-instruction)、
[`load`](https://llvm.org/docs/LangRef.html#load-instruction)、
[`store`](https://llvm.org/docs/LangRef.html#store-instruction)を正とする。

## Admissionと未決事項

最初のprofileではnumeric scalar、`ByteSize`、`Count`、`Address`、およびそれらから構成されるproductとsumをlayoutへ載せる候補を
採る。`Packed`はmal-ownedなEngramであり、それ自体のexternal layoutを持たない。`Symbol`は`Packed<UInt8>`のtransparent aliasとし、
`Packed<A>`はregionが持つ`Layout<A>`で要素ごとにadmitまたはobserveする。functionとexternal opaque typeなど
identity、ownership、またはhost固有の意味を持つ値は、retain、transfer、releaseとexternal representationを別に定めるまで除外する。

採択前に次を固定する。

- compile-time singletonである`Layout<A>`をbinding、parameter、resultへ書ける範囲と、specialization時の消去規則
- runtime carrierが必要な`Span`、`Cursor`、`Region`、`Packed`をsource signatureとpublic C ABIへ書ける範囲
- product field order、padding、tail paddingと、sum tag、payload、invalid representationの規則
- `#packed > #region`をstaticに排除できない場合にtrapするか、precondition違反とするか
- `Region<A>`と`Packed<A>`に対する`/`と`%`の境界値、evaluation、lowering規則
- `Symbol`をtransparent aliasにしたときのliteral、`+`、equality、`#`、indexingとC ABI名の扱い
- partial I/Oのzero progress、capacity超過、不正なconsumed count、途中failureの規則
- `Cursor<A>`のloadが現在位置を進めず、storeがstrideだけ進む規則とmemory chainのevaluation order
- region全体をadmitする途中でinvalid representationまたはallocation failureが生じた場合のcleanupとtrap
- `Packed<A> # index`のbounds preconditionとempty packed
- `<~`でaddress offsetを計算できるtargetと、Region、Cursor、prefix、remainderのalignment factを引き継ぐ規則
- alignment factをbinding、branch join、generic specialization、call boundaryで保存または弱める規則
- `Layout<Address>`を導入するbuilt-in layout literalのspelling
- hostから返すcountがcapacity以下であることをtrusted preconditionとruntime検査のどちらで扱うか
- `Packed<A>`をmal source間だけに限定するか、concrete specializationをpublic C ABIへ公開するか
- 現行`Ptr`から`Address`へのsource名と、対応するC ABI型名を同時に移行する範囲
- 現行の`T.size`、`T.load`、`T.store`、`Symbol.read`、`Symbol.write`からlayout、region、packedへの移行範囲
