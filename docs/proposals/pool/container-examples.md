# Pool-backed containerのsource sketch

Status: Exploratory example

この文書は[Poolとopaque型によるcontainer基盤](README.md)の候補modelを、将来のmal sourceに近い擬似codeで
具体化する。`opaque`、`Pool<State, T>`、file-privateなpack/openは未採択であり、現在のparserでは受理されない。
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
`poolReserve`だけがallocationを増やし、Stateと全slotを保ったまま指定したlogical capacityへ拡張するかtrapする。runtimeの
over-allocationは`poolCapacity`へ反映しない。Poolは自動的にgrowthせず、init対象は`index < poolCapacity(pool)`かつVacantでなければ
ならない。

`poolInitAt`と`poolPutAt`のvalue operandはowner successorである。execution ownershipは後続useのあるsourceを`Share`し、ownedな
last useを`Consume`する。`poolGetAt`はslotを残すためresultを`Share`し、`poolTakeAt`はstored responsibilityをresultへ
`Consume`する。`poolDropAt`はstored responsibilityを終了する。reserveによるrelocationはowner数を変えない。

Pool自身はStateにもelementにも格納できない。stable handleはこの例に必要なく、slot map、tree、graphが外部へkeyを返す実例から
nonowning Pool identityとgenerationのcontractを分離して検討する。

mutable arrayのidentity共有と、immutable arrayのcopy-on-writeは
[Pool array ownership example](array-ownership.md)で同じdense slot invariantを使って比較する。後者が仮定する
`poolWritableSuccessor`は基本slot APIではなく、参照数をsourceへ公開しないoptionalなtrusted operationである。

## opaque型の擬似operation

例ではopaque宣言が、宣言元fileだけから参照できるpack/open operationを概念上生成するとする。以下はsignatureを示すための擬似表記で、
通常のfunction application、call ABI、追加のShareまたはDropを発生させないrepresentation-preserving coercionである。

```mal
opaque Buffer<T> :: Pool<USize, T>;

_packBuffer<T> :: Pool<USize, T> -> Buffer<T>;
_openBuffer<T> :: Buffer<T> -> Pool<USize, T>;
```

実際のpack/open構文は未決定である。`Buffer<T>`は`Pool<USize, T>`とnominally異なり、別fileはrepresentationを開けない。
compilerはhidden representationから`Buffer<T>`のlifecycleと形成条件`Storable(T)`を導ける。

## Buffer

BufferはPoolのStateをlogical countにし、denseなlive prefixとgrowth policyを宣言元fileで維持する。

```mal
opaque Buffer<T> :: Pool<USize, T>;

makeBuffer<T> :: USize -> Buffer<T> := (initialCapacity) ->
    _packBuffer<T>(makePool<USize, T>(0usize, initialCapacity));

length<T> :: Buffer<T> -> USize := (buffer) ->
    poolState<USize, T>(_openBuffer<T>(buffer));

_nextCapacity :: (USize, USize) -> USize := (current, required) -> {
    doubled := current * 2usize;
    if (doubled < required) then required else doubled;
};

new<T> :: (Buffer<T>, T) -> USize := (buffer, value) -> {
    pool := _openBuffer<T>(buffer);
    count := poolState<USize, T>(pool);
    if (count == poolCapacity<USize, T>(pool))
    then poolReserve<USize, T>(pool, _nextCapacity(count, count + 1usize))
    else ();
    poolInitAt<USize, T>(pool, count, value);
    poolSetState<USize, T>(pool, count + 1usize);
    count;
};

get<T> :: (Buffer<T>, USize) -> T := (buffer, index) ->
    poolGetAt<USize, T>(_openBuffer<T>(buffer), index);

put<T> :: (Buffer<T>, USize, T) -> Unit := (buffer, index, value) ->
    poolPutAt<USize, T>(_openBuffer<T>(buffer), index, value);
```

BufferだけがStateを更新し、removeを公開しないため、live coordinateは常に`[0, length(buffer))`である。capacity 0からのgrowth、
overflow、`index < length(buffer)`のpreconditionはBuffer implementationが所有する。Poolはこれらをsequence ruleとして知らない。
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

_packMap<K, V> :: Pool<_MapState, _MapSlot<K, V>> -> Map<K, V>;
_openMap<K, V> :: Map<K, V> -> Pool<_MapState, _MapSlot<K, V>>;
```

Mapの宣言元fileはcapacity、load factor、probe、rehashをmalで実装する。

```mal
makeMap<K, V> :: USize -> Map<K, V> := (initialCapacity) ->
    _packMap<K, V>(
        makePool<_MapState, _MapSlot<K, V>>(
            (0usize, 0usize),
            initialCapacity
        )
    );

mapLength<K, V> :: Map<K, V> -> USize := (map) -> {
    (size, _) := poolState(_openMap<K, V>(map));
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
        _openMap<K, V>(map),
        hash(key),
        key,
        equal<K>
    );
```

`mapGet<K, V>`から`hash<K>`と`equal<K>`のoperation requirementが導かれる。Poolはhash、equality、load factor、probe順序を知らない。
Mapのcopyは同じPool identityを共有するため全aliasから更新を観測し、独立snapshotは新しいPoolへlive entryだけをrehashする。

rehashでMapのhidden representationを別Poolへ交換すると既存aliasが追随しない。現APIだけでidentityを保つ場合は、mutation前にtemporary
Poolへ全entryを新配置で構築し、元Poolのreserveが成功した後、元slotをdropしてtemporary slotをtake/initで戻す。allocationとhash計算を
元Poolのmutation前に完了させれば、commit部分はlifecycle遷移だけになる。これが代表的なMapで過度に高価なら、同一identity内のstorageを
transactionally交換するprimitiveを追加する根拠になる。

## この例が要求する境界

- Pool runtimeはStateとelementそれぞれについて、specializationが生成したlayout、share、drop glueを利用できる。
- `reserve`はStateと全slotのresponsibilityを変えず、物理carrierだけをrelocateする。
- 単一slotへの保存はowner successorとしてownership planへ現れ、borrowed sourceの`Share`とlast-use ownerの`Consume`を区別できる。
- runtime長の`fill`と`copy`には動的個数の`Share`、Pool破棄にはStateと全live slotの`Drop`が必要である。
- opaque型の宣言元fileだけがpack/openでき、他fileはStateとslot invariantを迂回できない。
- hidden representationのrequirementをopaque type constructorの形成条件として公開できる。
- `hash`と`equal`はstorage primitiveではなく、通常のoperation familyとしてcontainer algorithmが要求する。
- `Representable`とC host mappingはPool、Buffer、Mapの形成から自動的に導かない。
