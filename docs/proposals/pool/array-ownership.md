# Poolによるmutable arrayとimmutable array

Status: Exploratory example

この文書は[Pool-backed containerのsource sketch](container-examples.md)の`Pool<State, T>`を使い、共有mutable identityと
copy-on-write immutable valueの参照管理を比較する。構文は未採択の擬似codeであり、bounds failureの表現は省略する。

どちらもStateをlogical lengthとし、`[0, length)`だけがLiveであるdense slot invariantを持つ。違いは同じPool identityへの更新を
公開するか、更新前に独立したPool responsibilityを得るかにある。

この例だけがcore外のtrusted operation `poolWritableSuccessor`を仮定する。resultはcall開始時のState、logical capacity、全slotと
同じ値を持ち、その後resultを更新してもcall前から残る強いPool aliasの値を変えない。inputからresultへのowner-successor effectを持つが、
unique token、reference count、raw mutable carrierをsourceへ返さない。

```mal
poolWritableSuccessor<State, T> :: Pool<State, T> -> Pool<State, T>;

_reserveArraySlot<T> :: (Pool<USize, T>, USize) -> Unit :=
    (pool, required) -> {
        capacity := poolCapacity<USize, T>(pool);
        if (required <= capacity)
        then ()
        else {
            doubled := if (capacity == 0usize) then 1usize else capacity * 2usize;
            next := if (doubled < required) then required else doubled;
            poolReserve<USize, T>(pool, next);
        };
    };
```

`_reserveArraySlot`のoverflow処理は省略している。Poolはauto-growせず、mutable版とimmutable版が同じgeometric growth policyを選ぶ。

## Mutable array

mutable arrayはopaque valueのcopy後も同じPool identityを共有する。

```mal
opaque MutableArray<T> :: Pool<USize, T>;

_packMutableArray<T> :: Pool<USize, T> -> MutableArray<T>;
_openMutableArray<T> :: MutableArray<T> -> Pool<USize, T>;

makeMutableArray<T> :: USize -> MutableArray<T> := (capacity) ->
    _packMutableArray<T>(makePool<USize, T>(0usize, capacity));

mutableLength<T> :: MutableArray<T> -> USize := (array) ->
    poolState<USize, T>(_openMutableArray<T>(array));

mutableGet<T> :: (MutableArray<T>, USize) -> T := (array, index) ->
    poolGetAt<USize, T>(_openMutableArray<T>(array), index);

mutableSet<T> :: (MutableArray<T>, USize, T) -> Unit :=
    (array, index, value) ->
        poolPutAt<USize, T>(_openMutableArray<T>(array), index, value);

mutableAppend<T> :: (MutableArray<T>, T) -> USize := (array, value) -> {
    pool := _openMutableArray<T>(array);
    length := poolState<USize, T>(pool);
    _reserveArraySlot<T>(pool, length + 1usize);
    poolInitAt<USize, T>(pool, length, value);
    poolSetState<USize, T>(pool, length + 1usize);
    length;
};
```

`a1`をcopyして`a2`を作ると、両方が同じPool responsibilityをShareする。

```text
a1 ─┐
    ├── Pool P: State = 3、slots = [A, B, C]
a2 ─┘

mutableSet(a2, 1, X)

a1 ─┐
    ├── Pool P: State = 3、slots = [A, X, C]
a2 ─┘
```

`mutableSet`はPoolをBorrowし、new valueをslotへShareまたはConsumeする。array handleのretain/releaseとelementのretain/releaseは
Pool lifecycleが行い、array implementationは参照数を観測しない。

## Immutable array with copy-on-write extension

immutable arrayは破壊的な公開operationを持たず、更新後の値を返す。

