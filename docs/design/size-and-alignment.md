# `Address`、target size、layout、placementの試案

Status: Discussion draft (2026-09-17)

この文書はtarget依存のbyte量、値のcanonical memory representation、alignment、external storage上のplacementを
一つの候補として定める。有限regionとmal-owned sequenceのtransferは
[`Region`と`Packed`の試案](region-and-packed.md)、型parameterとの境界は
[parametric polymorphismの試案](parametric-polymorphism.md)が所有する。現行の規範は[`types`](../spec/types.md)、
[`memory primitive`](../spec/memory.md)、[`C host ABI`](../spec/c-host-abi.md)であり、この試案だけを根拠にsource、ABI、
compilerを変更しない。

## 目的と概念

現行仕様の`Ptr`、`UInt64`によるbyte offset、型別load/storeを次の一つのmodelへ置き換える。

| 概念 | 保持する意味 | 別のcontractが所有する意味 |
|---|---|---|
| `ByteSize` | targetがobject sizeとbyte offsetに使うunsigned量 | pointer representation、要素数、alignment保証 |
| `Count` | target上の有限collectionの要素数とindex | byte量、pointer representation |
| `Address` | external storageのopaqueなlocation | numeric value、型、extent、permission、ownership、alignment保証 |
| layout shape | 一要素のrepresentation、stride、required alignment | runtime payload、storage、lifetime、具体的なaddress |
| `Cursor<A>` | `Address`とcanonicalな`A`に一意なstatic layout | extent、initialization、permission、ownership、lifetime |
| `Region<A>` | 同じlayoutを持つ`Count`個のlocation | 利用可能なbuffer、初期化済みslice、permission、ownership |

layout shapeは`@`によるplacementまたは`#`によるstride queryの直後だけに置くcompile-time構文operandである。compilerは
shapeをtarget固有のconstantへ解決し、runtimeでは`Cursor<A>`と`Region<A>`が選択済みのstatic layoutを運ぶ。一つのcanonical
`A`とtargetの組は一つのartifact-local layoutを持つため、binding、parameter、result、field、capture、extern argumentには
通常のvalueとsource typeだけが現れる。

`Address`はcopyableなcapabilityであり、複製してもreferentのlifetimeを延長しない。literal、null、equality、Address同士の
加減算、integerとの相互変換は提供しない。`Address + ByteSize`と`Address - ByteSize`はinteger演算ではなく、同じstorage
capabilityからbyte位置を派生させるoperationである。

## 型形成

compilerは閉じた`Representable(A)` judgmentを持つ。最初のprofileでは`Unit`、全numeric scalar、`Address`、`ByteSize`、
`Count`、全fieldがrepresentableなproduct、全variantがrepresentableな二項以上のsumを再帰的にrepresentableとする。
transparent aliasは展開後の型で判定するため、`Bool`は`[Unit, Unit]`と同じlayoutを持つ。function、external opaque type、
`Packed<A>`、`Cursor<A>`、`Region<A>`、empty sumはrepresentableにしない。

