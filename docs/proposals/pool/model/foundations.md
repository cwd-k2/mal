# pointerとIxPoolの位置付け

Status: Exploratory support document; rebased on mal v0.7

この文書はpointerとIxPoolがmal全体のどのauthorityを担い、container policyとどこで分かれるかを定める。
operation lawは[意味論](semantics.md)、source surfaceは[pointerとIxPoolのAPI](../api/pool.md)、runtime representationは
[runtime contract](../runtime/contract.md)を正とする。

## 中心命題

`*A`は数値addressを持たないtyped location pointerである。pointerはlocationへのmanaged handleであり、copyしても
同じplaceへ到達する。pointeeはVacantまたはLive(A)であり、aliasからのstoreとtakeを共有観測する。

`IxPool<A>`はpointerを要素値として格納したcontainerではなく、有限なcoordinate空間と、各indexからplaceへ
到達する規則を持つidentityである。

```text
at : (IxPool<A>, USize) -> *A
```

同じpool identityとindexから得たpointerは同じplaceを指す。place identityはpayload addressと異なり、growth、
relocation、inline storageからflat storageへの移行で変わらない。IxPoolはcoordinateを縮めず、pointeeがVacantへ
戻ってもcoordinate自体は残る。

## 言語全体での位置

```text
value and control core
    immutable binding、lambda、application、product、sum

pointer authority
    *A、shared place、Vacant / Live、load、store、take

IxPool reachability identity
    finite indexed coordinates、growth、indexから*Aへの到達

container policy written in mal
    *Header、Live coordinateの形、growth policy、hash、free list、順序

implementation mechanism
    managed owner、occupancy bitmap、payload backing、reference count、LLVM layout
```

pointerとIxPoolは純粋なcore calculusから導かれないEngram storage primitiveである。source bindingはimmutableの
ままであり、可変性はpointer operationとして明示される。pointerはborrow、exclusive owner、integer addressではない。

## C pointerから残すものと外すもの

| C pointer | malの`*A` / IxPool |
|---|---|
| pointee type | `*A`のA |
| read | `load(p)`または`take(p)` |
| store | `store(p, value)` |
| aliasからの共有観測 | 保つ |
| pointee lifetime | pointer carrierがmanaged ownerを保持 |
| pointer arithmetic | 持たない。IxPool identityに相対的な`at(index)`を使う |
| address、ordering、integer conversion | 持たない |
| null / dangling | 初回案では持たない |
| uninitialized storage | danglingでなく正規のVacant state |
| `&binding` | 持たない。storage primitiveだけがpointerを作る |

`*`というtype spellingはaddressを約束せず、「このvalueはtyped locationを指す」というsource上の意味を
示す。IxPoolのindexはbyte offsetでなくlogical coordinateであり、異なるpoolの同じ数値indexは別のplaceを表す。

## containerへの分解

denseな可変列のstateを基礎へ分解すると次になる。

```text
Dense<A>
  = *USize             // countを指す
  + IxPool<A>          // indexed places
  + invariant [0, count) is Live
  + sequence policy
```

Map、Deque、heap、木も同じ二つの基礎を使い、Headerの型とLive coordinateの形だけを変える。IxPoolはcount、
head、root、hash、key、free listを知らない。本proposalではこの構造の説明に既存Bufferを参照することはあっても、
BufferやSymbolの採択済みsurfaceを同時に再設計しない。

## authority、responsibility、representation

| 層 | 問い | 答え |
|---|---|---|
| authority | どの変更をどのaliasから観測するか | pointerとIxPoolのshared identity |
| responsibility | carrierを誰が保持し、いつ終了するか | compilerの`Share`、`Consume`、`Drop` |
| representation | locationとpayloadをどこに置くか | backend/runtimeのowner、coordinate、backing allocation |

pointerを別bindingへ渡してもexclusive authorityは移らない。source valueは通常どおり再利用でき、compilerは
後続useがあればShare、last useならConsumeへlowerできる。runtime representationが一時的に一意でも、source
authorityをuniqueとは解釈しない。

IxPoolから得たpointerはpool identityへのresponsibilityを持つ。最後のIxPool handleが消えてもpointerが残れば
pool ownerと全coordinateは生き続ける。これはdanglingを作らないための到達性規則であり、一つのpointerがpool
全体のstorageを保持するcostを伴う。

## SlotとHeaderを基礎から外す

以前の案は`IxPool<Header, V>`の中に常にLiveなHeader placeと`Slot<V> = [Unit, V]`を持つindexed placeを
融合していた。pointerを基礎にすると、Headerは`*H`へ置ける。VacantとLiveはpointee stateなので、`Slot<V>`を
source valueとして読み書きする必要もない。

```text
以前: IxPool<H, V> = Header + indexed Slot<V> under one identity
現在: *H + IxPool<V>, at(pool, i) -> *V
```

この分解はmetadata placeとpoolという二つのidentityを作り得るが、opaque carrierへどう束ねるかはcontainerが選ぶ。
`(*H, IxPool<V>)`ならaliasは二carrierを直接共有し、pool identityを差し替えられない。`*(H, IxPool<V>)`ならaliasは
descriptor placeを共有し、そのpointeeにあるpool handleを交換できる。内部pointerがopaque境界からescapeしない限り、
どちらも同じ公開container semanticsを実装できる。runtimeはlogical identityとlifecycleを保つ範囲でco-allocateしてよい。

## minimality

- mutable cell、container metadata、out parameterを`*A`の一つのlawで説明する。
- read/writeを`load`、`take`、`store`へ寄せ、indexed storage専用operationを重複させない。
- vacancyをelement型の番兵値や公開`Slot<A>`へ押し込まない。
- Header、hash、free list、順序、key validityをIxPoolへ固定しない。
- address、allocator、borrow、lifetime annotation、明示freeをこの最小核の成立条件にしない。
- general lvalue、mutable binding、field addressabilityをpointer coreと同時には要求しない。
- growth後のphysical addressを保存せず、owner identityとcoordinateからplaceを解決する。

反対に、pointerを導入してもrecursive type、projection、stable domain key、snapshot、external resource lifecycleは
自動的に得られない。必要が現れたときに独立した規則として判断する。
