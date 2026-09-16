# `ByteSize`、`Count`、`Layout`、memory placement、`Chunk`の試案

Status: Discussion draft (2026-09-16)

この文書はtarget依存のbyte量、値のmemory representation、alignmentを満たすplacement、およびhostが提供する
storageとの境界を一つの候補として整理する。現行の規範は[`types`](../spec/types.md)、
[`Symbol`](../spec/symbols.md)、[`memory primitive`](../spec/memory.md)、[`program`](../spec/programs.md)、
[`C host ABI`](../spec/c-host-abi.md)を正とし、この試案だけを根拠にsource、ABI、compilerを変更しない。
型parameterそのものの範囲は
[`parametric polymorphismと型index付きprimitiveの試案`](parametric-polymorphism.md)に置く。

## 目的

現行仕様はstorage sizeとpointer offsetを`UInt64`で表し、scalar load/storeを型ごとの名前付きfunctionとして提供する。
すべてのload/storeはalignmentを要求せず、productとsumにはcanonicalなexternal memory representationを定めない。
この試案は次の摩擦をまとめて扱う。

- targetのaddress計算に使う量を固定幅の`UInt64`から分離する。
- size、alignment、padding、sum tagを型ごとに手計算せず、opaqueなplacement ruleへ閉じ込める。
- exactなaddressの解釈と、次のaligned addressへの移動を区別する。
- memory representation capabilityを明示的に渡したgeneric codeだけにparametricなload/storeを認める。
- allocation policyを言語が所有せず、host operationがsizeだけ、またはlayout情報を受け取れるようにする。

## 概念モデル

この試案では次の概念を区別する。`Layout`、`Span`、`Cursor`、`Region`、`Chunk`は内部表現をsourceから分解できない
型index付きのbuilt-in opaque valueとする候補であり、表中の型名は設計上の名前である。

| 概念 | 保持する意味 | 保持しない意味 |
|---|---|---|
| `ByteSize` | targetがobject sizeとbyte offsetとして扱えるunsigned量 | pointer representation、要素数、alignment保証 |
| `Count` | target上の有限collectionの要素数とindexに使うdimensionlessなunsigned量 | byte量、pointer representation |
| `Ptr` | 外部storageのlocationと、そのlocationから派生・accessするcapability | 型、length、ownership、alignment保証 |
| `Layout<A>` | `A`一個のrepresentation、admission、observation、size/stride、required alignment | storage、lifetime、具体的なaddress |
| `Span<A>` | 一つの`Layout<A>`、要素数、全byte extent、base alignment | storage、ownership |
| `Cursor<A>` | 現在の`Ptr`と一つの`Layout<A>` | bounds、ownership、lifetime、型としてのalignment保証 |
| `Region<A>` | `Ptr`へ`Span<A>`を適用した有限個の要素location | allocation identity、ownership、lifetime延長 |
| `Chunk<A>` | mal-ownedなimmutable有限要素列と要素数 | external layout、mutable storage、host resourceのlifetime |

`Cursor<A>`はstorageや所有者ではなく、現在位置へ`A`のrepresentation ruleを適用したmemory cursorである。書き込みは
同じlayoutのstrideだけ進んだ新しい`Cursor<A>`を返し、元のcursorとpointerを無効にしない。alignment factはcursorの
runtime fieldやsource typeではなく、cursor expressionについてcompilerが別に追跡する。raw `Ptr`には値のreadまたは
write規則を適用できず、先にlayoutを適用してcursorを作る必要がある。

alignmentは独立したsource valueを必須にせず、`pointer`と`layout`の関係として扱う。

```text
aligned(pointer, layout)
    iff pointerのaddressがlayoutのrequired alignmentを満たす
```

`p <~ layout`、layout-awareなhost contract、およびalignedなcursorからのstride単位の移動は、この関係を
compilerが利用できる形で確立または保存する。`Cursor<A>`と`Region<A>`を作っても、元の`Ptr`を供給したcontractが
定めるregion、permission、lifetimeは増えない。

