# sequence container

Status: Exploratory support document

## Stack

Stackは`(*USize, IxPool<A>)`を使う。`push`はcount位置へstoreしてcountを増やす。
`pop`はcountを減らし、旧末尾のpointerから`take`して返す。capacityは縮めず、Vacantな末尾を再利用する。

```text
push(a):  n := load(count) → ensure capacity → store(values.at(n), a) → store(count, n + 1)
pop():    n := load(count) - 1 → store(count, n) → take(values.at(n))
```

## Deque

```mal
opaque Deque<A> :: (*(USize, USize), IxPool<A>);
// metadata = (head, count)
```

capacityが0でなければ、論理位置`j`は`(head + j) mod capacity`へ写る。pushは対応するVacant placeへ
storeし、popは端のLive placeから`take`する。

growth時は新しいcoordinateを追加しただけではringの折り返しが直らない。新しい配置へ順序を保って
`take`/storeでMoveし、全要素が整った後に`head = 0`と新countをmetadataへ書く。移動先をVacantに保てば
AのShare/Dropは不要である。flat representationでは一時IxPoolから元のpoolへ戻し、共有descriptor representationでは
新しいpoolをdescriptorへcommitできる。移動中にcomparisonやuser callbackを呼ばない。

## binary heap

Heapも`(*USize, IxPool<A>)`で、`[0, count)`をLiveにする。挿入では末尾をLiveにし、親子の値を
`take`/storeまたは`swap`相当で移す。削除ではrootを取り出し、旧末尾値をholeへMoveしてsiftする。

comparisonが任意のMal codeを呼び得るなら、holeがある状態で呼ぶと公開invariantを観測され得る。
初回libraryはcomparisonを非再入とするか、比較に使う値を先に読み、各call前にholeを埋めてinvariantを
回復する必要がある。この選択はIxPoolでなくHeap APIが所有する。

## immutable sequence

以前の案にあった`ImPool`上のVectorは本proposalに含めない。first-classなpointerが残るIxPoolをsnapshotへ
変換するにはcopy、copy-on-write、outstanding pointerの排他のどれかが必要で、最小核から決まらないためである。
immutable sequenceは[Bufferとsnapshot候補](../api/buffer-vector.md#snapshotを分離する理由)で別課題とする。