```mal
opaque Array<T> :: Pool<USize, T>;

_packArray<T> :: Pool<USize, T> -> Array<T>;
_openArray<T> :: Array<T> -> Pool<USize, T>;

makeArray<T> :: USize -> Array<T> := (capacity) ->
    _packArray<T>(makePool<USize, T>(0usize, capacity));

arrayLength<T> :: Array<T> -> USize := (array) ->
    poolState<USize, T>(_openArray<T>(array));

arrayGet<T> :: (Array<T>, USize) -> T := (array, index) ->
    poolGetAt<USize, T>(_openArray<T>(array), index);

arraySet<T> :: (Array<T>, USize, T) -> Array<T> :=
    (array, index, value) -> {
        source := _openArray<T>(array);
        writable := poolWritableSuccessor<USize, T>(source);
        poolPutAt<USize, T>(writable, index, value);
        _packArray<T>(writable);
    };

arrayAppend<T> :: (Array<T>, T) -> Array<T> := (array, value) -> {
    source := _openArray<T>(array);
    length := poolState<USize, T>(source);
    writable := poolWritableSuccessor<USize, T>(source);
    _reserveArraySlot<T>(writable, length + 1usize);
    poolInitAt<USize, T>(writable, length, value);
    poolSetState<USize, T>(writable, length + 1usize);
    _packArray<T>(writable);
};
```

`poolWritableSuccessor`はinput Pool responsibilityを受け取り、call前から残る強いaliasとは独立して更新できるPoolを返す。参照数や
一意性をsourceへ返さず、次の二つを同じobservable semanticsとして選べる。

```text
inputが唯一:       Array A ── Pool P ── update in place ── Array B

inputにaliasあり:  Array A ── Pool P  = [A, B, C]
                                  share live elements
                   Array B ── Pool P' = [A, X, C]
```

callerが旧Arrayを後でも使う場合、ownership planはcallに渡すresponsibilityをShareする。そのためPoolにはaliasが残り、successorは
新しいPoolを作る。[D083](../../history/decisions/active/D083.md)のowned native entryへwritable-successorのowner effectを伝播できるcallで
旧Arrayがlast useなら、inputを`Consume`でき、他の強いaliasがなければ同じPoolを再利用できる。現在の保持解析はreturn、capture、
保持calleeへの転送だけを追うため、このprimitive-derived relationはPool導入時の追加事項である。borrowedまたはpinnedなcall経路では
calleeがowned responsibilityをShareしてcopyへfallbackしてよく、correctnessはcall conventionや再利用へ依存しない。

共有時のsuccessorはStateと各Live elementをShareする。managed `T`のpayloadをdeep copyせず、flatなslot carrierと占有metadataだけを
複製するが、処理量はO(length)である。chunk単位のCOWやpersistent treeはこのcopy量を減らせる一方、複数storageのownershipと
使われなくなったnodeの回収を追加で定める必要がある。

このprofileはPool identityまたはslotを指すstable handleを発行しない。identity-bearing handleが残る状態でstorageを再利用すると、
copyした場合との違いがhandle validityとして観測され得るためである。handle profileとの合成は、常に新identityを作るか、handleを
successorから切り離すか、handle存在を再利用条件へ含めるかを別途決める。

現在のruntimeにも同じ構造がある。immutableな`Symbol`のappendはbyte ownerのreference countが1ならallocationを再利用し、共有中なら
新しいownerへcopyする。現在の`Buffer`は逆に一つのbuffer objectをaliasが共有し、managed element用のretain/release callbackを持つ。
Pool案はこの二つの既存mechanismを、mutable identityとoptionalなwritable successorとして分離して一般化する。

外部のCOWとpersistent vectorとの対応、およびweak handleとの相互作用は
[Pool storageの関連事例](../../research/pool-storage-prior-art.md)にまとめる。

## 必要な境界

- Pool handleのcopyと終了を、それぞれEngram leafのShareとDropへlowerする。
- `poolWritableSuccessor`はowned inputをConsumeでき、残る強いaliasがなければstorageを再利用できる。
- 共有時のsuccessorはState、全Live slot、Vacant metadataを保存し、途中のallocation failureでinputを失わない。
- reference countやcompilerが作る一時responsibilityをsourceから観測させない。
- stable handleを同じprofileから発行せず、copyか再利用かをhandle validityから観測させない。
- immutableなのはArrayの構造であり、elementが運ぶExtern referentまでimmutableにはしない。
