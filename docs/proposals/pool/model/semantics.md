# Poolの意味論

Status: Exploratory support document

この文書は、Poolのstate、place、source carrierの観測則、operation lawを定める。言語全体での位置と採択理由は
[位置付けと根本モデル](foundations.md)、source APIの区分は[Pool primitive](../api/pool.md)、responsibilityの効果は
[runtime contract](../runtime/contract.md#responsibility)を正とする。

## Pool state

Poolの論理状態は次の三要素からなる。

```text
PoolState<H, V> = (h, n, slots)

h     : H
n     : USize
slots : Fin(n) -> Slot<V>

Slot<V> = Unit + V
```

`h`をHeader、`n`をlogical capacity、`[0, n)`をcoordinate空間と呼ぶ。各coordinateは`Slot<V>`を一つ持つ。
`Slot<V>`の第一項をVacant、第二項をLiveと呼ぶ。Vacantは値がない状態や未初期化の`V`ではなく、`Unit`を持つ正規のsum valueである。

Poolはcontainer上の意味を持たない。どのcoordinateがLiveか、coordinateが列の位置、heap node、hash bucket、またはobject identityの
どれを表すかは、Poolを使うoperationとinvariantが決める。

## place

Pool stateは、型の異なる二種類のplaceを持つ。

```text
Header              : H
Slot(i), i in Fin(n) : Slot<V>
```

すべてのplaceは値をちょうど一つ持つ。placeに共通する意味論の核はreadとexchangeである。

```text
read(p)               = pの値を返し、pを変えない
exchange(p, incoming) = pをincomingへ置き換え、以前の値を返す
```

ここで定めるのは、readがstateを保存し、exchangeが旧値と新値を入れ替えるというvalueとstateの規則までである。readを`Share`へ、
exchangeをresponsibilityの移動へlowerする規則は[runtime contract](../runtime/contract.md#responsibility)が所有する。意味論と実装効果を
分けても、HeaderとSlotが同じplace lawを持つことは変わらない。

source APIが`meta`と`peek`、`swapMeta`と`swap`に分かれるのは、Headerが`H`、Slotが`Slot<V>`を持ち、malがplaceの型をresultへ
依存させる型を持たないためである。別のlifecycle規則があるからではない。

## source carrierと観測

IxPoolとImPoolはどちらもsource valueであり、同じPool state lawを持つ。違いは同じcarrierを別bindingへ渡した後の更新をどう観測するかにある。

```text
IxPool<H, V> = handle value for identity i carrying PoolState<H, V>
ImPool<H, V> = snapshot value representing PoolState<H, V>
```

### IxPool

IxPool valueはidentityへのmanaged handleである。handleを別のbindingへ渡しても同じidentityを共有し、どのhandleからの更新も同じstateを変更する。
IxPoolのstate transitionはidentityを保存する。source APIがsuccessor IxPoolを返す形を採る場合、そのresultはinputと同じidentityで
あり、一意なstate tokenではない。別のIxPool handleはresultを受け取らなくても更新を観測する。

### ImPool

ImPool valueはPool stateのsnapshotを表す。更新はsuccessor snapshotを返し、inputが表すstateを変えない。

```text
update : State A -> State B
observe State A after update = observe State A before update
```

実装はinputへの今後の観測と区別できない場合にstorageをsuccessorへ移してよい。区別できるaliasがあればstorageを複製する。
この選択は物理的なcopy-on-writeであり、上のsnapshot semanticsを変えない。

## 構造のoperation

place operationだけではcoordinate空間を構成できないため、Poolはconstruction、capacity observation、extensionを持つ。

```text
pool(h)       = (h, 0, empty)
capacity(P)   = P.n
grow(P, k)    = P' where
    P'.h = P.h
    P'.n = P.n + k
    P'.slots[i] = P.slots[i]  when i < P.n
    P'.slots[i] = Vacant      when P.n <= i < P'.n
```

`grow`は既存coordinateを保存する。これはlogical placeの保存であり、payloadの物理address、backing allocation、strideの保存ではない。
Poolはcapacityを縮めない。containerが要素を削除または回収するときはSlotをVacantへし、coordinateの再利用規則を自身のinvariantで
定める。compactionや別Poolへの移動でcoordinateの意味が変わる場合は、container operationがremapを所有する。

`n + k`または必要なstorage sizeをtargetで表現できない場合とallocation failureは、既存Engram allocationと同じくtrapする。
このfailureはcallerがpreconditionとして事前に成立させられないため、未検査preconditionにはしない。

## operation law

operation lawはsource signatureより先に、Pool state間のtransitionとして定める。ここでは`P`と`P1`を更新前後のstateとする。

```text
pool       : H -> State<H, V>
grow       : (State<H, V>, USize) -> State<H, V>
capacity   : State<H, V> -> USize
peek       : (State<H, V>, USize) -> Slot<V>
swap       : (State<H, V>, USize, Slot<V>) -> (State<H, V>, Slot<V>)
meta       : State<H, V> -> H
swapMeta   : (State<H, V>, H) -> (State<H, V>, H)
```

IxPool APIはtransition後のstateを同じidentityへcommitするため、更新が`Unit`または旧値だけを返せる。ImPool APIはinput snapshotを
変えないため、successor stateをresultへ含める。IxPoolも同じidentityへのhandleを返す形へ揃えるかはAPIとcostの選択であり、state lawには
影響しない（[Pool primitive](../api/pool.md#impool)）。

次のlawが両carrierに共通する。`P1`はoperationが返すsuccessorとする。

```text
capacity(pool(h)) = 0
meta(pool(h)) = h

capacity(grow(P, k)) = capacity(P) + k
meta(grow(P, k)) = meta(P)
peek(grow(P, k), i) = peek(P, i)                  when i < capacity(P)
peek(grow(P, k), i) = Vacant                     when capacity(P) <= i

(P1, old) = swap(P, i, incoming)
peek(P1, i) = incoming
old = peek(P, i)
peek(P1, j) = peek(P, j)                         when i != j

(P1, old) = swapMeta(P, incoming)
meta(P1) = incoming
old = meta(P)
capacity(P1) = capacity(P)
peek(P1, i) = peek(P, i)
```

IxPoolではtransitionをcommitする前のstateを`P`、commit後に同じidentityから観測するstateを`P1`と読む。ImPoolでは更新後も
`P`を旧snapshot、`P1`をsuccessor snapshotとしてそれぞれ観測できる。

## 最小核の導出

核は、他のmal operationでは意味を表せない意味論の核と、意味は表せても計算量が変わる計算量の核に分かれる。

| 対象 | 意味論の核 | 計算量の核 | 派生 |
|---|---|---|---|
| construction | `pool` | — | — |
| coordinate空間 | `grow`、`capacity` | — | — |
| Slot place | `swap` | `peek` | `slot` |
| Header place | `meta`、`swapMeta` | — | `setMeta` |

### Slot place

SlotにはVacantという任意の`V`について構成できる値があるため、意味だけならreadをexchangeから導ける。

```text
(P1, old) = swap(P, i, Vacant)
(P2, _)   = swap(P1, i, old)
result    = (P2, old)
```

しかしこの分解はreadを二回のwriteにする。ImPoolでは共有中のstorageを複製する。IxPoolでも観測しか行わないoperationをwriteとして
backendへ見せる。このため`peek`は計算量の核である。

writeはexchangeの旧値を捨てるだけなので派生できる。

```text
slot(P, i, value) = first(swap(P, i, value))
```

### Header place

任意の`H`について一時的に置ける値はないため、`meta`を`swapMeta`だけから導けない。`swapMeta`も旧Headerを値として取り出す唯一の
operationである。したがって両方が意味論の核になる。writeは旧Headerを捨てて導く。

```text
setMeta(P, value) = first(swapMeta(P, value))
```

### coordinate空間

範囲外accessは結果を持たないため、`capacity`を`peek`の反復や失敗から導けない。`grow`はIxPool handleのreferentまたはImPool
snapshotのcoordinate空間を保存したまま広げる唯一のoperationである。新しいPoolへ要素を移すだけでは、IxPoolの既存handleが
同じidentityを観測できない。

## HeaderをPoolに置く理由

Headerはcontainer policyそのものではなく、Pool identityに属するdistinguished placeである。containerはcount、head、root、free listなど
必要なstateを`H`として選び、その解釈を自身のinvariantで定める。

malには可変bindingもproduct fieldをその場で更新するoperationもない。`(H, IxPool<Unit, V>)`というproductでは`H`が別の
product valueとして保持され、IxPool handleがHeaderの変更を共有できない。別の可変cellとIxPoolを組にすると、二つのidentityが常に対応するという
同期規約、二つのallocation、二つのlifetimeが必要になる。

概念上、Headerは同じidentityに融合した異型のplaceである。

```text
IxPool<H, V>
  ~= HeaderPlace<H> + IndexedPlaces<Slot<V>>
     under one identity and lifetime
```

`grow`しない`IxPool<H, V>`はHeader placeだけを持つmutable cellとして使えるが、これは別のcell mechanismを追加したのではなく、
capacity 0のPoolである。

ImPoolでは`(H, ImPool<Unit, V>)`というproductを更新してもsnapshot semanticsを保てるため、Header融合は意味論上必須ではない。
それでも同じPool stateをIx/Imで共有することで、containerのoperation lawとinvariantを一度だけ記述できる。

## precondition

核の未検査preconditionは`i < capacity(P)`だけである。範囲内ならSlotがVacantかLiveかにかかわらず`peek`と`swap`は定義される。
LiveかVacantかを仮定する`getAt`、`initAt`、`takeAt`、`putAt`は周辺operationであり、追加preconditionによってtag分岐、`Share`、
`Drop`を省く。

この分担により、核はuninitializedな`V`を読まず、Live/Vacant違反を新しいprimitive semanticsにしない。containerは核だけを使って
Slotを除去するか、自身のinvariantで状態を保証して周辺operationを使う。

## 表現からの独立

Pool stateは次を規定しない。

- Header、occupancy、payloadが同じallocationにあるか。
- occupancyがbyte tag、bitmap、container invariantからの導出のどれか。
- payloadがcanonical memory layoutかruntime value layoutか。
- physical capacityがlogical capacityより大きいか。
- growthが`realloc`、allocate-and-move、chunk追加のどれか。
- ImPoolがflat owner、view、または別の共有表現を使うか。

sourceから観測できるのはoperation law、handleとsnapshotの観測則、precondition、trap、約束された計算量だけである。C/LLVM loweringが守る境界は
[runtime contract](../runtime/contract.md#semantic-identityとallocation-object)に置く。

## malの設計原則との対応

Poolは[表現と関係を分ける](../../../design/representation-and-relations.md)のfinite carrierを提供する。Poolの型が保証するのはHeader、
finite coordinate、Slot state、authorityだけであり、tree、Map、sequenceといったdomain relationはoperationとinvariantが与える。

[authority](../../../design/authority.md)に対しては、PoolをEngramに閉じ、Extern resourceのpermissionやlifetimeを持ち込まない。
allocatorとreference countはauthorityではなく実装mechanismである。

[minimality](../../../design/minimality.md)に対しては、現行Bufferのshared identityを再利用し、vacancyをsum、移動をexchange、回収を既存の
managed responsibilityから導く。新しく必要な根は、container policyを持たないindexed place stateだけである。
