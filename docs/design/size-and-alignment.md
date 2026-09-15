# `ByteSize`、`Count`、`Layout`、memory placementの試案

Status: Discussion draft (2026-09-15)

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

この試案では次の概念を区別する。`Layout`、`Span`、`Address`、`Region`は内部表現をsourceから分解できない
型index付きのbuilt-in opaque valueとする候補であり、表中の型名は設計上の名前である。

| 概念 | 保持する意味 | 保持しない意味 |
|---|---|---|
| `ByteSize` | targetがobject sizeとbyte offsetとして扱えるunsigned量 | pointer representation、要素数、alignment保証 |
| `Count` | target上の有限collectionの要素数とindexに使うdimensionlessなunsigned量 | byte量、pointer representation |
| `Ptr` | 外部storageのlocationと、そのlocationから派生・accessするcapability | 型、length、ownership、alignment保証 |
| `Layout<A>` | `A`一個のrepresentation、admission、observation、size/stride、required alignment | storage、lifetime、具体的なaddress |
| `Span<A>` | 一つの`Layout<A>`、要素数、全byte extent、base alignment | storage、ownership |
| `Address<A>` | `Ptr`へ`Layout<A>`を適用した一要素のlocation | bounds、lifetime、必ずしもalignment保証 |
| `Region<A>` | `Ptr`へ`Span<A>`を適用した有限個の要素location | allocation identity、ownership、lifetime延長 |

alignmentは独立したsource valueを必須にせず、`address`と`layout`の関係として扱う。

```text
aligned(pointer, layout)
    iff pointerのaddressがlayoutのrequired alignmentを満たす
```

`p <~ layout`、layout-awareなhost contract、およびalignedなbaseからのstride単位のindexingは、この関係を
compilerが利用できる形で確立または保存する。`Address<A>`と`Region<A>`を作っても、元の`Ptr`を供給したcontractが
定めるregion、permission、lifetimeは増えない。

## Source notation

候補となるoperatorを役割ごとにまとめる。

| Notation | Result | Meaning |
|---|---|---|
| `#layout` | `ByteSize` | 一要素のbyte extent/strideを観測する |
| `layout # count` | `Span<A>` | 同じlayoutを`count`個並べる |
| `#span` | `ByteSize` | span全体のbyte extentを観測する |
| `left & right` | `Layout<(A, B)>` | product layoutを構成する |
| `left \| right` | `Layout<[A, B]>` | sum layoutを構成する |
| `value * layout` | `Placed<A>` | 値へstore representationを明示する |
| `pointer / placement` | `Address<A>`または`Region<A>` | 現在位置を動かさずlayoutまたはspanを適用する |
| `pointer <~ placement` | `Address<A>`または`Region<A>` | 次のrequired alignmentを満たす位置まで進めて適用する |
| `region # index` | `Address<A>` | regionの要素を選ぶ |
| `<-address` | `A` | addressから値を読む |
| `address <- value` | `Ptr` | addressへ値を書き、そのlayoutの直後を返す |
| `pointer <- placed` | `Ptr` | raw pointerへ値を書き、そのlayoutの直後を返す |
| `pointer + bytes`、`pointer - bytes` | `Ptr` | byte単位でlocationを派生させる |

`#`は既存の`#symbol`と`symbol # index`を、operand型ごとの閉じたprimitive familyへ拡張する。prefix形は
byte extentの観測、binary形はlayoutのrepetitionまたは有限regionのindexingに使う。binary `#`は引き続き
non-associativeとする。

`<~`と`<-`はmemory chain用の同じ最下位precedenceに置き、left-associativeとする候補を採る。既存のbinary `#`は
それらより強く結合するため、次は`p <~ (u64 # count)`を表す。

```mal
p <~ u64 # count
```

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
範囲に収まらなければcompile-time errorとする。suffixのないliteralは`layout # 10`のように周辺型から`Count`へ決まる場合に
省略できる。`Count * ByteSize`と`ByteSize * Count`だけはbyte extentを返すdimension付きの閉じたprimitiveとする。

