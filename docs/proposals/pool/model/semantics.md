# pointerとIxPoolの意味論

Status: Exploratory support document; rebased on mal v0.7

この文書は暗黙store、pointer、pointee state、IxPool identity、operation lawを定める。source surfaceは
[pointerとIxPoolのAPI](../api/pool.md)、responsibility effectは
[runtime contract](../runtime/contract.md#responsibility)を正とする。

## 暗黙storeとpointer

評価状態はsourceから直接参照できないstore `Σ`を持つ。

```text
Σ : PlaceId -> PlaceState<Value>
PlaceState<A> = Vacant | Live(A)
```

`p :: *A`というvalueは`PlaceId`へのmanaged pointerである。pointerをcopyしてもfreshなplaceを作らず、同じ
`PlaceId`へ到達する。Vacantは無効pointerや任意bit patternではなく、runtimeが認識する正規のpointee stateである。
LiveなplaceだけがAのcarrierとresponsibilityを持つ。

standalone placeのconstructionはfreshなidentityを作る。

```text
vacant<A>() = fresh p where Σ[p] = Vacant
place<A>(v) = fresh p where Σ[p] = Live(v)
```

pointer operationは次のstate transitionを持つ。

```text
isLive(p) = false                         when Σ[p] = Vacant
isLive(p) = true                          when Σ[p] = Live(v)

load(p) = v                              when Σ[p] = Live(v)

store(p, v):
    old = Σ[p]
    Σ[p] = Live(v)
    return Unit

take(p):
    require Σ[p] = Live(v)
    Σ[p] = Vacant
    return v
```

`load(p)`はvをShareして返す。`store(p, v)`はVacantを初期化し、Liveな旧値を置換する。`take`はvを
Moveして返す。loadとtakeのLive条件は未検査preconditionである。

storeは通常のfunction applicationとしてpointer expressionとvalue expressionを左から右へ各一度評価する。
source nameやaddressable expressionという分類は要求しない。従って次も同じlawを使う。

```mal
store(p, value);
store(pool.at(i), value);
```

`Slot<A> = [Unit, A]`をfirst-class valueへ返さないため、occupancyとpayloadを一度に観測するtotal operationは
核に含めない。Live同士の`swap(p, v)`は値のMoveを保つ周辺primitiveとして追加できるが、pointerの意味には
必要ない。

## IxPool state

IxPool identity `i`の論理状態はcoordinate数と各coordinateのplace stateからなる。

```text
PoolState<A> = (n, places)
n      : USize
places : Fin(n) -> PlaceState<A>
```

IxPool valueはidentity `i`へのmanaged handleである。handleをcopyしても同じidentityを指す。`at(i, k)`は
standalone placeを作らず、pool identityとcoordinateで決まるplaceへのpointerを返す。

```text
PlaceId = PoolPlace(i, k)                  when k < n
at(handle i, k) = pointer to PoolPlace(i, k)
```

同じidentityとcoordinateから何度`at`しても同じplaceを指す。返したpointerはpool ownerへのresponsibilityを
持ち、IxPool handleがなくなった後も有効である。

## IxPool operation law

```text
pool<A>() = fresh identity i carrying (0, empty)

capacity(i) = PoolState(i).n

grow(i, amount):
    old = PoolState(i).n
    next = old + amount
    preserve places[k]                     when k < old
    set places[k] = Vacant                 when old <= k < next
    set n = next

at(i, k) = pointer to PoolPlace(i, k)      when k < capacity(i)
```

`grow`はcoordinateを削除、並べ替え、再利用しない。既存pointerのlogical pointeeとstateを保存し、末尾に
Vacantなplaceを追加する。physical backing allocation、payload address、occupancy bitmapの位置は保存対象ではない。

IxPoolはcapacityを縮めない。containerが要素を削除するときはpointerを`take`してVacantへ戻し、coordinateを
いつ再利用するかを自身のinvariantで決める。別poolへの移動でdomain keyが変わる場合、remapはcontainerが所有する。

## identityの分離

次を同一視しない。

```text
IxPool semantic identity
logical place identity = (pool identity, coordinate)
pointer carrier
managed runtime owner
current backing allocation
payload address
pointee value
containerがcoordinateへ与えるdomain上の意味
```

growthは最初の二つを保存し、backing allocationとpayload addressを変更してよい。`*A` carrierは実装上owner
handleとcoordinateを持ち、pointer operation時にactive backingを解決できる。sourceはそのrepresentationを観測しない。

## container state

IxPoolはHeaderを持たない。container固有のmetadataは通常のplaceへ置く。例えば次の二表現を選べる。

```text
FlatContainer<H, V>       = (*H, IxPool<V>)
DescriptorContainer<H, V> = *(H, IxPool<V>)
```

flat formをcopyしたaliasはmetadata placeとpool identityの両方を直接共有するため、pool handleを差し替えず同じ
IxPoolを更新する。descriptor formのaliasは外側のplaceを共有するため、そのpointeeにあるpool handleを交換しても
追従する。file-local opaque typeが正しい組を構築し、公開operationがrelationとinvariantを保つ限り、この違いは
container内部の選択である。IxPool自身はHeaderの解釈も、どのcoordinateがLiveかも知らない。

内部の`*V`をopaque境界から返すことは、そのplaceへのload/store/take authorityを公開することである。通常の
container APIは値を返し、内部pointerをescapeさせない。pointerを返すAPIは、そのauthorityとcontainer invariantへの
影響を独自のcontractとして定める。

container operationは複数placeを順番に更新し、return時にinvariantを回復する。unknown functionを呼ぶ場合は、
その前に公開invariantを回復するか、再入をcontractで禁じる。

## precondition

| form | 未検査precondition |
|---|---|
| `load(pointer)`、`take(pointer)` | pointeeがLive |
| `at(pool, index)` | `index < capacity(pool)` |

`isLive`と`store`はVacantとLiveの両方で定義される。standalone construction、`pool`、`capacity`、`grow`は
未検査preconditionを持たない。container実装は公開preconditionをこれらへ写すinvariantを所有する。

## 表現からの独立

意味論は次を規定しない。

- standalone pointerとpool-derived pointerが同じcarrier layoutか。
- pointer carrierがownerとcoordinateをinlineに持つか。
- occupancyをbitmap、byte tag、container invariantのどれで表すか。
- payload、occupancy、owner metadataが同じallocationにあるか。
- growthが`realloc`、allocate-and-move、chunk追加のどれか。
- metadata placeとIxPoolをどうopaque carrierへ束ね、同じallocationへco-allocateするか。

sourceから観測できるのはidentityを通じたaliasing、pointee state transition、capacity、indexからplaceへの到達、
precondition、trapだけである。
