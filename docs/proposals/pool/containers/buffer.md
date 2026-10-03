# IxPool上のBuffer実装

Status: Exploratory example

この文書は、[AddressとBuffer](../../../spec/memory.md)が定める`Buffer<A>`のsequence operationを、IxPoolの核と周辺operation、
Vectorで実装した擬似codeを示す。host operationはVectorを正規形として意味を分解できるが、現行Buffer APIから除くかは
[BufferとVector](../api/buffer-vector.md#buffer)が別に管理する。IxPool primitiveの規則は
[runtime contract](../runtime/contract.md)、Bufferの各operationの意味は[AddressとBuffer](../../../spec/memory.md)を正とする。

以下は、このfileがpreludeとしてpredefinedな名前`make`、`new`、`get`、prefix `#`と`*`、receiver-first形を定義できると仮定する。

## 使うprimitive

[Pool primitive](../api/pool.md)のIxPoolの核と周辺operation、Vectorのprimitiveと、現行runtimeと同じoverflow trapをmalで起こすための
`trap`だけを使う。

## 表現とinvariant

```mal
opaque Buffer<A> :: IxPool<USize, A>;
```

Metaはcountである。`[0, count)`がLive、`[count, capacity)`がVacantであり、`count <= capacity`を保つ。
公開operationは利用者がBufferの[未検査precondition](../../../spec/memory.md#未検査precondition)を満たす限り、このinvariantから
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
        target := if (doubled < required) then required else doubled;
        grow<USize, A>(buffer, target - current);
    };
};

_extendCount<A> :: (Buffer<A>, USize) -> Unit := (buffer, end) ->
    when (end > meta<USize, A>(buffer)) setMeta<USize, A>(buffer, end);
```

`_rangeEnd`はcountとrange末尾を表現できない場合の現行trapを再現する。allocation byte数を表現できない場合とallocation failureは
`grow`がtrapする。growth policyはこのfileが所有する。現行runtimeは必要byte数を16以上の2の累乗へ丸めるが、
このfileはelement数で倍増し、byte単位の丸めは`capacity`から観測できない`grow`内部の選択として残す。

## 要素operation

```mal
make<A> :: USize -> Buffer<A> := (capacity) -> {
    buffer :: Buffer<A> := pool<USize, A>(0usize);
    grow<USize, A>(buffer, capacity);
    buffer;
};

length<A> :: Buffer<A> -> USize := (buffer) -> meta<USize, A>(buffer);

new<A> :: (Buffer<A>, A) -> USize := (buffer, value) -> {
    count := meta<USize, A>(buffer);
    end := _rangeEnd(count, 1usize);
    _ensureCapacity<A>(buffer, end);
    initAt<USize, A>(buffer, count, value);
    setMeta<USize, A>(buffer, end);
    count;
};

get<A> :: (Buffer<A>, USize) -> A := (buffer, index) -> getAt<USize, A>(buffer, index);

put<A> :: (Buffer<A>, USize, A) -> Unit := (buffer, index, value) ->
    putAt<USize, A>(buffer, index, value);
```

`get`と`put`はBufferの`index < #buffer`をinvariantでLive slotへ写すだけで、検査を追加しない。`new`の`count`はinvariantにより
Vacantであり、`_ensureCapacity`の後はcapacity内にある。

## Range operation

`fill`と`copy`の意味は、slot操作のloopで定める。runtimeは同じ結果になる一括処理で実装してよい。公開preconditionはcountを単位とし、
書いたrunの末尾までcountを延ばす。

```mal
_write<A> :: (Buffer<A>, USize, USize, A) -> Unit := (buffer, count, index, value) ->
    if (index < count)
    then putAt<USize, A>(buffer, index, value)
    else initAt<USize, A>(buffer, index, value);

_fillFrom<A> :: (Buffer<A>, USize, USize, USize, A) -> Unit :=
    (buffer, count, index, end, value) -> [return] => {
        when (index == end) return(());
        _write<A>(buffer, count, index, value);
        return(_fillFrom<A>(buffer, count, index + 1usize, end, value));
    };

fill<A> :: (Buffer<A>, USize, USize, A) -> Unit := (buffer, offset, length, value) -> {
    end := _rangeEnd(offset, length);
    _ensureCapacity<A>(buffer, end);
    _fillFrom<A>(buffer, meta<USize, A>(buffer), offset, end, value);
    _extendCount<A>(buffer, end);
};

_copyUp<A> :: (Buffer<A>, USize, USize, Buffer<A>, USize, USize, USize) -> Unit :=
    (destination, count, destinationOffset, source, sourceOffset, index, length) -> [return] => {
        when (index == length) return(());
        value := getAt<USize, A>(source, sourceOffset + index);
        _write<A>(destination, count, destinationOffset + index, value);
        return(_copyUp<A>(destination, count, destinationOffset, source, sourceOffset, index + 1usize, length));
    };

_copyDown<A> :: (Buffer<A>, USize, USize, Buffer<A>, USize, USize) -> Unit :=
    (destination, count, destinationOffset, source, sourceOffset, remaining) -> [return] => {
        when (remaining == 0usize) return(());
        index := remaining - 1usize;
        value := getAt<USize, A>(source, sourceOffset + index);
        _write<A>(destination, count, destinationOffset + index, value);
        return(_copyDown<A>(destination, count, destinationOffset, source, sourceOffset, index));
    };

copy<A> :: (Buffer<A>, USize, Buffer<A>, USize, USize) -> Unit :=
    (destination, destinationOffset, source, sourceOffset, length) -> {
        end := _rangeEnd(destinationOffset, length);
        _ensureCapacity<A>(destination, end);
        count := meta<USize, A>(destination);
        if (destinationOffset <= sourceOffset)
        then _copyUp<A>(destination, count, destinationOffset, source, sourceOffset, 0usize, length)
        else _copyDown<A>(destination, count, destinationOffset, source, sourceOffset, length);
        _extendCount<A>(destination, end);
    };
```

