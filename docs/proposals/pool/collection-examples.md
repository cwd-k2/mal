# collection例

Status: Exploratory example

この文書は、[IxPool上のcontainer](containers.md)で比べたstack、Deque、binary heap、open addressing Map、SlotMap、木と、ImPool上の
immutable arrayを、試作で動かしたcodeから要点を抜き出して示す。primitiveの名前と区分は[primitive一覧](primitives.md)、Bufferは[Buffer実装](buffer-implementation.md)、
試作そのものは[試作で確かめたこと](prototypes.md)を正とする。例は未採択の擬似codeである。

各例は共通の補助として、capacityを4以上の倍増で確保する`reserveAtLeast(pool, required)`を使う。これはprimitiveではなく、
growth policyを各containerで繰り返さないための関数である。

## stack

Bufferと同じく`[0, count)`をLiveに保つが、popは値を取り出してcountを減らす。

```mal
opaque Stack<T> :: IxPool<USize, T>;

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

## Deque

Stateを`(head, count)`とし、`head`から`count`個のcoordinateをcapacityで折り返してLiveに保つring bufferである。

```mal
opaque Deque<T> :: IxPool<(USize, USize), T>;

_slot<T> :: (Deque<T>, USize, USize) -> USize := (deque, head, offset) ->
    (head + offset) % capacity(deque);

// A full ring holds its tail in `[0, head)`. After doubling, that tail moves to follow the old end.
_unwrap<T> :: (Deque<T>, USize, USize, USize) -> Unit :=
    (deque, index, head, previous) -> [return] => {
        when (index == head) return(());
        moveAt(deque, index, previous + index);
        return(_unwrap(deque, index + 1usize, head, previous));
    };

_ensureRoom<T> :: Deque<T> -> Unit := (deque) -> {
    (head, count) := state(deque);
    current := capacity(deque);
    when (count == current) {
        reserve(deque, if (current == 0usize) then 4usize else current * 2usize);
        _unwrap(deque, 0usize, head, current);
    };
};

pushFront<T> :: (Deque<T>, T) -> Unit := (deque, value) -> {
    _ensureRoom(deque);
    (head, count) := state(deque);
    front := _slot(deque, head, capacity(deque) - 1usize);
    initAt(deque, front, value);
    setState(deque, (front, count + 1usize));
};

popBack<T> :: Deque<T> -> [Unit, T] := (deque) -> [empty, found] => {
    (head, count) := state(deque);
    when (count == 0usize) empty();
    value := takeAt(deque, _slot(deque, head, count - 1usize));
    setState(deque, (head, count - 1usize));
    found(value);
};
```

成長は`reserve`でcoordinateを保ったままcapacityを倍にし、折り返していた`[0, head)`を`moveAt`で旧capacityの後ろへ移す。
移動は`Share`も`Drop`も起こさない。`pushBack`と`popFront`も同じ形で書ける。

## binary heap

`[0, count)`をLiveに保ち、coordinate `i`の子を`2i + 1`と`2i + 2`とする。siftでは値を交換せず、Vacantな穴を動かす。

```mal
less<T> :: (T, T) -> Bool;
opaque Heap<T> :: IxPool<USize, T>;

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
opaque HashMap<K, V> :: IxPool<USize, (K, V)>;

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
requirementであり、IxPoolはhash、equality、load factor、probe順序を知らない。

Mapの公開operationは利用者にpreconditionを課さないため、IxPoolのpreconditionはすべて実装が満たす。capacity 0では`_locate`が
剰余を計算せずmissingを返し、以後のprobe coordinateは剰余で範囲内になる。`getAt`と`takeAt`は`isLive`がtrueだった
coordinateにだけ、`initAt`はfalseだったcoordinateか、直前に`takeAt`したcoordinateにだけ呼ぶ。`hash`と`equal`は変更中のMapへ
到達できないため、判定から呼び出しまでの間に状態は変わらない。

insertはload factorが3/4を超える前にrehashする。別のIxPoolへ移し替えると既存のaliasが追随しないため、entryを`takeAt`で一時的な
IxPoolへ移し、元のIxPoolを`reserve`してから新しいprobe位置へ`initAt`で戻す。移動はすべて`Consume`で、keyとvalueの`Share`も`Drop`も
起きない。一時的なallocationと二回の移動が代表的なMapで高価なら、同じidentityのstorageを交換するprimitiveを検討する。

## SlotMap

generationで古い`SlotKey<T>`を検出するmapを、IxPoolの上に書いた形である。要素の値、coordinateごとの
generation、空いたcoordinateのstackを別々のIxPoolに置く。Vacantなslotは値を持たないため、generationと
free listをvalueのIxPoolへ置けない。

