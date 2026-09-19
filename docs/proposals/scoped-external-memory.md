# scoped external memory surface

Status: Draft proposal; non-normative

この文書はexternal memoryとmal-owned sequenceのsource APIを、rawな`Address`、scopedな`Region`と`Buffer`、immutableな
`Packed`へ整理する案を定める。現在の規則は[external memory](../spec/memory.md)と
[`Region`と`Packed`](../spec/packed.md)を正とし、本proposalはまだlanguage ruleではない。

## Authority model

| 型 | authority | 視点 |
|---|---|---|
| `Address` | host-owned storage上のraw location。型、extent、permission、lifetimeを持たない | byte location |
| `Region<A>` | 引数として導入されたscope内だけ有効な、固定長external storageへのtyped view | 先頭 |
| `Packed<A>` | mal-owned immutable sequence | 先頭 |
| `Buffer<A>` | `Packed`を構築または編集するscoped mutable authority | 末尾 |

`Address`を渡したhost contractはstorageのextent、permission、initialization、valid representation、lifetimeについて、
実行するoperationが要求するpreconditionを保証する。`view`自体は未初期化のwritable storageにも形成でき、これらを検査せず、
referentのownershipも取得しない。

## scoped authorityのescape制約

`Region<A>`と`Buffer<A>`はscoped authorityとする。source syntaxへ新しいtype constructorは加えないが、compilerはこれらの
parameterを内部的に`Scoped<Region<A>>`、`Scoped<Buffer<A>>`として分類する。parameter valueがaggregate内にこれらを再帰的に
含む場合もaggregate全体をscopedとする。scopeはparameterを宣言したfunctionまたはlambdaの一回のinvocationであり、そのbodyの
完了までとする。

scoped parameterと、`/`、`%`、`set`などから得た派生Regionはlocal bindingへ置き、predefined operationへ渡せる。通常のhelperや
recursive callへ直接渡すこともでき、そのcallee parameterはcallee invocation中だけのscoped reborrowになる。一方、source functionや
lambdaはscoped parameter、その派生Region、それらを含むaggregateをresultとして返せない。nested lambdaによるcaptureも、closureの
実際のescapeや即時適用の有無にかかわらずcompile-time errorとする。通常のhelperからRegionを返すことも認めない。

Regionを返す`/`、`%`、`set`のresultはscopedに保つ。それ以外の`#region`、`get`、`new`、`put`のresultはscopedにしない。
特にelement型が`Address`でも、`get`で得たAddressは元Regionではなく自身のhost contractに従うcopyable capabilityである。
この規則はlexical checkだけで決まり、scope polymorphism、function valueのscope-flow summary、runtime lifetime check、
control-flowに依存するescape analysisを導入しない。

通常のgeneric type parameterへ、transparent aliasを展開した後にRegion、Buffer、またはこれらを再帰的に含む型を代入してはならない。
これにより`identity<A>(value) -> value`やaggregateを返すgeneric helperによるescape制約の迂回を防ぐ。generic signatureが
`Region<A>`または`Buffer<A>`を直接含むことは認め、そのparameterにはspecialization後も同じscoped classificationを適用する。

将来、RegionとBuffer以外のdeviceまたはexecution domain固有capabilityにも同じ制約を公開する場合は、`Scoped<T>`に相当する
source-level spellingを別途検討する。本proposalではbuiltin二型のchecker classificationに留める。

## Addressとlayout

`Address`に定義する派生は前方byte offsetだけとする。

```text
Address + ByteSize -> Address
```

数学的なoffsetがoverflowせず、resultが同じlive storage内または末尾直後にあることをpreconditionとする。
`Address - ByteSize`、Address同士の演算、equality、ordering、null、integer conversionは持たない。suffixのAddressを
受け取ったcalleeが、それより前のlocationを復元できるoperationを設けない。

canonical layout一要素のstrideは既存のclosed shape queryで得る。

```mal
#i64
#(i64, u8)
count * #i64
```

