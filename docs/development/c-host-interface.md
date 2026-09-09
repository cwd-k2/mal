# C host interface再設計案

Status: Accepted design; implementation pending; current v0.5 ABIではない

この文書はreference C backendのhost-facing interface候補を定める。現在の規範は
[C host ABI](../spec/c-host-abi.md)、意味上のauthorityは[EngramとExtern](../spec/engrams.md)、実装中のownershipは
[C backendのEngram ownership](../implementation/ownership.md)を正とする。実装と規範ABIの切り替えが完了するまでは既存ABIを
変更しない。利用例は
[C host interface再設計例](c-host-interface-examples.md)に分ける。実装順、test、ABI切替条件は
[C host interface実装手順](c-host-interface-implementation.md)を正とする。

## Host authorのモデル

host authorが理解する概念を次へ限定する。

1. `MAL_DEFINE_<operation>`は一つのexternal operation bodyを定義する。
2. `mal_call_t`は現在の一回のcallについてMalへ依頼できるcapabilityである。
3. `mal_<T>_t`はMal型`T`をhost側で読み、組み合わせるtyped C valueである。
4. bodyは型に属する`return` operationでresultを確定してcallを完了する。

## Abstract host interface notation

host interfaceの意味は`T::operation(arguments...)`というabstract notationで記述する。これはMalまたはCのsyntaxではなく、
operationのowner、authority argument、意味上の結果をC spellingと分離して表す仮想的な言語である。

```text
Symbol::to_bytes(call, value)
Symbol::from_bytes(span)

Status::make_0()
Status::make_1(error)

ReceiveResult::return_0(call, packet)
ReceiveResult::return_1(call, error)

File::to_bits(file)
File::from_bits(bits)
Call::trap(call, message)
```

authorityを必要とするかはabstract notationでも`call` argumentを省略せずに示す。意味上の設計、規範、比較はこのnotationで行い、
C identifierとgenerated loweringは後段で定める。

例えば、意味上の`Int64::return(call, value)`はCで次へlowerする。

```c
MAL_DEFINE_increment(call, value) {
    return mal_Int64_return(call, value + INT64_C(1));
}
```

`form`、admission、observation、designation、raw carrier、materialization、retain、drop、loweringはhost authorの
基本語彙にしない。これらはauthorityの説明またはgenerated implementationの語彙である。

## Authority rule

Mal storageを観測するoperationとresultを確定するoperationだけが`mal_call_t *`を要求する。host valueのpure constructionと
Extern capabilityの表現変換は`mal_call_t`を要求しない。

```text
Symbol::to_bytes(call, value)
Symbol::from_bytes(span)

Status::make_1(error)
File::to_bits(file)
File::from_bits(bits)
```

`mal_call_t`が許すのは、現在のcallにおけるEngramの観測要求、declared resultの確定、trapによる異常終了だけである。
Engram identity、storage、lifetime authorityや、`Ptr`とexternal opaque valueのreferent authorityを取得しない。

## Host value

型からhost valueへのmappingを一つ定める。

```text
HostValue(Unit)       = mal_Unit_t
HostValue(IntN)       = mal_IntN_t
HostValue(UIntN)      = mal_UIntN_t
HostValue(FloatN)     = mal_FloatN_t
HostValue(Bool)       = mal_Bool_t
HostValue(Symbol)     = mal_Symbol_t
HostValue(Ptr)        = mal_Ptr_t = void *
HostValue(External X) = mal_X_t

HostValue((A, B)) = struct {
    HostValue(A) field_0;
    HostValue(B) field_1;
}

HostValue([A, B]) = struct {
    uint32_t tag;
    union {
        HostValue(A) variant_0;
        HostValue(B) variant_1;
    } payload;
}
```

function型はextern signatureから除外する。scalarは対応するC scalar、`mal_Ptr_t`はC `void *`へのtypedefとする。
`Ptr`はpointee typeを持たず、変換operationを追加しない。productとsumはfieldごとにmappingを再帰適用し、aggregate全体を
一つのownershipまたはauthority単位にしない。

