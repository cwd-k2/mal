# IxPool上のcontainer

Status: Exploratory support document

この文書は、IxPool上に構成する代表的なcontainerを並べ、各containerが共通に使うものと自分で決めるものから、IxPoolが何を
抽象するかを示す。primitiveの一覧は[Pool primitive](../api/pool.md)、code sketchはBufferを[Buffer実装](buffer.md)、
その他のcontainerを[列のcontainer](sequences.md)と[keyで引くcontainer](keyed.md)が正とする。

## containerの比較

各containerは、Metaに何を置くか、Liveなcoordinateの集合をどんな形に保つか、coordinateに何の意味を与えるかで決まる。

| container | Meta | Liveなcoordinateの集合 | coordinateの意味 | 公開precondition |
|---|---|---|---|---|
| Buffer | count | `[0, count)` | 列の位置 | `index < #buffer` |
| stack | count | `[0, count)` | 列の位置 | なし。空のpopはmissingを返す |
| Deque | `(head, count)` | `head`から`count`個、capacityで折り返す | 輪の上の位置 | 位置が`count`未満 |
| binary heap | count | `[0, count)` | 暗黙の木の節点。子は`2i + 1`と`2i + 2` | なし |
| open addressing Map | 要素数 | probe列の途中にVacantを含まない任意の集合 | hashから始まるprobe順 | なし |
| SlotMap | 要素数とfree listの位置 | 任意 | 要素のidentity | `SlotKey<T>`の照合は検査する |
| 木 | root coordinateとfree list | 任意 | 節点のidentity。子をcoordinateで指す | 操作ごと |

現在のBufferはcountを減らせないため、Buffer以外は書けないか、sizeを別に持って取り出した値をstorageに残すか、tombstoneや
空値の`fill`を要する。IxPool上では違いがslot遷移の使い方に現れる。

- stackのpopとDequeの両端からの取り出しは、`takeAt`で値をcopyせずに取り出し、そのslotをVacantへ戻す。
- binary heapのsiftは、親子の値を`takeAt`と`initAt`で入れ替え、`Share`も`Drop`も起こさない。
- Mapの削除は、後続のentryを`takeAt`と`initAt`で穴へ詰め、tombstoneを持たない。
- SlotMapと木は、Vacantなslotを空き場所として再利用する。

## Bufferの前提を外すと変わること

現在のBufferでもMapやheapは書ける。[`generic-map`](../../../../examples/generic-map/map.mal)は、空きを表す`[Unit, (K, V)]`で
全bucketを`fill`してopen addressingを実装している。違いは、Bufferの前提がcontainerにどんなcostを課すかにある。

| Bufferの前提 | Bufferで書くcontainerが払うもの | IxPoolで変わること |
|---|---|---|
| 全slotが値を持つ | 空きを表す番兵値とsum tag、構築時の全slotへの`fill`、probeごとのsum分岐 | Vacantが空きを表し、要素型をそのまま置ける |
| countは増えるだけ | popも削除もできず、取り出した値は番兵で上書きするまでstorageに残る | `takeAt`で取り出した時点でresponsibilityがslotを離れる |
| 値は`get`の`Share`と`put`の`Drop`でしか動かない | rehash、sift、ringの展開で、移動ごとに`Share`と`Drop`が起こる | `takeAt`と`initAt`による移動は`Share`も`Drop`も起こさない |
| capacityはcountの延長としてしか増えない | 空き領域を作るたびに番兵を書く | `grow`で書き込みなしにVacantを増やす |
| 有効な範囲はcountだけで表す | container固有のinvariantを番兵値として要素の中へ符号化する | 占有状態は`Slot<V>`の値としてIxPoolが保ち、`peek`で読める |

`Symbol`などmanagedな要素では、二つ目と三つ目の差が大きい。Buffer上のMapで削除したentryは番兵で上書きするまでreferentを
保持し続け、rehashはentryごとにretainとreleaseを往復する。IxPool上では削除した値は取り出した時点で呼び出し元へ移り、rehashは
所有者の数を変えない。unmanagedな要素では、差は主に番兵の書き込み、sum tagの分岐、構築時の`fill`の量になる。

代わりにIxPool上のcontainerは次を払う。

- 占有状態のmetadataを持ち、IxPoolの終了時にはLiveなslotを探して破棄する。
- IxPoolのpreconditionを自分のinvariantで満たす責任を負う。違反はmanaged valueの二重破棄や未初期化carrierの読み出しになり得る。
- growth policy、free list、要素の列挙を自分で書く。

したがって、要素を末尾へ追加していくだけの列、[indexed-graph](../../../../examples/indexed-graph/graph.mal)のように一度作って読むだけの
表、snapshotを取って比べる用途ではBufferで足り、その方が単純である。途中を空ける、値を取り出す、要素を動かす、空きを値なしで
持つ必要があるcontainerで、IxPoolの前提が効く。

