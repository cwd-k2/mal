# Poolとopaque型によるcontainer基盤

Status: Exploratory; extern integration sections require a v0.7 rebase

> [!IMPORTANT]
> Poolのidentity、place、container semanticsの検討は継続するが、このdirectoryに残る`Address`、`Representable`、
> `HostMappable`、canonical host copyを前提としたhost integration案は[D098](../../history/decisions/active/D098.md)で廃止された。
> 次のPool試作はruntime carrierを直接扱う`extern`と`mal.h`のmanaged Buffer callback上で行い、旧境界を実装要件に使わない。

この文書は、現在の`Buffer<T>`が一つの組み込み型として持つEngram storageのauthorityとsequence policyを分け、少数のtrustedな
primitiveの上でcontainerをmal sourceとして定義する案の入口である。Poolの言語全体での位置とminimalityは
[位置付けと根本モデル](model/foundations.md)、状態とoperationの核は[意味論](model/semantics.md)を正とする。現行仕様は
[`Buffer`](../../spec/memory.md)、managed responsibilityは[D055](../../history/decisions/active/D055.md)と
[D083](../../history/decisions/active/D083.md)を正とする。

## 目的

現在の`Buffer<T>`は、mal-owned storage、共有mutable identity、denseな`[0, count)`、append、index access、growth、range operationを
一つの組み込み型として持つ。Map、Deque、heap、木を書くには、Bufferの全slotが値を持ちcountが増えるだけという前提へ、空きを表す
番兵値やtombstoneでcontainerの方針を押し込むことになる（[container](containers/overview.md#bufferの前提を外すと変わること)）。

本案は、既存Bufferがすでに持つmal-ownedな共有identityを、有限個のtyped placeを持つPool stateとして一般化する。layout、物理的な
allocation、relocationはtrusted implementationに残し、Liveなcoordinateの形、growth policy、free list、順序、hashはcontainerが
選ぶ。responsibilityはHeaderとslotというplaceのreadとexchangeから導く。

## 全体像

- Poolのauthority、responsibility、representationと既存Bufferから分離する理由は
  [位置付けと根本モデル](model/foundations.md)が所有する。
- `PoolState<Header, V>`、IxPool handle、ImPool snapshot、placeとoperationのlawは[意味論](model/semantics.md)が所有する。
- source APIと`Storable`の境界は[Pool primitive](api/pool.md)と[identity](model/identity.md)、物理表現とlifecycleは
  [runtime contract](runtime/contract.md)が所有する。
- Buffer、Vector、Map、Deque、heap、木へ課すrelationとinvariantは[container](containers/overview.md)以下が所有する。

本書はこれらの入口と採択単位だけを所有し、詳細なruleを再掲しない。containerをmal sourceで実装して現行runtimeと同じtrapを
起こすには、[primitive `trap`](../primitive-trap.md)の導入を前提とする。

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

## 推奨する最小採択単位

最初のlanguage changeは`IxPool<Header, V>`だけに切る。共有identity、indexed place、occupancy、containerと同じidentityに属する
Headerが、現行Bufferから分離する必要のある最小の意味である。`pool`、`grow`、`capacity`、`peek`、`swap`、`header`、
`swapHeader`を核とし、Live/Vacantを仮定するoperationはmalで書く周辺に置く。現在の文書とprototypeで`Meta`と書いている型parameterと
operation名は、採択時にはこの`Header`語彙へ揃える。

Headerを別identityへ分ける案は採らない。malにはproductの一fieldだけを更新するoperationがなく、Bufferのcount、Dequeの両端、
free-list headなどをslot storageとは別のshared identityに置くと、containerごとに二つのidentityのlifetimeと同期を規定することになる。
常に値を持つHeader placeとVacantになり得るslot placeを一つのidentityへ含める方が、authorityとinvariantの境界を小さくする。

最初の採択には次を含めない。

- `ImPool`、`freeze`、`thaw`、およびpublic `Vector`。structural snapshotの意味は本proposalで維持するが、copy、storage transfer、
  writable successorという独立したcost contractをIxPool導入の条件にしない。
- `moveRange`、snapshotのrange operation、live slot iterator。核のloopで意味を検証し、実際のIxPool loweringで定数倍または
  計算量が問題になるものだけを追加する。
- allocator parameter、arena、plugin crateによるruntime差し替え。C runtimeはbacking mechanismであり、source authorityではない。
- 現行Bufferのhost operation削除。IxPool上のBufferが現行semanticsとcost gateを満たしてから、Vectorを正規host valueにする変更を
  別に判断する。

この切り方でもImPoolを破棄するわけではない。IxPool kernelでStorable handle、managed payload、growth、終了時走査、occupancy表現を
検証した後、同じstate algebraへstructural snapshotを加える第二段階とする。Rust寄りのaffine responsibilityはcompiler内部の
`Store`/`Consume`、Haskell寄りのsnapshotはImPoolのsource semanticsに属し、両方を一つの初回primitiveへ束ねない。

`IxPool`は現行Bufferと同じpredefined mechanismとして全fileから参照可能にする。専用の`builtin`宣言や組み込みmodule機構は
IxPoolだけのためには追加しない。未検査preconditionを直接使う範囲は通常のAPI設計で狭め、file-local opaque型がcontainer invariantを
閉じる。更新operationは`Unit`または古いcarrierを返し、同じhandleを返すImPool共通signatureには揃えない。

## 後続段階で決める事項

- 周辺operationの公開名。周辺の意味のauthorityは通常のmal definitionに置き、backendの費用specializationは
  as-if loweringとする。どの周辺をspecializeするかと、Liveを仮定する除去を`unreachable :: Unit -> []`のような言語の
  primitiveへ寄せるかは、production kernelの生成物を測って決める。
- [`Storable`の拡張](model/identity.md#storableの原理)のうち、Buffer handleは
  [D096](../../history/decisions/active/D096.md)で、external opaque carrierは
  [D097](../../history/decisions/active/D097.md)で先に採択した。IxPool admissionは最初のPool decisionに含め、ImPool admissionは
  第二段階に残す。functionとempty sumは今回の範囲では除外する。
- transitive snapshot、serialization、Map keyなどに共通するstability judgmentが実際に必要か。ImPoolのstructural snapshotと
  handle nestingには不要なので、名称だけを先に追加しない。
- 第二段階でImPoolをpublicにするか。採る場合はuniqueness検査、Ix/Im共通source、`freeze`/`thaw`のcopy gateを一緒に決める。
- [測定後の候補](api/pool.md#測定後の候補)の`moveRange`とImPoolの範囲の写しを足すか。
- Vectorの公開API、現行Bufferのhost operationとの互換性、`Symbol`との型関係は
  [BufferとVectorの採択前に残る判断](api/buffer-vector.md#採択前に残る判断)を正とする。
- live slot iterationをcoreに持つか、core外のextensionにするか、containerに任せるか。
- IxPoolを直接使うcontainerが払う[占有tagの費用](runtime/implementation.md#占有tagの費用)。同じkernelのdirect Cとsafe Rustの
  測定ではbitmapを初期表現に選んだが、managed payload、growth、終了時走査と実際のcontainerを含む再測定が要る。
- opaque型のdiagnosticと、public APIがrepresentationを返せる範囲。
- IxPool callbackを既存Buffer callback emitterから一般化するか、共通`Lifecycle`を消費する別のemitterとして置くか。
- plugin crateのversion、reproducible build、artifact cache、runtime source選択のcontractはPool採択条件にしない。trusted extensionを
  外部配布する要求が生じたときのplugin設計が所有する。