```c
typedef struct {
    mal_UInt64_t field_0;
    mal_Symbol_t field_1;
} mal_Packet_t;

typedef struct {
    uint32_t tag;
    union {
        mal_Packet_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
} mal_ReceiveResult_t;
```

source aliasには`mal_<Alias>_t`を生成する。anonymous structural typeには`mal_repr_product_<id>_t`または
`mal_repr_sum_<id>_t`を生成し、raw型とstructural identityを共有する。source-level aliasだけをbody signatureと
host operationのspellingへ残す。

host valueは通常のC valueとしてlocal copy、product fieldの変更、再構成ができる。これは元のMal valueを変更せず、新しい
result valueを記述する。`Symbol`のprivate fieldとsum tagを直接構築せず、対応するgenerated operationを使う。

## Lifetime composition

host value全体へ一つのlifetimeを付けず、leafの規則を再帰適用する。

| Leaf | Contract |
|---|---|
| scalar | 通常のC copyとして保持できる |
| Mal由来`Symbol` value | current callを越えて使用しない |
| `Symbol::to_bytes`が返すspan | current callを越えて使用しない |
| `Symbol::from_bytes`へ渡すspan | 対応するresult `return`が完了するまで有効にする |
| `Ptr`、external opaque value | copyと保持は個別Extern contractに従う |
| `Ptr`、external opaque valueのreferent | carrierをcopyしてもlifetimeは延長しない |

aggregateをcall後にそのまま保持してはならない。保持が必要なscalarまたはExtern capability leafだけを取り出し、その
Extern contractに従う。

## Symbol

`mal_Symbol_t`はhost側で扱う一つのSymbol valueである。Mal由来かhost bytes由来かによらず同じ型とoperationを使う。

```c
typedef struct {
    const uint8_t *data;
    uint64_t length;
} mal_span_t;
```

`mal_span_t`はMal value、Engram、authorityまたはstorageではない。`const uint8_t *`と`uint64_t`を対応する一組として運ぶ
形だけに関心を持つ、汎用のnon-owning C support typeである。`mal_`はgenerated headerの名前衝突を避けるnamespace prefixに
すぎず、Mal固有の意味を示さない。span自体にlifetime延長またはcopyの意味はなく、受け渡すoperationが有効期間を定める。

```text
Symbol::to_bytes(call, value) -> span
Symbol::from_bytes(span) -> Symbol
```

`mal_Symbol_t`はby-valueでcopyできるが、fieldはopaque-by-contractとする。source discriminant、raw `MalType_Symbol`、
flat/rope、ownership pointer、capacity、admission stateをhost-facing contractへ出さない。C ABIのためcomplete representationが
headerに必要な場合も、generated private fieldとして扱う。

`Symbol::to_bytes`はMal storageを観測するため`mal_call_t`を要求する。必要な場合だけruntimeがmaterializeし、返した
`mal_span_t`はnon-owning spanである。`Symbol::from_bytes`はspanを記録するpure constructionで、allocationまたはcopyを
行わない。実際のbyte copyは対応するresult `return`中に完了する。

同じSymbolをresultの複数fieldへ置いてもhostはclone回数を扱わない。generated loweringがfieldごとに必要なEngram shareを作る。

## Product、sum、Bool

productはC compound literalで構築する。sumはvalid tagを保証するvariant-specific `make`をnested constructionに使う。

```text
Status::make_0() -> Status
Status::make_1(error) -> Status
```

top-level sum resultはvariant-specific `return`で構築する。

```text
ReceiveResult::return_0(call, packet)
ReceiveResult::return_1(call, error)
```

Mal由来のobserved sumはvalidだが、C `switch`の末尾は`Call::trap`のC realizationとする。nested sumのterminal loweringは
tagを検査してからactive payloadを読む。`Bool` resultも`mal_false`または`mal_true`だけをvalidとし、terminal loweringで
検査する。

