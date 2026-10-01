# IxPoolとImPoolによるmutable arrayとimmutable array

Status: Exploratory example

この文書は、[primitive一覧](primitives.md)の対になる二つのprimitiveで同じ配列を書き、共有mutable identityと
copy-on-write immutable valueの参照管理を比較する。mutable arrayは`IxPool<State, T>`、immutable arrayは`ImPool<State, T>`を
representationにする。構文は未採択の擬似codeである。indexは公開precondition`index < length`に従い、
[Buffer](buffer-implementation.md#表現とinvariant)と同じinvariantでpool preconditionへ移るため、範囲検査を書かない。

どちらもStateをlogical lengthとし、`[0, length)`だけがLiveであるdense slot invariantを持つ。違いは、同じidentityへの更新を
公開するか、更新のたびに独立した値を返すかにある。

```mal
_reserveArraySlot<T> :: (IxPool<USize, T>, USize) -> Unit :=
    (pool, required) -> {
        current := capacity(pool);
        when (required > current) {
            reserve(pool, if (current == 0usize) then 1usize else current * 2usize);
        };
    };
```

`_reserveArraySlot`のoverflow処理は省略している。どちらのarrayも同じgeometric growth policyを選ぶ。

## Mutable array

mutable arrayはopaque valueのcopy後も同じIxPool identityを共有する。

```mal
opaque MutableArray<T> :: IxPool<USize, T>;

makeMutableArray<T> :: USize -> MutableArray<T> := (capacity) ->
    makeIxPool<USize, T>(0usize, capacity);

mutableLength<T> :: MutableArray<T> -> USize := (array) ->
    state<USize, T>(array);

mutableGet<T> :: (MutableArray<T>, USize) -> T := (array, index) ->
    getAt<USize, T>(array, index);

mutableSet<T> :: (MutableArray<T>, USize, T) -> Unit :=
    (array, index, value) ->
        putAt<USize, T>(array, index, value);

mutableAppend<T> :: (MutableArray<T>, T) -> USize := (array, value) -> {
    length := state<USize, T>(array);
    _reserveArraySlot<T>(array, length + 1usize);
    initAt<USize, T>(array, length, value);
    setState<USize, T>(array, length + 1usize);
    length;
};
```

`a1`をcopyして`a2`を作ると、両方が同じIxPool responsibilityをShareする。

```text
a1 ─┐
    ├── IxPool P: State = 3、slots = [A, B, C]
a2 ─┘

mutableSet(a2, 1, X)

a1 ─┐
    ├── IxPool P: State = 3、slots = [A, X, C]
a2 ─┘
```

`mutableSet`はIxPoolをBorrowし、new valueをslotへShareまたはConsumeする。array handleのretain/releaseとelementのretain/releaseは
IxPool lifecycleが行い、array implementationは参照数を観測しない。

## Immutable array

immutable arrayは破壊的な公開operationを持たず、更新後の値を返す。ImPoolの更新primitiveがそのままsuccessorを返す。

```mal
opaque Array<T> :: ImPool<USize, T>;

makeArray<T> :: USize -> Array<T> := (capacity) -> makeImPool(0usize, capacity);

arrayLength<T> :: Array<T> -> USize := (array) -> imState(array);

arrayGet<T> :: (Array<T>, USize) -> T := (array, index) -> imGetAt(array, index);

arraySet<T> :: (Array<T>, USize, T) -> Array<T> := (array, index, value) ->
    imPutAt(array, index, value);

arrayAppend<T> :: (Array<T>, T) -> Array<T> := (array, value) -> {
    length := imState(array);
    current := imCapacity(array);
    grown := if (length < current)
        then array
        else imReserve(array, if (current == 0usize) then 1usize else current * 2usize);
    imSetState(imInitAt(grown, length, value), length + 1usize);
};
```

ImPoolの各更新は入力のresponsibilityを`Store`で受け取り、内部でwritable successorを作ってから変更する。参照数や一意性を
sourceへ返さず、次の二つを同じobservable semanticsとして選ぶ。

```text
inputが唯一:       Array A ── storage P ── update in place ── Array B

inputにaliasあり:  Array A ── storage P  = [A, B, C]
                                  share live elements
                   Array B ── storage P' = [A, X, C]
```

callerが旧Arrayを後でも使う場合、ownership planはcallに渡すresponsibilityをShareする。そのためstorageにはaliasが残り、更新は
新しいstorageを作る。[D083](../../history/decisions/active/D083.md)のowned native entryへ`Store`を伝播できるcallで旧Arrayが
last useなら、inputを`Consume`でき、他の強いaliasがなければ同じstorageを再利用できる。`arrayAppend`のように更新を続けると、
最初の更新が一意なsuccessorを作るため、以後の更新はその場で行われる。borrowedまたはpinnedなcall経路ではcalleeが
owned responsibilityをShareしてcopyへfallbackしてよく、correctnessはcall conventionや再利用へ依存しない。

共有時の更新はStateと各Live elementをShareする。managed `T`のpayloadをdeep copyせず、flatなslot carrierと占有metadataだけを
複製するが、処理量はO(length)である。chunk単位のCOWやpersistent treeはこのcopy量を減らせる一方、複数storageのownershipと
使われなくなったnodeの回収を追加で定める必要がある。

`Array<T>`は`ImPool<USize, T>`から
型形成条件を導き、`T`が`Storable`なら`Storable`になるため、`Array<Array<T>>`やMapのvalueにできる。

現在のruntimeにも同じ構造がある。immutableな`Symbol`の連結はbyte ownerのreference countが1ならallocationを再利用し、共有中なら
新しいownerへcopyする。現在の`Buffer`は逆に一つのbuffer objectをaliasが共有し、managed element用のretain/release callbackを持つ。
Pool案はこの二つの既存mechanismを、IxPoolとImPoolとして一般化する。

外部のCOWとpersistent vectorとの対応、およびweak handleとの相互作用は
[Pool storageの関連事例](../../research/pool-storage-prior-art.md)にまとめる。

## 必要な境界

- IxPoolとImPoolのhandleのcopyと終了を、それぞれEngram leafのShareとDropへlowerする。
- ImPoolの更新はowned inputをConsumeでき、残る強いaliasがなければstorageを再利用できる。
- 共有時の更新はState、全Live slot、Vacant metadataを保存する。allocation failureはtrapであり、inputの保存を要求しない。
- reference countやcompilerが作る一時responsibilityをsourceから観測させない。
- immutableなのはArrayの構造であり、elementが運ぶExtern referentまでimmutableにはしない。