`#shape`は`ByteSize`、`#value`はsequenceの`USize`を返す。複数要素のextentとAddress移動には通常の乗算と加算を使い、
新しい`stride`または`extent` intrinsicは導入しない。

## zero-stride element

sequenceの論理countとelement storageのfootprintを分ける。`stride(A) = 0`なら、任意のcountについてfootprintはemptyである。
これは`Unit`に加え、canonical layoutがzero strideになるUnitだけからなるproductとtransparent aliasにも一貫して適用する。

zero-stride elementについて、Regionの`get`はboundsだけを検査して唯一の値を構成し、`put`と`set`はboundsまたはcountだけを検査する。
`view`と`pack`は`offsetStart <= offsetEnd`だけをstorageに関するpreconditionとし、Addressのreferent、extent、permission、
initialization、valid representationを要求しない。`pack`は`offsetEnd - offsetStart`をcountとするPackedをelement storageなしで返す。

PackedとBufferもelement storageを持たず、countだけを保持してよい。`make`のcapacityは通常どおり評価するが、nonzeroでも
element storage allocationを要求しない。`new`はcountを増やし、`get`はbounds内の唯一の値を返し、`put`はbounds内で何も変更しない。
countのoverflow規則はzero-strideでも維持する。

## scoped `Region`

Addressからroot `Region<A>`を形成できるoperationは`view`だけとする。

```mal
view<A> ::
    (
        Address,
        USize,
        USize,
        Region<A> -> Unit
    )
    -> Unit;
```

二つの`USize`はbase Addressからのelement offsetによる`offsetStart`と`offsetEnd`で、対象は半開区間
`[offsetStart, offsetEnd)`とする。`A`は`Representable(A)`を満たす。Address、二つのoffset、callbackを通常のapplication順で
一度ずつ評価した後にRegionを形成し、callbackを一度適用する。strideがnonzeroならRegionの先頭は
`address + offsetStart * stride(A)`である。zero strideではAddressを派生または観測せず、logical countだけを持つRegionを形成する。
どちらもlengthは`offsetEnd - offsetStart`である。
`view`はallocationせず、元のAddressを変更しない。
`Region<A>`にはsource-level constructorがなく、RepresentableでもHostMappableでもない。callback parameterにはscoped authorityの
escape制約を適用し、Regionをcallback完了後へ到達させない。

Regionは固定長で先頭を基準とする。

```text
#Region<A>                 -> USize
Region<A> / USize         -> Region<A>
Region<A> % USize         -> Region<A>
Region<A>.get(USize)      -> A
Region<A>.put(USize, A)   -> Unit
Region<A>.set(Packed<A>)  -> Region<A>
```

receiver-first表記は独立したmethod lookupではなく、`get(region, index)`、`put(region, index, value)`、
`set(region, packed)`と同じpredefined operation identityを指す。

各operationは次の未検査preconditionを持つ。

| operation | precondition |
|---|---|
| `view<A>(address, start, end, ...)` | `start <= end`である。strideがnonzeroなら両offsetとstrideから得るbyte offsetがoverflowせず、半開区間が同じlive storage内にある。empty区間では先頭が末尾直後でもよい |
| `region / count`、`region % count` | `count <= #region`である。strideがnonzeroならprefix extentがoverflowしない |
| `region.get(index)` | indexが範囲内である。strideがnonzeroならoffsetがoverflowせず、locationがreadable、初期化済み、valid representationを持つ |
| `region.put(index, value)` | indexが範囲内である。strideがnonzeroならoffsetがoverflowせず、locationがwritableである |
| `region.set(packed)` | `#packed <= #region`である。strideがnonzeroなら対象prefixがwritableで、extentがoverflowしない |

`region / count`はprefix、`region % count`はremainderを返し、storageをdereferenceしない。`get`と`put`はindex位置を
観測または変更する。

`region.set(packed)`はRegionの先頭から`#packed`要素を書き、書いた直後を先頭とするremainder Regionを返す。
`#packed <= #region`をpreconditionとし、empty Packedではstorageへ触れず元のRegionを返す。

```mal
remaining := region.set(first).set(second);
```

