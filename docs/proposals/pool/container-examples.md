# Pool-backed containerのsource sketch

Status: Exploratory example

この文書は[Poolとopaque型によるcontainer基盤](README.md)の候補modelを、将来のmal sourceに近い擬似codeで
具体化する。`opaque`とfile-localなrepresentation viewは採択済みだが、`Pool<State, T>`は未採択であり、例全体は現在のparserで受理されない。
完全なsyntaxではなく、Poolとcontainer policyの責務を比較するための例である。

## 仮定するPool primitive

Poolは挿入位置、logical size、free list、growth policyを決めない。共有mutableな`State`と、coordinateで選ぶtyped slotの
Live/Vacant状態だけを持つ。

```mal
Pool<State, T>

makePool<State, T> :: (State, USize) -> Pool<State, T>;
poolState<State, T> :: Pool<State, T> -> State;
poolSetState<State, T> :: (Pool<State, T>, State) -> Unit;
poolCapacity<State, T> :: Pool<State, T> -> USize;
poolReserve<State, T> :: (Pool<State, T>, USize) -> Unit;

poolIsLive<State, T> :: (Pool<State, T>, USize) -> Bool;
poolInitAt<State, T> :: (Pool<State, T>, USize, T) -> Unit;
poolGetAt<State, T> :: (Pool<State, T>, USize) -> T;
poolPutAt<State, T> :: (Pool<State, T>, USize, T) -> Unit;
poolTakeAt<State, T> :: (Pool<State, T>, USize) -> T;
poolDropAt<State, T> :: (Pool<State, T>, USize) -> Unit;
```

