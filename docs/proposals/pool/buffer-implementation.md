# IxPool上のBuffer実装

Status: Exploratory example

この文書は、[AddressとBuffer](../../spec/memory.md)が定める`Buffer<A>`の全operationを、core IxPool APIと少数のcompiler primitiveで
実装した擬似codeを示す。IxPool primitiveの規則は[lifecycle contract](lifecycle-contract.md)、`from`、`into`、`copy`、`fill`の
familyと意味は[run protocol](run-protocol.md)を正とする。

predefinedな名前`make`、`new`、`get`、prefix `#`と`*`、receiver-first形を通常のmal fileへ結ぶ規則は本書の対象外である。
以下はそのfileがpreludeとしてこれらの名前を定義できると仮定する。

## 使うprimitive

[primitive一覧](primitives.md)のslot、run、Symbol、制御primitiveを使う。`trap`は現行runtimeと同じoverflow trapを
malで起こすため、`symbol`と`loadSymbol`は`Buffer<UInt8>`の`*`のために使う。

## 表現とinvariant

```mal
opaque Buffer<A> :: IxPool<USize, A>;
```

Stateはcountである。`[0, count)`がLive、`[count, capacity)`がVacantであり、`count <= capacity`を保つ。
公開operationは利用者がBufferの[未検査precondition](../../spec/memory.md#未検査precondition)を満たす限り、このinvariantから
IxPool preconditionを導く。利用者が違反した場合はinvariantが壊れ得るが、結果を保証しない点は現行Bufferと同じである。

## 補助

```mal
_maxUSize :: USize := -1usize;

_rangeEnd :: (USize, USize) -> USize := (offset, length) -> {
    when (length > _maxUSize - offset) trap("Buffer range overflow")[];
    offset + length;
};

_ensureCapacity<A> :: (Buffer<A>, USize) -> Unit := (buffer, required) -> {
    current := capacity<USize, A>(buffer);
    when (required > current) {
        doubled := if (current > _maxUSize / 2usize) then _maxUSize else current * 2usize;
        reserve<USize, A>(buffer, if (doubled < required) then required else doubled);
    };
};

_extendCount<A> :: (Buffer<A>, USize) -> Unit := (buffer, end) ->
    when (end > state<USize, A>(buffer)) setState<USize, A>(buffer, end);
```

`_rangeEnd`はcountとrange末尾を表現できない場合の現行trapを再現する。allocation byte数を表現できない場合とallocation failureは
`reserve`がtrapする。growth policyはこのfileが所有する。現行runtimeは必要byte数を16以上の2の累乗へ丸めるが、
このfileはelement数で倍増し、byte単位の丸めは`capacity`から観測できない`reserve`内部の選択として残す。

## 要素operation

```mal
make<A> :: USize -> Buffer<A> := (capacity) -> makeIxPool<USize, A>(0usize, capacity);

length<A> :: Buffer<A> -> USize := (buffer) -> state<USize, A>(buffer);

new<A> :: (Buffer<A>, A) -> USize := (buffer, value) -> {
    count := state<USize, A>(buffer);
    end := _rangeEnd(count, 1usize);
    _ensureCapacity<A>(buffer, end);
    initAt<USize, A>(buffer, count, value);
    setState<USize, A>(buffer, end);
    count;
};

get<A> :: (Buffer<A>, USize) -> A := (buffer, index) -> getAt<USize, A>(buffer, index);

put<A> :: (Buffer<A>, USize, A) -> Unit := (buffer, index, value) ->
    putAt<USize, A>(buffer, index, value);
```

`get`と`put`はBufferの`index < #buffer`をinvariantでLive slotへ写すだけで、検査を追加しない。`new`の`count`はinvariantにより
Vacantであり、`_ensureCapacity`の後はcapacity内にある。

## Range operation

Bufferは[run protocol](run-protocol.md)のfamilyを実装する。runの公開preconditionはcountを単位とし、書いたrunの末尾までcountを延ばす。

```mal
fill<Buffer<E>, E> :: (Buffer<E>, USize, USize, E) -> Unit := (buffer, offset, length, value) -> {
    end := _rangeEnd(offset, length);
    _ensureCapacity<E>(buffer, end);
    writeRange<USize, E>(buffer, offset, length, value);
    _extendCount<E>(buffer, end);
};

copy<Buffer<A>> :: (Buffer<A>, USize, Buffer<A>, USize, USize) -> Unit :=
    (destination, destinationOffset, source, sourceOffset, length) -> {
        end := _rangeEnd(destinationOffset, length);
        _ensureCapacity<A>(destination, end);
        copyRange<USize, A>(destination, destinationOffset, source, sourceOffset, length);
        _extendCount<A>(destination, end);
    };
```

公開preconditionの`offset <= #buffer`により、destination rangeはLiveな部分とそれに続くVacantな部分からなり、穴を作らない。
`copy`の`sourceOffset + length <= #source`はsource rangeが全てLiveであることを与える。`_ensureCapacity`がsourceと同じIxPoolを
relocateしてもcoordinateは変わらない。countの更新はprimitiveの後に行い、その間にmal codeは走らない。

`writeRange`が性能だけのためのprimitiveであることは、同じ遷移をIxPool callで書けることで分かる。

```mal
_fillFrom<A> :: (Buffer<A>, USize, USize, USize, A) -> Unit :=
    (buffer, count, index, end, value) -> [return] => {
        when (index == end) return(());
        if (index < count)
        then putAt<USize, A>(buffer, index, value)
        else initAt<USize, A>(buffer, index, value);
        return(_fillFrom<A>(buffer, count, index + 1usize, end, value));
    };
```

どちらもslotごとにvalueを一回`Share`するので、lifecycle上の差はない。差はcall回数と、占有状態の分岐をloopの外へ出せるかである。

## Host境界とSymbol

```mal
from<Buffer<A>> :: (Address, USize, USize) -> Buffer<A> := (address, offset, length) -> {
    buffer := makeIxPool<USize, A>(0usize, length);
    load<USize, A>(buffer, 0usize, address, offset, length);
    setState<USize, A>(buffer, length);
    buffer;
};

into<Buffer<A>> :: (Buffer<A>, Address, USize, USize) -> Unit := (buffer, address, offset, length) ->
    store<USize, A>(buffer, offset, length, address);

snapshot :: Buffer<UInt8> -> Symbol := (buffer) ->
    symbol<USize>(buffer, 0usize, state<USize, UInt8>(buffer));

bytes :: Symbol -> Buffer<UInt8> := (symbol) -> {
    buffer := makeIxPool<USize, UInt8>(0usize, #symbol);
    loadSymbol<USize>(buffer, 0usize, symbol);
    setState<USize, UInt8>(buffer, #symbol);
    buffer;
};
```

`from`と`into`はgeneric implementationで`Representable(A)`を要求するため、それをrequirementとして伝播させる規則が要る
（[run protocol](run-protocol.md#未決定事項)）。`snapshot`と`bytes`はprefix `*`の二方向に当たり、run protocolに含めない
`Buffer<UInt8>`だけの操作である。

## 現行Bufferとの差分

- 各operationの意味、評価順、alias、trap条件は変えない。trapのmessageはruntimeではなくBuffer fileが決める。
- growth policy、count、invariantはruntimeからこのfileへ移り、runtimeはIxPool primitiveとrun primitiveだけを持つ。
- 現行runtimeはBuffer storageをSymbolと同じbyte ownerで持つため、`*symbol`でstorageを共有できる。`IxPool<State, UInt8>`は
  [canonical layout](lifecycle-contract.md#runtime-representation)のbyte列を持つので、slot storageをbyte ownerにすれば共有を保てる。
- C runtimeの`mal_runtime_buffer_from_arguments`は、`main`へ渡す`Buffer<Symbol>`をIxPool representationとState=countで構築する。
  これはentry ABIがこのfileのrepresentation選択へ依存することを意味する。
- predefined名、prefix `#`と`*`、receiver-first形をpreludeのmal定義へ結ぶ規則が新たに必要になる。
- `from`はcontainer型をkeyとするfamilyになるため、`from<UInt8>(...)`の明示形は`from<Buffer<UInt8>>(...)`になる。期待result型が
  `Buffer<UInt8>`なら現行どおり型argumentを省ける。
