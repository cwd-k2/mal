# identityと型判定

Status: Exploratory support document

この文書は、共通の[Pool state](semantics.md#pool-state)に対するsource carrierの観測則と、Poolに関係する
`Storable`、`Stable`、`Representable`、`HostMappable`の境界を管理する。Pool全体のauthorityは
[根本モデル](foundations.md#authority)、responsibilityの遷移は
[runtime contract](../runtime/contract.md#responsibility)、ImPoolのAPIは[Pool primitive](../api/pool.md#impool)、copy-on-writeの動作例は
[Vector](../containers/sequences.md#vector)を正とする。現行の`Storable` judgmentは
[AddressとBuffer](../../../spec/memory.md#storable)に定める。

## 四つの判定が答える問い

ここで分類するのはsource authorityと境界の可否であり、compilerが移すresponsibilityやruntime allocationの物理的な一意性ではない。
四つの判定は同じ強さを段階的に表すtype classではなく、異なる境界の問いに答える。

| 判定 | 問い | authorityまたはmechanism |
|---|---|---|
| `Storable(T)` | Mal-managedなtyped placeが`T`を保持し、lifecycleを完結できるか | Engram storage mechanism |
| `Stable(T)`（案） | `T`からMal-ownedなshared mutable identityを観測できないか | source value authority |
| `Representable(T)` | `T`にcanonical memory representationがあり、host storageと値をcopyできるか | admissionとobservation |
| `HostMappable(T)` | `T`をpublic extern signatureに出すC carrierとbridge contractがあるか | extern boundary |

`HostMappable`は残る三つと直交する。external opaque typeは`HostMappable`だが、Malがlifecycleを支配しないので`Storable`ではなく、
canonical memory copyの対象でもない。反対に`Symbol`はMal-managedな値だが`Representable`でも`HostMappable`でもない。
`Representable`はruntime representationが存在するという意味ではなく、固定されたcanonical memory representationとの変換が
存在するという狭い判定である。

`Address`は四判定を混同しやすい。copyableなcapability carrierとしてstorage、canonical memory、extern boundaryへ置けるが、
referentのlifetimeと変更のauthorityはExternに残る。`Stable(Address)`が意味するのはAddressからMal-owned mutable identityを
観測しないことであり、外部storageの内容が不変であることではない。

## source carrierとidentity

IxPool、ImPool、Address、scalarはすべてsource valueである。分類するのはvalueかどうかではなく、同じcarrierを別bindingへ渡した後にどの変更を
観測できるかである。

| source carrier | 到達先 | 別bindingへ渡した後の観測 |
|---|---|---|
| IxPool handle value | Mal-owned shared identity | どのhandleからの更新も他のhandleから観測する |
| ImPool snapshot value | Pool state snapshot | 更新はsuccessorを返し、以前のsnapshotは変わらない |
| `Address` capability value | Extern-owned referent | referentの観測はextern contractに従い、carrierはlifetimeを延長しない |
| plain data | carrier自身 | operationが返す新しいvalueだけが異なる |

IxPool handleはreferentのlifetimeを支える。ImPool snapshotもmanaged carrierを持ち得るが、shared mutable identityをsourceへ公開しない。
runtime allocation identityの有無はこの分類に関係しない。

現在の`Storable`はMal-managed placeへ保持できることに加え、Mal-ownedなshared mutable identityへ到達しないことも要求する。
これは[D075](../../../history/decisions/active/D075.md)がBufferの要素を値に限った選択である。安全性だけから必要な制約ではなく、
Buffer elementを後から変化しない値に限るsource semanticsでもある。

owner cycleは現行の除外理由ではない。malは表現に寄与する再帰型を持たず、functionも`Storable`でないため、storageの要素から出る
owner edgeは型構造に沿って真に小さくなる。将来plugin leafを加える場合は、この前提をplugin contractで別途保つ必要がある。

## Poolの最小案

Poolだけを導入する段階では`Stable`を新しいcompiler judgmentにしない。IxPoolとImPoolのMetaとelementは、どちらも現行のclosed
`Storable`を要求する。

```text
IxPool<M, V> is well-formed if Storable(M) and Storable(V)
ImPool<M, V> is well-formed if Storable(M) and Storable(V)

Storable(ImPool<M, V>) if Storable(M) and Storable(V)
not Storable(IxPool<M, V>)
```

したがって`opaque Buffer<T> :: IxPool<USize, T>`は`Storable`にならず、`opaque Vector<T> :: ImPool<USize, T>`は`Storable(T)`のもとで
`Storable`になる。IxPoolは`Representable`でも`HostMappable`でもなく、ImPoolもruntime representationをそのままhost境界へ出さない。
opaque型の判定とlifecycleはhidden representationから再帰的に導き、wrapperが制約を迂回できない。

この案ではsnapshotの入れ子はできるが、shared identityへのhandleの入れ子はできない。Pool state、carrierの観測則、responsibility
loweringの検証にはそれで足り、既存のgeneric requirement、specialization、diagnostic、plugin contractを増やさない。

## `Storable`と`Stable`の分割案

identityをstorageへ入れる具体的な用途が現れた場合だけ、現在の`Storable`が兼ねる二つの条件を分ける。

```text
Storable(T) = Mal-managed placeがTのShare、Drop、relocationを実装できる
Stable(T)   = Storable(T) and TからMal-owned shared mutable identityを観測できない
```

productとsumは要素から、file-local opaque型はhidden representationから導く。分割後の関係は次になる。

```text
Representable(T) implies Stable(T)
Stable(T) implies Storable(T)
HostMappable(T) is independent
```

| 型 | Storable（現行） | Storable（分割後） | Stable | Representable | HostMappable |
|---|---|---|---|---|---|
| `Unit`、numeric scalar、`ByteSize`、`USize`、`Address` | ○ | ○ | ○ | ○ | ○ |
| `Symbol` | ○ | ○ | ○ | × | × |
| `ImPool<M, V>` | ― | ○ | `M`と`V`がStableなら○ | × | × |
| `Buffer<A>` | × | `A`がStorableなら○ | × | × | × |
| `IxPool<M, V>` | ― | `M`と`V`がStorableなら○ | × | × | × |
| external opaque | × | × | × | × | ○ |
| function | × | × | × | × | × |

分割後は、IxPoolが`Storable(Meta)`と`Storable(V)`を、ImPoolが`Stable(Meta)`と`Stable(V)`を要求する。`freeze`もMetaとelementの
`Stable`を要求する。これによりIxPoolやBufferはmutable identityを保持できる一方、ImPoolのsnapshotからMal-owned mutationを
観測する経路は作らない。Mapのkeyには別途equalityとhashのcontractが要り、`Stable`だけでkey invariant全体を表したことにはしない。

`fill`で同じ内側のBufferを複数のplaceへ置けば、それらは意図どおりaliasになり、`copy`もshallowになる。これは`Storable`の
lifecycle contractには反しない。snapshot semanticsが必要な型とoperationだけが`Stable`を要求する。

## minimality監査

現行の三判定は統合できない。`Symbol`は`Storable`だが`Representable`でなく、external opaque typeは`HostMappable`だが
`Storable`でも`Representable`でもない。file-local opaque representationはcanonical memory helperの対象になり得ても、opaque identityを
public C ABIへ出さないため`HostMappable`でない。したがって`Representable`と`HostMappable`の両方を要求する境界も、片方から他方を
導けない。三判定はそれぞれstorage lifecycle、canonical memory copy、public extern ABIという独立contractを一つずつ所有している。

`Stable`を分けると、`Buffer<Buffer<T>>`、IxPoolを要素にするIxPool、mutable identityへのhandleをpayloadに持つMapを書ける。一方で、独立contractは
確実に増える。

- type formationとgeneric requirementに`Stable`を追加する。
- operation familyのspecializationが`Storable`と`Stable`を区別する。
- opaque型、plugin leaf、diagnostic、conformance testが両判定を説明する。
- `freeze`、ImPool、Map keyなど、どのoperationがどちらを要求するかを利用者が調べる。
- 現行`Storable`の意味を広げるため、D075とBuffer element semanticsを見直す。

この費用はhandle nestingという一つの能力のために発生する。snapshotの入れ子とcoordinateの入れ子で用途を表せる間は、現行`Storable`を
再利用するPoolの最小案の方がsystem全体で小さい。分割を採択する条件は、少なくとも一つの代表的containerでhandle nestingが
relationの不自然な平坦化や重複実装を実際に減らし、その利益が上の静的・説明上のcontract増加を上回ることである。

現在の`Storable`を残して、広い判定を`Placeable`などの別名で足す案は採らない。保存できる型の一部が`Storable`でないという名前と
規則の逆転を作り、BufferとIxPoolの型形成に二種類のstorage判定が残る。分割が必要になった時点で、`Storable`をmechanismへ狭く定義し、
従来兼ねていたvalue条件だけを`Stable`へ出す方が独立contractは少ない。

名称だけを先に追加したり、`Stable`をruntime uniquenessの証明として使ったり、`HostMappable`を包含階層へ入れたりしない。

## 入れ子

現在の判定では、snapshotは入れ子にでき、shared identityへのhandleは入れ子にできない。`Vector<Vector<T>>`やpayloadが
`Vector<V>`のMapは書けるが、
`Buffer<Buffer<T>>`、valueが`Buffer<V>`のMap、IxPool上のMapを要素にするcontainerは書けない。containerを利用者が書く
基盤として、この穴の扱いを決める必要がある。入れ子を作る経路は三つある。

| 経路 | 内側 | 内側の更新 | 必要なもの |
|---|---|---|---|
| snapshotの入れ子 | ImPool上のcontainer | 外側から`takeAt`で取り出して更新し、戻す | 内側のsnapshot版container |
| coordinateの入れ子 | 一つのIxPoolにまとめた要素 | coordinateを引いてその場で更新する | coordinateの回収をcontainerが行う |
| handleの入れ子 | IxPool上のcontainer | handleを通してその場で更新する | [`Storable`と`Stable`の分割案](#storableとstableの分割案)と[D075](../../../history/decisions/active/D075.md)の見直し |

snapshotの入れ子は、判定を変えずに使える。費用はO(1)にもO(n)にもなり、外側のplaceにaliasを残さずresponsibilityを移す書き方が要る
（[更新の費用](../api/pool.md#更新の費用)）。内側のsnapshot版containerは、containerを核と周辺の返すpoolを引き回す形で一度書けば、
constructorを替えるだけで得られる。[試作](../prototypes.md#constructorについてgenericなcontainer)では、同じsourceのbinary heapが
IxPool上ではhandleを通じて共有更新され、ImPool上ではsnapshotとsuccessorとして動いた。ただしこれは、IxPoolの更新も同じhandleを
返すfamilyの形を採る場合に限られる（[Pool primitive](../api/pool.md#impool)）。

coordinateの入れ子は、木やgraphのように外部や要素からidentityを参照する構造に合う。handleの入れ子は、内側を複数の場所から
共有して変更する場合だけに要る。snapshotの入れ子とcoordinateの入れ子で足りない用途が見つかるまで、判定の分割は採らずにおける。
