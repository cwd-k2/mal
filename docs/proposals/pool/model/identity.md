# identityとStorable

Status: Exploratory support document

この文書は、Pool案のidentity軸、すなわち値の変更を誰が観測するかと、それによって決まる`Storable`の可否を管理する。
所有権の遷移は[runtime contract](../runtime/contract.md#所有権)、ImPoolの意味は[Pool primitive](../api/pool.md#impool)、copy-on-writeの動作例は
[Vector](../containers/sequences.md#vector)を正とする。現行の`Storable` judgmentは
[AddressとBuffer](../../../spec/memory.md#storable)に定める。

## 所有権とidentity

所有権は誰がresponsibilityを持つかを、identityはaliasが同じ変更を観測するかを表す。二つは独立しており、Pool案の型は
次のように並ぶ。

| | identityを共有する | identityを持たない |
|---|---|---|
| 所有する | `IxPool<Meta, V>` | `ImPool<Meta, V>` |
| 所有しない | `Address`、coordinateやhandleのような参照 | 通常のdata |

`Storable`にできないのは、所有とidentity共有を両方持つ型である。identityを共有する値をstorageへ保存すると、その後のmutationを
storage内のaliasから観測できるためである。これは[D075](../../../history/decisions/active/D075.md)がBufferの要素を値に限った理由と
同じであり、安全性ではなく言語の意味の選択である。

owner cycleはこの除外の理由にならない。malは表現に寄与する再帰型を持たず、functionは`Storable`でないため、storageの要素から出る
owner edgeは常に真に小さい型の値を指す。`IxPool<M, IxPool<M, V>>`のように自身を要素にする型は書けず、owner cycleは型の上で
生じない。

`ImPool`はidentityを捨て、参照は所有を捨てることで、それぞれ`Storable`になる。`IxPool<Meta, V>`は`Storable`でも
`Representable`でも`HostMappable`でもなく、`IxPool<Meta, IxPool<...>>`のような入れ子も認めない。

## 型形成条件

IxPoolと`ImPool`の形成は、現在のclosed judgmentである`Storable(Meta)`と`Storable(V)`を要求する。opaque型の`Storable`、
`Representable`、lifecycleはcompilerがhidden representationから導き、opaque型がこれらのpropertyを新たに宣言して
representationの制約を迂回することはできない。したがって`opaque Buffer<T> :: IxPool<USize, T>`は`Storable`にならず、
`opaque Vector<T> :: ImPool<USize, T>`は`Storable(T)`のもとで`Storable`になる。

将来plugin leafを`Storable`へ追加するには、storage内のShareが安全であること、aliasが後の
mal-owned mutationを観測しないこと、container edgeからowner cycleを作らないことを登録時に示す。

## 判定の分割（案）

現在の`Storable`は、storageに保持できることと、保持した値の意味が後から変わらないことの二つを一つの判定で表している。
owner cycleが型の上で生じない以上、前者だけならBufferやIxPoolも保持できる。そこで次の案を検討する。この案は
[D075](../../../history/decisions/active/D075.md)の見直しを伴い、採択していない。

| 判定 | 問い | 要求される場所 |
|---|---|---|
| `Storable` | malのstorageに保持できるか | Bufferの要素、IxPoolの要素とMeta |
| `Stable` | 保持した値の意味が後から変わらないか | ImPoolの要素とMeta、`freeze`するIxPoolの要素とMeta、Mapのkeyのように値の意味を前提にする場所 |
| `Representable` | hostとcopyできるcanonical layoutを持つか | Vectorの`from`と`into`、canonical memory helper |
| `HostMappable` | extern境界をそのまま渡れるか | externのparameterとresult |

| 型 | Storable（現在） | Storable（案） | Stable | Representable | HostMappable |
|---|---|---|---|---|---|
| `Unit`、numeric scalar、`ByteSize`、`USize`、`Address` | ○ | ○ | ○ | ○ | ○ |
| `Symbol` | ○ | ○ | ○ | × | × |
| `ImPool<M, V>` | ― | ○ | ○ | × | × |
| `Buffer<A>` | × | `A`がStorableなら○ | × | × | × |
| `IxPool<M, V>` | ― | ○ | × | × | × |
| external opaque | × | × | × | × | ○ |
| function | × | × | × | × | × |

IxPoolの形成は`Storable(Meta)`と`Storable(V)`を、ImPoolの形成は`Stable(Meta)`と`Stable(V)`を要求するため、形成できるIxPoolと
ImPoolは表の値を無条件に持つ。productとsumは要素から、file-local opaque型はhidden representationから導く。案では`Representable`⊂`Stable`⊂`Storable`となり、
`Stable`は現在の`Storable`に`ImPool`を加えたものになる。`Address`は背後のものを指すが値そのものは変わらないため、
`Stable`である。external opaqueは値が変わるからではなく、寿命がEngramの回収と結びつかないため
`Storable`の外に残る。

案を採ると`Buffer<Buffer<T>>`、`HashMap<K, Buffer<V>>`、IxPoolを要素にするIxPoolを書ける。代わりに、
`fill`で同じ内側のBufferをすべての位置へ置くとそれらがaliasになり、`copy`は浅くなる。値の意味が要る場所は`Stable`を要求して、
このaliasを型で排除する。

## 入れ子

現在の判定では、値は入れ子にでき、identityは入れ子にできない。`Vector<Vector<T>>`やvalueが`Vector<V>`のMapは書けるが、
`Buffer<Buffer<T>>`、valueが`Buffer<V>`のMap、IxPool上のMapを要素にするcontainerは書けない。containerを利用者が書く
基盤として、この穴の扱いを決める必要がある。入れ子を作る経路は三つある。

| 経路 | 内側 | 内側の更新 | 必要なもの |
|---|---|---|---|
| 値の入れ子 | ImPool上のcontainer | 外側から`takeAt`で取り出して更新し、戻す | 内側の値版のcontainer |
| coordinateの入れ子 | 一つのIxPoolにまとめた要素 | coordinateを引いてその場で更新する | coordinateの回収をcontainerが行う |
| identityの入れ子 | IxPool上のcontainer | aliasを通してその場で更新する | [判定の分割案](#判定の分割案)と[D075](../../../history/decisions/active/D075.md)の見直し |

値の入れ子は、判定を変えずに使える。費用はO(1)にもO(n)にもなり、内側を一意に保つ書き方が要る
（[更新の費用](../api/pool.md#更新の費用)）。内側の値版のcontainerは、containerを核と周辺の返すpoolを引き回す形で一度書けば、
constructorを替えるだけで得られる。[試作](../prototypes.md#constructorについてgenericなcontainer)では、同じsourceのbinary heapが
IxPool上ではidentityとして、ImPool上では値として動いた。ただしこれは、IxPoolの更新も同じidentityを返すfamilyの形を採る場合に
限られる（[Pool primitive](../api/pool.md#impool)）。

coordinateの入れ子は、木やgraphのように外部や要素からidentityを参照する構造に合う。identityの入れ子は、内側を複数の場所から
共有して変更する場合だけに要る。値の入れ子とcoordinateの入れ子で足りない用途が見つかるまで、判定の分割は採らずにおける。