## Source notation

候補となるoperatorを役割ごとにまとめる。

| Notation | Result | Meaning |
|---|---|---|
| `#layout` | `ByteSize` | 一要素のbyte extent/strideを観測する |
| `layout ^ count` | `Span<A>` | 同じlayoutを`count`個並べる |
| `#span` | `ByteSize` | span全体のbyte extentを観測する |
| `left & right` | `Layout<(A, B)>` | product layoutを構成する |
| `left \| right` | `Layout<[A, B]>` | sum layoutを構成する |
| `pointer <- layout` | `Cursor<A>` | 現在位置を動かさずlayoutを適用する |
| `cursor <- layout` | `Cursor<B>` | 現在位置を動かさず別のlayoutへ切り替える |
| `region <- layout` | `Cursor<B>` | region終端を動かさずlayoutを適用する |
| `pointer <- span`、`cursor <- span`、`region <- span` | `Region<A>` | 現在のplacement frontierを動かさず有限spanを適用する |
| `pointer <~ placement`、`cursor <~ placement`、`region <~ placement` | `Cursor<A>`または`Region<A>` | placement frontierから次のrequired alignmentを満たす位置まで進めてlayoutまたはspanを適用する |
| `<-cursor` | `A` | cursorの現在位置から値を読む |
| `cursor <- value` | `Cursor<A>` | 値を書き、同じlayoutのstrideだけ進む |
| `<-region` | `Chunk<A>` | region全体を先頭からadmitする |
| `region <- chunk` | `Region<A>` | 同じ要素数のchunkをregion全体へ先頭からobserveする |
| `#chunk` | `Count` | chunkの要素数を観測する |
| `chunk # index` | `A` | chunkの要素を選ぶ |
| `pointer + bytes`、`pointer - bytes` | `Ptr` | byte単位でlocationを派生させる |

型の関係だけを取り出すと、memory placementとaccessの代数は次になる。ここでleft operandの`Cursor<B>`または
`Region<B>`にある`B`は、切替先の`A`と同じである必要はない。

```text
Layout<A> ^ Count         -> Span<A>

Ptr <- Layout<A>          -> Cursor<A>
Cursor<B> <- Layout<A>    -> Cursor<A>
Region<B> <- Layout<A>    -> Cursor<A>
Ptr <~ Layout<A>          -> Cursor<A>
Cursor<B> <~ Layout<A>    -> Cursor<A>
Region<B> <~ Layout<A>    -> Cursor<A>

Ptr <- Span<A>            -> Region<A>
Cursor<B> <- Span<A>      -> Region<A>
Region<B> <- Span<A>      -> Region<A>
Ptr <~ Span<A>            -> Region<A>
Cursor<B> <~ Span<A>      -> Region<A>
Region<B> <~ Span<A>      -> Region<A>

Cursor<A> <- A            -> Cursor<A>
<-Cursor<A>               -> A

<-Region<A>               -> Chunk<A>
Region<A> <- Chunk<A>     -> Region<A>
#Chunk<A>                 -> Count
Chunk<A> # Count          -> A
```

placement operatorのleft operandは、それぞれ次の位置をfrontierとして使う。

```text
frontier(Ptr)       = pointer自身の位置
frontier(Cursor<A>) = cursorの現在位置
frontier(Region<A>) = regionのbase + #span
```

`left <- placement`はfrontierへexact placementし、`left <~ placement`はfrontierからalign-upしてplacementする。
`Region<A>`のfrontierはzero-countでもbaseと同じ位置に定まり、region終端から次の異なるsectionを配置できる。

```mal
header := pointer <- u8 ^ headerLength;
items := header <~ itemLayout ^ itemCount;
trailer := items <- u32;
end := trailer <- checksum;
```

