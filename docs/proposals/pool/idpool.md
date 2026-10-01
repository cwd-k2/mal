# IdPool

Status: Exploratory support document

この文書は、identityで要素を引く`IdPool<T>`と、そのhandle `Id<T>`を管理する。IxPoolのprimitiveは[primitive一覧](primitives.md)、
`Id<T>`のidentity軸上の位置と`Storable`の根拠は[identity](identity.md)、IdPoolをIxPoolの上に書いたcodeは
[collection例](collection-examples.md#idpool)を正とする。

## 位置づけ

IdPoolはprimitiveではなく、IxPool上のcontainerの一つである。新しいprimitiveなしにIxPoolの上に書け、Map、Deque、heapと同じ層に
ある。Bufferが位置で要素を引くのに対し、IdPoolは検査付きのhandleで引き、古い参照をmissingとして安全に扱える。この性質から、
木、graph、entityのように外へ参照を返すcontainerの標準として置く価値がある。標準libraryに含めるかは未決定である。

| 型 | 引き方 | precondition | 主な用途 |
|---|---|---|---|
| IxPool | coordinate | 未検査。実装者の責任 | container実装 |
| Buffer | 位置 | 公開precondition `index < #buffer` | 列、run、hostとの交換 |
| IdPool | `Id<T>` | 検査してmissingを返す | 木、graph、entity、外へ返す参照 |

IdPoolはIxPoolの上に書ける。逆にIxPoolはIdPoolの上に書けず、heapやopen addressing Mapのような位置の計算をIdPoolは持たない。
IdPoolの実装は、generationやfree listをVacantなslotの中へ置く最適化をtrusted layerで行ってよい。

## API

```mal
IdPool<T>
Id<T>

makeIdPool<T> :: Unit -> IdPool<T>;
idInsert<T> :: (IdPool<T>, T) -> Id<T>;
idGet<T> :: (IdPool<T>, Id<T>) -> [Unit, T];
idRemove<T> :: (IdPool<T>, Id<T>) -> [Unit, T];
idCount<T> :: IdPool<T> -> USize;
```

`idInsert`は空いた場所を再利用するか新しい場所を確保して要素を置き、その要素を指す`Id<T>`を返す。`idGet`は要素を`Share`して返し、
`idRemove`は要素のresponsibilityを呼び出し元へ移してその場所を空ける。どちらも`Id<T>`が今の要素を指さなければmissingを返す。
IdPoolの形成は`Storable(T)`を要求し、IdPoolのcopyはIxPoolと同じくidentityを共有する。

## 照合

IxPoolのcoordinateは[未検査](lifecycle-contract.md#未検査precondition)だが、`Id<T>`の照合は検査してsumで返す。coordinateは
container実装がfile-local invariantで保証できる一方、`Id<T>`は利用者がいつまでも保持でき、古いかどうかをcontainer実装も
利用者も静的に知らないためである。照合によって、IdPoolの利用者のmemory safetyはinvariantに依存しない。

照合には次が必要である。

- IdPoolのidentityを、`Id<T>`が残り得る間は再利用しない。process内で単調に増える64-bit counterを候補とする。
- 場所ごとのgenerationを、要素を取り除くたびに進める。generationが上限に達した場所はretireし、再利用しない。
- `Id<T>`は発行したIdPoolのidentity、場所、generationを持ち、別のIdPoolや取り除いた後の要素を指せば照合で外れる。
- IdPoolの破棄後は照合に要るIdPoolのoperandが存在しないため、`Id<T>`は使いようがない。

## `Id<T>`

`Id<T>`はIdPoolをretainしない不変のdataであり、`Storable`にできる。そのため要素の中へ置いて木やgraphを作れ、要素から自身の
IdPoolを指してもowner cycleにならない。根拠は[identity](identity.md#所有権とidentity)で扱う。

`Id<T>`を`Representable`と`HostMappable`にはしない。hostが任意のbit列から`Id<T>`を作れると、照合がforgeryへの防御を担い、
identityとgenerationの幅がsecurity contractになるためである。sourceもIdPool以外から`Id<T>`を構築できない。

ownerを要求せず、processのregistryから要素を直接引くweakなidも採らない。その照合結果は最後の強いaliasがいつ消えたかを表すが、
[memory](../../spec/memory.md#buffer)はstorage回収をsource-levelで規定せず、reference countも観測させない。IdPoolでは要素が
消える時点は`idRemove`とIdPoolの破棄に限られる。

## IxPoolを要素にする場合

IxPoolは`Storable`でないため、`IdPool<IxPool<S, T>>`は作れない。nodeごとの隣接listのように、要素ごとに可変長のIxPoolを持ちたい
場合は、IxPoolだけを保持する一段のowner `Arena<S, T>`を別に置き、handleを`Id<IxPool<S, T>>`とする。

```mal
Arena<S, T>

makeArena<S, T> :: Unit -> Arena<S, T>;
arenaAdd<S, T> :: (Arena<S, T>, S, USize) -> Id<IxPool<S, T>>;
arenaGet<S, T> :: (Arena<S, T>, Id<IxPool<S, T>>) -> [Unit, IxPool<S, T>];
arenaRemove<S, T> :: (Arena<S, T>, Id<IxPool<S, T>>) -> Unit;
```

Arena自身は`Storable`でなく、ArenaのIxPoolがArenaやIxPoolを要素に持つこともできないため、owner edgeは深さ1で終わる。
`arenaGet`は所有しているIxPoolを`Share`して返す。取り出したIxPoolがlocalやclosureに残っていれば、`arenaRemove`の後も通常の
lifecycleで生き続ける。`ImPool`の入れ子との使い分けは[identity](identity.md#入れ子構造の選び方)で扱う。

## 未決定事項

- generationの幅と、retireした場所をIdPoolの大きさへどう反映するか。
- IdPoolの要素を列挙する操作を持つか。
- `Id<T>`にequalityとhashを与え、Mapのkeyにできるようにするか。
- Arenaの`State`を個々のIxPoolごとに持つか、Arena全体で一つ持つか。
