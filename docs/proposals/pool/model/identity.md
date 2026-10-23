# identityと型判定

Status: Exploratory support document; rebased on mal v0.7

この文書はpointerとIxPoolのcarrier、`Storable`、lifecycle、extern admissionの境界を管理する。現行規則は
[`Buffer`](../../../spec/memory.md)、[`extern`](../../../spec/extern.md)、
[`C host value API`](../../../spec/c-host-api.md)を正とする。

## carrierと到達性

| source carrier | 到達先 | copyしたcarrierからの観測 |
|---|---|---|
| `*A` | 一つの共有されたlogical place | pointee stateと値の変更を共有観測する |
| `IxPool<A>` | 有限で成長可能なcoordinate空間 | growthと各placeの変更を共有観測する |
| external opaque carrier | extern-owned referent | extern contractに従う |
| plain data | carrier自身 | 新しいvalueだけが異なる |

pointerはpointee valueのidentityではなく、valueを置けるplaceのidentityへ到達する。standalone pointerは自身の
ownerを持ち、`at(pool, i)`のresultはpool identityとcoordinateの組からlogical placeへ到達する。同じpoolと
coordinateから得たpointerはcarrierが別でもstateを共有する。異なるpoolの同じ数値indexは別のplaceを指す。

IxPool handleを失っても、そこから得たpointerが残る間はpool ownerを回収できない。これはdanglingを防ぐ
lifecycle規則であり、sourceへaddressや明示freeを導入するものではない。

## `Storable`

初回採択では現行の回収可能性とcycle制約を変えず、pointee AとIxPool element Aの両方に`Storable(A)`を要求する。

```text
Storable(Unit | numeric scalar | ByteSize | USize | Symbol)
Storable(external opaque type)
Storable((A...))                 if all Storable(A)
Storable([A...])                 if the sum has at least two variants and all Storable(A)
Storable(Buffer<A>)              if Storable(A)
Storable(*A)                     if Storable(A)
Storable(IxPool<A>)              if Storable(A)
not Storable(function)
not Storable(empty sum)
```

この再帰によりpointerやIxPool handleを別のplaceへ保存できる。保存されるのはcarrierへのresponsibilityであり、
referentのdeep copyやexclusive authorityではない。recursive value typeを導入せず、file-local opaque representationの
既存の再帰検査も保つ。functionを除く現行理由も維持し、closure captureに隠れたowner back-edgeを許さない。

`Storable`はhash安定性やserialization可能性を意味しない。Map key、deep snapshot、永続表現は各APIが別に要求する。

## lifecycle

```text
Lifecycle(T) = Trivial | Owned(share glue, drop glue)
```

`*A`と`IxPool<A>`はOwnedである。Shareは同じidentityへの到達性を一つ増やし、Dropはその到達性を終了する。
最後のownerが終了すると、standalone placeはLiveなpointeeを、IxPoolは全Live coordinateの値を一度ずつDropする。
Aのcarrier layoutとglueは既存のtarget-specific planを再利用する。

pointerでguardされたrecursive opaque typeを将来認めるとowner cycleを作り得る。非循環のlistやtreeは、最後の
外部responsibilityからpointeeをDropし、そのpointer fieldをDropする再帰で回収できる。strong cycleはreference countを
0にしないため自動回収を保証せず、利用者が最後の外部到達を失う前に`take`や`store`でcycleを切る。cycle collectionや
weak reachabilityはrecursive typeの成立条件にせず、必要なら独立して判断する。

初回採択は現行のStorable境界とrecursive representationの拒否を保つ。既存のopaque型検査で循環を排除できない例が
見つかった場合は、コードだけで挙動を選ばずspec issueとして記録する。

## extern admission

pointerをplaceへ保存できることと、C signatureへ出せることは別の判定である。初回採択では`*A`とIxPoolを
extern signatureへ認めず、runtime operationも`mal.h`のpublic APIにしない。後続で公開する場合はtarget carrier、
borrow/move、型別Share/Drop、growth後の再解決を一つのABI decisionとして追加する。`*A`のspellingからraw C
pointerとのrepresentation compatibilityは導かれない。
