# 列のcontainer

Status: Exploratory example

この文書は、[IxPool上のcontainer](overview.md)のうち、Liveなcoordinateが列をなすstack、Deque、binary heapと、ImPool上の
Vectorを、試作で動かしたcodeから要点を抜き出して示す。primitiveの名前と区分は[Pool primitive](../api/pool.md)、試作は
[試作で確かめたこと](../prototypes.md)を正とする。例は未採択の擬似codeである。

各例は共通の補助として、capacityが`required`未満なら4以上の倍増で`grow`する`ensureCapacity(pool, required)`を使う。これはprimitiveではなく、
growth policyを各containerで繰り返さないための関数である。

## stack

Bufferと同じく`[0, count)`をLiveに保つが、popは値を取り出してcountを減らす。

```mal
opaque Stack<T> :: IxPool<USize, T>;

push<T> :: (Stack<T>, T) -> Unit := (stack, value) -> {
    count := meta(stack);
    ensureCapacity(stack, count + 1usize);
    initAt(stack, count, value);
    setMeta(stack, count + 1usize);
};

pop<T> :: Stack<T> -> [Unit, T] := (stack) -> [empty, found] => {
    count := meta(stack);
    when (count == 0usize) empty();
    top := count - 1usize;
    value := takeAt(stack, top);
    setMeta(stack, top);
    found(value);
};
```

`takeAt`は値のresponsibilityを呼び出し元へ移し、slotをVacantへ戻す。現在のBufferでは、取り出した値は上書きするまでstorageに残る。

## Deque

Metaを`(head, count)`とし、`head`から`count`個のcoordinateをcapacityで折り返してLiveに保つring bufferである。

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
    (head, count) := meta(deque);
    current := capacity(deque);
    when (count == current) {
        grow(deque, if (current == 0usize) then 4usize else current);
        _unwrap(deque, 0usize, head, current);
    };
};

pushFront<T> :: (Deque<T>, T) -> Unit := (deque, value) -> {
    _ensureRoom(deque);
    (head, count) := meta(deque);
    front := _slot(deque, head, capacity(deque) - 1usize);
    initAt(deque, front, value);
    setMeta(deque, (front, count + 1usize));
};

popBack<T> :: Deque<T> -> [Unit, T] := (deque) -> [empty, found] => {
    (head, count) := meta(deque);
    when (count == 0usize) empty();
    value := takeAt(deque, _slot(deque, head, count - 1usize));
    setMeta(deque, (head, count - 1usize));
    found(value);
};
```

成長は`grow`でcoordinateを保ったままcapacityを倍にし、折り返していた`[0, head)`を`moveAt`で旧capacityの後ろへ移す。
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
    count := meta(heap);
    when (count == 0usize) empty();
    top := takeAt(heap, 0usize);
    last := count - 1usize;
    setMeta(heap, last);
    when (last > 0usize) _siftDown(heap, 0usize, takeAt(heap, last), last);
    found(top);
};
```

`_siftDown`は小さい方の子を`moveAt`で穴へ上げる。移動は`takeAt`と`initAt`だけで、比較の`getAt`以外に`Share`も`Drop`も起こさない。
`less`は[operation family](../../../spec/operation-families.md)のrequirementとしてheapへ渡る。

## Vector

ImPoolの上で、更新のたびに新しい値を返す平らな列である。Metaは長さで、`[0, length)`だけがLiveである。Bufferが組み立てる
ための可変な列であるのに対し、Vectorは確定した値の列であり、Bufferから`freeze`で作れる。byte列ではこの対が`Buffer<UInt8>`と
`Symbol`に当たる。Clojure、ScalaのVectorのように木で構造を共有せず、共有中の更新はcopyする。hostとの交換と`Symbol`の構築は、組み込み
libraryとしてのVectorが[primitive](../api/buffer-vector.md#vector)として持つ。

```mal
opaque Vector<T> :: ImPool<USize, T>;

vectorSet<T> :: (Vector<T>, USize, T) -> Vector<T> := (vector, index, value) ->
    putAt(vector, index, value);

vectorAppend<T> :: (Vector<T>, T) -> Vector<T> := (vector, value) -> {
    length := meta(vector);
    current := capacity(vector);
    grown := if (length < current)
        then vector
        else grow(vector, if (current == 0usize) then 1usize else current);
    setMeta(initAt(grown, length, value), length + 1usize);
};

_copyFrom<T> :: (Vector<T>, Vector<T>, USize, USize, USize) -> Vector<T> :=
    (source, target, offset, index, length) -> [return] => {
        when (index == length) return(target);
        return(_copyFrom(source, initAt(target, index, getAt(source, offset + index)), offset, index + 1usize, length));
    };

// Precondition: `offset + length <= meta(vector)`.
slice<T> :: (Vector<T>, USize, USize) -> Vector<T> := (vector, offset, length) ->
    _copyFrom(vector, setMeta(grow(pool(0usize), length), length), offset, 0usize, length);
```

ImPoolの更新は参照数や一意性をsourceへ返さず、次の二つを同じ意味として選ぶ。

```text
inputが唯一:       Vector A ── storage P ── update in place ── Vector B

inputにaliasあり:  Vector A ── storage P  = [A, B, C]
                                  share live elements
                   Vector B ── storage P' = [A, X, C]
```

callerが旧Vectorを後でも使う場合、call siteは渡すresponsibilityをShareするため、更新は新しいstorageを作る。旧Vectorがlast useなら
inputを`Consume`でき、他のaliasがなければ同じstorageを再利用する。`vectorAppend`のように更新を続けると、最初の更新が一意な
successorを作るため、以後の更新はその場で行われる。borrowedなcall経路ではcopyへfallbackしてよく、意味はcall conventionに
依存しない。`Vector<T>`は`T`が`Storable`なら`Storable`になるため、`Vector<Vector<T>>`やMapのvalueにできる。

chunk単位のCOWやpersistent vectorは共有時のcopy量を減らせる一方、複数storageの所有と使われなくなったnodeの回収を別途定める
必要がある。外部の事例は[Pool storageの関連事例](../../../research/pool-storage-prior-art.md)にまとめる。

