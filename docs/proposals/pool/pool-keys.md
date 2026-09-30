# Pool key extension

Status: Exploratory

この文書は、core Pool APIの外に置くnonowning keyを管理する。core Poolはcoordinateだけでslotを選び、keyを発行しない。
keyはslot map、tree、graphのように、container外へ参照を返す実例がある場合だけ追加する。Pool primitiveの規則は
[lifecycle contract](lifecycle-contract.md)、keyのidentity軸上の位置と`Storable`の根拠は[identity](identity.md)を正とする。

## 二つのkey

keyは指す対象で二種類に分かれる。

- `SlotKey<State, T>`は、特定のPool内の一つのslotを指す。slot mapやgraphのnode handleとして外部へ返す。
- `PoolKey<State, T>`は、Pool自体を指す。nodeごとの隣接listのように、Poolの集まりをelementから参照する。

どちらもPoolをretainせず、解決にはliveなowner operandを要求する。`SlotKey`のownerは指す先のPoolであり、既存のPoolだけで成立する。
`PoolKey`のownerになれる値は現在の案に存在しないため、後述の`Arena`を追加する必要がある。

```mal
SlotKey<State, T>
PoolKey<State, T>
Arena<State, T>

keyAt<State, T> :: (Pool<State, T>, USize) -> SlotKey<State, T>;
resolve<State, T> :: (Pool<State, T>, SlotKey<State, T>) -> [Unit, USize];

makeArena<State, T> :: USize -> Arena<State, T>;
arenaAdd<State, T> :: (Arena<State, T>, State, USize) -> PoolKey<State, T>;
arenaPool<State, T> :: (Arena<State, T>, PoolKey<State, T>) -> [Unit, Pool<State, T>];
arenaRemove<State, T> :: (Arena<State, T>, PoolKey<State, T>) -> Unit;
```

`keyAt`はLive slotを指すcoordinateをpreconditionとする。`resolve`は、keyが同じPool identityから発行され、そのslotが
発行時から一度もVacantになっていない場合だけcoordinateを返す。`arenaPool`はArenaが所有するPoolを`Share`して返し、
`arenaRemove`はArenaの所有を終える。取り出し済みのPoolがlocalやclosureに残っていれば、そのPoolは通常のlifecycleで生き続ける。

## hostとsourceからの構築

keyが`Storable`になる理由は[identity](identity.md#所有権とidentity)で扱う。keyを`Representable`と`HostMappable`にはしない。
hostが任意のbit列からkeyを作れると、照合がforgeryへの防御を担うことになり、identityとgenerationの幅がsecurity contractに
なるからである。sourceもprimitive以外からkeyを構築できない。

## 照合とprecondition

coordinateのpreconditionは[未検査](lifecycle-contract.md#未検査precondition)だが、keyの照合は検査してsumで返す。coordinateは
container実装がfile-local invariantで保証できる一方、外部へ返したkeyは利用者がいつまでも保持でき、staleかどうかを
container実装も利用者も静的に知らないためである。照合を未検査にするとkeyを持つ意味がない。

照合には次が必要である。

- Pool identityを、keyが残り得る間は再利用しない。process内で単調に増える64-bit counterを候補とする。
- slotごとのgenerationを、Live slotがVacantになるたびに進める。generationが上限に達したslotはretireし、再利用しない。
- Pool破棄後のkeyは、解決に必要なowner operandが同じidentityで存在しないため、照合で自然にmissingになる。

## weak keyを採らない理由

ownerを要求せず、processのregistryからPoolを直接解決するweak keyも考えられる。しかしその照合結果は、最後の強いaliasが
いつ消えたかを表す。[memory](../../spec/memory.md#buffer)はstorage回収をsource-levelで規定せず、reference countも観測させないため、
weak keyはこの原則を破る。Arenaは回収時点をsourceの`arenaRemove`とArenaの破棄に限るため、この問題を持たない。

## Arenaの境界

Arenaは`Pool<State, T>`だけを保持する一段のownerである。Arena自身は`Storable`でなく、ArenaのPoolがArenaやPoolをelementに
持つこともできないため、owner edgeは深さ1で終わり、cycleを作らない。`Pool<State, Pool<...>>`を許す一般化はPoolを
`Storable`にすることと同じであり、採らない。同じ理由でArenaは既存のPoolの組み合わせでは表せない。
`ValuePool`の入れ子との使い分けは[identity](identity.md#入れ子構造の選び方)で扱う。

## 例：隣接listを持つgraph

nodeごとの隣接listを可変長にするには、node slotから隣接list Poolへの参照が要る。Bufferは`Storable`でないため
現在は[indexed-graph](../../../examples/indexed-graph/graph.mal)のように全edgeを一つの列へflattenするが、Arenaを使うと
nodeごとのPoolを持てる。

```mal
_Node<T> :: (T, PoolKey<USize, USize>); // node value、隣接nodeのcoordinate列
opaque Graph<T> :: (Pool<USize, _Node<T>>, Arena<USize, USize>);

_adjacency<T> :: (Graph<T>, USize) -> Pool<USize, USize> := ((nodes, lists), node) -> {
    (_, key) := getAt<USize, _Node<T>>(nodes, node);
    arenaPool<USize, USize>(lists, key)[
        () -> trap("graph adjacency list was removed")[],
        (pool) -> pool
    ];
};
```

Graphがnodeを削除しない限り、keyはfile-local invariantにより常に解決できる。ここでmissingになるならGraph実装の誤りなので、
[primitive `trap`](../primitive-trap.md)で終了させる。利用者へnode handleを返す場合は、node Poolに対する`SlotKey`を使い、
`resolve`のmissingを利用者へ返す。

## 未決定事項

- `SlotKey`のgeneration幅と、retireしたslotを`capacity`へどう反映するか。
- live slot iterationと`SlotKey`を同じextensionで提供するか。
- Arenaの`State`を個々のPoolごとに持つか、Arena全体で一つ持つか。
- keyにequalityとhashを与え、Mapのkeyにできるようにするか。
