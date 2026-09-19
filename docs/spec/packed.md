# `Region`と`Packed`

Status: Accepted v0.6 profile

この文書はexternal location列`Region<A>`とmal-owned immutable sequence `Packed<A>`のtransfer、slice、構築、編集、
`Symbol`変換、host境界を定める。layoutとtyped viewは[external memory](memory.md)を正とする。

## Authority

`Region<A>`はAddress、USize、canonical layoutを運ぶexternal storage viewであり、initialization、permission、allocation identity、
ownership、lifetimeは構成元のprogramまたはhost contractが定める。

`Packed<A>`はmal-ownedなimmutable有限要素列とUSizeである。runtimeはbacking storageを必要な間保持し、最後のviewを失った後に
解放する。連続した表現は値の意味に含まれない。`A`がAddressを含んでもPacked storageだけをmalが支配し、各Addressのreferent lifetimeは延長しない。
`Representable(A)`によりelementはmal-managed ownerを含まない。

## Scoped authority

`Region<A>`と`Buffer<A>`はscoped authorityである。compilerはこれらのparameterと、これらを再帰的に含むaggregate parameterを
内部的な`Scoped<T>`として分類する。scopeはparameterを宣言したfunctionまたはlambdaの一回のinvocationが完了するまでである。

scoped valueはlocal binding、predefined operation、通常のhelper、recursive callへ直接渡せる。callee parameterはそのinvocation中だけの
scoped reborrowになる。scoped parameter、その派生Region、これらを含むaggregateをsource functionまたはlambdaから返してはならず、
nested lambdaによるcaptureも即時適用の有無にかかわらずcompile-time errorとする。通常のhelperからRegionを返してはならない。

Regionを返す`/`、`%`、`set`のresultはscopedに保つ。`#region`、`get`、`new`、`put`のresultはscopedにしない。
通常のgeneric type parameterへ、alias展開後にRegion、Buffer、またはこれらを再帰的に含む型を代入してはならない。
generic signatureが`Region<A>`または`Buffer<A>`を直接含むことは認め、そのparameterをspecialization後もscopedに分類する。

## Operation

```text
Region<A> / USize           -> Region<A>
Region<A> % USize           -> Region<A>
Region<A>.set(Packed<A>)    -> Region<A>
Packed<A> / USize           -> Packed<A>
Packed<A> % USize           -> Packed<A>
Packed<A> + Packed<A>       -> Packed<A>
#Region<A>                  -> USize
#Packed<A>                  -> USize
Packed<A> # USize           -> A
*Packed<UInt8>              -> Symbol
*Symbol                     -> Packed<UInt8>
```

`#region`はlocation数、`#packed`は読み出せる実在要素数を返す。`region.set(packed)`は先頭からobserveし、書いた範囲の直後から
始まるsuffix Regionを返す。zero-count Packedでは元のRegionを返し、storageをdereferenceしない。

`value / count`はprefix、`value % count`はremainderを返す。Regionはstorageをdereferenceせずviewを分ける。Packedは同じbacking
storageを共有するslice viewを返す。operandは通常のexpressionと同じ順で一度だけ評価する。

`left + right`はleftの全要素にrightの全要素を続けたimmutable Packedを返す。operandは引き続き有効で、empty Packedをidentityとする。
copy、owner共有、dead intermediateのstorage再利用、連結chainの一括allocationはobservable semanticsを変えない実装detailである。

## External storage admission

```mal
pack<A> :: (Address, USize, USize) -> Packed<A>;
```

二つのUSizeはelement単位の`offsetStart`と`offsetEnd`で、半開区間`[offsetStart, offsetEnd)`を表す。引数を通常のapplication順で
一度ずつ評価する。strideがnonzeroなら`address + offsetStart * stride(A)`から`offsetEnd - offsetStart`要素をindex順にadmitする。
zero strideではAddressを派生または観測せず、logical countだけを持つPackedを返す。external storageは変更しない。

## Symbol conversion

`Symbol`は`Packed<UInt8>`のaliasではない。prefix `*`はこの二型の間だけのclosed conversion familyである。
`*packed`は同じbytesのSymbol、`*symbol`は同じbytesのPackedを返す。両型はmal-owned byte storageのownerとviewを共有し、
変換は新しいownerを割り当てずにresultのshareを得る。operandとresultは変換後も独立に有効である。変換は
storage allocation、byte copy、allocation failureを追加しない。
external storageを直接ownerにするzero-copy Packed viewはない。

