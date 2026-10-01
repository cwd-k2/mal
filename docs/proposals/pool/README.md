# Poolとopaque型によるcontainer基盤

Status: Exploratory

この文書は、現在の`Buffer<T>`が一つの組み込み型として持つstorageの仕組みとsequenceの方針を分け、少数のtrustedなprimitiveの上で
containerをmal sourceとして定義する案の要点を管理する。現行仕様は[AddressとBuffer](../../spec/memory.md)、managed responsibilityは
[D055](../../history/decisions/active/D055.md)と[D083](../../history/decisions/active/D083.md)を正とする。本案はまだsource syntax、
API、plugin ABIを定めない。

## 目的

現在の`Buffer<T>`は、mal-owned storage、共有mutable identity、denseな`[0, count)`、append、index access、growth、range operationを
一つの組み込み型として持つ。Map、Deque、heap、木を書くには、Bufferの全slotが値を持ちcountが増えるだけという前提へ、空きを表す
番兵値やtombstoneでcontainerの方針を押し込むことになる（[container](containers.md#bufferの前提を外すと変わること)）。

本案は、malが所有するstorageの仕組みを少数のprimitiveとして切り出し、Bufferを含むcontainerをその上にmalで定義する。layout、
lifecycle、allocation failureはtrusted layerに残し、State、growth、free list、順序、hashはcontainerが選ぶ。利用者へraw storage、
未初期化の値、手動の`drop`は公開しない。

## 要点

本案は次の三つからなる。

1. storageのprimitiveは、変更の扱いが異なる対である。`IxPool<State, T>`はidentityを共有してその場で書き換え、coordinateで引く
   slotの線形空間と、slotごとのLiveとVacantを持つ。`ImPool<State, T>`はidentityを持たず、更新のたびにsuccessorを返す。
2. containerは、この対の上にmalで書く。BufferはIxPoolに「Liveなslotは`[0, count)`」というinvariantを課したものであり、
   hostと`Symbol`との交換だけを自分のprimitiveとして持つ。Map、Deque、heap、木もIxPool上に、immutable arrayはImPool上に置く。
3. IxPoolのslotに関する条件は未検査のpreconditionであり、IxPoolを直接呼べるのは[file-local opaque type](../../spec/types.md#file-local-opaque-type)
   を宣言したfileだけである。containerの実装がinvariantでその条件を満たし、利用者はcontainerの公開preconditionだけを見る。

この対は、一つのLiveなrunへ特化した型と、byte列へ特化した既存の型でも同じ形を取る。現在のBufferと`Symbol`は、この対を
byte列に特化して既に実装したものに当たる。

| | identityを共有する | 値 |
|---|---|---|
| primitive | IxPool | ImPool |
| 一つのLiveなrun | Buffer（`[0, count)`） | Array（`[0, length)`） |
| byte列 | `Buffer<UInt8>` | `Symbol` |

対の間は`freeze`と`thaw`で変換し、byte列ではこれが`Buffer<UInt8>`と`Symbol`の間の`*`に当たる。

ImPoolはIxPoolの上に書けない。storageを再利用できるかは参照数で決まり、malは参照数をsourceへ見せないためである。BufferはIxPoolの
上に追加のcostなしに書けるが、逆はslotごとのsum tagと移動ごとの`Share`と`Drop`を払う（[試作](prototypes.md#ixpoolとbufferの非対称)）。
このため、仕組みの層ではIxPoolがBufferより基本的である。

## 設計の軸

本案が決める事項は六つの軸に分かれる。

| 軸 | 問い | 本案の答え | 所有する文書 |
|---|---|---|---|
| 所有権 | 誰がresponsibilityを持つか | slotへの`init`と`take`、operand effect `Store` | [runtime contract](runtime.md#所有権) |
| lifecycle | carrierをいつ確保、移動、解放するか | `reserve`、IxPool終了、型別glue | [runtime contract](runtime.md#runtime-representation) |
| identity | 変更を誰が観測し、何を`Storable`にできるか | IxPoolとImPool | [identity](identity.md) |
| 妥当性 | どのslotがLiveで、誰がそれを保証するか | 未検査preconditionとcontainer invariant | [runtime contract](runtime.md#未検査precondition) |
| 権限 | 誰がIxPoolへ直接触れるか | file-local opaque型の宣言元file | 本書 |
| 表現 | どのbitで保持し、hostとどう交換するか | runtime representationとBufferのprimitive | [runtime contract](runtime.md#runtime-representation) |

妥当性と権限は対になる。IxPoolのpreconditionを未検査にできるのは、IxPoolを直接呼ぶfileをopaque型で一つに閉じ込め、そのfileの
invariantでpreconditionを満たせるからである。

```mal
opaque Buffer<T> :: IxPool<USize, T>;
```

hidden representationを観察できるのは宣言元fileだけであり、この権限は`require`先へ移らない。同じfileでは`Buffer<T>`をそのまま
IxPool operationへ渡せ、このviewはallocation、copy、新しいidentityを作らない。opaque型はgeneric specializationとoperation familyの
keyにdeclaration identityを残す。element equality、hash、orderingはIxPoolへ埋め込まず、operation familyのrequirementとして
containerが要求する。

## 文書の構成

- [primitive一覧](primitives.md)：IxPoolとBufferの責務分担、IxPool、Buffer、ImPoolのprimitive、`freeze`と`thaw`
- [identity](identity.md)：所有権とidentityの関係、`Storable`の条件、`Storable`と`Stable`の分割案
- [runtime contract](runtime.md)：representation、所有権の遷移、未検査precondition、compilerとruntimeの分担、検証の段階
- [IxPool上のcontainer](containers.md)：containerの比較、Bufferの前提を外すと変わること、IxPoolの輪郭
- [Buffer実装](buffer-implementation.md)：BufferをIxPoolの上に書いた参照実装と、現行Bufferとの差分
- [collection例](collection-examples.md)：stack、binary heap、open addressing Map、SlotMap、木、immutable array
- [試作で確かめたこと](prototypes.md)：C host試作とBuffer上のemulationの結果

## 非目標

- raw allocation、pointer arithmetic、manual `init`、manual `drop`、`free`を一般mal codeへ公開しない。
- IxPoolだけを理由にborrow checker、linear type、user-defined finalizer、tracing GCを導入しない。
- `Representable(T)`やpublic C carrierを`Storable(T)`から自動的に導かない。
- plugin ABI、dynamic loading、package system、friend fileをIxPoolの初期実装条件にしない。
- 現在のBufferを、semantic parityと代表的なperformance測定なしに置き換えない。

## 未決定事項

- IxPoolを使えるfileを、どのfileにも開くか、標準libraryとtrustedなcodeだけに限るか。後者では利用者はBufferと標準の
  containerだけを見る。
- primitiveの名前と、IxPoolの名前を`require`したfileだけへ導入する規則。
- [ImPool](primitives.md#impool)をIxPoolと対のprimitiveとして持つか。持つ場合のAPIと、uniqueness検査をruntimeへ置く範囲。
- [freezeとthaw](primitives.md#freezeとthaw)でstorageを共有するか。共有するとIxPoolへの書き込みのたびに共有中かの確認が入る。
- [`Storable`と`Stable`の分割案](identity.md#判定の分割案)。採ると`Buffer<Buffer<T>>`やIxPoolの入れ子を書ける。
  [D075](../../history/decisions/active/D075.md)の見直しを伴う。
- [測定後の候補](primitives.md#測定後の候補)の`moveRange`、IxPoolとhostの直接の交換、ImPoolの範囲の写しを足すか。
- live slot iterationをcoreに持つか、core外のextensionにするか、containerに任せるか。
- Vacantを末尾だけに限ったdense primitiveへ`[Unit, T]`を載せる形との比較。意味は同じであり、占有metadataとIxPool終了時の
  走査に対する、slotごとのsum tagと移動ごとの`Share`と`Drop`のcostは測っていない。
- opaque型のdiagnosticと、public APIがrepresentationを返せる範囲。
- IxPool callbackを既存Buffer callbackから一般化するか、共通lifecycle planを先に抽出するか。
- plugin crateのversion、reproducible build、artifact cache、runtime source選択のcontract。
