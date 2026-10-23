# keyed container

Status: Exploratory support document

## HashMap

最小のopen-addressing表現は、要素数を指すpointerとbucketのIxPoolである。

```mal
opaque Map<K, V> :: (*USize, IxPool<(K, V)>);
```

Vacant bucketは`isLive(buckets.at(i)) == false`で表す。payloadへ`[Unit, (K, V)]`やsentinel keyを
入れない。lookupはprobe順にpointerを調べ、Liveならloadしたentryのkeyを比較する。insertはVacant
placeへstoreし、既存keyなら同じplaceへstoreして旧entryをDropする。removeは`take`でentryを返すか
Dropし、clusterが切れないよう後続entryをMoveして詰める。

```text
bucket := buckets.at(probe);
if isLive(bucket) {
    entry := load(bucket);
    ... compare entry.key ...
} else {
    store(bucket, (key, value));
}
```

flat representationのrehashでは、一時IxPoolへ旧entryを`take`/storeで退避し、元のpoolを`grow`してから新しい
probe位置へ戻せる。共有descriptor representationなら、新しいpoolへ配置した後でdescriptorのhandleを交換できる。
どちらも値carrierはMoveでき、K/VのShareは不要である。途中の穴とprobe invariantをuser-supplied hash/equalityへ
公開してはならない。

`Storable(K)`はhashの時間的安定性を保証しない。keyから共有mutable identityへ到達できる場合、格納後の
変更でhash/equalityが変わり得る。Map APIはimmutableなhash projectionを要求するか、利用者のpreconditionと
して別に定める。これはpointerやIxPoolの形成条件へ混ぜない。

## SlotMap

generation付きkeyは、values、generations、free coordinatesを別のIxPoolへ分けて表せる。

```mal
opaque SlotKey<A> :: (USize, UInt64);
opaque SlotMap<A> :: (
    *USize,             // live count
    IxPool<A>,          // entry places
    IxPool<UInt64>,     // issued coordinateのgeneration
    *USize,
    IxPool<USize>       // free coordinate stack
);
```

removeはkeyのgenerationを検査し、value pointerから`take`し、generationを増やしてfree stackへcoordinateを
積む。insertはfree stackから再利用するか新coordinateをgrowする。古いkeyの拒否はpublic resultであり、
IxPoolの未検査preconditionへ流さない。

このsketchの`SlotKey`は別SlotMapの同じindex/generationを区別しない。必要ならcontainer identity tokenを
keyへ含めるが、pointer equalityをそのためだけに公開しない。

## treeとgraph

nodeを`IxPool<Node>`へ置き、子linkを`USize` coordinateで表せる。rootとsizeは
`*(USize, USize)`、free coordinateは別のstackに置く。node削除はpointerから`take`してfree stackへ
coordinateを戻し、再挿入で同じcoordinateを使える。

coordinateはaddressではなく一つのpool identity内だけで意味を持つ。別poolへnodeを移す場合はlinkを
書き換える必要があり、数値が同じでも同じplaceとはみなさない。generationなしでcoordinateを外部へ公開すると
再利用後の別nodeを指すため、stable external keyにはSlotMapと同じgenerationを加える。