公開preconditionの`offset <= #buffer`により、destination rangeはLiveな部分とそれに続くVacantな部分からなり、穴を作らない。
`copy`の`sourceOffset + length <= #source`はsource rangeが全てLiveであることを与える。`_ensureCapacity`がsourceと同じIxPoolを
relocateしてもcoordinateは変わらない。countの更新はloopの後に行い、その間に利用者のcodeは走らない。

malはsourceとdestinationが同じidentityかを知れないため、`copy`はoffsetの大小だけでloopの向きを選ぶ。destinationが前にあれば
昇順、後ろにあれば降順に写すと、同じBufferで範囲が重なっても、まだ読んでいないsourceの要素を先に上書きしない。別のBufferなら
どちらの向きでも結果は同じである。どちらのoperationもslotごとに値を一回`Share`し、置き換えたLiveな値を一回`Drop`する。

## Symbolとの変換

hostとの交換はVectorを正規形として説明でき、現行Bufferの`from`と`into`はそのcompatibility operationとして実装できる
（[BufferとVector](../api/buffer-vector.md#buffer)）。`Symbol`との変換は、BufferとVectorを同じpreludeのfileで定義すると仮定し、
`freeze`したIxPoolをそのままVectorとして扱って書く。
型、意味、preconditionは現行の[Symbol conversion](../../../spec/memory.md#symbol-conversion)のままである。

```mal
_toSymbol :: Buffer<UInt8> -> Symbol := (buffer) -> symbol(freeze<USize, UInt8>(buffer));
```

`*buffer`はbyte列の`freeze`に当たる`_toSymbol`であり、`*symbol`はbyte列の`thaw`に当たり、`symbol # index`を`new`で積むloopで
書ける。as-ifで実装するBufferとVectorは、`freeze`と`thaw`のstorageの移動で、組み立ててから変換する用途を現行と同じ費用に実装できる。

## 現行Bufferとの差分

- elementの型形成は[D096](../../../history/decisions/active/D096.md)でplace lifecycleとしての`Storable`へ揃い、
  `Buffer<Buffer<T>>`を認めた。Pool採択時には`Buffer<IxPool<M, V>>`と`Buffer<ImPool<M, V>>`を同じ規則へ加える。external opaque
  carrierはruntime-value Trivial storageの後続decisionに分ける。handle elementの`get`、`fill`、`copy`は同じidentityへのauthorityを
  保存するshallowなcarrier operationであり、functionはclosure cycleのため引き続き除外する。
- `from`と`into`の意味はVectorのadmissionとobservationから導ける。既存名を残すかは互換性のdecisionとし、他のoperationの意味、
  評価順、alias、trap条件は変えない。trapのmessageはruntimeではなくBuffer fileが決める。
- growth policy、count、invariantはruntimeからこのfileへ移る。runtimeはIxPoolとVectorのprimitiveを持つ。
- 現行runtimeは小容量の初期Buffer storageをobjectへinline化し、それ以外をSymbolと同じ形のbyte ownerで持つが、`*`の意味は
  物理表現によらず独立したsnapshotである。inline storageはcopyし、flat ownerはlast useと一意性を確認できたときだけ移す。
  `IxPool<Meta, UInt8>`もcanonical byte列を持てるため、last useとruntime上の一意性を証明できる場合だけownerを移す。同時にliveな
  Buffer aliasとSymbolまたはVectorの間でwritable storageを共有することは要求しない。
- `*`と、`main`へ渡す`Buffer<Symbol>`を構築するC runtimeの`mal_runtime_buffer_from_arguments`は、このfileの
  representation選択とMeta=countの意味へ依存する。representationを変えるときはruntimeも合わせて変える。
- predefined名、prefix `#`と`*`、receiver-first形をpreludeのmal定義へ結ぶ規則が新たに必要になる。

公開operationと未検査preconditionを保つため、`managed-bytes`、`canonical-memory`、`indexed-graph`など`Buffer`を使う
[`examples/`](../../../../examples)のsourceは変わらない。変わるのは各operationの実装場所とcostである。

| operation | 現行Buffer | IxPool上の実装 |
|---|---|---|
| `get`、`put` | 範囲を検査しない | 範囲も占有状態も検査しない |
| `new` | runtimeがgrowthを決める | Buffer fileが`grow`とgrowth policyを呼ぶ |
| `fill`、`copy` | runtimeのloopとretain callback | IxPool callのloop、またはruntimeの一括処理とshare callback |
| `from`、`into` | runtimeのbulk copy | Vectorのadmissionとobservationに対応するcompatibility operation。中間snapshotを作らず一段にできる |
| `*` | runtimeのbulk copy | Vectorとの`freeze`と`thaw`を経由する。Bufferがlast useで区別可能なaliasがなければstorageを移す |
| managed elementの`put` | Borrowしてruntimeがretain | 一時値とlast useは`Consume` |
| 破棄 | `[0, count)`をrelease | 占有tagを走査してLive slotをDrop |

`indexed-graph`のDijkstraが`fill`で初期化する距離表のように、unmanagedな要素の`fill`はIxPool callのloopにすると要素数だけcallが
増える。現行runtimeのloopと同等にするには、runtime representationとcanonical layoutが一致する型の一括処理が要る。