## Result return

すべてのresult型でterminal `return` operationを必須にする。scalarと`Unit`も例外にしない。

```text
Int64::return(call, value)
Unit::return(call)
Packet::return(call, packet)
ReceiveResult::return_0(call, packet)
```

abstract `T::return`単体ではなく、それをlowerした`return mal_<T>_return...(call, value)`というC return expression全体が
boundary completionである。
host bodyはこの形でちょうど一度完了する。helperの結果を保存したり、helper後に処理を継続したりしない。

専用のpublic return carrier familyは生成しない。`MAL_DEFINE_<operation>`が隠すbodyのC return typeにはraw result carrierを
直接使い、host authorはそれを綴らない。`Unit`ではprivate bodyがinternal Unit carrierを返し、raw `void` wrapperが評価して
破棄する。result operationはhost local storageが有効な間に次を再帰的に完了する。

| Host value leaf | Raw lowering |
|---|---|
| scalar | validity検査後にidentity transport |
| product | fieldごとにraw aggregateを構成 |
| sum | tag検査後にactive variantだけを構成 |
| Mal由来`Symbol` | ownership shareを一つ作る |
| bytes由来`Symbol` | allocationとbyte copyを一回行う |
| external opaque value | capability bitsをtransport |
| `Ptr` | C `void *`としてidentity transport |

allocation failureがtrapする現在のruntimeでは、途中まで構成したresultをrecoverするframeは不要である。将来これをrecoverable
failureへ変える場合は別途rollback contractが必要になる。

## C realization

abstract notationをCへ次のようにlowerする。

| Abstract notation | C spelling |
|---|---|
| Mal型`T` | `mal_<T>_t` |
| `T::operation(arguments...)` | `mal_<T>_<operation>(arguments...)` |
| `T::operation_<variant>(arguments...)` | `mal_<T>_<operation>_<variant>(arguments...)` |
| `T::tag_<variant>` | `mal_<T>_tag_<variant>` |
| `Call::trap(call, message)` | `mal_call_trap(call, message)` |
| `span` | `mal_span_t` |

Cのhost-facing identifierを次へ統一する。

| 対象 | 形 |
|---|---|
| current call capability | `mal_call_t` |
| named host value | `mal_<Type>_t` |
| anonymous structural host value | `mal_repr_<kind>_<id>_t` |
| generic C support type | `mal_<name>_t` |
| type-specific operation | `mal_<Type>_<operation>` |
| sum variant operation | `mal_<Type>_<operation>_<variant>` |
| sum tag enum constant | `mal_<Type>_tag_<variant>` |
| current call operation | `mal_call_<operation>` |

固定fragmentはlowercase、`<Type>`と`<operation>`はMal sourceのspellingをそのまま保つ。caseを正規化して異なるsource identifierを
衝突させない。uppercase `MAL_` prefixは`MAL_DEFINE_<operation>`など実際のpreprocessor macroだけに使う。

使用する主要動詞は次に限定する。

- `to_bytes`: Symbolをnon-owning byte spanとして観測する。
- `from_bytes`: byte spanからSymbol host valueをpureに構築する。
- `make_<variant>`: nested sum host valueをpureに構築する。
- `return[_<variant>]`: declared resultを確定してC return expressionを構成する。
- `to_bits` / `from_bits`: Extern capability carrierとhost representationをpureに変換する。
- `trap`: current callを異常終了する。

`Observe` namespace、`form`、`commit`、`clone`、`move`、`drop`をhost-facing familyとして追加しない。

## Generated wrapperと実装責務

`MAL_DEFINE_<name>`は同じhost translation unitへcompiler-facing raw wrapperと`static` host bodyを定義する。概念上は次になる。

