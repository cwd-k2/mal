# Pool上のcontainer

Status: Exploratory support document

この文書は、Pool上に構成する代表的なcontainerを並べ、各containerが共通に使うものと自分で決めるものから、Poolが何を
抽象するかを示す。primitiveの一覧は[primitive一覧](primitives.md)、code sketchは[source sketch](container-examples.md)と
[既存exampleとの差分](current-examples.md)、keyとArenaは[Pool key extension](pool-keys.md)を正とする。

## containerの比較

各containerは、Stateに何を置くか、Liveなcoordinateの集合をどんな形に保つか、coordinateに何の意味を与えるかで決まる。

| container | State | Liveなcoordinateの集合 | coordinateの意味 | 公開precondition |
|---|---|---|---|---|
| Buffer | count | `[0, count)` | 列の位置 | `index < #buffer` |
| stack | count | `[0, count)` | 列の位置 | なし。空のpopはmissingを返す |
| Deque | `(head, count)` | `head`から`count`個、capacityで折り返す | 輪の上の位置 | 位置が`count`未満 |
| binary heap | count | `[0, count)` | 暗黙の木の節点。子は`2i + 1`と`2i + 2` | なし |
| open addressing Map | 要素数 | probe列の途中にVacantを含まない任意の集合 | hashから始まるprobe順 | なし |
| slot map | 要素数とfree listの位置 | 任意 | 要素のidentity | keyの照合は検査する |
| 木 | root coordinateとfree list | 任意 | 節点のidentity。子をcoordinateで指す | 操作ごと |

現在のBufferはcountを減らせないため、Buffer以外は書けないか、sizeを別に持って取り出した値をstorageに残すか、tombstoneや
空値の`fill`を要する。Pool上では違いがslot遷移の使い方に現れる。

- stackのpopとDequeの両端からの取り出しは、`takeAt`で値をcopyせずに取り出し、そのslotをVacantへ戻す。
- binary heapのsiftは、親子の値を`takeAt`と`initAt`で入れ替え、`Share`も`Drop`も起こさない。
- Mapの削除は、後続のentryを`takeAt`と`initAt`で穴へ詰め、tombstoneを持たない。
- slot mapと木は、Vacantなslotを空き場所として再利用する。

## runの語彙を使えるcontainer

runの操作は、読むrunが全てLiveであることを要求する。Liveな集合が区間になるcontainerだけが、その区間をrunとして公開できる。

- Bufferとstackは`[0, count)`をそのままrunにでき、[run protocol](run-protocol.md)を実装する。
- Dequeの論理的なrunは最大二つの区間に分かれ、`into`などを二回のrun primitiveで実装する。
- binary heapは`[0, count)`がLiveだが、coordinateの順は要素の順序ではないため、runとして公開する意味は薄い。
- Map、slot map、木のLiveな集合は区間にならず、runを公開しない。rehashや複製では内部でslot遷移を使う。

## Vacantが値を持たないこと

Vacantなslotは値を持たない。free listを使うslot mapと木は、次の空きcoordinateをVacantなslotへ書けないため、free listをStateに
置くか、`Pool<Unit, USize>`のような別のPoolへ積む。これはPoolが未初期化carrierを公開しない代わりに生じる制約であり、Vacantな
slotに値を置く必要があるcontainerは、要素を`[Unit, T]`のような直和にしてLiveなまま空きを表す。

Liveなslotを列挙する操作もPoolにはない。Map、slot map、木の全要素を訪れるには`isLive`でcapacity全体を走査するか、container自身が
要素の並びを持つ。live slot iterationをどの層が持つかは[README](README.md#未決定事項)の未決定事項である。

## Poolの輪郭

どのcontainerも共通に使い、Poolが提供するものは次である。

- `0`から`capacity - 1`までの線形なcoordinate空間
- coordinateごとのLiveとVacant、およびその間の遷移`initAt`と`takeAt`
- 同じidentityに置く共有mutableなState
- 明示的な`reserve`によるcapacityの拡張と、coordinateを保つrelocation
- 要素型ごとの型付きstorageと、寿命と所有権の正しさ

containerごとに異なり、Poolが決めないものは次である。

- Liveなcoordinateの集合の形
- coordinateの意味：列の位置、輪の位置、木の節点、hashのprobe順、要素のidentity
- growth policy、free list、順序、hash、要素の列挙
- 公開preconditionと、それをPoolのpreconditionへ写すinvariant

したがってPoolは、共有Stateを伴う型付きplaceの線形空間であり、どのplaceを使うかを持ち主が決めるもの、と言える。Bufferは
この空間の使い方の一つにすぎず、Poolの意味はBufferに依存しない。
