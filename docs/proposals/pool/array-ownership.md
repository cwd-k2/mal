# IxPoolによるmutable arrayとimmutable array

Status: Exploratory example

この文書は[primitive一覧](primitives.md)の`IxPool<State, T>`を使い、共有mutable identityと
copy-on-write immutable valueの参照管理を比較する。構文は未採択の擬似codeである。indexは公開precondition`index < length`に従い、
[Buffer](buffer-implementation.md#表現とinvariant)と同じinvariantでIxPool preconditionへ移るため、範囲検査を書かない。

どちらもStateをlogical lengthとし、`[0, length)`だけがLiveであるdense slot invariantを持つ。違いは同じIxPool identityへの更新を
公開するか、更新前に独立したIxPool responsibilityを得るかにある。

この例だけがcore外のtrusted operation `writableSuccessor`を仮定する。resultはcall開始時のState、logical capacity、全slotと
同じ値を持ち、その後resultを更新してもcall前から残る強いIxPool aliasの値を変えない。inputからresultへのowner-successor effectを持つが、
unique token、reference count、raw mutable carrierをsourceへ返さない。

```mal
writableSuccessor<State, T> :: IxPool<State, T> -> IxPool<State, T>;

_reserveArraySlot<T> :: (IxPool<USize, T>, USize) -> Unit :=
    (pool, required) -> {
        current := capacity<USize, T>(pool);
        if (required <= current)
        then ()
        else {
            doubled := if (current == 0usize) then 1usize else current * 2usize;
            next := if (doubled < required) then required else doubled;
            reserve<USize, T>(pool, next);
        };
    };
```

`_reserveArraySlot`のoverflow処理は省略している。IxPoolはauto-growせず、mutable版とimmutable版が同じgeometric growth policyを選ぶ。

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

## Immutable array with copy-on-write extension

immutable arrayは破壊的な公開operationを持たず、更新後の値を返す。

```mal
opaque Array<T> :: IxPool<USize, T>;

makeArray<T> :: USize -> Array<T> := (capacity) ->
    makeIxPool<USize, T>(0usize, capacity);

arrayLength<T> :: Array<T> -> USize := (array) ->
    state<USize, T>(array);

arrayGet<T> :: (Array<T>, USize) -> T := (array, index) ->
    getAt<USize, T>(array, index);

arraySet<T> :: (Array<T>, USize, T) -> Array<T> :=
    (array, index, value) -> {
        writable := writableSuccessor<USize, T>(array);
        putAt<USize, T>(writable, index, value);
        writable;
    };

arrayAppend<T> :: (Array<T>, T) -> Array<T> := (array, value) -> {
    length := state<USize, T>(array);
    writable := writableSuccessor<USize, T>(array);
    _reserveArraySlot<T>(writable, length + 1usize);
    initAt<USize, T>(writable, length, value);
    setState<USize, T>(writable, length + 1usize);
    writable;
};
```

`writableSuccessor`はinput IxPool responsibilityを受け取り、call前から残る強いaliasとは独立して更新できるIxPoolを返す。参照数や
一意性をsourceへ返さず、次の二つを同じobservable semanticsとして選べる。

```text
inputが唯一:       Array A ── IxPool P ── update in place ── Array B

inputにaliasあり:  Array A ── IxPool P  = [A, B, C]
                                  share live elements
                   Array B ── IxPool P' = [A, X, C]
```

callerが旧Arrayを後でも使う場合、ownership planはcallに渡すresponsibilityをShareする。そのためIxPoolにはaliasが残り、successorは
新しいIxPoolを作る。[D083](../../history/decisions/active/D083.md)のowned native entryへwritable-successorのowner effectを伝播できるcallで
旧Arrayがlast useなら、inputを`Consume`でき、他の強いaliasがなければ同じIxPoolを再利用できる。現在の保持解析はreturn、capture、
保持calleeへの転送だけを追うため、このprimitive-derived relationはIxPool導入時の追加事項である。borrowedまたはpinnedなcall経路では
calleeがowned responsibilityをShareしてcopyへfallbackしてよく、correctnessはcall conventionや再利用へ依存しない。

共有時のsuccessorはStateと各Live elementをShareする。managed `T`のpayloadをdeep copyせず、flatなslot carrierと占有metadataだけを
複製するが、処理量はO(length)である。chunk単位のCOWやpersistent treeはこのcopy量を減らせる一方、複数storageのownershipと
使われなくなったnodeの回収を追加で定める必要がある。

このprofileは`Id<T>`を発行しない。理由と合成の選択肢は[identity](identity.md#idとの合成)で扱う。

現在のruntimeにも同じ構造がある。immutableな`Symbol`のappendはbyte ownerのreference countが1ならallocationを再利用し、共有中なら
新しいownerへcopyする。現在の`Buffer`は逆に一つのbuffer objectをaliasが共有し、managed element用のretain/release callbackを持つ。
Pool案はこの二つの既存mechanismを、mutable identityとoptionalなwritable successorとして分離して一般化する。

外部のCOWとpersistent vectorとの対応、およびweak handleとの相互作用は
[IxPool storageの関連事例](../../research/pool-storage-prior-art.md)にまとめる。

## 必要な境界

- IxPool handleのcopyと終了を、それぞれEngram leafのShareとDropへlowerする。
- `writableSuccessor`はowned inputをConsumeでき、残る強いaliasがなければstorageを再利用できる。
- 共有時のsuccessorはState、全Live slot、Vacant metadataを保存する。allocation failureはtrapであり、inputの保存を要求しない。
- reference countやcompilerが作る一時responsibilityをsourceから観測させない。
- `Id<T>`を同じprofileから発行せず、copyか再利用かを`Id<T>`の有効性から観測させない。
- immutableなのはArrayの構造であり、elementが運ぶExtern referentまでimmutableにはしない。

## Storableなimmutable array

`Array<T>`の型形成条件はhidden representationの`IxPool<USize, T>`から導かれるため、`Array<T>`は値としてimmutableでも
`Storable`にならない。更新がsuccessorを返す`ImPool`をrepresentationにすれば`Storable`にでき、その条件は
[identity](identity.md#impool)で扱う。