## 書く側から見た比較

[列のcontainer](sequences.md)と[keyで引くcontainer](keyed.md)の例をC host上で書いた経験では、削除、取り出し、移動、可変のmetadataを持つcontainerはBufferより
書きやすかった。最も効いたのはMetaである。malには可変のbindingがないため、Buffer上のDequeや木は`head`、`count`、`root`の置き場として
別の`Buffer<USize>`を用意するか、要素の一つへ埋め込む必要がある。IxPoolでは`(head, count) := meta(deque)`のようにstorageと
同じidentityから読める。番兵値を置かず要素型をそのままslotに置けることと、`takeAt`、`initAt`、`moveAt`でアルゴリズムどおりに
値を動かせることも、code量と読みやすさの両方に効いた。

負担は、占有状態とMetaを自分で正しく保つことに集約された。

- `grow`してから`initAt`し、`initAt`の後にMetaを更新する、という対を毎回書く。Bufferの`new`はこれを一つの操作で行う。
- 要素の列挙がないため、木の検査には中順の再帰を書き、Mapの全要素には`peek`でcapacity全体を走査する。
- SlotMapのgenerationや木のfree listのように、Vacantなslotに関する情報の置き場を最初に設計する。

この負担は、よく使う対を`ensureCapacity`のような補助関数へまとめること、占有状態を検査するruntimeでcontainerをtestすること、
[primitive `trap`](../../primitive-trap.md)でcontainer操作単位のtrap messageを出すことで軽くできる。試作で要素型ごとのIxPool実装や
IxPoolの明示的な解放が必要だったのはC hostを経由したためであり、IxPoolの性質ではない。

## runの語彙

`fill`と`copy`というrunの語彙はBufferだけが持ち、hostとの交換は`Host<A>`が持つ（[語彙の分担](../api/buffer-host.md#語彙の分担)）。
他のcontainerがrunを必要とする場面は少なく、必要な場合もslot操作かBufferの経由で足りる。

- stackは`[0, count)`がLiveなので、まとめたpushやpopもslot操作のloopで書ける。
- Dequeの論理的な列は最大二つの区間に分かれる。hostとの交換は`Host<A>`を経由し、成長時に折り返した部分を動かす操作は値を写す
  `copy`ではなく、`takeAt`と`initAt`で移す。
- binary heapは`[0, count)`がLiveだが、coordinateの順は要素の順序ではない。範囲を使うのは配列からの一括構築くらいである。
- Map、SlotMap、木のLiveな集合は区間にならない。rehashや複製は内部でslot遷移を使う。

移す操作を一括にする`moveRange`は[測定後の候補](../api/pool.md#測定後の候補)に置く。

## Vacantが`Unit`だけを持つこと

Vacantなslotは`Unit`しか持たない。IxPoolの上にmalで書くSlotMapと木は、次の空きcoordinateをVacantなslotへ書けないため、free listをMetaに
置くか、`IxPool<USize, USize>`のような別のIxPoolへ積む。これはIxPoolが未初期化carrierを公開しない代わりに生じる制約であり、
空きslotに情報を置く必要があるcontainerは、要素を`[USize, T]`のような直和にして、Liveなslotの第一項で空きと次の空きcoordinateを表す。

Liveなslotを列挙する操作もIxPoolにはない。Map、SlotMap、木の全要素を訪れるには`peek`でcapacity全体を走査するか、container自身が
要素の並びを持つ。live slot iterationをどの層が持つかは[README](../README.md#未決定事項)の未決定事項である。

## IxPoolの輪郭

どのcontainerも共通に使い、IxPoolが提供するものは次である。

- `0`から`capacity - 1`までの線形なcoordinate空間
- coordinateごとの`Slot<V>`の値と、それを読み書きする`peek`、`slot`、`swap`
- 同じidentityに置く共有mutableなMeta
- 明示的な`grow`によるcapacityの拡張と、coordinateを保つrelocation
- 要素型ごとの型付きstorageと、寿命と所有権の正しさ

containerごとに異なり、IxPoolが決めないものは次である。

- Liveなcoordinateの集合の形
- coordinateの意味：列の位置、輪の位置、木の節点、hashのprobe順、要素のidentity
- growth policy、free list、順序、hash、要素の列挙
- 公開preconditionと、それをIxPoolのpreconditionへ写すinvariant

したがってIxPoolは、共有Metaを伴う型付きplaceの線形空間であり、どのplaceを使うかを持ち主が決めるもの、と言える。Bufferは
この空間の使い方の一つにすぎず、IxPoolの意味はBufferに依存しない。
