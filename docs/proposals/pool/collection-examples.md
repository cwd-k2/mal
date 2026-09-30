# Pool上のcollection例

Status: Exploratory example

この文書は、[Pool上のcontainer](containers.md)で比べたstack、binary heap、open addressing Map、slot map、木を、試作で動かした
codeから要点を抜き出して示す。primitiveの名前と区分は[primitive一覧](primitives.md)、Bufferは[Buffer実装](buffer-implementation.md)、
試作そのものは[試作で確かめたこと](prototypes.md)を正とする。例は未採択の擬似codeである。

各例は共通の補助として、capacityを4以上の倍増で確保する`reserveAtLeast(pool, required)`を使う。これはprimitiveではなく、
growth policyを各containerで繰り返さないための関数である。

## stack

Bufferと同じく`[0, count)`をLiveに保つが、popは値を取り出してcountを減らす。

```mal
opaque Stack<T> :: Pool<USize, T>;

push<T> :: (Stack<T>, T) -> Unit := (stack, value) -> {
    count := state(stack);
    reserveAtLeast(stack, count + 1usize);
    initAt(stack, count, value);
    setState(stack, count + 1usize);
};

pop<T> :: Stack<T> -> [Unit, T] := (stack) -> [empty, found] => {
    count := state(stack);
    when (count == 0usize) empty();
    top := count - 1usize;
    value := takeAt(stack, top);
    setState(stack, top);
    found(value);
};
```

`takeAt`は値のresponsibilityを呼び出し元へ移し、slotをVacantへ戻す。現在のBufferでは、取り出した値は上書きするまでstorageに残る。

## binary heap

`[0, count)`をLiveに保ち、coordinate `i`の子を`2i + 1`と`2i + 2`とする。siftでは値を交換せず、Vacantな穴を動かす。

```mal
less<T> :: (T, T) -> Bool;
opaque Heap<T> :: Pool<USize, T>;

// The slot at `hole` is Vacant. Parents greater than `value` move down into it.
_siftUp<T> :: (Heap<T>, USize, T) -> Unit := (heap, hole, value) -> [return] => {
    when (hole == 0usize) {
        initAt(heap, 0usize, value);
        return(());
    };
    parent := (hole - 1usize) / 2usize;
    when (!less(value, getAt(heap, parent))) {
        initAt(heap, hole, value);
        return(());
    };
    moveAt(heap, parent, hole);
    return(_siftUp(heap, parent, value));
};

heapPop<T> :: Heap<T> -> [Unit, T] := (heap) -> [empty, found] => {
    count := state(heap);
    when (count == 0usize) empty();
    top := takeAt(heap, 0usize);
    last := count - 1usize;
    setState(heap, last);
    when (last > 0usize) _siftDown(heap, 0usize, takeAt(heap, last), last);
    found(top);
};
```

`_siftDown`は小さい方の子を`moveAt`で穴へ上げる。移動は`takeAt`と`initAt`だけで、比較の`getAt`以外に`Share`も`Drop`も起こさない。
`less`は[operation family](../../spec/operation-families.md)のrequirementとしてheapへ渡る。

## open addressing Map

linear probingのMapである。Stateは要素数で、どのprobe列も途中にVacantを含まない。削除はtombstoneを置かず、後続のentryを穴へ
詰めてこのinvariantを保つ。

```mal
equal<K> :: (K, K) -> Bool;
hash<K> :: K -> UInt64;
opaque HashMap<K, V> :: Pool<USize, (K, V)>;

_find<K, V> :: (HashMap<K, V>, K, USize, USize) -> [Unit, USize] :=
    (map, key, index, remaining) -> [missing, found] => {
        when (remaining == 0usize) missing();
        when (!isLive(map, index)) missing();
        (storedKey, _) := getAt(map, index);
        when (equal(storedKey, key)) found(index);
        _find(map, key, _next(map, index), remaining - 1usize)[missing, found];
    };

// Moves each following entry back into the gap unless its home lies cyclically in (gap, index].
_closeGap<K, V> :: (HashMap<K, V>, USize, USize) -> Unit := (map, gap, index) -> [return] => {
    when (!isLive(map, index)) return(());
    entry := takeAt(map, index);
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
    setState(map, state(map) - 1usize);
    _closeGap(map, index, _next(map, index));
    found(value);
};
```

`_home`はhashをcapacityで割ったcoordinate、`_next`は一つ先のcoordinateを返す。`hash`と`equal`はoperation familyの
requirementであり、Poolはhash、equality、load factor、probe順序を知らない。

