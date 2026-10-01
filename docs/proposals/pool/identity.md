# IxPool identityとStorable

Status: Exploratory support document

この文書は、Pool案のidentity軸、すなわち値の変更を誰が観測するかと、それによって決まる`Storable`の可否を管理する。
所有権の遷移は[所有権primitive](ownership-primitives.md)、`Id<T>`の照合とArenaは[IdPool](idpool.md)、
copy-on-writeの動作例は[array ownership](array-ownership.md)を正とする。現行の`Storable` judgmentは
[AddressとBuffer](../../spec/memory.md#storable)に定める。

## 所有権とidentity

所有権は誰がresponsibilityを持つかを、identityはaliasが同じ変更を観測するかを表す。二つは独立しており、Pool案の型は
次のように並ぶ。

| | identityを共有する | identityを持たない |
|---|---|---|
| 所有する | `IxPool<State, T>` | `ImPool<State, T>` |
| 所有しない | `Id<T>` | 通常のdata |

`Storable`にできないのは、所有とidentity共有を両方持つ型である。identityを共有する値をstorageへ保存すると、その後のmutationを
storage内のaliasから観測できるためである。これは[D075](../../history/decisions/active/D075.md)がBufferの要素を値に限った理由と
同じであり、安全性ではなく言語の意味の選択である。

owner cycleはこの除外の理由にならない。malは表現に寄与する再帰型を持たず、functionは`Storable`でないため、storageの要素から出る
owner edgeは常に真に小さい型の値を指す。`IxPool<S, IxPool<S, T>>`のように自身を要素にする型は書けず、owner cycleは型の上で
生じない。

`ImPool`はidentityを捨て、`Id<T>`は所有を捨てることで、それぞれ`Storable`になる。`IxPool<State, T>`は`Storable`でも
`Representable`でも`HostMappable`でもなく、`IxPool<State, IxPool<...>>`のような入れ子も認めない。

## 型形成条件

IxPoolと`ImPool`の形成は、現在のclosed judgmentである`Storable(State)`と`Storable(T)`を要求する。opaque型の`Storable`、
`Representable`、lifecycleはcompilerがhidden representationから導き、opaque型がこれらのpropertyを新たに宣言して
representationの制約を迂回することはできない。したがって`opaque Array<T> :: IxPool<USize, T>`は`Storable`にならず、
`opaque Array<T> :: ImPool<USize, T>`は`Storable(T)`のもとで`Storable`になる。

将来plugin leafを`Storable`へ追加する場合も、layoutとdropだけから導かない。storage内のShareが安全であること、aliasが後の
mal-owned mutationを観測しないこと、container edgeからowner cycleを作らないことを登録時に示す。shared mutableなIxPool、Buffer、
function、external opaque valueを除外する現在の制約を、opaque wrapperやplugin registrationで迂回させない。

## 判定の分割（案）

現在の`Storable`は、storageに保持できることと、保持した値の意味が後から変わらないことの二つを一つの判定で表している。
owner cycleが型の上で生じない以上、前者だけならBufferやIxPoolも保持できる。そこで次の案を検討する。この案は
[D075](../../history/decisions/active/D075.md)の見直しを伴い、採択していない。

| 判定 | 問い | 要求される場所 |
|---|---|---|
| `Storable` | malのstorageに保持できるか | Buffer、IxPool、IdPool、ImPoolの要素、IxPoolのState |
| `Stable` | 保持した値の意味が後から変わらないか | ImPoolの要素とState、Mapのkeyのように値の意味を前提にする場所 |
| `Representable` | hostとcopyできるcanonical layoutを持つか | `from`、`into`、canonical memory helper |
| `HostMappable` | extern境界をそのまま渡れるか | externのparameterとresult |

| 型 | Storable（現在） | Storable（案） | Stable | Representable | HostMappable |
|---|---|---|---|---|---|
| `Unit`、numeric scalar、`ByteSize`、`USize`、`Address` | ○ | ○ | ○ | ○ | ○ |
| `Symbol` | ○ | ○ | ○ | × | × |
| `Id<T>` | ― | ○ | ○ | × | × |
| `ImPool<S, T>` | ― | `S`と`T`がStorableなら○ | `S`と`T`がStableなら○ | × | × |
| `Buffer<A>` | × | `A`がStorableなら○ | × | × | × |
| `IxPool<S, T>`、`IdPool<T>` | ― | ○ | × | × | × |
| external opaque | × | × | × | × | ○ |
| function | × | × | × | × | × |

productとsumは要素から、file-local opaque型はhidden representationから導く。案では`Representable`⊂`Stable`⊂`Storable`となり、
`Stable`は現在の`Storable`に`Id<T>`と`ImPool`を加えたものになる。`Address`と`Id<T>`はどちらも背後のものを指すが値そのものは
変わらないため、同じく`Stable`である。external opaqueは値が変わるからではなく、寿命がEngramの回収と結びつかないため
`Storable`の外に残る。

案を採ると`Buffer<Buffer<T>>`や`HashMap<K, Buffer<V>>`を書け、IxPoolを要素にするためのArenaも不要になる。代わりに、
`fill`で同じ内側のBufferをすべての位置へ置くとそれらがaliasになり、`copy`は浅くなる。値の意味が要る場所は`Stable`を要求して、
このaliasを型で排除する。

## ImPool

`ImPool<State, T>`は、更新するたびにsuccessorを返す、identityを持たないPoolであり、IxPoolと対になるprimitiveの候補である。
位置で引く点はIxPoolと同じであり、APIは[primitive一覧](primitives.md#impool-primitive候補)に置く。

各更新operationはinputを`Store`で受け取り、内部で[writable successor](ownership-primitives.md#拡張operation)を作ってから変更して返す。
inputが唯一のresponsibilityならstorageを再利用し、共有中ならcopyする。

- 更新前のvalueをsourceから変更する手段がないため、storage内でShareしても後のmutationを観測しない。
- 同じvalueを別のslotへ保存したoperandはShareされてuniquenessが成り立たず、以後の更新はcopyへfallbackする。

更新を`Unit`を返すIxPool操作とsuccessor取得の二つへ分けると、Shareしたsuccessorを更新しないことが未検査preconditionになり、
違反は別のvalueの変更として現れる。更新operation自体がsuccessorを返す形なら、`Storable`の健全性をcontainer実装の
invariantへ依存させない。uniqueness検査は更新ごとに一回の比較であり、last useを`Consume`できるcallではstorageを再利用する。

## freezeとthaw

IxPoolとImPoolの間は、次の二つの変換で行き来する。

- `freeze(pool)`は、呼び出し時点のStateとslotを持つImPoolを返す。元のIxPoolはidentityを保ったまま使い続けられ、以後の変更は
  返した値から観測されない。
- `thaw(value)`は、同じStateとslotを持つ新しいidentityのIxPoolを返す。返したIxPoolへの変更は、元の値からも、同じ値から
  thawした別のIxPoolからも観測されない。

可変なIxPoolで効率よく組み立ててから値として公開すること、値から編集用の可変なcopyを作ることに使う。`Buffer<UInt8>`と
`Symbol`の`*`は、この対をbyte列へ特化した変換に当たる。

意味は要素を一つずつcopyするloopで定まる。primitiveにするのはstorageを共有するためであり、`freeze`はIxPoolのstorageを
ImPoolと共有して、IxPool側への後の書き込みでcopyする。`thaw`は入力が唯一のresponsibilityならstorageを移し、共有中なら
書き込みでcopyする。共有を許すと、IxPoolへの書き込みのたびにstorageが共有中かの確認が一回入る。現在のBufferも`Symbol`と
byte ownerを共有するため同じ確認を持つが、全要素型のIxPoolへ広げるか、`freeze`を常にcopyにして確認を省くかは未決定である。

## Idとの合成

`Id<T>`とwritable successorを同じ型へ合成しない。storageを再利用したかcopyしたかが`Id<T>`の有効性として観測され、
reference countをsourceへ漏らすためである。`ImPool`は`Id<T>`を発行せず、`Id<T>`はidentityを共有するIdPoolとArenaだけが
発行する。合成が必要になった場合は、常に新identityを作るか、`Id<T>`をsuccessorから切り離すか、`Id<T>`の存在を再利用条件へ
含めるかを別途決める。

## 入れ子構造の選び方

`Array<Array<T>>`に相当する構造は、identity軸のどちらを選ぶかで作り方が分かれる。

- `ImPool`を入れ子にする。値として振る舞い、到達できなくなった内側の配列は自動で回収される。cycleは作れない。
- [Arena](idpool.md#ixpoolを要素にする場合)へ内側のIxPoolを置き、外側のslotへ`Id<IxPool<S, T>>`を保存する。identityを共有し、cycleを作れるが、
  回収は`arenaRemove`かArenaの破棄による。

tree、graph、IdPoolのように外部やelementからidentityを参照する構造は後者、immutable arrayやsnapshotのような値は前者を使う。
