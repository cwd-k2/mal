# Poolの意味論

Status: Exploratory support document

この文書は、Poolの意味論の核を導き、malの設計方針との対応を管理する。核と周辺の一覧と区分は
[Pool primitive](../api/pool.md)、placeに対するresponsibilityの動きは[responsibilityの図](responsibility.md#responsibilityの動き)、各operationの
`share`と`drop`の回数と順序は[runtime contract](../runtime/contract.md#所有権)を正とする。

## 三つの層

Poolの記述は三つの層に分かれる。sourceから観測できるのは意味論の層だけである。`Share`、`Consume`、`Drop`は
[managed valueのownership](../../../implementation/ownership.md)の語彙であり、[仕様](../../../spec)には現れない。費用の層は、意味論が同じ
operationの間の違いを実装へ約束する。

```mermaid
flowchart TB
  S["意味論<br/>Pool = (m, n, slots)<br/>Slot&lt;V&gt; = [Unit, V]"]
  P["precondition<br/>核は i &lt; n だけ<br/>周辺はLiveかVacantかを加える。未検査"]
  C["費用と所有権<br/>Share、Move、Dropの回数と順序<br/>ImPoolのstorage再利用"]
  S --> P --> C
```

## placeと値

IxPoolは`(m, n, slots)`を一つの共有identityとして持つ。Metaの`m`と各slotは、どちらも値をちょうど一つ持つplaceである。

```text
place  meta       : Meta
place  slot 0..n-1 : Slot<V>      Slot<V> :: [Unit, V]
```

Vacantは値がない状態ではなく、`Slot<V>`の第一項という値である。Liveは第二項である。VacantとLiveという区別はmalの直和の
variantとして現れ、Poolが別に持つ概念ではない。

### Metaとslot

Metaとslotは同じ規則のplaceであり、違いは持つ値の型だけである。この違いはplaceを作るoperationから来る。

- `pool(m)`はMetaの初期値を受け取るため、Metaは最初から`Meta`の値を持つ。
- `grow(pool, k)`はk個のslotを値なしで作る。任意の`V`には既定値がないため、新しいslotには`Unit`を置き、slotの型は`Slot<V>`になる。
- slotから値を取り出した後も、placeは何かの値を持つ必要があり、`Unit`を置く。

## 最小核の導出

核は`pool`、`grow`、`capacity`、`peek`、`swap`、`meta`、`swapMeta`である。核は、他の操作で表せない意味論の核と、
他の操作で書けるが書くと計算量が変わる計算量の核に分かれる（[区分](../api/pool.md#区分)）。

| place | 意味論の核 | 計算量の核 | 派生 |
|---|---|---|---|
| slot | `swap` | `peek` | `slot` |
| Meta | `meta`、`swapMeta` | — | `setMeta` |

### slot

意味の上では、slotは`swap`だけで閉じる。`swap`はplaceの値を入れ替えて古い値を返し、Vacantという値があるため、交換の間に
置いておく値に困らない。

```text
slot(i, s)     = swap(i, s)の結果を捨てる
peek(i)        = r := swap(i, vacant()); swap(i, r); r
```

`peek`の分解で`r`を二回使えるのは、malの値が再利用できるからである。それでも`peek`を計算量の核に置くのは、読み出しを
書き込みにしないためである。分解すると読み出しが二回の書き込みになり、共有中のImPoolや[freeze](../api/pool.md#freezeとthaw)で
共有したstorageを読むたびにO(n)のcopyが起きる。ImPoolでは読み出しがsuccessorを返す更新になる。

逆に`slot`は核に置かない。`swap`の結果を呼び出し側でDropしても、primitive内で旧値をDropしても費用は同じであり、`slot`だけ
では値をMoveで取り出せない。

他のslot operationは`peek`と`swap`の合成である。Liveかどうかは結果の除去で分かる。

```text
isLive(i)      = peek(i)[() -> false, (_) -> true]
initAt(i, v)   = slot(i, live(v))
putAt(i, v)    = slot(i, live(v))
dropAt(i)      = slot(i, vacant())
takeAt(i)      = swap(i, vacant())のLiveの値
moveAt(a, b)   = slot(b, swap(a, vacant()))
```

### Meta

Metaの型には、任意の型について用意できる値がない。交換の間に置いておく値がないため、`meta`は`swapMeta`から導けず、
読み出しと交換の二つが意味論の最小になる。

```text
setMeta(m)     = swapMeta(m)の結果を捨てる
swapMeta(m)    = r := meta(); setMeta(m); r
```

containerがMetaを`[Unit, X]`のような直和にすれば、Metaも`swapMeta`だけで閉じる。これはcontainerの選択である。

### 構造の操作

`capacity`は`n`を読む。`n`はPool自身の構造であり、`peek`と`swap`のpreconditionが参照する。

`grow`は、identityを保ったままcoordinate空間を広げる唯一のoperationである。新しいPoolを作って要素を移すと、古いPoolの
aliasは新しいPoolを追えない。identityを共有するというIxPoolの性質を成長の後も保つために`grow`が要る。

`pool`は新しいidentityを作る唯一のoperationである。`n`を`0`で始め、大きさは`grow`で与える。

### MetaをPoolに置く理由

malには可変なbindingもproductのfieldをその場で書き換える手段もない。`(Meta, IxPool<Unit, V>)`というproductのMetaは値として
copyされるため、変更をaliasが観測できない。Metaはどこかのidentityの中に置く必要がある。

意味の上では、Metaは容量1のPoolのslot 0を同じidentityへ融合したものである。

```text
IxPool<Meta, V>  ≅  (IxPool<Unit, Meta>, IxPool<Unit, V>)    二つが同じidentityを共有し、第一Poolのslot 0は常にLive

meta(pool)         = peek(metaPool, 0)のLiveの値
swapMeta(pool, m)  = swap(metaPool, 0, live(m))のLiveの値
```

融合すると、分離した形でcontainerのinvariantだった「slot 0は常にLive」を型が保証する。Metaは`Slot`で包まず、preconditionも
持たない。分離した形との違いは、identity、allocation、handleのcopyごとのretainが一つで済むことである。

`grow`しない`IxPool<Meta, V>`は、Metaだけを持つ可変なcellとして使える。

ImPoolでは、Metaは`(Meta, ImPool<Unit, V>)`というproductと同じであり、productごと更新できる。MetaをPoolに置く必要があるのは
identityを共有するIxPoolだけである。

## malの設計方針との対応

[minimality](../../../design/minimality.md)、[表現と関係を分ける](../../../design/representation-and-relations.md)、
[値、解釈、control](../../../design/value-interpretation-and-control.md)に照らすと、このモデルには次の性質がある。

意味論が新しく持ち込むのは、可変なidentityと、`n`を広げる`grow`だけである。slotの占有はmalの直和`[Unit, V]`で表し、Metaと
slotは同じ規則のplaceである。IxPoolは、Metaと`Slot<V>`の有限列を持つ共有identityと言える。
[Buffer上のemulation](../prototypes.md#二つの試作)が同じ意味を再現できたのは、Bufferが同じplaceの列を持つからである。

Moveは値の消費ではなく、placeの値の入れ替えとして現れる。malの値は再利用できるcarrierであり、affineなのはcontrolだけである。
`swap`は値を消費せずplaceへ`Unit`を残すため、所有権の移動にlinear typeやborrow checkerを要しない。

IxPoolはcoordinateで引く有限carrierであり、Liveな集合の形とcoordinateの意味はcontainerのoperationとinvariantが与える。これは
`finite carrier + relation operations + invariants = domain structure`の形にそのまま当てはまる。

核の未検査preconditionは範囲`i < n`だけである。LiveとVacantの一致は核では直和の除去で扱われ、違反はmemory safetyではなく
返る値に現れる。周辺の`getAt`、`initAt`、`putAt`、`takeAt`はこの一致を再び未検査preconditionにして費用を下げる。containerは
核だけで書くか、invariantで一致を保証して周辺を使うかを選べる。

IxPoolと`Buffer<[Unit, V]>`の違いは、`swap`によるMoveと、tagをslotの外に置けるlayoutの二点に絞られる。どちらも費用の層であり、
[論理構造とlayoutを分ける](../../../design/representation-and-relations.md#論理構造とlayout)方針の一例になる。