Mapの公開operationは利用者にpreconditionを課さないため、Poolのpreconditionはすべて実装が満たす。capacity 0では`_locate`が
剰余を計算せずmissingを返し、以後のprobe coordinateは剰余で範囲内になる。`getAt`と`takeAt`は`isLive`がtrueだった
coordinateにだけ、`initAt`はfalseだったcoordinateか、直前に`takeAt`したcoordinateにだけ呼ぶ。`hash`と`equal`は変更中のMapへ
到達できないため、判定から呼び出しまでの間に状態は変わらない。

insertはload factorが3/4を超える前にrehashする。別のPoolへ移し替えると既存のaliasが追随しないため、entryを`takeAt`で一時的な
Poolへ移し、元のPoolを`reserve`してから新しいprobe位置へ`initAt`で戻す。移動はすべて`Consume`で、keyとvalueの`Share`も`Drop`も
起きない。一時的なallocationと二回の移動が代表的なMapで高価なら、同じidentityのstorageを交換するprimitiveを検討する。

## slot map

要素の値、coordinateごとのgeneration、空いたcoordinateのstackを別々のPoolに置く。Vacantなslotは値を持たないため、generationと
free listをvalueのPoolへ置けない。

```mal
SlotKey :: (USize, UInt64); // coordinate、発行時のgeneration

// values: Stateは要素数。generations: 発行した全coordinateでLive、Stateは発行数。free: 空いたcoordinateのstack。
opaque SlotMap<T> :: (Pool<USize, T>, Pool<USize, UInt64>, Pool<USize, UInt64>);

_current<T> :: (SlotMap<T>, SlotKey) -> Bool := ((_, generations, _), (index, generation)) ->
    index < state(generations) && getAt(generations, index) == generation;

slotRemove<T> :: (SlotMap<T>, SlotKey) -> [Unit, T] := (map, key) -> [missing, found] => {
    when (!_current(map, key)) missing();
    (values, generations, free) := map;
    (index, generation) := key;
    value := takeAt(values, index);
    setState(values, state(values) - 1usize);
    putAt(generations, index, generation + 1u64);
    vacated := state(free);
    reserveAtLeast(free, vacated + 1usize);
    initAt(free, vacated, index.u64);
    setState(free, vacated + 1usize);
    found(value);
};
```

挿入は`free`の先頭からcoordinateを再利用し、なければ新しいcoordinateを発行する。keyの照合は利用者が古いkeyを持ち続けるため
検査してmissingを返し、Poolのpreconditionへは流さない。この例はkeyの照合をmalで書いており、
[Pool key extension](pool-keys.md)の`SlotKey`をprimitiveにする前の形である。

## 木

節点`(key, value, left, right)`をslotに置き、子をcoordinateで指す二分探索木である。`nodes`のStateは`(root, size)`で、空いた
coordinateは`free`のstackで再利用する。

```mal
_Node :: (UInt64, UInt64, UInt64, UInt64);
opaque Tree :: (Pool<(USize, USize), _Node>, Pool<USize, UInt64>);

_release :: (Tree, UInt64) -> Unit := ((nodes, free), link) -> {
    _ := takeAt(nodes, link.usize);
    (root, size) := state(nodes);
    setState(nodes, (root, size - 1usize));
    vacated := state(free);
    reserveAtLeast(free, vacated + 1usize);
    initAt(free, vacated, link);
    setState(free, vacated + 1usize);
};
```

削除は、子が一つ以下の節点をその子で置き換えて`_release`し、子が二つの節点には右部分木の最小の節点を移して、その節点を
`_release`する。親の子linkを書き換えるときは`putAt`で節点全体を置き換える。削除したkeyを入れ直すと、空いたcoordinateを
再利用するためcapacityは増えない。

## 試作での確認

試作では、五つの例を次の条件で動かし、全Poolの解放まで確認した。

- stack：2000個をpushし、逆順にpopする。
- binary heap：擬似乱数の2000個をpushし、popの結果が減少しないことと個数を確かめる。
- slot map：2000個を挿入し、3個に1個を削除した後、同数を挿入し直す。削除したkeyはmissingになり、残したkeyと新しいkeyは
  値を返す。
- Map：2000個を挿入し、100個を置き換え、1000個を削除した後、全keyの存在と値を確かめる。rehashを含む。
- 木：擬似乱数のkeyを2000個挿入し、半分を削除して、残りの存在、削除したkeyの不在、中順の単調性を確かめる。削除したkeyを
  入れ直してもcapacityは増えない。

C host上の試作ではword Poolの`getAt`を`takeAt`と`initAt`で実装しているため、heapと木の比較や走査がhost callを倍にする。これは
[primitive一覧](primitives.md#slot-primitive)が`getAt`をprimitiveに残す理由の一つである。
