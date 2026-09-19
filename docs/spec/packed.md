# `Region`と`Packed`

Status: Accepted v0.6 profile

この文書はexternal location列`Region<A>`とmal-owned immutable sequence `Packed<A>`のtransfer、slice、構築、編集、
`Symbol`変換、host境界を定める。layoutとplacementは[external memory](memory.md)を正とする。

## Authority

`Region<A>`はAddress、USize、canonical layoutを運ぶexternal storage viewであり、initialization、permission、allocation identity、
ownership、lifetimeは構成元のprogramまたはhost contractが定める。

`Packed<A>`はmal-ownedなimmutable有限要素列とUSizeである。runtimeはbacking storageを必要な間保持し、最後のviewを失った後に
解放する。連続した表現は値の意味に含まれない。`A`がAddressを含んでもPacked storageだけをmalが支配し、各Addressのreferent lifetimeは延長しない。
`Representable(A)`によりelementはmal-managed ownerを含まない。

## Operation

```text
<-Region<A>                 -> Packed<A>
Region<A> <- Packed<A>      -> Region<A>
Region<A> / USize           -> Region<A>
Region<A> % USize           -> Region<A>
Packed<A> / USize           -> Packed<A>
Packed<A> % USize           -> Packed<A>
#Region<A>                  -> USize
#Packed<A>                  -> USize
Region<A> # USize           -> Cursor<A>
Packed<A> # USize           -> A
*Packed<UInt8>              -> Symbol
*Symbol                     -> Packed<UInt8>
```

`#region`はlocation数、`#packed`は読み出せる実在要素数を返す。`region # index`はindex番目のexternal locationをCursorとして返し、
storageをdereferenceしない。`<-region`は全要素をindex順にadmitして新しいPackedを返す。
`region <- packed`は先頭からobserveし、書いた範囲の直後から始まるsuffix Regionを返す。zero-count Regionのadmissionはempty
Packed、zero-count Packedのstoreは元のRegionを返し、storageをdereferenceしない。`Packed<Unit>`はUSizeだけで表現してよい。

`value / count`はprefix、`value % count`はremainderを返す。Regionはstorageをdereferenceせずviewを分ける。Packedは同じbacking
storageを共有するslice viewを返す。operandは通常のexpressionと同じ順で一度だけ評価する。

## Symbol conversion

`Symbol`は`Packed<UInt8>`のaliasではない。prefix `*`はこの二型の間だけのclosed conversion familyである。
`*packed`は同じbytesのSymbol、`*symbol`は同じbytesのPackedを返す。両型はmal-owned byte storageのownerとviewを共有し、
変換は新しいownerを割り当てずにresultのshareを得る。operandとresultは変換後も独立に有効である。変換は
storage allocation、byte copy、allocation failureを追加しない。
external storageを直接ownerにするzero-copy Packed viewはない。

## Scoped constructionとediting

`pack`と`edit`はpredefined generic intrinsicである。`Representable(A)`を満たす型argumentを明示し、構築中だけ有効な
`Buffer<A>` authorityをcallbackへ渡す。`Buffer<A>`にはsource-level constructorがなく、`pack`または`edit`だけが作る。

```mal
pack<A> ::
    (Buffer<A> -> Unit)
    -> Packed<A>;

edit<A> ::
    (
        Packed<A>,
        Buffer<A> -> Unit
    )
    -> Packed<A>;
```

`pack<A>(callback)`はcount 0のbuilderを作る。`edit<A>(source, callback)`はsourceと同じ要素列とcountから始まるbuilderを
作る。`source.edit<A>(callback)`はreceiver-first applicationによる同じoperationである。callback parameter `buffer`について、
`buffer.new(value)`は末尾へ追加してその安定したindexを返し、`buffer.get(index)`は現在値を返し、
`buffer.put(index, value)`は現在値を置換する。これらはそれぞれ`new(buffer, value)`、`get(buffer, index)`、
`put(buffer, index, value)`というreceiver-firstでない同じoperationとしても書ける。element型はBuffer operandから決まり、
明示的なtype argumentを取らない。
先行するoperationの結果は後続のoperationから観測できる。

引数は通常のapplication順で一度ずつ評価する。`pack`ではcallbackを評価してからbuilderを作る。`edit`ではsource、callbackの順に
評価してからbuilderを作る。callbackが`Unit`で正常完了するとbuilderをfreezeし、count要素のowned `Packed<A>`を返す。
Bufferはhelper、nested closure、recursive frameへ渡せるが、callbackの正常完了後には到達できない。callback resultの`Unit`、
Mal内部に留まるfunction value、immutable capture、result binderのcapture規則がこのscopeを構成する。

`edit`のsourceとresultは独立したimmutable valueとして振る舞う。source、そのslice、および`A = UInt8`の場合にownerを共有する
`Symbol`は編集前と同じ値を返す。実装はcallbackの開始前または最初の変更時にwritable storageを確立し、変更がない場合はownerを共有してよい。
source responsibilityとruntime ownerがともに一意な場合のstorage再利用は、observable semanticsを変えないoptimizationである。

count、`count * stride(A)`、owner allocation sizeがtargetで表現できない場合と、必要なallocationのfailureはtrapする。
`Packed<Unit>`ではstorageを持たず、`new`の回数をcountへ加えてよい。growth policy、余剰capacity、copy-on-writeと再利用の選択は
implementation detailである。

## Partial I/O

external allocation、deallocation、failure、ownershipはprogram固有のextern contractが定める。host operationはAddress、ByteSize、
USize、またはoperation固有のconcrete contractを受け取り、layout shapeをABIへ渡さない。

partial inputはcapacity以下のUSizeと、そのprefixを初期化したというpostconditionを返す。partial outputはinput以下のUSizeと、
そのprefixを消費したというpostconditionを返す。mal codeはRegion/Packedをprefixとremainderへ分け、admitまたはretryする。
`0usize`がEOF、would-block、empty input、zero progressのどれかはoperation固有のcontractが定める。

## 未検査precondition

| Operation | Precondition |
|---|---|
| `Region / USize`、`Region % USize` | `count <= #region`かつprefix extentがoverflowしない |
| `Packed / USize`、`Packed % USize` | `count <= #packed` |
| `Region # USize` | `index < #region`かつoffset計算がoverflowしない |
| `Packed # USize` | `index < #packed` |
| scoped `buffer.get(index)`、`buffer.put(index, value)` | `index <` builderの現在count |
| `<-Region<A>` | 全locationがreadable、初期化済み、valid representationで、allocation sizeがtargetで表現可能 |
| `Region<A> <- Packed<A>` | `#packed <= #region`で対象prefixがwritable |
| host partial input | result USizeがcapacity以下で、そのprefixが初期化済み |
| host partial output | result USizeがinput以下で、そのprefixがconsumed済み |

primitiveはこれらを検査せず、違反時の結果を保証しない。host postcondition違反もtrusted contractへの違反である。
防御的trapはimplementation detailである。Unit elementはreadability、writability、initialization、storage extentを要求しない。

host operationへ渡せる型とbyte列をAddressで運ぶ規則は[`extern`](extern.md#host-mappable-type)を正とする。