```c
static MalType_ReceiveResult mal_detail_receivePacket(
    mal_call_t *call,
    mal_Socket_t socket
);

MalType_ReceiveResult mal_ext_receivePacket(
    MalContext *context,
    MalType_Socket raw_socket
) {
    mal_call_t call = { .context = context };
    return mal_detail_receivePacket(
        &call,
        mal_detail_Socket_from_raw(raw_socket)
    );
}
```

bodyのraw return type、`mal_ext_*`、parameter flattening、raw conversionはmacro expansion内のgenerated detailであり、
host-facing extension pointではない。lazy Symbol observationへ移行すると現v0.5の「call前に全Symbolをmaterializeする」raw ABIと
互換にならないため、`mal_ext_*`をcompilerとgenerated wrapper間のinternal ABIへ降格し、direct-definition escape hatchを
削除する。

既存実装からの移行責務は次になる。

- [`TypeRegistry`](../../compiler/src/c_emit/types/mod.rs)がraw型に加えて`HostValue(T)`のC型を管理する。
- [host type collection](../../compiler/src/c_emit/types/collect.rs)がraw型とhost valueでstructural identityを共有する。
- [product](../../compiler/src/c_emit/types/host/product.rs)と[sum](../../compiler/src/c_emit/types/host/sum.rs)のhelperをhost valueの
  construction、observation、terminal loweringへ組み替える。
- [extern call emission](../../compiler/src/c_emit/body/expression/mod.rs)にあるcall前の再帰的Symbol materializationを除き、raw
  designationをgenerated wrapperへ渡す。
- [managed lifetime generation](../../compiler/src/c_emit/types/lifetime.rs)はraw resultのcopy/destroyを引き続き所有する。
- runtimeは`mal_Symbol_to_bytes`からlazy materializerを呼べるinternal linkageを提供する。

一般のtranslation boundaryは[compiler responsibilities](../implementation/responsibilities.md)を正とする。

## Extern cleanup

`mal_call_t`はdynamic value graph、owner list、generic cleanup stackを持たない。Extern resource取得後からresult transferまでのcleanupは
operation固有のhost contractに残す。Engram cleanupとExtern resource cleanupを同じauthorityへ統合しない。

## Performance conditions

- scalar-only externにheap allocation、retain、release、materializationを追加しない。
- productとsumのhost value変換にgeneric runtime nodeを追加しない。
- bytesを要求しないrope Symbolをmaterializeしない。
- Mal由来Symbolのpassthroughはbyte copyせず、必要なshare一回以内にする。
- host bytesから返すSymbolはallocation一回とbyte copy一回以内にする。
- host valueのSymbol leafがraw carrierより大きくなるcostとaggregate copyをClang `-O2`生成物で測定する。
- wrapper call、binary size、wall-clock、materialization count、終了時live allocationを既存suiteで比較する。

optional direct Symbol outputは基本経路のcopyが独立したcost centerだと測定された場合だけ検討する。unfinished outputのcleanup stateが
必要になるため、statelessな基本`mal_call_t`へ先に組み込まない。

検証は[test policy](testing.md)と[generated program最適化計画](generated-program-optimization.md)に従い、generated header、native C
adapter、invalid Bool/sum、failure path、materialization counter、ASan/UBSan pressure caseを同時に更新する。

## 非目標

- callback、re-entry、asyncまたはMal由来Symbol viewのcall越しretention
- external opaque resourceの自動close/free
- C type systemによるtrusted adapterの完全な検証
- generated header version間のABI compatibility
- named product fieldまたはsum variantの推測
- managed runtimeをregion、arenaまたはtracing方式へ固定すること

## 実装前に測定または試作する事項

1. `mal_Symbol_t`のprivate complete representationとaggregate size。
2. raw wrapperとhost bodyを含むgenerated macroがClangのdiagnostic、debugger、formatterで許容できるか。
3. wrapper、host body、type-directed conversionが`-O2`でどこまで消えるか。
4. optional direct Symbol outputが独立した効果を持つか。持たなければ導入しない。