RegionからAddressへのprojectionは設けない。host operationを構成するcodeは元のAddressを保持し、Regionと同じ論理量だけ
明示的に前進させる。Regionだけを委譲されたcalleeはraw Addressを取得できない。

```mal
nextAddress := address + consumed * #i64;
remaining := region % consumed;
```

Regionは1より強いalignmentを保証せず、backendはunaligned-safeなaccessを生成する。実Addressのalignmentをruntimeで判定して
高速なpathを選ぶことはsource semanticsを変えないoptimizationだが、source-levelのalign-upまたはalignment assertionは設けない。
`Packed`と`Buffer`は内部表現に応じて安全にaccessし、storage alignment自体をsource semanticsへ含めない。

## Addressからの`pack`

external storageの半開区間をmal-owned sequenceへadmitするoperationを`pack`とする。

```mal
pack<A> ::
    (
        Address,
        USize,
        USize
    )
    -> Packed<A>;
```

二つのoffsetは`view`と同じelement単位の`offsetStart`と`offsetEnd`である。引数を通常のapplication順で一度ずつ評価し、
strideがnonzeroなら`address + offsetStart * stride(A)`から`offsetEnd - offsetStart`要素をindex順にadmitする。zero strideでは
Addressを派生または観測せず、logical countだけを持つPackedを作る。元のexternal storageは変更しない。

`offsetStart <= offsetEnd`で、strideがnonzeroなら両offsetとstrideから得るbyte offsetがoverflowせず、対象区間が同じlive storage内に
あり、各locationがreadable、初期化済みでvalid representationを持つことを未検査preconditionとする。result countとallocation sizeがtargetで
表現できない場合もprecondition違反とし、allocation failureはtrapする。empty区間はstorageをdereferenceせずempty Packedを返す。

## `Packed`と`Buffer`

Packedはimmutableで先頭を基準とする。既存のlength、index、prefix、remainderに、同じelement型の連結を加える。

```text
#Packed<A>                 -> USize
Packed<A> # USize         -> A
Packed<A> / USize         -> Packed<A>
Packed<A> % USize         -> Packed<A>
Packed<A> + Packed<A>     -> Packed<A>
```

`left + right`はleftの全要素にrightの全要素を続けた新しいimmutable valueを返す。operandは引き続き有効で、empty Packedを
identityとする。copy、owner共有、dead intermediateのstorage再利用、連結chainの一括allocationはobservable semanticsを変えない
実装detailとする。countまたはstorage sizeをtargetで表現できない場合はprecondition違反、必要なallocationのfailureはtrapとする。

byte sequence conversionは維持する。

```text
*Packed<UInt8> -> Symbol
*Symbol        -> Packed<UInt8>
```

Bufferは末尾を基準とするgrowable builderであり、`get`、`put`、`new`だけを持つ。`Region`の`/`、`%`、`set`、Address projection、
および途中のPacked snapshotを持たない。複数の完成済みPackedは`+`で結合し、必要ならそのresultを`edit`するため、
`Buffer.extend(Packed)`も初期surfaceには加えない。

```text
Buffer<A>.get(USize)    -> A
Buffer<A>.put(USize, A) -> Unit
Buffer<A>.new(A)        -> USize
```

このreceiver-first表記も、Buffer operandを取る同名のpredefined operationを指す。

## `make`と`edit`

既存のscoped builder `pack`と`bulk`を、初期capacity付き`make`へ統合する。

```mal
make<A> ::
    (
        USize,
        Buffer<A> -> Unit
    )
    -> Packed<A>;

edit<A> ::
    (
        Packed<A>,
        Buffer<A> -> Unit
    )
    -> Packed<A>;
```

capacityとcallbackを通常のapplication順で一度ずつ評価した後にbuilderを形成する。`make`の第一引数は初期countでも上限でもなく、
callback開始前に要求するcapacityである。Bufferの初期countは0とする。
capacity 0またはzero strideではelement storageを事前確保せず、それ以外ではcapacity要素分をcallback開始前に確保する。`new`はcapacityを超えて
growできる。最終Packedのlengthは`new`の回数であり、初期size overflowと初期allocation failureはcallback開始前にtrapする。

