# Pool APIとprimitive一覧

Status: Exploratory support document

この文書は、PoolとBufferの責務分担と、Pool案が仮定するprimitiveの一覧を管理する。slot primitiveのpreconditionは
[lifecycle contract](lifecycle-contract.md#未検査precondition)、run primitiveのpreconditionと意味は
[run protocol](run-protocol.md#poolのrun-primitive)、所有権の効果は[所有権primitive](ownership-primitives.md)を正とする。名前は`pool`接頭辞を外した仮のものであり、Poolの名前を`require`した
fileだけへ導入する規則と合わせて決める。

## PoolとBufferの責務

memory操作は、抽象する単位で三つの語彙に分かれる。

| 語彙 | 抽象する単位 | 操作 | 使える型 |
|---|---|---|---|
| slot | 一つのcoordinateのLiveまたはVacant | `init`、`take`、`get`、`reserve`、State | Poolだけ |
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
| `putAt`、`dropAt` | 派生 | `take`と`init`、`take`と破棄 |

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
