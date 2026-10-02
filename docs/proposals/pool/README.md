# Poolとopaque型によるcontainer基盤

Status: Exploratory

この文書は、現在の`Buffer<T>`が一つの組み込み型として持つstorageの仕組みとsequenceの方針を分け、少数のtrustedなprimitiveの上で
containerをmal sourceとして定義する案の要点を管理する。現行仕様は[AddressとBuffer](../../spec/memory.md)、managed responsibilityは
[D055](../../history/decisions/active/D055.md)と[D083](../../history/decisions/active/D083.md)を正とする。

## 目的

現在の`Buffer<T>`は、mal-owned storage、共有mutable identity、denseな`[0, count)`、append、index access、growth、range operationを
一つの組み込み型として持つ。Map、Deque、heap、木を書くには、Bufferの全slotが値を持ちcountが増えるだけという前提へ、空きを表す
番兵値やtombstoneでcontainerの方針を押し込むことになる（[container](containers/overview.md#bufferの前提を外すと変わること)）。

本案は、malが所有するstorageの仕組みを少数のprimitiveとして切り出し、Bufferを含むcontainerをその上にmalで定義する。layout、
lifecycle、allocation failureはtrusted layerに残し、Meta、growth、free list、順序、hashはcontainerが選ぶ。
responsibilityはMetaとslotというplaceの値の入れ替えで動かす。

## 要点

本案は次の三つからなる。

1. storageのprimitiveは、変更の扱いが異なる対である。`IxPool<Meta, V>`はidentityを共有してその場で書き換え、Metaと、coordinateで
   引く`Slot<V> :: [Unit, V]`の線形空間を持つ。`ImPool<Meta, V>`は同じ状態を値として持ち、更新のたびにsuccessorを返す。核は`pool`、
   `grow`、`capacity`、`peek`、`swap`、`meta`、`swapMeta`である（[Poolの意味論](model/semantics.md)）。
2. containerは、この対の上にmalで書く。BufferはIxPoolに「Liveなslotは`[0, count)`」というinvariantを課したものであり、
   自分のprimitiveを持たない。hostとの交換は値の列であるVectorが持ち、Bufferはそれを経由して書く。Map、Deque、
   heap、木もIxPool上に、値の列であるVectorはImPool上に置く。
3. 核の未検査preconditionはcoordinateの範囲だけであり、LiveかVacantかを仮定する周辺operationがそれを加える。どちらもBufferと
   同じ未検査のpreconditionであり、primitiveはどのfileからも呼べる。
   containerの実装がinvariantでその条件を満たし、利用者はcontainerの公開preconditionだけを見る。

この対は、一つのLiveなrunへ特化した型と、byte列へ特化した既存の型でも同じ形を取る。現在のBufferと`Symbol`は、この対を
byte列に特化して既に実装したものに当たる。

| | identityを共有する | 値 |
|---|---|---|
| primitive | IxPool | ImPool |
| 一つのLiveなrun | Buffer（`[0, count)`） | Vector（`[0, length)`） |
| byte列 | `Buffer<UInt8>` | `Symbol` |

対の間は`freeze`と`thaw`で変換し、byte列ではこれが`Buffer<UInt8>`と`Symbol`の間の`*`に当たる。

ImPoolはIxPoolの上に書けない。storageを再利用できるかは参照数で決まり、malは参照数をsourceへ見せないためである。IxPoolとBufferは
互いの上に書けるが、Buffer上のIxPoolがslotごとのsum tagと移動ごとの`Share`と`Drop`を払うのに対し、IxPool上のBufferが払うのは
占有tagの更新と終了時の走査である（[試作](prototypes.md#ixpoolとbufferの非対称)）。そこで意味の層ではIxPoolをBufferより
基本的なものとし、BufferとVectorはpreludeに置いて参照実装で意味を定め、as-ifで実装する（[BufferとVector](api/buffer-vector.md#bufferとvectorの対)）。
占有tagの費用はIxPoolを直接使うcontainerだけが払う（[占有tagの費用](runtime/implementation.md#占有tagの費用)）。

containerが現行runtimeと同じtrapをmalで起こすため、本案は[primitive `trap`](../primitive-trap.md)の導入を前提とする。

## 設計の軸

本案が決める事項は六つの軸に分かれる。

| 軸 | 問い | 本案の答え | 所有する文書 |
|---|---|---|---|
| 所有権 | 誰がresponsibilityを持つか | placeのread、write、swap、operand effect `Store` | [runtime contract](runtime/contract.md#所有権) |
| lifecycle | carrierをいつ確保、移動、解放するか | `grow`、IxPool終了、型別glue | [runtime contract](runtime/contract.md#runtime-representation) |
| identity | 変更を誰が観測し、何を`Storable`にできるか | IxPoolとImPool | [identity](model/identity.md) |
| 妥当性 | どのslotがLiveで、誰がそれを保証するか | `Slot<V>`の除去、または周辺operationの未検査preconditionとcontainer invariant | [runtime contract](runtime/contract.md#未検査precondition) |
| 権限 | 誰がIxPoolへ直接触れるか | どのfileも。opaque型はcontainerのinvariantを宣言元fileへ閉じる | 本書 |
| 表現 | どのbitで保持し、hostとどう交換するか | runtime representationとVectorのprimitive | [runtime contract](runtime/contract.md#runtime-representation) |

IxPoolはどのfileからも使え、そのpreconditionを未検査にするのは現行Bufferの未検査preconditionと同じ選択である。
containerは[file-local opaque type](../../spec/types.md#file-local-opaque-type)でrepresentationを隠すことで、
IxPool preconditionを満たす責任を宣言元fileのinvariantへ集め、利用者へ公開preconditionだけを見せられる。

```mal
opaque Buffer<T> :: IxPool<USize, T>;
```

hidden representationを観察できるのは宣言元fileだけであり、この権限は`require`先へ移らない。同じfileでは`Buffer<T>`をそのまま
IxPool operationへ渡せ、このviewはallocation、copy、新しいidentityを作らない。opaque型はgeneric specializationとoperation familyの
keyにdeclaration identityを残す。element equality、hash、orderingはIxPoolへ埋め込まず、operation familyのrequirementとして
containerが要求する。

## 文書の構成

文書は読む目的ごとにdirectoryへ分ける。

- `model/`：Poolが何を意味するか
  - [Poolの意味論](model/semantics.md)：三つの層、placeと値、最小核の導出、Metaの位置づけ、malの設計方針との対応
  - [responsibilityの図](model/responsibility.md)：slotの状態遷移、responsibilityの動き、費用が違うoperation、ImPoolの更新
  - [identity](model/identity.md)：所有権とidentityの関係、`Storable`の条件、`Storable`と`Stable`の分割案
- `api/`：primitive
  - [Pool primitive](api/pool.md)：区分、IxPoolの核と周辺、ImPool、`freeze`と`thaw`、測定後の候補
  - [BufferとVector](api/buffer-vector.md)：語彙の分担、BufferとVectorの対、Buffer、Vectorとhostとの交換
- `runtime/`：trusted layerの契約と実装
  - [runtime contract](runtime/contract.md)：representation、所有権の遷移、未検査precondition
  - [compilerとruntimeの実装](runtime/implementation.md)：compilerとruntimeの分担、検証の段階
- `containers/`：Poolの上のcontainer
  - [IxPool上のcontainer](containers/overview.md)：containerの比較、Bufferの前提を外すと変わること、IxPoolの輪郭
  - [Buffer実装](containers/buffer.md)：BufferをIxPoolの上に書いた参照実装と、現行Bufferとの差分
  - [列のcontainer](containers/sequences.md)：stack、Deque、binary heap、Vector
  - [keyで引くcontainer](containers/keyed.md)：open addressing Map、SlotMap、木
- [試作で確かめたこと](prototypes.md)：C host試作とBuffer上のemulationの結果

## 未決定事項

- IxPoolとImPoolをsourceへ見せるか、見せる場合にpreludeへ常に置くか、`builtin "ixpool";`のように宣言したfileだけへ導入するか。
  後者は組み込みmoduleの提供という仕組みを[program](../../spec/programs.md)へ新たに持ち込む。直接見せる場合、IxPoolとImPoolは
  containerを書く利用者のAPIになるため、名前、signatureの形、周辺operationの集合をAPIとして設計し直す。
  [試作](prototypes.md#二つの試作)では、更新がpoolを返す形にsignatureを揃え、constructorをkeyに持つ
  [operation family](../../spec/operation-families.md)で一つの名前にまとめられ、containerを一度書けば値版も得られた
  （[入れ子](model/identity.md#入れ子)）。ImPoolを見せる場合は、uniqueness検査をruntimeへ置く範囲も決める。
- [Meta](model/semantics.md#metaをpoolに置く理由)をPoolに融合したまま持つか、容量1のPoolとの組へ分離するか。ImPoolのMetaはproductで足りる。
- 核と周辺の名前、特にMetaの呼び方。周辺operationのうちどれを費用primitiveとして持つか、Liveを仮定する除去を
  `unreachable :: Unit -> []`のような言語のprimitiveへ寄せるか。
- [`Storable`と`Stable`の分割案](model/identity.md#判定の分割案)。採ると`Buffer<Buffer<T>>`やIxPoolの入れ子を書ける。
  [D075](../../history/decisions/active/D075.md)の見直しを伴う。値の入れ子とcoordinateの入れ子で足りない
  用途が見つかるまで採らずにおける（[入れ子](model/identity.md#入れ子)）。
- [測定後の候補](api/pool.md#測定後の候補)の`moveRange`とImPoolの範囲の写しを足すか。
- `Symbol`を`Vector<UInt8>`とどこまで同一視するか。`*`による`Symbol`との変換も、`from`と`into`と同じくBufferからVectorへ寄せるか。
- live slot iterationをcoreに持つか、core外のextensionにするか、containerに任せるか。
- IxPoolを直接使うcontainerが払う[占有tagの費用](runtime/implementation.md#占有tagの費用)。測っておらず、大きい場合はtagの表現を見直す。
- opaque型のdiagnosticと、public APIがrepresentationを返せる範囲。
- IxPool callbackを既存Buffer callbackから一般化するか、共通lifecycle planを先に抽出するか。
- plugin crateのversion、reproducible build、artifact cache、runtime source選択のcontract。
