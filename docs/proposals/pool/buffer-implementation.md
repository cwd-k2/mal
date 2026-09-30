# Pool上のBuffer実装

Status: Exploratory example

この文書は、[AddressとBuffer](../../spec/memory.md)が定める`Buffer<A>`の全operationを、core Pool APIと少数のcompiler primitiveで
実装した擬似codeを示す。Pool primitiveの規則は[lifecycle contract](lifecycle-contract.md)、Bufferの最小sketchは
[source sketch](container-examples.md#buffer)を正とする。

predefinedな名前`make`、`new`、`get`、prefix `#`と`*`、receiver-first形を通常のmal fileへ結ぶ規則は本書の対象外である。
以下はそのfileがpreludeとしてこれらの名前を定義できると仮定する。

## 追加するcompiler primitive

core Pool APIに加え、次を仮定する。

```mal
trap :: Symbol -> [];

poolWriteRange<State, T> :: (Pool<State, T>, USize, USize, T) -> Unit;
poolCopyRange<State, T> :: (Pool<State, T>, USize, Pool<State, T>, USize, USize) -> Unit;
poolLoadSymbol<State> :: (Pool<State, UInt8>, USize, Symbol) -> Unit;

poolLoad<State, T> :: (Pool<State, T>, USize, Address, USize, USize) -> Unit;
poolStore<State, T> :: (Pool<State, T>, USize, USize, Address) -> Unit;
poolSymbol<State> :: (Pool<State, UInt8>, USize, USize) -> Symbol;
```

`trap`は[primitive `trap`案](../primitive-trap.md)のものである。rangeを書き込むprimitiveは、destination range内の各slotを
Liveにする。Live slotは新valueを成立させてから旧valueをDropし、Vacant slotはinitする。

| primitive | 引数の意味 | precondition | 必要な理由 |
|---|---|---|---|
| `poolWriteRange` | `(pool, offset, length, value)` | `offset + length <= capacity` | 性能。Pool callのloopで書ける |
| `poolCopyRange` | `(dest, destOffset, source, sourceOffset, length)` | dest rangeがcapacity内、source rangeが全てLive | 性能。loopで書ける |
| `poolLoadSymbol` | `(pool, offset, symbol)` | `offset + #symbol <= capacity` | 性能。`symbol # index`のloopで書ける |
| `poolLoad` | `(pool, offset, address, hostOffset, length)` | pool rangeがcapacity内、host rangeは`from`と同じ | 必須。malはAddressを読めない |
| `poolStore` | `(pool, offset, length, address)` | pool rangeが全てLive、host rangeは`into`と同じ | 必須。malはAddressへ書けない |
| `poolSymbol` | `(pool, offset, length)` | rangeが全てLive | 必須。bytes列からSymbolを作る操作がない |

`poolCopyRange`はsourceとdestinationが同じidentityでもよく、operation開始時点のsource rangeを写した結果になる。
range primitiveの`init`と`take`への分解と`share`、`drop`の回数は[所有権primitive](ownership-primitives.md#派生operation)に示す。`poolLoad`と`poolStore`は`Representable(T)`を
要求し、host側のoffset計算を表現できない場合は現行の`from`、`into`と同じくtrapする。

## 表現とinvariant

```mal
opaque Buffer<A> :: Pool<USize, A>;
```

Stateはcountである。`[0, count)`がLive、`[count, poolCapacity)`がVacantであり、`count <= poolCapacity`を保つ。
公開operationは利用者がBufferの[未検査precondition](../../spec/memory.md#未検査precondition)を満たす限り、このinvariantから
Pool preconditionを導く。利用者が違反した場合はinvariantが壊れ得るが、結果を保証しない点は現行Bufferと同じである。

## 補助

```mal
_maxUSize :: USize := -1usize;

_rangeEnd :: (USize, USize) -> USize := (offset, length) -> {
    when (length > _maxUSize - offset) trap("Buffer range overflow")[];
    offset + length;
};

_ensureCapacity<A> :: (Buffer<A>, USize) -> Unit := (buffer, required) -> {
    capacity := poolCapacity<USize, A>(buffer);
    when (required > capacity) {
        doubled := if (capacity > _maxUSize / 2usize) then _maxUSize else capacity * 2usize;
        poolReserve<USize, A>(buffer, if (doubled < required) then required else doubled);
    };
};

_extendCount<A> :: (Buffer<A>, USize) -> Unit := (buffer, end) ->
    when (end > poolState<USize, A>(buffer)) poolSetState<USize, A>(buffer, end);
```

`_rangeEnd`はcountとrange末尾を表現できない場合の現行trapを再現する。allocation byte数を表現できない場合とallocation failureは
`poolReserve`がtrapする。growth policyはこのfileが所有する。現行runtimeは必要byte数を16以上の2の累乗へ丸めるが、
このfileはelement数で倍増し、byte単位の丸めは`poolCapacity`から観測できない`poolReserve`内部の選択として残す。

## 要素operation

```mal
make<A> :: USize -> Buffer<A> := (capacity) -> makePool<USize, A>(0usize, capacity);

length<A> :: Buffer<A> -> USize := (buffer) -> poolState<USize, A>(buffer);

new<A> :: (Buffer<A>, A) -> USize := (buffer, value) -> {
    count := poolState<USize, A>(buffer);
    end := _rangeEnd(count, 1usize);
    _ensureCapacity<A>(buffer, end);
    poolInitAt<USize, A>(buffer, count, value);
    poolSetState<USize, A>(buffer, end);
    count;
};

get<A> :: (Buffer<A>, USize) -> A := (buffer, index) -> poolGetAt<USize, A>(buffer, index);

put<A> :: (Buffer<A>, USize, A) -> Unit := (buffer, index, value) ->
    poolPutAt<USize, A>(buffer, index, value);
```

`get`と`put`はBufferの`index < #buffer`をinvariantでLive slotへ写すだけで、検査を追加しない。`new`の`count`はinvariantにより
Vacantであり、`_ensureCapacity`の後はcapacity内にある。

## Range operation

```mal
fill<A> :: (Buffer<A>, USize, USize, A) -> Unit := (buffer, offset, length, value) -> {
    end := _rangeEnd(offset, length);
    _ensureCapacity<A>(buffer, end);
    poolWriteRange<USize, A>(buffer, offset, length, value);
    _extendCount<A>(buffer, end);
};

copy<A> :: (Buffer<A>, USize, Buffer<A>, USize, USize) -> Unit :=
    (destination, destinationOffset, source, sourceOffset, length) -> {
        end := _rangeEnd(destinationOffset, length);
        _ensureCapacity<A>(destination, end);
        poolCopyRange<USize, A>(destination, destinationOffset, source, sourceOffset, length);
        _extendCount<A>(destination, end);
    };
```

公開preconditionの`offset <= #buffer`により、destination rangeはLiveな部分とそれに続くVacantな部分からなり、穴を作らない。
`copy`の`sourceOffset + length <= #source`はsource rangeが全てLiveであることを与える。`_ensureCapacity`がsourceと同じPoolを
relocateしてもcoordinateは変わらない。countの更新はprimitiveの後に行い、その間にmal codeは走らない。

`poolWriteRange`が性能だけのためのprimitiveであることは、同じ遷移をPool callで書けることで分かる。

```mal
_fillFrom<A> :: (Buffer<A>, USize, USize, USize, A) -> Unit :=
    (buffer, count, index, end, value) -> [return] => {
        when (index == end) return(());
        if (index < count)
        then poolPutAt<USize, A>(buffer, index, value)
        else poolInitAt<USize, A>(buffer, index, value);
        return(_fillFrom<A>(buffer, count, index + 1usize, end, value));
    };
```

どちらもslotごとにvalueを一回`Share`するので、lifecycle上の差はない。差はcall回数と、占有状態の分岐をloopの外へ出せるかである。

## Host境界とSymbol

```mal
from<A> :: (Address, USize, USize) -> Buffer<A> := (address, offset, length) -> {
    buffer := makePool<USize, A>(0usize, length);
    poolLoad<USize, A>(buffer, 0usize, address, offset, length);
    poolSetState<USize, A>(buffer, length);
    buffer;
};

into<A> :: (Buffer<A>, Address, USize, USize) -> Unit := (buffer, address, offset, length) ->
    poolStore<USize, A>(buffer, offset, length, address);

snapshot :: Buffer<UInt8> -> Symbol := (buffer) ->
    poolSymbol<USize>(buffer, 0usize, poolState<USize, UInt8>(buffer));

bytes :: Symbol -> Buffer<UInt8> := (symbol) -> {
    buffer := makePool<USize, UInt8>(0usize, #symbol);
    poolLoadSymbol<USize>(buffer, 0usize, symbol);
    poolSetState<USize, UInt8>(buffer, #symbol);
    buffer;
};
```

`from`と`into`の`Representable(A)` requirementは、primitiveのrequirementからgeneric bindingへ導かれる。`snapshot`と`bytes`は
prefix `*`の二方向に当たる。

## 現行Bufferとの差分

- 各operationの意味、評価順、alias、trap条件は変えない。trapのmessageはruntimeではなくBuffer fileが決める。
- growth policy、count、invariantはruntimeからこのfileへ移り、runtimeはPool primitiveと上のrange primitiveだけを持つ。
- 現行runtimeはBuffer storageをSymbolと同じbyte ownerで持つため、`*symbol`でstorageを共有できる。`Pool<State, UInt8>`は
  [canonical layout](lifecycle-contract.md#runtime-representation)のbyte列を持つので、slot storageをbyte ownerにすれば共有を保てる。
- C runtimeの`mal_runtime_buffer_from_arguments`は、`main`へ渡す`Buffer<Symbol>`をPool representationとState=countで構築する。
  これはentry ABIがこのfileのrepresentation選択へ依存することを意味する。
- predefined名、prefix `#`と`*`、receiver-first形をpreludeのmal定義へ結ぶ規則が新たに必要になる。