pointer offset、`Symbol`のbyte lengthとbyte index、各process argumentのbyte lengthには`ByteSize`を使う。spanとregionの
要素数とindex、process argument countには`Count`を使う。file format、network protocol、hashなど固定幅自体に意味がある値には
引き続き`UInt32`や`UInt64`を使う。CとRustは`size_t`または`usize`をbyte量と要素数の両方へ使うが、この試案では
source上の単位を区別し、ABI representationだけを共有する。

従来どおりbyte arithmeticを記述できる。

```mal
p + count * #u64
p - count * #u64
p + #(u64 # count)
```

`#(layout # count) == count * #layout`を満たす。pointerの派生が同じlive region内または末尾の直後に収まることと、
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

binary `#`は一つのlayoutを有限のspanにする。

```text
# :: (Layout<A>, Count) -> Span<A>
#(layout # count) == count * #layout
```

`Span<A>`はallocationそのものではない。layout、count、extent、base alignmentを持つplacement requestであり、
storageと結び付くのは`/`、`<~`、またはそれを受け取るhost operationのcontractによる。

## Exact placementとaligned placement

`/`はpointerを変更しない。Cで`void *`を`T *`へ変換する操作に近いが、Cのeffective typeやprovenanceをそのまま
source semanticsへ取り込まない。

```mal
address := p / u64;
region := p / (u64 # count);
```

`<~`はlayoutのrequired alignmentを満たす最初のaddressまで前方へ進める。

```text
alignUp(pointer, layout)
    = pointer + ((-address(pointer)) mod requiredAlignment(layout))
```

```mal
address := p <~ u64;
region := p <~ u64 # count;
```

`p = 513`、`#u64 = 8bytes`、required alignmentが8 bytesなら、`p / u64`は513を保ち、`p <~ u64`は520を指す。
unaligned accessは「実際にmisalignedである」という意味ではなく、alignmentをpreconditionとしてcode generatorへ渡さない
accessである。`/`のresultが偶然alignedでも正しく動作する。

`<~`は新しいstorageやauthorityを作らない。spanを適用する場合、align-up後に`#span` bytesが元のlive regionへ収まり、
必要なpermissionを持つことをpreconditionとする。最大`alignment - 1` bytes進む可能性があるため、alignmentを保証しない
byte allocatorの結果へ適用するcallerは余剰storageとdeallocation用の元pointerを管理する必要がある。

## Accessとcompiler alignment fact

`Address<A>`はlayoutを保持するため、genericなload/storeが型`A`だけからrepresentationを探索する必要はない。

```mal
value := <-(p / u64);
next := (p / u64) <- value;

alignedValue := <-(p <~ u64);
alignedNext := (p <~ u64) <- value;
```

raw pointerへ異なるlayoutを順に置く場合は`Placed<A>`を使う。

```mal
end := p
    <- header * u8
    <- payload * u64
    <- trailer * i32;
```

alignment paddingを挟む場合、left-associativeなmemory chainで次のように書ける。

```mal
end := p
    <- tag * u8
    <~ u64
    <- payload;
```

`p <- tag * u8`が返す`Ptr`へ`<~ u64`を適用して`Address<UInt64>`を作るため、最後のstoreではlayoutを繰り返さない。

compilerはsource typeとは別に、各address expressionが保証する最小alignmentをfactとして追跡する。

| Origin | Guaranteed alignment used for lowering |
|---|---:|
| `p / layout` | 1、または明示されたhost contractから導ける値 |
| `p <~ layout` | layoutのrequired alignment |
| alignedな`Region<A> # index` | element layoutのrequired alignment |
| guaranteeを保存できないjoinまたはcall boundary | 保守的な値へ弱める |