```mal
opaque SlotKey<T> :: (USize, UInt64); // coordinate、発行時のgeneration

// values: Stateは要素数。generations: 発行した全coordinateでLive、Stateは発行数。free: 空いたcoordinateのstack。
opaque SlotMap<T> :: (IxPool<USize, T>, IxPool<USize, UInt64>, IxPool<USize, UInt64>);

_current<T> :: (SlotMap<T>, SlotKey<T>) -> Bool := ((_, generations, _), (index, generation)) ->
    index < state(generations) && getAt(generations, index) == generation;

slotMapRemove<T> :: (SlotMap<T>, SlotKey<T>) -> [Unit, T] := (pool, id) -> [missing, found] => {
    when (!_current(pool, id)) missing();
    (values, generations, free) := pool;
    (index, generation) := id;
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

挿入は`free`の先頭からcoordinateを再利用し、なければ新しいcoordinateを発行する。`SlotKey<T>`の照合は利用者が古い`SlotKey<T>`を持ち
続けるため検査してmissingを返し、IxPoolのpreconditionへは流さない。このsketchは`SlotKey<T>`にSlotMapのidentityを含めないため、別の
SlotMapの`SlotKey<T>`を区別しない。区別が要るなら、SlotMapごとの番号をStateに持って`SlotKey<T>`へ含める。

## 木

節点`(key, value, left, right)`をslotに置き、子をcoordinateで指す二分探索木である。`nodes`のStateは`(root, size)`で、空いた
coordinateは`free`のstackで再利用する。

```mal
_Node :: (UInt64, UInt64, UInt64, UInt64);
opaque Tree :: (IxPool<(USize, USize), _Node>, IxPool<USize, UInt64>);

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

## immutable array

ImPoolの上で、更新のたびに新しい値を返す配列である。Stateは長さで、`[0, length)`だけがLiveである。mutable arrayに当たる
ものはBufferであり、同じinvariantをIxPoolの上に置く。

```mal
opaque Array<T> :: ImPool<USize, T>;

arraySet<T> :: (Array<T>, USize, T) -> Array<T> := (array, index, value) ->
    putAt(array, index, value);

arrayAppend<T> :: (Array<T>, T) -> Array<T> := (array, value) -> {
    length := state(array);
    current := capacity(array);
    grown := if (length < current)
        then array
        else reserve(array, if (current == 0usize) then 1usize else current * 2usize);
    setState(initAt(grown, length, value), length + 1usize);
};
```

ImPoolの更新は参照数や一意性をsourceへ返さず、次の二つを同じ意味として選ぶ。

```text
inputが唯一:       Array A ── storage P ── update in place ── Array B

inputにaliasあり:  Array A ── storage P  = [A, B, C]
                                  share live elements
                   Array B ── storage P' = [A, X, C]
```

callerが旧Arrayを後でも使う場合、call siteは渡すresponsibilityをShareするため、更新は新しいstorageを作る。旧Arrayがlast useなら
inputを`Consume`でき、他のaliasがなければ同じstorageを再利用する。`arrayAppend`のように更新を続けると、最初の更新が一意な
successorを作るため、以後の更新はその場で行われる。borrowedなcall経路ではcopyへfallbackしてよく、意味はcall conventionに
依存しない。`Array<T>`は`T`が`Storable`なら`Storable`になるため、`Array<Array<T>>`やMapのvalueにできる。

chunk単位のCOWやpersistent vectorは共有時のcopy量を減らせる一方、複数storageの所有と使われなくなったnodeの回収を別途定める
必要がある。外部の事例は[Pool storageの関連事例](../../research/pool-storage-prior-art.md)にまとめる。

## 試作での確認

試作では、六つの例を次の条件で動かし、全IxPoolの解放まで確認した。

- stack：2000個をpushし、逆順にpopする。
- Deque：両端へpushして折り返しを作り、両端からpopした後、折り返したまま成長させても順序が保たれることを確かめる。
- binary heap：擬似乱数の2000個をpushし、popの結果が減少しないことと個数を確かめる。
- SlotMap：2000個を挿入し、3個に1個を削除した後、同数を挿入し直す。削除した`SlotKey<T>`はmissingになり、残した`SlotKey<T>`と新しい`SlotKey<T>`は
  値を返す。
- Map：2000個を挿入し、100個を置き換え、1000個を削除した後、全keyの存在と値を確かめる。rehashを含む。
- 木：擬似乱数のkeyを2000個挿入し、半分を削除して、残りの存在、削除したkeyの不在、中順の単調性を確かめる。削除したkeyを
  入れ直してもcapacityは増えない。

C host上の試作ではword IxPoolの`getAt`を`takeAt`と`initAt`で実装しているため、heapと木の比較や走査がhost callを倍にする。これは
[primitive一覧](primitives.md#ixpool)が`getAt`をprimitiveに残す理由の一つである。
