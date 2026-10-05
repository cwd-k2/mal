# keyで引くcontainer

Status: Exploratory example

この文書は、[IxPool上のcontainer](overview.md)のうち、Liveなcoordinateの集合が区間にならないopen addressing Map、
SlotMap、木を、試作で動かしたcodeから要点を抜き出して示す。primitiveの名前と区分は[Pool primitive](../api/pool.md)、試作は
[試作で確かめたこと](../prototypes.md)を正とする。例は未採択の擬似codeである。

各例は共通の補助として、capacityが`required`未満なら4以上の倍増で`grow`する`ensureCapacity(pool, required)`を使う。これはprimitiveではなく、
growth policyを各containerで繰り返さないための関数である。

## open addressing Map

linear probingのMapである。Headerは要素数で、どのprobe列も途中にVacantを含まない。削除はtombstoneを置かず、後続のentryを穴へ
詰めてこのinvariantを保つ。

```mal
equal<K> :: (K, K) -> Bool;
hash<K> :: K -> UInt64;
opaque HashMap<K, V> :: IxPool<USize, (K, V)>;

_find<K, V> :: (HashMap<K, V>, K, USize, USize) -> [Unit, USize] :=
    (map, key, index, remaining) -> [missing, found] => {
        when (remaining == 0usize) missing();
        (storedKey, _) := peek(map, index)[() -> missing(), (entry) -> entry];
        when (equal(storedKey, key)) found(index);
        _find(map, key, _next(map, index), remaining - 1usize)[missing, found];
    };

// Moves each following entry back into the gap unless its home lies cyclically in (gap, index].
_closeGap<K, V> :: (HashMap<K, V>, USize, USize) -> Unit := (map, gap, index) -> [return] => {
    entry := swap(map, index, vacant())[() -> return(()), (entry) -> entry];
    (key, _) := entry;
    home := _home(map, key);
    stays := if (gap < index) then home > gap && home <= index else home > gap || home <= index;
    when (stays) {
        initAt(map, index, entry);
        return(_closeGap(map, gap, _next(map, index)));
    };
    initAt(map, gap, entry);
    return(_closeGap(map, index, _next(map, index)));
};

mapRemove<K, V> :: (HashMap<K, V>, K) -> [Unit, V] := (map, key) -> [missing, found] => {
    index := _locate(map, key)[missing, (index) -> index];
    (_, value) := takeAt(map, index);
    setHeader(map, header(map) - 1usize);
    _closeGap(map, index, _next(map, index));
    found(value);
};
```

`_home`はhashをcapacityで割ったcoordinate、`_next`は一つ先のcoordinateを返す。`hash`と`equal`はoperation familyの
requirementであり、IxPoolはhash、equality、load factor、probe順序を知らない。

Mapの公開operationは利用者にpreconditionを課さないため、IxPoolのpreconditionはすべて実装が満たす。capacity 0では`_locate`が
剰余を計算せずmissingを返し、以後のprobe coordinateは剰余で範囲内になる。`_find`と`_closeGap`は核の`peek`と`swap`の
結果を除去してVacantを判定し、Live/Vacantのpreconditionを持つ周辺operationは、`mapRemove`の`takeAt`を`_locate`が見つけた
coordinateにだけ、`initAt`を直前に`swap`でVacantにしたcoordinateにだけ呼ぶ。

handle nestingだけでは、表現に寄与する再帰型とfunction storageがないため、keyからそれを保持する同じMapへのowner back-edgeは
作れない。ただしmutable referentの内容をhashまたはequalityへ使うkeyは、格納後の変更でprobe invariantを壊す。
`Storable(K)`はこれを防がないため、Map keyのrequirementはhashとequalityの時間的安定性を別に定める。caller-supplied closureを
将来受け取るoperationは、同じMapをcaptureした再入を別途考慮する。

insertはload factorが3/4を超える前にrehashする。別のIxPoolへ移し替えると既存のaliasが追随しないため、entryを`takeAt`で一時的な
IxPoolへ移し、元のIxPoolを`grow`してから新しいprobe位置へ`initAt`で戻す。移動はすべて`Consume`で、keyとvalueの`Share`も`Drop`も
起きない。一時的なallocationと二回の移動が代表的なMapで高価なら、同じidentityのstorageを交換するprimitiveを検討する。

## SlotMap

generationで古い`SlotKey<T>`を検出するmapを、IxPoolの上に書いた形である。要素の値、coordinateごとの
generation、空いたcoordinateのstackを別々のIxPoolに置く。Vacantなslotは`Unit`しか持たないため、
generationとfree listを要素用のIxPoolへ置けない。

```mal
opaque SlotKey<T> :: (USize, UInt64); // coordinate、発行時のgeneration

// values: Headerは要素数。generations: 発行した全coordinateでLive、Headerは発行数。free: 空いたcoordinateのstack。
opaque SlotMap<T> :: (IxPool<USize, T>, IxPool<USize, UInt64>, IxPool<USize, UInt64>);

_current<T> :: (SlotMap<T>, SlotKey<T>) -> Bool := ((_, generations, _), (index, generation)) ->
    index < header(generations) && getAt(generations, index) == generation;

slotMapRemove<T> :: (SlotMap<T>, SlotKey<T>) -> [Unit, T] := (pool, id) -> [missing, found] => {
    when (!_current(pool, id)) missing();
    (values, generations, free) := pool;
    (index, generation) := id;
    value := takeAt(values, index);
    setHeader(values, header(values) - 1usize);
    putAt(generations, index, generation + 1u64);
    vacated := header(free);
    ensureCapacity(free, vacated + 1usize);
    initAt(free, vacated, index.u64);
    setHeader(free, vacated + 1usize);
    found(value);
};
```

挿入は`free`の先頭からcoordinateを再利用し、なければ新しいcoordinateを発行する。`SlotKey<T>`の照合は利用者が古い`SlotKey<T>`を持ち
続けるため検査してmissingを返し、IxPoolのpreconditionへは流さない。このsketchは`SlotKey<T>`にSlotMapのidentityを含めないため、別の
SlotMapの`SlotKey<T>`を区別しない。区別が要るなら、SlotMapごとの番号をHeaderに持って`SlotKey<T>`へ含める。

## 木

節点`(key, value, left, right)`をslotに置き、子をcoordinateで指す二分探索木である。`nodes`のHeaderは`(root, size)`で、空いた
coordinateは`free`のstackで再利用する。

```mal
_Node :: (UInt64, UInt64, UInt64, UInt64);
opaque Tree :: (IxPool<(USize, USize), _Node>, IxPool<USize, UInt64>);

_release :: (Tree, UInt64) -> Unit := ((nodes, free), link) -> {
    takeAt(nodes, link.usize);
    (root, size) := header(nodes);
    setHeader(nodes, (root, size - 1usize));
    vacated := header(free);
    ensureCapacity(free, vacated + 1usize);
    initAt(free, vacated, link);
    setHeader(free, vacated + 1usize);
};
```

削除は、子が一つ以下の節点をその子で置き換えて`_release`し、子が二つの節点には右部分木の最小の節点を移して、その節点を
`_release`する。親の子linkを書き換えるときは`putAt`で節点全体を置き換える。削除したkeyを入れ直すと、空いたcoordinateを
再利用するためcapacityは増えない。