Poolの形成は`Storable(State)`と`Storable(T)`を要求する。Poolのcopyは同じState、capacity、slotを持つidentityを共有する。
`poolReserve`だけがallocationを増やし、Poolは自動的にgrowthしない。各operationのownership effectと
[未検査precondition](lifecycle-contract.md#未検査precondition)はlifecycle contractが所有する。以下のsketchでは、各Pool callの
直前にそのpreconditionをどのinvariantが満たすかを本文で示す。

Pool自身はStateにもelementにも格納できない。keyはこの例に必要なく、[Pool key extension](pool-keys.md)で扱う。
このsketchの全operationを実装した形は[Pool上のBuffer実装](buffer-implementation.md)に示す。

mutable arrayのidentity共有と、immutable arrayのcopy-on-writeは
[Pool array ownership example](array-ownership.md)で同じdense slot invariantを使って比較する。後者が仮定する
`poolWritableSuccessor`は基本slot APIではなく、参照数をsourceへ公開しないoptionalなtrusted operationである。

## opaque型のrepresentation view

例では[opaque宣言](../../spec/types.md#file-local-opaque-type)の宣言元fileだけがhidden representationを観察できる。専用のpack/open operationは
生成せず、通常のexpression type checkingで`Buffer<T>`を`Pool<USize, T>`が期待されるoperandへ渡し、逆にPoolを
`Buffer<T>`が期待されるresultにできる。

```mal
opaque Buffer<T> :: Pool<USize, T>;
```

`Buffer<T>`は`Pool<USize, T>`と異なるcanonical typeであり、別fileはrepresentationを観察できない。representation viewはcall ABI、
allocation、追加のShareまたはDropを発生させない。compilerはhidden representationから`Buffer<T>`のlifecycleと形成条件
`Storable(T)`を導ける。

## Buffer

BufferはPoolのStateをlogical countにし、denseなlive prefixとgrowth policyを宣言元fileで維持する。

```mal
opaque Buffer<T> :: Pool<USize, T>;

makeBuffer<T> :: USize -> Buffer<T> := (initialCapacity) ->
    makePool<USize, T>(0usize, initialCapacity);

length<T> :: Buffer<T> -> USize := (buffer) ->
    poolState<USize, T>(buffer);

_nextCapacity :: (USize, USize) -> USize := (current, required) -> {
    doubled := current * 2usize;
    if (doubled < required) then required else doubled;
};

new<T> :: (Buffer<T>, T) -> USize := (buffer, value) -> {
    count := poolState<USize, T>(buffer);
    if (count == poolCapacity<USize, T>(buffer))
    then poolReserve<USize, T>(buffer, _nextCapacity(count, count + 1usize))
    else ();
    poolInitAt<USize, T>(buffer, count, value);
    poolSetState<USize, T>(buffer, count + 1usize);
    count;
};

get<T> :: (Buffer<T>, USize) -> T := (buffer, index) ->
    poolGetAt<USize, T>(buffer, index);

put<T> :: (Buffer<T>, USize, T) -> Unit := (buffer, index, value) ->
    poolPutAt<USize, T>(buffer, index, value);
```

BufferだけがStateを更新し、removeを公開しないため、live coordinateは常に`[0, length(buffer))`であり、
`length(buffer) <= poolCapacity(buffer)`である。Poolはこれらをsequence ruleとして知らない。

このinvariantによりpreconditionは次のように移る。

- `get`と`put`：利用者が公開precondition`index < #buffer`を満たせば、`index`はLive slotを指す。Bufferは検査を追加せず、
  利用者の違反は現行Bufferと同じく結果を保証しない。
- `new`：`count`はlive prefixの直後なのでVacantであり、直前の`reserve`で`count < poolCapacity`になる。これは利用者に
  preconditionを課さず、Buffer実装だけが満たす。
- `count + 1usize`のoverflowと`_nextCapacity`のoverflowはこのsketchでは省略している。現行Bufferと同じくtrapさせるなら、
  Buffer実装が比較して[primitive `trap`](../primitive-trap.md)のようなmal-level trapを呼ぶ必要がある。

Bufferのaliasは同じPool Stateを開くので、一方からの`new`は他方の`length`へ反映される。

prefix `#buffer`を維持する場合は`length`をoperator familyへ結ぶ規則が別途必要になる。`fill`とoverlapping `copy`もBuffer policyであり、
動的個数のShareをprogram非依存runtimeへ委ねるか、mal controlとして展開するかを別途測定する。

## Map

open addressingのMapは、Map固有のStateとslot表現を同じPoolへ置ける。PoolのVacantは一度も使われていないbucket、liveな`tombstone`は
probeを継続すべき削除済みbucketを表す。

```mal
_MapState :: (USize, USize); // logical size、occupiedまたはtombstoneになったbucket数
_MapSlot<K, V> :: [Unit, (UInt64, K, V)]; // tombstone、entry

opaque Map<K, V> :: Pool<_MapState, _MapSlot<K, V>>;
```

Mapの宣言元fileはcapacity、load factor、probe、rehashをmalで実装する。

```mal
makeMap<K, V> :: USize -> Map<K, V> := (initialCapacity) ->
    makePool<_MapState, _MapSlot<K, V>>(
        (0usize, 0usize),
        initialCapacity
    );

mapLength<K, V> :: Map<K, V> -> USize := (map) -> {
    (size, _) := poolState<_MapState, _MapSlot<K, V>>(map);
    size;
};
```

lookupはhashから始めたcoordinateをprobeし、`poolIsLive`がfalseならmissing、liveなtombstoneなら継続、entryならkeyを比較する。
insertはload factorを見て必要なら同じPool identityのlogical capacityを拡張してrehashする。空bucketには`poolInitAt`、tombstoneまたは同じkeyには
`poolPutAt`を使う。removeはentryを`tombstone`へreplaceし、Stateのlogical sizeだけを減らす。rehashだけがtombstoneをVacantへ戻す。

```mal
equal<T> :: (T, T) -> Bool;
hash<T> :: T -> UInt64;

mapGet<K, V> :: (Map<K, V>, K) -> [Unit, V] := (map, key) ->
    _mapProbeGet<K, V>(
        map,
        hash(key),
        key,
        equal<K>
    );
```

`mapGet<K, V>`から`hash<K>`と`equal<K>`のoperation requirementが導かれる。Poolはhash、equality、load factor、probe順序を知らない。
Mapのcopyは同じPool identityを共有するため全aliasから更新を観測し、独立snapshotは新しいPoolへlive entryだけをrehashする。

Mapの公開operationは利用者にpreconditionを課さないので、Pool preconditionはすべてMap実装が満たす。

- capacity 0では`% poolCapacity`を計算せずmissingを返す。以後のprobe coordinateは`% poolCapacity`で常に範囲内になる。
- `poolGetAt`と`poolPutAt`は、同じcoordinateで`poolIsLive`がtrueだった直後にだけ呼ぶ。
- `poolInitAt`は`poolIsLive`がfalseだったcoordinateにだけ呼ぶ。`hash`と`equal`は変更中のMapへ到達できないため、判定から
  呼び出しまでの間に状態は変わらない。

rehashでMapのhidden representationを別Poolへ交換すると既存aliasが追随しない。現APIだけでidentityを保つ場合は、元Poolの
entryを`poolTakeAt`でtemporary Poolへ移し、tombstoneを`poolDropAt`し、元Poolを`reserve`してから新しいprobe位置へ
`poolTakeAt`と`poolInitAt`で戻す。entryの移動はすべて`Consume`であり、K、Vの`Share`や`Drop`は起きない。途中状態は
[lifecycle contract](lifecycle-contract.md#primitive-transitionとcontainer-invariant)のとおり観測されないため、順序はfailureではなく
algorithmだけで決めてよい。temporary allocationとentryの2回移動が代表的なMapで過度に高価なら、同一identity内のstorageを
交換するprimitiveを追加する根拠になる。

## この例が要求する境界

- Pool runtimeはStateとelementそれぞれについて、specializationが生成したlayout、share、drop glueを利用できる。
- Pool primitiveは[lifecycle contract](lifecycle-contract.md)のownership effect、遷移順序、未検査preconditionに従う。
- container実装はfile-local invariantから、公開preconditionを満たすcallのPool preconditionを導ける。
- opaque型の宣言元fileだけがrepresentation viewを使え、他fileはStateとslot invariantを迂回できない。
- hidden representationのrequirementをopaque type constructorの形成条件として公開できる。
- `hash`と`equal`はstorage primitiveではなく、通常のoperation familyとしてcontainer algorithmが要求する。
- `Representable`とC host mappingはPool、Buffer、Mapの形成から自動的に導かない。
