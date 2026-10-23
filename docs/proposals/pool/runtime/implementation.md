# compilerとruntimeの実装

Status: Exploratory support document; rebased on mal v0.7

この文書は、[runtime contract](contract.md)をcompilerとruntimeへ接続する境界と検証順を管理する。

## compilerとruntimeの分担

現行Bufferで使う`RuntimeCarrier(T)`、`Lifecycle(T) = Trivial | Owned(share, drop)`、execution ownershipの
`Borrow | Share | Consume | Drop | Store`を再利用する。pointer専用のownership体系やportable layoutは作らない。

| source operation | operand effect | result |
|---|---|---|
| `place(a)`、`store(p, a)` | `a`は`Store`、pointerは`Borrow` | owned pointer / `Unit` |
| `load(p)` | pointerは`Borrow` | Share済みのowned `A` |
| `take(p)` | handleは`Borrow` | Moveされたowned `A` |
| `pool()` | なし | owned IxPool |
| `at(pool, i)` | poolとindexは`Borrow` | ownerをShareしたowned pointer |
| `grow`、`capacity` | handleは`Borrow` | `Unit` / `USize` |

`Store`はsource operandを必ず消費する意味ではない。use planが後続利用に応じてShareまたはConsumeへlowerし、
calleeがowned carrierを受け取る。pointerやIxPool handleをproduct、sum、Buffer、別placeへ保存した場合も既存の
型再帰的なShare/Dropを使う。

## lowering boundary

| 層 | 所有する判断 |
|---|---|
| parsing / type checking | type constructor `*A`、pointer operation、`IxPool<A>`の形成と`Storable(A)` |
| execution ownership | operandとresultのresponsibility plan、`Store`のShare/Consume |
| target layout | specialization後のAのsize、alignment、stride、glue |
| LLVM lowering | typed access、occupancy順序、runtime call、growth後の再解決 |
| C runtime | owner count、allocation、growth、occupancy、physical capacity |

runtimeはAの意味を再解釈せず、compilerが渡すlayoutとShare/Drop glueだけを使う。compilerはreference countや
physical capacityからsource authorityを推論しない。

## runtime object

初期実装は概念上、次の二種類を持てる。

```text
Standalone pointer owner
    managed owner count
    occupancy
    A carrier storage

IxPool owner
    managed owner count
    logical and physical capacity
    occupancy storage
    current payload storage
    A layout and lifecycle glue

Pool-derived pointer carrier
    IxPool owner responsibility
    coordinate
```

これはprivate ABIの模式であり、固定layoutやallocation数ではない。standalone placeのpayloadをownerへinline化し、
小さいIxPool storageをownerへinline化してよい。containerのmetadata pointerとIxPool ownerも、logical identity、
alias、lifecycleを保てる特殊化では同じallocationへco-allocateしてよい。

既存の`runtime/c11/pool.c`はstable owner、bitmap、growth、Header carrier、slot exchangeを検査した非公開kernelである。
bitmap、relocation、overflow、managed payload glueは再利用できる。一方、次は現proposalをまだ検査していない。

- `Header`を外してcontainer metadataを独立したpointerへ置くこと
- `at`がfirst-class pointer carrierを返すこと
- pool handleよりpointerが長生きすること
- pointerを通常値として保存、Share、Dropすること
- 同じpoolとcoordinateから得た複数pointerの共有観測

したがって既存kernelを実装済みのsource featureとはみなさず、pointerのowner + coordinate carrierへ合わせて境界を
書き換える。

## accessとgrowth

pool-derived pointerの各operationはownerからlogical capacity、occupancy、active payload pointerを取得する。
同じregionでcacheしてよいが、次をclobber boundaryとする。

- `grow`またはphysical relocationを行い得るruntime call
- 同じIxPoolへ到達し得るnative Mal call
- 将来callback/reentryを認める境界

growth前のpayload pointerをgrowth後にload/storeへ使わない。Cの`realloc`が同じ数値addressを返しても新しい
LLVM allocation objectとして扱う。source pointer carrierはaddressでなくownerとcoordinateを保持する。

storeはpayloadを初期化してからLiveをcommitし、置換では新carrierを観測可能にしてから旧carrierをDropする。
`take`はcarrierをresultへMoveしてVacantをcommitする。primitive内部の順序は、lifecycle glueがMal codeへ再入しない
現行前提に依存する。再入を認めるならcommit pointを改めてspec化する。

## 費用の論点

IxPoolは任意のcoordinateがVacantになれるためoccupancyを払う。dense Bufferは長さからoccupancyを導いて省ける。
初期kernelのbitmap測定は表現候補の根拠になるが、first-class pointer carrierの二word化、owner count、`at`の生成、
metadata pointerとの二identity化は未測定である。

評価では次を分離する。

- bitmap、byte tag、inline sumのoccupancy費用
- `at`をinline化して一時pointer carrierを消せる場合と、pointerがescapeする場合
- metadata pointerとIxPool ownerをco-allocateできるcontainerと、独立aliasが残る場合
- managed AのShare/Dropと、単なるrelocation
- 現行Bufferのdense lower bound

## 検証の段階

1. `*A`、load/store/take、IxPoolのsyntaxと型形成、unsupported syntaxの拒否をfrontend testで固定する。
2. standalone pointerでVacant/Live、load/store/take、zero-sized A、managed AのShare/Dropを検査する。
3. `Header`を外したIxPool ownerでcapacity、bitmap、growth、overflow、終了時の全Live Dropを検査する。
4. `at`のpointerを複数作り、共有観測、growth後の有効性、IxPool handle終了後の存続、最後のpointerでの回収を検査する。
5. pointer/IxPool handleをproduct、sum、Buffer、別placeへ保存し、通常値と同じlifecycle再帰を使うことを検査する。
6. `Store` effectを接続し、last-useでConsume、再利用時にShareとなることを生成物で検査する。
7. IxPool上のBuffer相当を現行Bufferとalias、range、overlap、lifecycleで比較する。
8. HashMap、Deque、heap、SlotMap、treeでVacant再利用とcoordinate安定性を検査する。
9. 意味と費用が妥当な場合だけ、Buffer内部表現の統合、snapshot、extern admissionを別々に判断する。

## 実装を再開する位置

次回はpointer type、load/store/takeとcarrierを先に実装し、そのcarrierがstandalone ownerとIxPool owner + coordinateの
両方へ到達できる最小表現を決める。その後、既存pool kernelからHeaderを外し、`at`が返したpointerのlifetimeを
owner countへ接続する。

この境界が通る前にkernelを汎用allocator、snapshot storage、closure arena、public C extensionへ広げない。
