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
| 所有する | `IxPool<State, T>` | `ValuePool<State, T>` |
| 所有しない | `Id<T>` | 通常のdata |

`Storable`にできないのは、所有とidentity共有を両方持つ型である。identityを共有する値をstorageへ保存すると、その後のmutationを
storage内のaliasから観測できるためである。これは[D075](../../history/decisions/active/D075.md)がBufferの要素を値に限った理由と
同じであり、安全性ではなく言語の意味の選択である。

owner cycleはこの除外の理由にならない。malは表現に寄与する再帰型を持たず、functionは`Storable`でないため、storageの要素から出る
owner edgeは常に真に小さい型の値を指す。`IxPool<S, IxPool<S, T>>`のように自身を要素にする型は書けず、owner cycleは型の上で
生じない。

`ValuePool`はidentityを捨て、`Id<T>`は所有を捨てることで、それぞれ`Storable`になる。`IxPool<State, T>`は`Storable`でも
`Representable`でも`HostMappable`でもなく、`IxPool<State, IxPool<...>>`のような入れ子も認めない。

## 型形成条件

IxPoolと`ValuePool`の形成は、現在のclosed judgmentである`Storable(State)`と`Storable(T)`を要求する。opaque型の`Storable`、
`Representable`、lifecycleはcompilerがhidden representationから導き、opaque型がこれらのpropertyを新たに宣言して
representationの制約を迂回することはできない。したがって`opaque Array<T> :: IxPool<USize, T>`は`Storable`にならず、
`opaque Array<T> :: ValuePool<USize, T>`は`Storable(T)`のもとで`Storable`になる。

将来plugin leafを`Storable`へ追加する場合も、layoutとdropだけから導かない。storage内のShareが安全であること、aliasが後の
mal-owned mutationを観測しないこと、container edgeからowner cycleを作らないことを登録時に示す。shared mutableなIxPool、Buffer、
function、external opaque valueを除外する現在の制約を、opaque wrapperやplugin registrationで迂回させない。

## ValuePool

`ValuePool<State, T>`は、更新するたびにsuccessorを返すIxPoolである。

```mal
ValuePool<State, T>

valuePoolPutAt<State, T> :: (ValuePool<State, T>, USize, T) -> ValuePool<State, T>;
valuePoolInitAt<State, T> :: (ValuePool<State, T>, USize, T) -> ValuePool<State, T>;
valuePoolReserve<State, T> :: (ValuePool<State, T>, USize) -> ValuePool<State, T>;
```

各更新operationはinputを`Store`で受け取り、内部で[writable successor](ownership-primitives.md#拡張operation)を作ってから変更して返す。
inputが唯一のresponsibilityならstorageを再利用し、共有中ならcopyする。

- 更新前のvalueをsourceから変更する手段がないため、storage内でShareしても後のmutationを観測しない。
- 同じvalueを別のslotへ保存したoperandはShareされてuniquenessが成り立たず、以後の更新はcopyへfallbackする。

更新を`Unit`を返すIxPool操作とsuccessor取得の二つへ分けると、Shareしたsuccessorを更新しないことが未検査preconditionになり、
違反は別のvalueの変更として現れる。更新operation自体がsuccessorを返す形なら、`Storable`の健全性をcontainer実装の
invariantへ依存させない。uniqueness検査は更新ごとに一回の比較であり、last useを`Consume`できるcallではstorageを再利用する。

## Idとの合成

`Id<T>`とwritable successorを同じ型へ合成しない。storageを再利用したかcopyしたかが`Id<T>`の有効性として観測され、
reference countをsourceへ漏らすためである。`ValuePool`は`Id<T>`を発行せず、`Id<T>`はidentityを共有するIdPoolとArenaだけが
発行する。合成が必要になった場合は、常に新identityを作るか、`Id<T>`をsuccessorから切り離すか、`Id<T>`の存在を再利用条件へ
含めるかを別途決める。

## 入れ子構造の選び方

`Array<Array<T>>`に相当する構造は、identity軸のどちらを選ぶかで作り方が分かれる。

- `ValuePool`を入れ子にする。値として振る舞い、到達できなくなった内側の配列は自動で回収される。cycleは作れない。
- [Arena](idpool.md#ixpoolを要素にする場合)へ内側のIxPoolを置き、外側のslotへ`Id<IxPool<S, T>>`を保存する。identityを共有し、cycleを作れるが、
  回収は`arenaRemove`かArenaの破棄による。

tree、graph、IdPoolのように外部やelementからidentityを参照する構造は後者、immutable arrayやsnapshotのような値は前者を使う。
