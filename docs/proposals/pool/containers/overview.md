# containerの構成

Status: Exploratory support document

この文書は、containerをpointerと`IxPool`から組み立てる共通形を示す。APIは
[pointerとIxPool](../api/pool.md)、値の移動は[responsibility](../model/responsibility.md)を正とする。

## 共通形

containerは、可変metadataを置く一つ以上のplaceと、coordinateから要素placeへ到達するIxPoolを組み合わせる。

```text
container value
  ├─ Header place      count、head、rootなど
  └─ IxPool<Element>   capacityとcoordinate空間
          └─ at(i) → *Element
```

`Header`はIxPoolの組み込みfieldや型引数ではなく、containerが選ぶ通常の型である。単純な長さなら
`*USize`、Dequeなら`*(USize, USize)`にできる。あるいは`*(Header, IxPool<Element>)`として、metadataとpool handleを
共有descriptorへ格納してもよい。

| container | metadata pointer | Live coordinate | coordinateの意味 |
|---|---|---|---|
| Buffer / Stack | `count` | `[0, count)` | sequence position |
| Deque | `(head, count)` | ring上のcount個 | backing position |
| Heap | `count` | `[0, count)` | heap position |
| HashMap | `count`など | occupied bucket | probe position |
| SlotMap | count、free headなど | allocated entry | stable slot number |
| Tree | `(root, size)` | allocated node | node link |

pointeeの`Vacant | Live(A)`はfirst-classな`Slot<A>`値ではない。探索は`isLive(pool.at(i))`、読み出しは
`load(pool.at(i))`、挿入と置換は`store(pool.at(i), value)`、削除と移動は`take`を使う。

## Bufferとの差

Bufferは`[0, count)`だけがLiveというdense invariantを持つ。このためoccupancyをcountから導ける。
IxPoolは途中のcoordinateをVacantにできるため、HashMap、free list、tree node arenaをsentinelや
`[Unit, A]` payloadなしで表せる。その代わりoccupancy storageと終了時のscanを払う。

IxPoolを全dense containerの一律な物理表現にはしない。containerのopaque representationが内部pointerとIxPool
handleを外へ出さなければ、logical identityとlifecycleを保ったままmetadataとstorageをco-allocateし、不要な
occupancyを除去するas-if最適化が可能である。

## 共通helper

container libraryには意味を増やさない次のhelperを置ける。

```mal
ensureCapacity<A> :: (IxPool<A>, USize) -> Unit := (p, required) -> {
    current := capacity(p);
    when (current < required) grow(p, required - current);
};

transfer<A> :: (*A, *A) -> Unit := (from, to) -> {
    store(to, take(from));
};
```

実際のgrowth factor、overflow判定、reserve policyは各containerが選ぶ。IxPoolの`grow`は要求した分だけ
logical capacityを増やし、physical over-allocationはruntime内部に隠す。

## invariantの責任

opaque containerは公開operationのpreconditionをpointer/IxPoolの未検査preconditionへ写す責任を持つ。
たとえばBufferの`get(i)`は`i < count`から`at(values, i)`が範囲内かつLiveだと導く。

flatな`(*H, IxPool<V>)`を選ぶcontainerは同じIxPoolを`grow`して更新し、別poolを使う再配置では完了時に要素を
元のIxPoolへ戻す。`*(H, IxPool<V>)`という共有descriptorを選ぶcontainerは、内部pointerが残らない境界で新しい
pool handleへ交換できる。どちらを選ぶかは公開operationと費用に基づくopaque representationの判断である。

内部の要素pointerを返すと、利用者は`take`でLive coordinateをVacantにできる。公開APIがそのauthorityを意図して
与えるのでなければ、内部pointerをopaque境界からescapeさせず、`load`した値やcontainer operationのresultを返す。

値を移動中に一時的な穴を作ってよいが、未知のhash、equality、comparison、callbackや観測可能なDropを
呼ぶ前には公開invariantを回復する。再入を許さない設計なら、その制限をoperation contractに明記する。
