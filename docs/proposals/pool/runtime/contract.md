# runtime contract

Status: Exploratory support document; rebased on mal v0.7

この文書は、pointerが指すplaceとIxPoolのsemantic identityを物理allocationから分け、runtimeが守るlifecycleと
growthの境界を定める。source operationは[API](../api/pool.md)、状態遷移は
[responsibility](../model/responsibility.md)、現行の実行規則は
[`execution`](../../../spec/execution.md)と[managed valueのownership](../../../implementation/ownership.md)を正とする。

## authority boundary

pointer ownerとIxPoolはmal-controlledなEngramである。allocatorやC pointerは実装機構であり、sourceへallocator
capability、address、明示的な`free`を公開しない。allocation failureとsize overflowは既存Engram allocationと
同じterminalなtrap、値は現行runtimeと同じくthread-confinedとする。

初回採択では`*A`とIxPoolをextern signatureや`mal.h`へ公開しない。将来公開する場合も、generated C headerと
LLVM moduleが同じtarget carrier planを使い、managed parameterのborrow、managed resultのowned move、
型別Share/Dropを一緒に定める。

## identityと表現

実装は次を区別する。

```text
logical place identity
IxPool semantic identity
poolに相対的なcoordinate
managed owner object
occupancy storage
payload backing allocation
LiveなAのcarrier
```

standalone pointerは安定したowner objectにあるplaceを指せる。IxPoolは安定したowner objectから、交換可能な
occupancyとpayload storageを指せる。`at(pool, i)`のpointer carrierは少なくともpool ownerとcoordinateを意味し、
ownerへのresponsibilityを持つ。元のIxPool handleが終了しても、そのpointerが残る間は
ownerと全coordinateが有効である。

同じpool identityとcoordinateは常に同じlogical placeを表す。異なるpoolの同じindexや、standalone placeとは
別identityである。初回採択はidentity equalityを公開しないため、実装がcarrierをinternする必要はない。

`grow`は同じpool identityと既存coordinateを保ってbacking allocationを交換できる。pointer carrierへpayload
pointerを固定してはならず、accessごとにownerとcoordinateからactive backingを解決する。LLVM上でもgrowth後に
古いallocation objectのpointerを使わない。`llvm.lifetime.start/end`はLive/Vacantの状態を表さない。

## occupancyとgrowth

IxPoolはlogical capacity `n`、各coordinateのoccupancy、Live coordinateの`A` carrierを持つ。
Vacant payloadを`A`としてloadしない。`size<A> == 0`でもcoordinateとoccupancyは存在し、状態遷移とDrop回数は
通常の型と同じである。

`grow(pool, k)`は次のtransactionとして実装する。

1. `n + k`、occupancy、payload sizeのoverflowを判定する。
2. 必要なら新しいbacking storageを確保する。
3. occupancyとLive carrierをShare/Dropせず移す。
4. ownerのactive backingをcommitし、古いbackingを解放する。
5. logical capacityを更新し、新coordinateをVacantにする。

trapはterminalなので失敗後の状態をsourceへ定めない。physical over-allocationとphysical capacityは観測させない。
初期表現にはbitmapを使えるが、occupancy表現はsource ABIではない。

## responsibility

| operation | carrierの効果 | primitive内のShare | primitive内のDrop |
|---|---|---|---|
| `vacant()` | Vacant ownerを作る | なし | なし |
| `place(a)` | `a`をMoveしてLive ownerを作る | なし | なし |
| `isLive(p)` | 状態だけを読む | なし | なし |
| `load(p)` | Live carrierを返す | 1 | なし |
| `store(p, a)` | `a`をStoreして置換する | なし | 旧値がLiveなら1 |
| `take(p)` | Live carrierを結果へMoveしVacantにする | なし | なし |
| `pool()` | capacity 0のownerを作る | なし | なし |
| `at(pool, i)` | ownerへの到達性を持つpointerを返す | owner 1 | なし |
| `grow(pool, k)` | carrierをrelocateしVacantを追加する | なし | なし |
| pointer ownerの最後のresponsibility | standaloneなら状態を終了する | なし | Liveなら1 |
| IxPoolの最後のowner | 全coordinateを終了する | なし | Live数 |

storeはincoming carrierとLive状態を確立してから旧値をDropする。`take`とrelocationはMoveであり、Aの
Share/Dropを呼ばない。call siteは`Store` operandを後でも使うならShare、last useならConsumeして渡す。

pool-derived pointerとIxPool handleは同じownerへ別々のresponsibilityを持つ。pointer carrierのShare/Dropは
owner countを増減するだけで、coordinateの値をShare/Dropしない。最後のownerでだけ全Live値を終了する。

## precondition

| operation | 未検査precondition |
|---|---|
| `load(p)`、`take(p)` | pointeeがLive |
| `at(pool, i)` | `i < capacity(pool)` |
| `store`、`isLive`、`pool`、`capacity` | なし |

`grow`の表現不能なsizeとallocation failureはprecondition違反でなくtrapである。未検査preconditionへの違反は
現行Bufferと同様に実行結果を保証しない。debug runtimeは検査してよい。

container利用者はcontainerの公開preconditionを守り、container実装はそこから到達するpointer/IxPool formの
preconditionをopaque representationのinvariantで保証する。複数operationはtransactionではないため、unknown
callbackや観測可能なDropを呼ぶ前にinvariantを回復するか、再入を明示的に禁止する。

## Bufferとの関係

`(*USize, IxPool<A>)`でdense sequenceを表せるが、現行Bufferをそのまま二allocationにする必要はない。
長さからLive範囲が分かるdense Bufferはoccupancyを省き、metadataとstorageを同じallocationへ置ける。
as-if表現はsourceのalias、lifecycle、range operationを保てばよい。逆に、汎用IxPoolから返したpointerが残る場合は
ownerとcoordinateの到達性を保ち、co-allocationを理由にlogical identityを潰したりdanglingさせたりしてはならない。