`Cursor<A>`、`Region<A>`、`Packed<A>`は`Representable(A)`の場合だけwell-formedである。generic signatureから型形成条件を
導く規則は[parametric polymorphismの試案](parametric-polymorphism.md#型形成条件)に置く。この条件はstorageにvalidな
representationが存在することを証明しない。

## Layout shape

primitive shapeはnumeric literal suffixと同じ短いspellingを使う。型が現れる構文と混同せず、既知のsuffixとの対応を
再利用するため、`UInt64`ではなく`u64`と書く。

```text
unit
i8 i16 i32 i64
u8 u16 u32 u64
f32 f64
address bytesize count
```

productとsumはsource typeと同じdelimiterの内側にshapeを書き、flatなn項構造とnested構造を区別する。

```text
(i8, u64, i32)
((i8, u64), i32)
[unit, i32, address]
[unit, [i32, address]]
```

一要素product、一要素sum、empty sum shapeは認めない。transparent aliasはcanonical typeへ展開し、representableなら
shapeとして使える。`#shape`は一要素のstrideを`ByteSize`で返すtarget constantである。

```mal
#i8
#(u64, i32)
#Bool
```

### PrimitiveとUnit

numeric scalar、`Address`、`ByteSize`、`Count`のstrideとrequired alignmentはtarget data layoutから決める。numeric scalarの
storage幅はbit幅、`Address`はdefault address spaceのpointer storage幅、`ByteSize`と`Count`はpointer index幅を使う。
異なるscalar shapeで同じbytesを観測した場合のbyte orderとrepresentationはbackend host ABIが定める。

`Unit`はstride 0、required alignment 1とする。load/storeはstorageをdereferenceせず、次のCursorは同じlocationになる。
`Region<Unit>`はstorageを消費せず任意のCountを持てる。

### Product

fieldをsource orderに配置する。先頭offsetは0、後続offsetは直前fieldの末尾からそのfieldのrequired alignmentまで前方へ
丸める。全体alignmentは全fieldの最大値、strideは最後のfieldの末尾から全体alignmentまで前方へ丸めた値とする。
nested productはflattenしない。

### Sum

0-based variant indexのtag、padding、全variantで共有するpayload領域の順に配置する。tagはvariant数を表せる最小のbuiltin
unsigned型とする。

```text
2 .. 2^8 variants          UInt8
2^8 + 1 .. 2^16 variants  UInt16
2^16 + 1 .. 2^32 variants UInt32
2^32 + 1 .. 2^64 variants UInt64
```

payload alignmentは全variantの最大値、payload offsetはtag末尾からそのalignmentまで前方へ丸める。payload extentは全variantの
strideの最大値、sum alignmentはtagと全variantの最大値、sum strideはpayload末尾から全体alignmentまで前方へ丸める。
`2^64`を超えるvariantを持つsumは拒否する。

storeはproduct field、sum tag、選択payloadだけを書き、paddingと非選択payloadを変更しなくてよい。loadはそれらを読まず、
paddingを含むbytewise equalityを提供しない。このlayoutは同じartifactと対応adapterの範囲だけで有効であり、mal runtime
representation、public C aggregate carrier、file、network、永続storageのformatではない。

## `ByteSize`と`Count`

両型はdefault address spaceのpointer index幅を持つ別々のunsigned source typeである。

```text
source type          ByteSize          Count
literal suffix       bytes             count
example              10bytes           10count
public C ABI type    mal_ByteSize_t    mal_Count_t
C representation     size_t            size_t
```

literal、同じ型同士の加減算と比較、明示的numeric conversionを認める。`Count`には乗除算とremainderも認める。
`Count * ByteSize`と`ByteSize * Count`は`ByteSize`、`ByteSize * ByteSize`はerrorとする。加減乗算はtarget幅でwrapし、
divisionとremainderはdivisorがzeroでないことをpreconditionとする。memory extentとして使う数学的な積がoverflowしないことは、
そのmemory operationまたはhost contractのpreconditionである。

address offsetとprocess argumentのbyte lengthには`ByteSize`、Region、Packed、Symbolの要素数とindex、process argument countには
`Count`を使う。target非依存checkはliteralの型を決め、target幅に収まるかはartifact生成時に検査する。

```mal
p + count * #u64
p - count * #u64
p + 8 * #u64
```

numeric conversionは`.i8`、`.u32`、`.f64`、`.bytes`、`.count`のclosed postfix familyへ統一する候補とする。通常のliteralは
既存suffixを使い、conversionは既存integerのmodulo規則を保つ。

```mal
300u8          // range error
(300i64).u8    // 44
```

## Placementとaccess

```text
Address + ByteSize          -> Address
Address - ByteSize          -> Address
Address@Shape               -> Cursor<A>
Cursor<A>@Count             -> Region<A>
?Cursor<A>                  -> Address
?Region<A>                  -> Address
Cursor<A>!                  -> Cursor<A>
Region<A>!                  -> Region<A>
Cursor<A> <- A              -> Cursor<A>
<-Cursor<A>                 -> (A, Cursor<A>)
```

`Address@Shape`はlocationを動かさず、shapeに対応するcanonical `A`のCursorを作る。Cursorへ別shapeを直接適用できず、
`?cursor`でAddressへ戻してから切り替える。`Cursor<A>@Count`は現在locationから同じstrideを繰り返すRegionを作る。
Cursorと一要素Regionは同一視しない。

loadは現在位置の値とstrideだけ進んだCursorを返し、storeも同じ次Cursorを返す。どちらもexternal storageをconsumeせず、
referentのlifetimeを変更しない。product patternまたは既存のvalue-first applicationで連続readを表せる。

```mal
(value, next) := <-(address@u64);
(<-(address@u64))[(value, next) -> use(value, next)]

end := address@u8
    <- first
    <- second
    <- third;
```

すべてのexact Cursor accessはunaligned accessを認める。backendは保証されたalignmentがなければalignment 1のload/storeまたは
同等のbyte accessへlowerする。compilerは性能のために最小alignment factを追跡してよいが、失えばalignment 1へ弱め、過大な
LLVM alignmentを指定してはならない。

postfix `!`はassertionではなく、現在位置から`A`のrequired alignmentを満たす最初のlocationへのalign-upである。Regionでは
Countを保存し、count 0でもlocationをalign-upする。prefix `!`はBool negationのまま残す。

```mal
alignedCursor := address@u64!;
alignedRegion := address@u64@count!;
```

exact placementとunaligned accessはすべてのbackendが実装するbaselineとする。postfix `!`はtarget capabilityであり、backendは
opaque pointer provenanceを保ってalign-upを実装できるtargetでだけadmitする。integral pointer targetではAddressのbitsからpaddingを
計算し、元のAddressへbyte offsetを適用してresultを派生する。実装できないtargetは`!`を使うprogramだけをartifact生成時に拒否する。

required alignmentの数値queryとgeneric aligned allocatorは最初のprofileへ入れない。arbitrary allocatorからのalign-upには最大paddingの
余剰storageとdeallocation用の元Addressが必要であり、concreteなhost operationのcontractへ閉じ込める。

## 未検査precondition

このmemory algebraは、callerまたはAddressを提供したhost contractが次の条件を満たす未検査mechanismである。

| Operation | Precondition |
|---|---|
| `Address +/- ByteSize` | 数学的offsetがoverflowせず、resultが同じlive region内または末尾の直後にある |
| `Address@Shape` | なし。形成だけではstorageをaccessしない |
| `Cursor@Count` | `Count * stride(A)`がoverflowせず、全locationが同じlive region内にある。Count 0またはstride 0では先頭が末尾の直後でもよい |
| Cursor load | 現在の一要素がreadableかつ初期化済みでvalid representationを持つ |
| Cursor store | 現在の一要素がwritableである |
| Cursor load/storeのresult | 次locationが同じregion内または末尾の直後にある |
| `Cursor<A>!` | skipするpaddingと一要素分のextentが同じlive regionに収まる |
| `Region<A>!` | skipするpaddingとCount要素分のextentが同じlive regionに収まる。Count 0ではpaddingだけを対象とする |
| Region/Packed operation | [`Region`と`Packed`のprecondition](region-and-packed.md#未検査precondition)に従う |

valid Address representation、sum tag、bounds、permission、initialization、lifetime、extent、overflowをprimitiveは検査しない。
違反時の特定の結果を保証しない。実装は内部memory corruptionを避けるために検査してtrapしてよいが、そのtrapはsource-level
contractではない。preconditionを満たしたoperationがmal-owned storageを必要とし、実際のallocationに失敗した場合はtrapする。

末尾の直後を指すCursorは保持、Addressへの投影、Count 0のRegion形成には使えるが、通常のload/storeには使えない。Unit accessと
`Region<Unit>`はdereferenceせず、permission、initialization、storage extentを要求しない。stride 0なので同じlocationを返す。

## TargetとC ABI

backendはsource memory layout専用のtarget layout planを作り、mal runtime valueの内部layoutを再利用しない。default address
spaceのpointer representation幅、pointer index幅、primitive ABI alignmentをtarget data layoutから別々に取得する。layout計算を
targetのobject sizeで表現できない型はartifact生成時に拒否する。

reference C backendは`ByteSize`と`Count`を`size_t`へ写す。LLVM pointer index幅とC `size_t`幅が一致するtargetだけをadmitし、
generated C artifactのcompile-time assertionを含むABI admissionで検証する。pointer representation幅との一致は要求しない。
LLVM module、C shim、runtime、host sourceは同じtargetへcompileする。別backendは自身のindex型に対応するhost mappingを定める。

extern parameterとresultへ出せる型は[`Region`と`Packed`のABI規則](region-and-packed.md#host-abi)に従う。

## 採択前に固定すること

- `#value`と`#shape`、`Address@Shape`と`Cursor@Count`、postfix `!`を区別するgrammar、precedence、formatter規則
- postfix numeric conversionへ現行`T(value)`と`value[T]`を置き換える移行範囲
- 現行`Ptr`から`Address`へのsource名、C ABI型名、process argument ABIの移行範囲
- precondition表のpositive、one-past、zero-count、overflow、invalid representation corpus
- exact/unaligned baselineとpostfix `!` target capabilityのartifact生成diagnostic