`items`はalignedなitem region、`trailer`はその終端にexact placementした`Cursor<UInt32>`である。placement resultは新しく
適用した部分だけを表し、`header`、alignment padding、`items`を一つの結合regionにはしない。

`Region<A>`のcontent operationは常にbaseからregion全体を対象にする。loadはindex順に各要素をlayoutでadmitして
mal-ownedな`Chunk<A>`を作り、storeは同じ要素数のchunkをindex順にobserveする。zero-count regionのloadはempty chunkを返し、
load/storeともstorageをdereferenceしない。store resultは同じbaseとspanを持つregionであり、次のplacementを終端から継続できる。
`Region<A>`自体は移動するcursorではなく、baseを先頭とする区間である。終端は次のplacementにだけ派生して使う。

```mal
chunk := <-region;
filled := outputRegion <- chunk;
next := filled <~ nextLayout;
```

`Region<A>`から一要素のcursorを直接得る`#` operationは設けない。external regionの要素を個別に扱う場合は全体をchunkへ
admitして`chunk # index`で選ぶか、元の`Ptr`からbyte offsetとlayoutを明示してcursorを作る。`chunk # index`は
`index < #chunk`をpreconditionとし、region storeはregionのspan countと`#chunk`の一致を要求する。count不一致をtrapと
precondition違反のどちらにするかは採択前に固定する。

region終端は元のregionに対する有効なone-past locationだが、その位置からpaddingを進めたり新しい値をaccessしたりするauthorityを
追加しない。`Region<A> <- placement`と`Region<A> <~ placement`では、paddingと新しいplacementの全byteが元の`Ptr`を供給した
host contractのlive regionに収まり、必要なpermissionを持つことをpreconditionとする。

`#`は既存の`#symbol`と`symbol # index`を、operand型ごとの閉じたprimitive familyへ拡張する。prefix形は`Layout`、`Span`、
`Symbol`のbyte extentを`ByteSize`、`Chunk`の要素数を`Count`で観測する。binary形は`Symbol`または`Chunk`のindexingに使い、
引き続きnon-associativeとする。
`^`はintegerに対する既存のbit XORに加えて、`Layout<A>`と`Count`から`Span<A>`を作る閉じたprimitiveとする。

`<~`とbinary `<-`はmemory chain用の同じ最下位precedenceに置き、left-associativeとする候補を採る。既存の`^`と
binary `#`はそれらより強く結合するため、次は`p <- (u64 ^ count)`を表す。

```mal
p <- u64 ^ count
```

したがって通常の算術をstoreする場合も右辺を括弧で囲む必要はない。

```mal
end := p <- i32 <- a + 1 <- b * 2;
```

これは`(((p <- i32) <- (a + 1)) <- (b * 2))`として評価する。各operandは通常のoperator規則どおり左から右へ
一度ずつ評価する。prefix `<-cursor`はloadであり、binary chainとは別に現在位置を進めない。

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
範囲に収まらなければcompile-time errorとする。suffixのないliteralは`layout ^ 10`のように周辺型から`Count`へ決まる場合に
省略できる。`Count * ByteSize`と`ByteSize * Count`だけはbyte extentを返すdimension付きの閉じたprimitiveとする。

pointer offset、`Symbol`のbyte lengthとbyte index、各process argumentのbyte lengthには`ByteSize`を使う。spanとregionの
要素数、`Chunk`のlengthとindex、process argument countには`Count`を使う。file format、network protocol、hashなど固定幅自体に意味がある値には
引き続き`UInt32`や`UInt64`を使う。CとRustは`size_t`または`usize`をbyte量と要素数の両方へ使うが、この試案では
source上の単位を区別し、ABI representationだけを共有する。

従来どおりbyte arithmeticを記述できる。

```mal
p + count * #u64
p - count * #u64
p + #(u64 ^ count)
```

