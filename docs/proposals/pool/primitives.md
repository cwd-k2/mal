# Pool APIとprimitive一覧

Status: Exploratory support document

この文書は、PoolとBufferの責務分担と、Pool案が仮定するprimitiveの一覧を管理する。slot primitiveのpreconditionは
[lifecycle contract](lifecycle-contract.md#未検査precondition)、run primitiveのpreconditionと意味は
[run protocol](run-protocol.md#poolのrun-primitive)、所有権の効果は[所有権primitive](ownership-primitives.md)を正とする。
名前は仮のものであり、Poolの名前を`require`したfileだけへ導入する規則と合わせて決める。

## 層

Pool案のmemoryは次の層からなる。

- Engram lifecycle：`Share`、`Consume`、`Drop`と回収により、すべてのmanaged valueの寿命を自動で管理する。Poolを使わない
  `Symbol`、closure environment、productとsumもここに属する。
- Pool：malが所有する可変storageのmodelである。identity、State、slotを持ち、各slotのLiveとVacantは使う側が管理する。
- run protocol：連続したcoordinate範囲をまとめて扱う語彙である。
- Buffer：Pool上の、占有状態とcapacityを自動で管理するsequenceであり、一つのLiveなrun `[0, count)`だけを持つ。
- `Symbol`：不変のrunであり、`Buffer<UInt8>`とstorageを共有して相互に変換できる。
- `Address`：hostのstorageであり、run protocolを通してだけ交換する。

PoolとBufferはどちらもEngramであり、寿命の管理に差はない。Bufferが自動で管理するのは、slotの占有状態とcapacityである。

## coordinateの線形性

Poolのslotは`0`から`capacity - 1`までの`USize` coordinateで選び、coordinate空間は順序を持ち途中に抜けがない。この線形性により
`[offset, offset + length)`という区間、つまりrunが意味を持つ。runの意味はPool storageの物理配置に依存しない。実装は
`Representable`な要素をcanonical layoutで連続に置くことを選べ、その場合runの操作はbulk copyになる。

coordinate空間が線形でも、占有状態には穴があり得る。runを読む操作が全coordinateのLiveを要求するのはそのためである。Bufferは
Liveなcoordinateの集合が0から始まる一つの区間であることをinvariantにする。他のcontainerがこの線形空間をどう使うかは
[Pool上のcontainer](containers.md)で比較する。[key](pool-keys.md)は順序ではなくidentityでslotを指すため、runを作らない。

## PoolとBufferの責務

memory操作は、抽象する単位で三つの語彙に分かれる。

| 語彙 | 抽象する単位 | 操作 | 使える型 |
|---|---|---|---|
| slot | 一つのcoordinateのLiveまたはVacant | `initAt`、`takeAt`、`getAt`、`reserve`、State | Poolだけ |
| run | 連続したcoordinate範囲 | `from`、`into`、`copy`、`fill` | run protocolを実装したcontainer |
| sequence | 一つのLiveなrun `[0, count)` | `make`、`new`、`#`、growth policy | Bufferだけ |

Poolはslotを、Bufferは一つのLiveなrunを抽象する。PoolはBufferの意味を定義できるが、一般の利用者へ見せる安全な語彙を持たず、
Bufferはslotを空ける語彙を持たない。

| | slotの語彙 | runの語彙 |
|---|---|---|
| Pool | すべて使える | primitiveとして使え、runの状態は実装者が保証する |
| Buffer | 使えない | run protocolとして使え、invariantが条件を保証する |

Bufferは言語の組み込み型ではなく、Pool上のpreludeのopaque型であり、sequenceの語彙とrun protocolの実装を持つ。
`Buffer<UInt8>`と`Symbol`の`*`だけはBufferに固有の操作である。

## 区分

各primitiveを次のどれかに分類する。

- 核：所有権を動かす遷移。他の操作はこれらへ分解できる。
- 必須：malの他の操作では表せない。
- 維持：意味は他の操作へ分解できるが、分解するとstorageの共有を壊すためprimitiveに残す。
- 性能：他の操作のloopで書けるが、call数や分岐を減らすためprimitiveにする。
- 派生：他のprimitiveで書く通常の関数。性能のためprimitiveにしてもよく、その場合も意味は分解と同じである。

## slot primitive

```mal
Pool<State, T>

makePool<State, T> :: (State, USize) -> Pool<State, T>;
state<State, T> :: Pool<State, T> -> State;
setState<State, T> :: (Pool<State, T>, State) -> Unit;
capacity<State, T> :: Pool<State, T> -> USize;
reserve<State, T> :: (Pool<State, T>, USize) -> Unit;
isLive<State, T> :: (Pool<State, T>, USize) -> Bool;
initAt<State, T> :: (Pool<State, T>, USize, T) -> Unit;
takeAt<State, T> :: (Pool<State, T>, USize) -> T;
getAt<State, T> :: (Pool<State, T>, USize) -> T;
putAt<State, T> :: (Pool<State, T>, USize, T) -> Unit;
dropAt<State, T> :: (Pool<State, T>, USize) -> Unit;
```

| primitive | 区分 | 役割 |
|---|---|---|
| `makePool` | 必須 | 初期StateとlogicalなcapacityでPoolを作る |
| `state`、`setState` | 必須 | 共有identityのStateを読み、置き換える |
| `capacity`、`reserve` | 必須 | logical capacityを読み、増やす。Poolは自動でgrowthしない |
| `isLive` | 必須 | coordinateの占有状態を読む |
| `initAt`、`takeAt` | 核 | Vacant → Live、Live → Vacant |
| `getAt` | 維持 | Liveなslotの値を`Share`する |
| `putAt`、`dropAt` | 派生 | `takeAt`して`initAt`、`takeAt`して結果を捨てる |

Poolの形成は`Storable(State)`と`Storable(T)`を要求する。Poolのcopyは同じState、capacity、slotを持つidentityを共有する。

## run primitive

```mal
load<State, T> :: (Pool<State, T>, USize, Address, USize, USize) -> Unit;
store<State, T> :: (Pool<State, T>, USize, USize, Address) -> Unit;
writeRange<State, T> :: (Pool<State, T>, USize, USize, T) -> Unit;
copyRange<State, T> :: (Pool<State, T>, USize, Pool<State, T>, USize, USize) -> Unit;
```

| primitive | 引数 | 区分 | 理由 |
|---|---|---|---|
| `load` | `(pool, offset, address, hostOffset, length)` | 必須 | malはAddressを読めない |
| `store` | `(pool, offset, length, address)` | 必須 | malはAddressへ書けない |
| `writeRange` | `(pool, offset, length, value)` | 性能 | slot操作のloopで書ける |
| `copyRange` | `(destination, destinationOffset, source, sourceOffset, length)` | 性能 | loopで書ける |

## Symbol primitive

byte Poolと`Symbol`の変換は、`Buffer<UInt8>`の`*`の実装に使う。

```mal
symbol<State> :: (Pool<State, UInt8>, USize, USize) -> Symbol;
loadSymbol<State> :: (Pool<State, UInt8>, USize, Symbol) -> Unit;
```

| primitive | 区分 | 役割 |
|---|---|---|
| `symbol` | 必須 | Liveなrunの`Symbol`を作る。bytes列から`Symbol`を作る操作がmalにない |
| `loadSymbol` | 性能 | `offset + #symbol <= capacity`のrunへ`Symbol`のbytesを書く。`symbol # index`のloopでも書ける |

どちらもbyte Poolのstorageを`Symbol`と共有してよく、書き込みはcopy-on-writeにする
（[representation](lifecycle-contract.md#runtime-representation)）。

## 制御

```mal
trap :: Symbol -> [];
```

[primitive `trap`案](../primitive-trap.md)のprimitiveであり、Bufferなどのcontainerが現行runtimeと同じoverflow trapをmalで
起こすのに使う。Pool案はこの採択に依存する。

## core外の拡張

次はcore Pool APIに含めず、所有する文書で扱う。

- `writableSuccessor`と`ValuePool`の更新操作：[identity](identity.md#valuepool)
- `SlotKey`、`PoolKey`、`Arena`：[Pool key extension](pool-keys.md)