```text
旧 pack<A>(callback)           -> make<A>(0usize, callback)
旧 bulk<A>(capacity, callback) -> make<A>(capacity, callback)
```

`edit`はsource、callbackを通常のapplication順で一度ずつ評価した後にbuilderを形成する。builderはsourceと同じ要素列とcountから
始め、callback完了時にresultをfreezeする。`make`と`edit`のcallback parameterにはscoped authorityのescape制約を適用し、Bufferを
callback完了後へ到達させない。BufferからPackedを得る通常の境界は`make`または`edit`のcallback完了だけとする。

## Editorでのpredefined API表示

`view`、`pack`、`make`、`edit`はsource declarationを持たないため、predefined identityのmetadataに表示名、generic signature、
短い説明、主要なpreconditionを持たせる。RegionとBufferの`get`、`put`、`set`、`new`も同じ対象とし、receiver-first applicationと
通常のapplicationは同じidentityとdocumentationを参照する。`get`や`put`のように複数の型へ適用できるoperationでは、check済みの
receiverとoperation variantに対応する説明を表示する。

この仕組みはmemory primitive専用にせず、predefined typeとvalue全体へ適用する。少なくともbuiltin scalar、`Address`、`ByteSize`、
`USize`、`Symbol`、`Region`、`Packed`、`Buffer`では、型の役割、type argumentの制約、ownershipまたはscope上の性質をhoverで確認
できるようにする。source上の宣言位置は捏造せず、definitionとrenameの対象外である現行規則は維持する。

resolver、checker、editorが別々のpredefined一覧や説明文を持たないよう、名前とreserved identityを所有するpredefined declarationを
metadataのauthorityとする。generic signatureはcheckerとは別の表示用文字列として保持せず、同じstructured declarationからrenderする。
主要なpreconditionの説明はidentityまたはcheck済みoperation variantごとの単一documentation fieldに置く。semantic hoverとcompletion
documentationは同じmetadataを使い、LSP層は表示形式への変換だけを担う。
採択時には[editor tooling](../development/editor-tooling.md)の「predefined symbolにはdocumentationを付けない」という現行規則を
この規則へ置き換える。

## Surfaceの整理

このproposalは次を置き換える。

- `Cursor<A>`を削除し、一要素accessも長さ1のRegionで表す。
- `Address@Shape`と`Cursor@USize`を半開offset区間を取る`view<A>`へ置き換え、`@` placementを削除する。
- Cursor load/storeを`get`、`put`へ、Region admissionの`<-`をAddressと半開offset区間を取る`pack`へ置き換える。
- `Region <- Packed`を`region.set(packed)`へ置き換える。
- Cursor/RegionからAddressを得る`?`を削除する。
- postfix `!`とsource-level alignment operationを削除する。
- `Address - ByteSize`を削除する。
- scoped builderの`pack`と`bulk`をcapacity付き`make`へ統合する。

closed layout shapeと`#shape`、Region/Packedの`/`と`%`、Packed/Symbolの`*`は維持する。

## 採択前の確認

- Typical90のRegion variantを新surfaceへ移し、Addressを明示的に運ぶ負担とgenerated codeを比較する。
- `Packed + Packed`のbaseline実装と、Symbol concatenationと同じunique-owner optimizationの適用範囲を測定する。
- scoped parameter、派生Region、aggregate result、nested capture、helperとrecursive callを含むpositive/negative testでlexical escape制約を固定する。
- zero-stride elementがlogical countだけを運び、Region、Packed、Bufferのoperationでstorage accessとelement allocationを要求しないことを固定する。
- predefined type、`view`、`pack`、`make`、`edit`、Region/Buffer operationのhoverとcompletion documentationをeditor testで固定する。
- Regionのunaligned-safe accessとPacked/Bufferの内部表現に応じたsafe accessが、代表的なscalar、product、sumで正しいartifactを生成することを確認する。