`#(layout ^ count) == count * #layout`を満たす。pointerの派生が同じlive region内または末尾の直後に収まることと、
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

`^`は一つのlayoutを有限のspanにする。

```text
^ :: (Layout<A>, Count) -> Span<A>
#(layout ^ count) == count * #layout
```

`Span<A>`はallocationそのものではない。layout、count、extent、base alignmentを持つplacement requestであり、
storageと結び付くのは`<-`、`<~`、またはそれを受け取るhost operationのcontractによる。

## `Chunk`と`Symbol`

`Chunk<A>`はmal-ownedなimmutable有限要素列であり、external storageのlayout、permission、lifetimeを保持しない。
`<-region`はregion全体を先頭からadmitし、元のexternal storageが失効しても保持できるchunkを作る。実装はobservableな
identityを追加しない範囲でstorageを共有できるが、source semanticsではexternal regionから独立した値である。

```mal
region := pointer <- u8 ^ count;
bytes := <-region;
byte := bytes # index;
```

`Chunk<A>`自体に`Layout<Chunk<A>>`を暗黙に与えない。external storageへ戻すときは、要素の`Layout<A>`を持つregionを作り、
同じcountのchunkをregion全体へ書く。

```mal
output := outputPointer <- u8 ^ #bytes;
filled := output <- bytes;
```

`Symbol`のexternal read/writeは専用layoutで直接行わず、`Chunk<UInt8>`とのlosslessな組み込み変換を介する。

```text
Symbol.fromChunk :: Chunk<UInt8> -> Symbol
Symbol.toChunk   :: Symbol -> Chunk<UInt8>
```

```mal
bytes := <-(inputPointer <- u8 ^ count);
symbol := Symbol.fromChunk(bytes);

outputBytes := Symbol.toChunk(symbol);
output := outputPointer <- u8 ^ #outputBytes;
filled := output <- outputBytes;
```

変換はbyte sequenceを保存し、identityは観測できない。実装はcopy、共有、storage transferのどれを使ってもよい。
`#symbol`はbyte量を`ByteSize`で返し、`#chunk`は要素数を`Count`で返すため、同じruntime表現になり得てもsource上の単位は
区別する。`Symbol`は引き続きimmutableなbyte value、`Chunk<UInt8>`は要素型を持つfinite collectionである。

## Exact placementとaligned placement

binary `<-`でlayoutまたはspanを適用するexact placementはfrontierを変更せず、その位置にplacementを適用する。`Ptr`または
`Cursor`へのlayout適用はCで`void *`を`T *`へ変換する操作に近いが、Cのeffective typeやprovenanceをそのままsource
semanticsへ取り込まない。`Region`ではbaseではなく終端をfrontierにする。

```mal
cursor := p <- u64;
region := p <- u64 ^ count;
```

`<~`はlayoutのrequired alignmentを満たす最初のaddressまで前方へ進める。

```text
alignUp(pointer, layout)
    = pointer + ((-address(pointer)) mod requiredAlignment(layout))
```

```mal
cursor := p <~ u64;
region := p <~ u64 ^ count;
```

`p = 513`、`#u64 = 8bytes`、required alignmentが8 bytesなら、`p <- u64`は513を保ち、`p <~ u64`は520を指す。
unaligned accessは「実際にmisalignedである」という意味ではなく、alignmentをpreconditionとしてcode generatorへ渡さない
accessである。exact placementのresultが偶然alignedでも正しく動作する。

`<~`は新しいstorageやauthorityを作らない。spanを適用する場合、align-up後に`#span` bytesが元のlive regionへ収まり、
必要なpermissionを持つことをpreconditionとする。最大`alignment - 1` bytes進む可能性があるため、alignmentを保証しない
byte allocatorの結果へ適用するcallerは余剰storageとdeallocation用の元pointerを管理する必要がある。

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