## Scoped constructionとediting

`make`、`edit`はpredefined generic intrinsicである。`Representable(A)`を満たす型argumentを明示し、構築中だけ有効な
`Buffer<A>` authorityをcallbackへ渡す。`Buffer<A>`にはsource-level constructorがなく、これらのintrinsicだけが作る。

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

`make<A>(capacity, callback)`はcount 0のbuilderを作り、callback開始前に`capacity`要素までのstorageを確保する。
capacityはhintではなく、表現可能なsizeとallocationをこの時点で検査する要求である。
callbackが追加する要素数はcapacity以下に制限されず、超えた場合も通常どおりgrowthする。
`edit<A>(source, callback)`はsourceと同じ要素列とcountから始まるbuilderを作る。
`source.edit<A>(callback)`はreceiver-first applicationによる同じoperationである。callback parameter `buffer`について、
`buffer.new(value)`は末尾へ追加してその安定したindexを返し、`buffer.get(index)`は現在値を返し、
`buffer.put(index, value)`は現在値を置換する。これらはそれぞれ`new(buffer, value)`、`get(buffer, index)`、
`put(buffer, index, value)`というreceiver-firstでない同じoperationとしても書ける。element型はBuffer operandから決まり、
明示的なtype argumentを取らない。
先行するoperationの結果は後続のoperationから観測できる。

引数は通常のapplication順で一度ずつ評価する。`make`ではcapacity、callbackの順、`edit`ではsource、callbackの順に評価してから
builderを作る。callbackが`Unit`で正常完了するとbuilderをfreezeし、count要素の
owned `Packed<A>`を返す。
Buffer callback parameterにはscoped authorityのescape制約を適用する。

`edit`のsourceとresultは独立したimmutable valueとして振る舞う。source、そのslice、および`A = UInt8`の場合にownerを共有する
`Symbol`は編集前と同じ値を返す。実装はcallbackの開始前または最初の変更時にwritable storageを確立し、変更がない場合はownerを共有してよい。
source responsibilityとruntime ownerがともに一意な場合のstorage再利用は、observable semanticsを変えないoptimizationである。

count、`count * stride(A)`、owner allocation sizeがtargetで表現できない場合と、必要なallocationのfailureはtrapする。
zero-stride elementではstorageを持たず、capacityはelement allocationを要求しない。`new`の回数をcountへ加えてよい。
growth policy、余剰capacity、copy-on-writeと再利用の選択は
implementation detailである。

## Partial I/O

external allocation、deallocation、failure、ownershipはprogram固有のextern contractが定める。host operationはAddress、ByteSize、
USize、またはoperation固有のconcrete contractを受け取り、layout shapeをABIへ渡さない。

partial inputはcapacity以下のUSizeと、そのprefixを初期化したというpostconditionを返す。partial outputはinput以下のUSizeと、
そのprefixを消費したというpostconditionを返す。mal codeはRegionをprefixとremainderへ分け、保持するprefixを元Addressから
`pack`し、残りをretryする。
`0usize`がEOF、would-block、empty input、zero progressのどれかはoperation固有のcontractが定める。

## 未検査precondition

| Operation | Precondition |
|---|---|
| `Region / USize`、`Region % USize` | `count <= #region`で、strideがnonzeroならprefix extentがoverflowしない |
| `Packed / USize`、`Packed % USize` | `count <= #packed` |
| `Packed # USize` | `index < #packed` |
| scoped `buffer.get(index)`、`buffer.put(index, value)` | `index <` builderの現在count |
| `pack<A>(address, start, end)` | `start <= end`である。strideがnonzeroならbyte offsetがoverflowせず、対象区間が同じlive storage内にあり、全locationがreadable、初期化済みでvalid representationを持つ。allocation sizeがtargetで表現可能 |
| `region.set(packed)` | `#packed <= #region`である。strideがnonzeroなら対象prefixがwritableでextentがoverflowしない |
| host partial input | result USizeがcapacity以下で、そのprefixが初期化済み |
| host partial output | result USizeがinput以下で、そのprefixがconsumed済み |

primitiveはこれらを検査せず、違反時の結果を保証しない。host postcondition違反もtrusted contractへの違反である。
防御的trapはimplementation detailである。zero-stride elementはreadability、writability、initialization、storage extentを要求しない。

host operationへ渡せる型とbyte列をAddressで運ぶ規則は[`extern`](extern.md#host-mappable-type)を正とする。
