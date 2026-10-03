# Poolとopaque型によるcontainer基盤

Status: Exploratory

この文書は、現在の`Buffer<T>`が一つの組み込み型として持つEngram storageのauthorityとsequence policyを分け、少数のtrustedな
primitiveの上でcontainerをmal sourceとして定義する案の入口である。Poolの言語全体での位置とminimalityは
[位置付けと根本モデル](model/foundations.md)、状態とoperationの核は[意味論](model/semantics.md)を正とする。現行仕様は
[AddressとBuffer](../../spec/memory.md)、managed responsibilityは[D055](../../history/decisions/active/D055.md)と
[D083](../../history/decisions/active/D083.md)を正とする。

## 目的

現在の`Buffer<T>`は、mal-owned storage、共有mutable identity、denseな`[0, count)`、append、index access、growth、range operationを
一つの組み込み型として持つ。Map、Deque、heap、木を書くには、Bufferの全slotが値を持ちcountが増えるだけという前提へ、空きを表す
番兵値やtombstoneでcontainerの方針を押し込むことになる（[container](containers/overview.md#bufferの前提を外すと変わること)）。

本案は、既存Bufferがすでに持つmal-ownedな共有identityを、有限個のtyped placeを持つPool stateとして一般化する。layout、物理的な
allocation、relocationはtrusted implementationに残し、Liveなcoordinateの形、growth policy、free list、順序、hashはcontainerが
選ぶ。responsibilityはHeaderとslotというplaceのreadとexchangeから導く。

## 要点

本案は次の五つからなる。

1. 根は`PoolState<Header, V> = (header, logical capacity, slots)`である。Headerと各slotはtyped placeであり、slotは
   `Slot<V> :: [Unit, V]`を持つ。placeの核はreadとexchange、構造の核はconstruction、capacity observation、extensionである。
2. `IxPool<Header, V>`と`ImPool<Header, V>`はどちらもsource valueであり、別々のstorage modelではない。IxPool valueはshared
   identityへのhandleで、どのhandleからの更新も同じstateを変更する。ImPool valueはstateのstructural snapshotで、更新前の
   Meta、capacity、slot carrierを変えずsuccessorを返す。slot carrierがhandleなら、そのreferentの変更は共有観測する。
3. containerは、この対の上にmalで書く。BufferはIxPoolに「Liveなslotは`[0, count)`」というinvariantを課したものであり、
   VectorはImPoolへ同じdense sequence invariantを課したものである。hostとの交換はVectorを正規形とする候補だが、現行Bufferの
   `from`と`into`を残すかは別に決める。Map、Deque、heap、木はIxPool上に置く。
4. handleもplaceへ保存できるvalueである。`Storable`はimmutable valueの分類でなく、typed placeがcarrier lifecycleを完結できる
   ことを表す。Pool採択時には現行Bufferも同じ判定へ揃え、BufferやIxPoolの入れ子を認める。
5. 核の未検査preconditionはcoordinateの範囲だけであり、LiveかVacantかを仮定する周辺operationがそれを加える。どちらもBufferと
   同じ未検査のpreconditionであり、primitiveはどのfileからも呼べる。
   containerの実装がinvariantでその条件を満たし、利用者はcontainerの公開preconditionだけを見る。

この五点を支える境界は、authority、responsibility、representationを分けることである。source valueは再利用可能なまま、compilerは
実行時responsibilityをaffineに移し、runtimeは物理storageの一意性を観測不能な最適化に使う。Rustの`Box`に似た単一ownerはlowering上の
状態として使えるが、IxPoolやImPoolに加える第三のsource authorityではない。

利用者向けの語彙は三つに絞る。handleはshared identityを観測するsource value、snapshotは更新前の構造を保存するsource value、
successorはsnapshotの構造を変えずに更新後のstateを表す新しいsnapshotである。同じcarrierを別のbindingへ渡すことをcopyとは呼ばず、
値またはstorageを実際に複製する場合だけcopyと呼ぶ。

snapshotはdeep immutabilityを意味しない。保存したslot carrierがhandleなら、旧snapshotとsuccessorは同じreferent authorityを
持ち得る。外側のslot置換は互いに独立し、handle先の変更は共有される。このstructural snapshotを型形成の根にし、推移的な
stabilityはそれを必要とするAPIだけが別途要求する。

この対は、一つのLiveなrunへ特化した型と、byte列へ特化した既存の型でも同じ形を取る。現在の`Buffer<UInt8>`と`Symbol`の
snapshot変換は、この対をbyte列に限定して先に実装したものとみなせる。

| | handle value | snapshot value |
|---|---|---|
| primitive | IxPool | ImPool |
| 一つのLiveなrun | Buffer（`[0, count)`） | Vector（`[0, length)`） |
| byte列 | `Buffer<UInt8>` | `Symbol` |

対の間は`freeze`と`thaw`で変換し、byte列ではこれが`Buffer<UInt8>`と`Symbol`の間の`*`に当たる。

ImPoolのsnapshot semanticsは、更新ごとにstate全体を複製してsuccessorを作る形でも実装できるが、writable successorによるstorage
再利用はruntimeだけが実装できる。再利用可能性をsourceへ返さず、更新前のsnapshotを変えないas-if ruleに閉じる。IxPoolとBufferは
互いの上に書けるが、Buffer上のIxPoolがslotごとのsum tagと移動ごとの`Share`と`Drop`を払うのに対し、IxPool上のBufferが払うのは
占有tagの更新と終了時の走査である（[試作](prototypes.md#ixpoolとbufferの非対称)）。そこで意味の層ではIxPoolをBufferより
基本的なものとし、BufferとVectorはpreludeに置いて参照実装で意味を定め、as-ifで実装する（[BufferとVector](api/buffer-vector.md#bufferとvectorの対)）。
占有tagの費用はIxPoolを直接使うcontainerだけが払う（[占有tagの費用](runtime/implementation.md#占有tagの費用)）。

containerが現行runtimeと同じtrapをmalで起こすため、本案は[primitive `trap`](../primitive-trap.md)の導入を前提とする。

## authorityとimplementationの境界

PoolはEngramであり、public C ABIへ渡すExtern resourceではない。C runtimeのallocatorはPoolのbacking storageを実装するが、allocator、
pointer、physical capacity、layoutをsourceへ公開しない。IxPoolのsemantic identity、managed runtime object、current backing allocation、
coordinate、slot valueは互いに異なる。`grow`はsemantic identityと既存coordinateを保存するが、backing allocationとslot addressを
保存しない（[位置付けと根本モデル](model/foundations.md#cとllvmとの境界)）。

## 設計の軸

本案が決める事項は七つの軸に分かれる。

| 軸 | 問い | 本案の答え | 所有する文書 |
|---|---|---|---|
| authority | 同じcarrierを別bindingへ渡した後の更新を誰が観測するか | IxPool handleとImPool snapshot | [位置付けと根本モデル](model/foundations.md#authorityresponsibilityrepresentation) |
| responsibility | 誰がcarrierを保持するか | placeのread、write、exchange、operand effect `Store` | [runtime contract](runtime/contract.md#responsibility) |
| lifecycle | carrierをいつ確保、移動、解放するか | `grow`、IxPool終了、型別glue | [runtime contract](runtime/contract.md#runtime-representation) |
| 型形成 | どのcarrierをmanaged placeやhost境界へ置けるか | `Storable`をplace lifecycleへ純化し、host境界は`Representable`と`HostMappable`を保つ | [identity](model/identity.md) |
| 妥当性 | どのslotがLiveで、誰がそれを保証するか | `Slot<V>`の除去、または周辺operationの未検査preconditionとcontainer invariant | [runtime contract](runtime/contract.md#未検査precondition) |
| 権限 | 誰がIxPoolへ直接触れるか | どのfileも。opaque型はcontainerのinvariantを宣言元fileへ閉じる | 本書 |
| 表現 | どのbitで保持し、hostとどう交換するか | runtime representationとdense sequenceのhost operation | [runtime contract](runtime/contract.md#runtime-representation) |

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
  - [位置付けと根本モデル](model/foundations.md)：authority、minimality、C/LLVM・Rust・Haskellとの比較、採択条件
  - [Poolの意味論](model/semantics.md)：Pool state、place、source carrierの観測、operation law、最小核の導出
  - [responsibilityの図](model/responsibility.md)：slotの状態遷移、responsibilityの動き、費用が違うoperation、ImPoolの更新
  - [identity](model/identity.md)：handle保存、structural snapshot、`Storable`、stability、host判定の境界
- `api/`：primitive
  - [Pool primitive](api/pool.md)：区分、IxPoolの核と周辺、ImPool、`freeze`と`thaw`、測定後の候補
  - [BufferとVector](api/buffer-vector.md)：語彙の分担、BufferとVectorの対、Buffer、Vectorとhostとの交換
- `runtime/`：trusted layerの契約と実装
  - [runtime contract](runtime/contract.md)：authority boundary、semantic identityとallocation object、representation、responsibility、precondition
  - [compilerとruntimeの実装](runtime/implementation.md)：compiler、LLVM、C runtimeの分担、検証の段階
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
  [operation family](../../spec/operation-families.md)で一つの名前にまとめられ、containerを一度書けばsnapshot版も得られた
  （[入れ子](model/identity.md#入れ子)）。ImPoolを見せる場合は、uniqueness検査をruntimeへ置く範囲も決める。
- [Header](model/semantics.md#headerをpoolに置く理由)をPoolに融合したまま持つか、別identityとの組へ分離するか。ImPoolのHeaderは
  productで足りるが、Ix/Imで同じstate algebraとcontainer invariantを使う利点との比較になる。
- 核と周辺の名前、特にMetaの呼び方。周辺operationのうちどれを費用primitiveとして持つか、Liveを仮定する除去を
  `unreachable :: Unit -> []`のような言語のprimitiveへ寄せるか。
- [`Storable`の拡張](model/identity.md#storableの原理)のうち、Buffer handleは
  [D096](../../history/decisions/active/D096.md)で、external opaque carrierは
  [D097](../../history/decisions/active/D097.md)で先に採択した。残るdecisionはPool採択時のIxPool、
  ImPool admissionである。functionとempty sumは今回の範囲では除外する。
- transitive snapshot、serialization、Map keyなどに共通するstability judgmentが実際に必要か。ImPoolのstructural snapshotと
  handle nestingには不要なので、名称だけを先に追加しない。
- [測定後の候補](api/pool.md#測定後の候補)の`moveRange`とImPoolの範囲の写しを足すか。
- Vectorの公開API、現行Bufferのhost operationとの互換性、`Symbol`との型関係は
  [BufferとVectorの採択前に残る判断](api/buffer-vector.md#採択前に残る判断)を正とする。
- live slot iterationをcoreに持つか、core外のextensionにするか、containerに任せるか。
- IxPoolを直接使うcontainerが払う[占有tagの費用](runtime/implementation.md#占有tagの費用)。測っておらず、大きい場合はtagの表現を見直す。
- opaque型のdiagnosticと、public APIがrepresentationを返せる範囲。
- IxPool callbackを既存Buffer callback emitterから一般化するか、共通`Lifecycle`を消費する別のemitterとして置くか。
- plugin crateのversion、reproducible build、artifact cache、runtime source選択のcontract。