異なるlayoutへ切り替えるexact placementもpointerを進めない。`Cursor<A> <- Layout<B>`と`Cursor<A> <- A`は
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
| `left <- layout` | frontierから保守的に導ける値。保証がなければ1 |
| `left <~ layout` | layoutのrequired alignment |
| alignedな`Region<A>`のbulk load/store | element layoutのrequired alignment |
| alignedな`Cursor<A> <- value` | 同じlayoutのrequired alignmentを保存 |
| guaranteeを保存できないjoinまたはcall boundary | 保守的な値へ弱める |

保証を持たないcursorはLLVMの`align 1` load/storeへ、保証を持つcursorはlayoutのrequired alignmentを指定した
load/storeへ変換できる。LLVMのalignment operandは性能hintではなく、過大申告するとundefined behaviorになるcontractである。
保証をsource typeとして区別する必要が実例から生じた場合に限り、`AlignedCursor<A>`などのrefinementを別途検討する。
exact placementで作ったregionのbulk accessはbaseの保守的なalignment factを使う。aligned placementで作ったregionでは、
layoutのstrideがrequired alignmentの倍数であるため、index順に処理するすべての要素でrequired alignmentを保存する。
region終端へのexact placementはbase alignmentとspan extentから安全に導けるfrontierのfactだけを引き継ぎ、切替先layoutの
required alignmentを仮定しない。aligned placementは終端からalign-upするため、切替先layoutのrequired alignmentを確立する。

## Host allocationとの境界

allocation、failure、ownership、deallocationはこの試案でもpredefined primitiveにしない。言語が提供するのは、host operationが
必要に応じて`ByteSize`またはopaqueな`Span`を受け取れる表現と、そのresultへplacementを適用するmechanismである。
次の例はhost operationの成功resultから`raw`を取り出した後だけを示す。

```mal
span := u64 ^ count;
raw := hostAllocateBytes(#span);
region := raw <- span;
```

host contractがbase alignmentも保証する場合、compilerはそのfactをexact placementへ引き継げる。

別のhostはlayout-awareなoperationを提供できる。

```mal
span := u64 ^ count;
raw := hostAllocateLayout(span);
region := raw <~ span;
```

後者のhost contractは、成功時のpointerがspanのbase alignmentを満たし、少なくとも`#span` bytesのlive regionを指すと
定められる。`<~`はそのcontractが守られていればaddressを動かさず、compiler alignment factを明示的に確立する。
failureをnull `Ptr`で表さず、sumまたはoperation固有のexternal opaque valueを使う現行規則は維持する。

alignmentを保証しないallocatorへ正確なbyte数だけ要求してから`<~`を適用してはならない。一般にはpayload extentに
`alignment - 1`を加え、派生pointerとは別に元pointerをdeallocationまで保持する必要がある。opaque layoutからalignmentを
数値として公開しないprofileでは、この処理をlayout-awareなhost adapterへ閉じ込める。

## CとRustとの比較