保証を持たないaddressはLLVMの`align 1` load/storeへ、保証を持つaddressはlayoutのrequired alignmentを指定した
load/storeへ変換できる。LLVMのalignment operandは性能hintではなく、過大申告するとundefined behaviorになるcontractである。
保証をsource typeとして区別する必要が実例から生じた場合に限り、`AlignedAddress<A>`などのrefinementを別途検討する。

## Host allocationとの境界

allocation、failure、ownership、deallocationはこの試案でもpredefined primitiveにしない。言語が提供するのは、host operationが
必要に応じて`ByteSize`またはopaqueな`Span`を受け取れる表現と、そのresultへplacementを適用するmechanismである。
次の例はhost operationの成功resultから`raw`を取り出した後だけを示す。

```mal
raw := hostAllocateBytes(#(u64 # count));
region := raw / (u64 # count); // host contractがalignmentを保証する場合
```

別のhostはlayout-awareなoperationを提供できる。

```mal
raw := hostAllocateLayout(u64 # count);
region := raw <~ u64 # count;
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
| typed interpretation | `T *`への変換。addressは変えない | `*const T`へのcast。addressは変えない | `p / layout`がaddressを変えず`Address<A>`または`Region<A>`を作る |
| type layout | `sizeof`、`alignof`、field paddingとして型へ静的に付随し、first-class descriptorはない | type layoutにsize、alignment、field offsetがあり、別に`std::alloc::Layout`がある | opaqueな`Layout<A>`がrepresentation、size、alignment、padding、tagを明示的に運ぶ |
| alignment evidence | typed pointer自体には保持せず、allocatorとprogramのcontractに置く | raw pointer自体には保持せず、referenceとoperationのsafety contractに置く | `<~`、host contract、region indexingからcompiler factとして導く |
| ordinary access | typed lvalue accessは型のalignmentを要求する | referenceと`ptr::read`/`write`はproper alignmentを要求する | `/`由来ではalignmentを仮定せず、layoutのrepresentationでaccessする |
| unaligned access | 一般のtyped primitiveはなく、portableなbyte copyには`memcpy`を使う | `read_unaligned`、`write_unaligned`がalignmentだけを緩和する | `/`から作るaddressをalignment 1のload/storeへlowerする |
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
`Address<T>`も標準で構成しない。raw pointerのvalidityはproper alignmentを含まず、通常のoperationとは別に
[`read_unaligned`](https://doc.rust-lang.org/std/ptr/fn.read_unaligned.html)と`write_unaligned`を提供する。
正確なtype layoutとraw pointer alignmentの規則は
[`Rust Reference`](https://doc.rust-lang.org/reference/type-layout.html)と
[`std::ptr`](https://doc.rust-lang.org/std/ptr/index.html)を参照する。

このmal試案はRustのallocator用`Layout`よりrepresentation capabilityを強くし、CとRustが主にunsafeまたは外部contractへ
残す「pointerへどのlayoutを適用したか」を`Address<A>`と`Region<A>`へ明示する。一方、bounds、ownership、lifetimeを
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

最初のprofileではnumeric scalar、`ByteSize`、`Count`、`Ptr`、およびそれらだけから構成されるproductとsumをlayoutへ
載せる候補を採る。
`Symbol`、function、external opaque typeなどownershipまたはhost固有の意味を持つ値は、retain、transfer、releaseと
external representationを別に定めるまで除外する。

採択前に次を固定する。

- `Layout`、`Span`、`Address`、`Region`をsource signatureに書ける型として公開する範囲
- product field order、padding、tail paddingと、sum tag、payload、invalid representationの規則
- `Address<A>`へのstoreが返す`Ptr`とmemory chainのevaluation order
- `Region<A> # index`のbounds preconditionとzero-count span
- `<~`でpointer offsetを計算できるtargetと、one-past pointerを含むregion contract
- alignment factをbinding、branch join、generic specialization、call boundaryで保存または弱める規則
- opaque `Layout`と`Span`をpublic C ABIでhostが解釈するgenerated helper
- 現行の`T.size`、`T.load`、`T.store`、`Symbol.read`、`Symbol.write`からの移行範囲
