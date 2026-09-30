# 既存exampleで見るPool化の差分

Status: Exploratory example

この文書は、現在の[`examples/`](../../../examples/)をPool案へ移したときに、source、ownership、precondition、costの何が
変わるかを示す。現在の動作は[AddressとBuffer](../../spec/memory.md)、Pool primitiveの規則は
[lifecycle contract](lifecycle-contract.md)、Bufferのmal実装は[source sketch](container-examples.md#buffer)を正とする。
Pool側のcodeは未採択の擬似codeである。

## Bufferの利用者

`Buffer<T>`の公開operationと未検査preconditionを保つ限り、`managed-bytes`、`canonical-memory`、`indexed-graph`、
`main :: Buffer<Symbol> -> Int32`を使う各exampleのsourceは変わらない。変わるのは各operationの実装場所とcostである。

`managed-bytes`の次の行は、それぞれ異なる層へ移る。

```mal
bytes := from<UInt8>(address, 0usize, length);
snapshot := *bytes;
alias.put(0usize, 'M');
alias.copy(#alias, *"!", 0usize, 1usize);
bytes.into(address, 0usize, #bytes);
```

- `alias.put(0usize, 'M')`はBuffer fileの`put`から`poolPutAt`になる。利用者は現行どおり`0 < #alias`を満たし、Buffer実装は
  live prefix invariantでslotがLiveであることを導く。
- `alias.copy(#alias, ...)`は現在のcountを越えるrangeを書く。Buffer実装は`reserve`してから、Liveなslotを置換しVacantなslotを
  initするrange primitiveを呼び、最後にStateのcountを更新する。primitiveがなければPool callのloopになる。
- `from`、`*bytes`、`*"!"`、`into`はAddressやSymbolのbytesとPool slotの間の一括copyであり、core Pool APIだけでは書けない。
  必要なprimitiveと、それを使った各operationは[Pool上のBuffer実装](buffer-implementation.md)に示す。

`main`の`Buffer<Symbol>`はC runtimeが`mal_runtime_buffer_from_arguments`でentry前に構築する。Bufferをmalで実装しても、
このruntime関数はPool representationと、Buffer fileが選んだStateの意味（count）を知る必要がある。entry ABIが
opaque Bufferのrepresentationへ依存する点は、Buffer置換の判断で別に扱う。

`indexed-graph`は`_copyBuffer`で`make`と全体`copy`を、Dijkstraのworkspaceで`fill`を使う。

```mal
distances := make<UInt64>(nodeCount);
distances.fill(0usize, nodeCount, infinity);
```

elementがunmanagedなのでShareもDropも起きないが、Pool上の`fill`は`nodeCount`回の`poolInitAt`になる。現行runtimeのloopと
同等にするには、runtime representationとcanonical layoutが一致する型のbulk fast pathが要る。

## Bufferの上に書いたcontainer

[`generic-map`](../../../examples/generic-map/map.mal)は、empty entryを持つsumをBufferの全coordinateへ`fill`して
open addressingを実装している。

```mal
HashEntry<K, V> :: [Unit, (K, V)];
opaque HashMap<K, V> :: Buffer<HashEntry<K, V>>;

hashMap<K, V> :: USize -> HashMap<K, V> := (capacity) -> {
    slots :: HashMap<K, V> := make(capacity);
    slots.fill(0usize, capacity, _emptyEntry());
    slots;
};
```

Poolへ直接置くと、PoolのVacantがempty entryを表し、slotは`(K, V)`だけになる。Stateは使わないので`Unit`にする。

```mal
opaque HashMap<K, V> :: Pool<Unit, (K, V)>;

hashMap<K, V> :: USize -> HashMap<K, V> := (capacity) ->
    makePool<Unit, (K, V)>((), capacity);

_hashMapGetFrom<K, V> :: (HashMap<K, V>, K, USize, USize) -> HashLookup<V> :=
    (slots, key, index, remaining) -> [missing, found] => {
        when (remaining == 0usize) missing();
        when (!poolIsLive<Unit, (K, V)>(slots, index)) missing();
        (storedKey, storedValue) := poolGetAt<Unit, (K, V)>(slots, index);
        when (equal(storedKey, key)) found(storedValue);
        slots._hashMapGetFrom(
            key,
            (index + 1usize) % poolCapacity<Unit, (K, V)>(slots),
            remaining - 1usize
        )[missing, found];
    };

_hashMapPutFrom<K, V> :: (HashMap<K, V>, K, V, USize, USize) -> Bool :=
    (slots, key, value, index, remaining) -> [return] => {
        when (remaining == 0usize) return(false);
        when (!poolIsLive<Unit, (K, V)>(slots, index)) {
            poolInitAt<Unit, (K, V)>(slots, index, (key, value));
            return(true);
        };
        (storedKey, _) := poolGetAt<Unit, (K, V)>(slots, index);
        when (equal(storedKey, key)) {
            poolPutAt<Unit, (K, V)>(slots, index, (key, value));
            return(true);
        };
        return(slots._hashMapPutFrom(
            key,
            value,
            (index + 1usize) % poolCapacity<Unit, (K, V)>(slots),
            remaining - 1usize
        ));
    };
```

`hashMapGet`と`hashMapPut`は`#slots`を`poolCapacity`へ置き換えるだけで、capacity 0の早期returnを含めて現行と同じである。
差分は次のとおりである。

- 構築は`capacity`個のempty entryを書かず、占有metadataの初期化だけになる。
- lookupは`get`の結果をsumで分岐する代わりに、`poolIsLive`で分岐してからLive slotだけを読む。
- 空bucketへの挿入は`poolInitAt`、同じkeyの置換は`poolPutAt`になり、Bufferの`put`一つが二つの遷移に分かれる。
- `(key, value)`はcall内で作った一時値なので`Consume`でslotへ移る。現行はentry構築でkeyとvalueをShareした後、
  Bufferの`put`がBorrowしたentryをruntimeがもう一度retainし、一時entryをreleaseする。`K = Symbol`では挿入ごとに
  retainとreleaseの往復が一組減る。
- keyの比較のために`poolGetAt`が`(K, V)`全体をShareする点は、現行の`slots.get(index)`と同じである。

deletionとresizeは現行exampleと同じく省略する。resize自体は現行Bufferでも`new`で伸ばして書き直せるが、Pool上では
[Mapのrehash](container-examples.md#map)のように`poolTakeAt`でentryを移し、K、VのShareとDropを起こさずに済む。

## preconditionの責任

同じ範囲外accessでも、どのpreconditionが破れたかで責任の所在が変わる。

利用者が公開preconditionに違反する例では、結果は現行Bufferと同じく保証されない。

```mal
bytes.get(#bytes);
```

`index == #bytes`はBufferの`index < #buffer`に違反し、Buffer実装はそのまま`poolGetAt`をVacant slotへ呼ぶ。Poolは検査しないので、
現在の未検査preconditionと同じ扱いになる。

利用者が公開preconditionを守っても、container実装の誤りでPool preconditionへ違反し得る。例えば`new`が`count == capacity`での
`reserve`を忘れると、`poolInitAt(buffer, count, value)`は`index < poolCapacity`に違反する。これはBuffer実装の誤りであり、
占有状態を検査するtest用runtimeで検出する対象である。

HashMapは公開operationにpreconditionを持たないため、すべてのPool preconditionを実装が満たす。上のcodeでは、probe coordinateが
`% poolCapacity`で範囲内になり、`poolGetAt`と`poolPutAt`は`poolIsLive`がtrueの直後、`poolInitAt`はfalseの直後だけに呼ぶ。
capacity 0の早期returnは剰余のpreconditionのためにも必要であり、Poolが代わりに検査することはない。

## costの比較

| operation | 現行Buffer | Pool上の実装 |
|---|---|---|
| `get`、`put` | 範囲を検査しない | 範囲も占有状態も検査しない |
| `new` | runtimeがgrowthを決める | Buffer fileが`reserve`とgrowth policyを呼ぶ |
| `fill`、`copy` | runtimeのloopとretain callback | range primitiveのloopとshare callback、またはPool callのloop |
| `from`、`into`、`*` | runtimeのbulk copy | host境界とSymbol用のcompiler primitive |
| HashMapの構築 | capacity個のempty entryを`fill` | 占有metadataの初期化 |
| managed elementの挿入 | Borrowしてruntimeがretain | 一時値とlast useは`Consume` |
| 破棄 | `[0, count)`をrelease | 占有metadataを走査してLive slotをDrop |