| Concern | C | Rust | このmal試案 |
|---|---|---|---|
| byte量と要素数 | `size_t`を`sizeof` result、byte数、要素数に共用する | `usize`を`size_of` result、slice length、indexに共用する | `ByteSize`と`Count`をsourceで分け、ABI representationだけを共有する |
| raw storage location | `void *`が近いがC object modelの規則を受ける | `*const u8`、`*mut u8`などのraw pointer | `Ptr`は型、length、ownershipを持たないcapability-bearing location |
| finite collection | borrowed arrayと別途length、またはowned allocation | sliceと`Vec<T>`などでviewとowned sequenceを分ける | `Region<A>`をexternal view、`Chunk<A>`をmal-owned immutable sequenceとして分ける |
| typed interpretation | `T *`への変換。addressは変えない | `*const T`へのcast。addressは変えない | `p <- layout`がaddressを変えず`Cursor<A>`を作る |
| type layout | `sizeof`、`alignof`、field paddingとして型へ静的に付随し、first-class descriptorはない | type layoutにsize、alignment、field offsetがあり、別に`std::alloc::Layout`がある | opaqueな`Layout<A>`がrepresentation、size、alignment、padding、tagを明示的に運ぶ |
| alignment evidence | typed pointer自体には保持せず、allocatorとprogramのcontractに置く | raw pointer自体には保持せず、referenceとoperationのsafety contractに置く | `<~`、host contract、region indexingからcompiler factとして導く |
| ordinary access | typed lvalue accessは型のalignmentを要求する | referenceと`ptr::read`/`write`はproper alignmentを要求する | exact placement由来ではalignmentを仮定せず、cursorのlayoutでaccessする |
| unaligned access | 一般のtyped primitiveはなく、portableなbyte copyには`memcpy`を使う | `read_unaligned`、`write_unaligned`がalignmentだけを緩和する | exact placementから作るcursorをalignment 1のload/storeへlowerする |
| align-up | castはaddressを動かさず、必要ならprogramまたはallocatorが別に計算する | pointer arithmeticまたはallocatorが担当する | `p <~ layout`が次の適合位置へ進む |
| allocation input | `malloc(size)`、`aligned_alloc(alignment, size)` | allocatorへ`Layout`を渡す | hostごとに`ByteSize`または`Span<A>`を受け取るextern contractを選べる |
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

このmal試案はRustのallocator用`Layout`よりrepresentation capabilityを強くし、CとRustが主にunsafeまたは外部contractへ
残す「pointerへどのlayoutを適用したか」を`Cursor<A>`と`Region<A>`へ明示する。一方、bounds、ownership、lifetimeを
同時に証明するsafe referenceにはしない。

## ABIとbackend

`Layout`と`Span`をhostへ渡す場合、sourceから内部fieldを観測できるC structに固定せず、`Symbol`と同様のmanaged primitive
またはgenerated helperを介してhostが解釈できる形を候補とする。host adapterは少なくともtotal extentとrequired alignmentを
取得できる。concrete specializationだけがpublic ABIへ現れ、openな型parameterをC headerへ公開しない。

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

最初のprofileではnumeric scalar、`ByteSize`、`Count`、`Ptr`、およびそれらから構成されるproductとsumをlayoutへ載せる候補を
採る。`Symbol`と`Chunk`はmal-ownedなEngramであり、それ自体のexternal layoutを持たない。`Symbol`は`Chunk<UInt8>`へ変換して
regionとの間で移し、`Chunk<A>`はregionが持つ`Layout<A>`で要素ごとにadmitまたはobserveする。functionとexternal opaque typeなど
identity、ownership、またはhost固有の意味を持つ値は、retain、transfer、releaseとexternal representationを別に定めるまで除外する。

採択前に次を固定する。

- `Layout`、`Span`、`Cursor`、`Region`、`Chunk`をsource signatureに書ける型として公開する範囲
- product field order、padding、tail paddingと、sum tag、payload、invalid representationの規則
- `Region<A>`と`Chunk<A>`のcount不一致をtrapするか、precondition違反とするか
- `Cursor<A>`のloadが現在位置を進めず、storeがstrideだけ進む規則とmemory chainのevaluation order
- region全体をadmitする途中でinvalid representationまたはallocation failureが生じた場合のcleanupとtrap
- `Chunk<A> # index`のbounds preconditionとempty chunk
- `<~`でpointer offsetを計算できるtargetと、one-past pointerを含むregion contract
- alignment factをbinding、branch join、generic specialization、call boundaryで保存または弱める規則
- opaque `Layout`と`Span`をpublic C ABIでhostが解釈するgenerated helper
- `Chunk<A>`をmal source間だけに限定するか、concrete specializationをpublic C ABIへ公開するか
- 現行の`T.size`、`T.load`、`T.store`、`Symbol.read`、`Symbol.write`からlayout、region、chunkへの移行範囲
