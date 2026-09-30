# run protocol

Status: Exploratory support document

この文書は、containerの連続したcoordinate範囲を扱う公開語彙、run protocolを管理する。Poolのslot遷移は
[所有権primitive](ownership-primitives.md)、slot preconditionは[lifecycle contract](lifecycle-contract.md)、Bufferによる実装は
[Buffer実装](buffer-implementation.md)を正とする。

## 三つの語彙

Pool案のmemory操作は、抽象する単位で三つの語彙に分かれる。

| 語彙 | 抽象する単位 | 操作 | 使える型 |
|---|---|---|---|
| slot | 一つのcoordinateのLiveまたはVacant | `init`、`take`、`get`、`reserve`、State | Poolだけ |
| run | 連続したcoordinate範囲 | `from`、`into`、`copy`、`fill` | run protocolを実装したcontainer |
| sequence | 一つのLiveなrun `[0, count)` | `make`、`new`、`#`、growth policy | Bufferだけ |

Poolはslotを、Bufferは一つのLiveなrunを抽象する。PoolはBufferの意味を定義できるが、一般の利用者へ見せる安全な語彙を持たず、
Bufferはslotを空ける語彙を持たない。run protocolは、Poolの仕組みとcontainerごとのinvariantをつなぐ公開語彙である。

## family

run protocolは次のoperation familyからなる。

```mal
from<A> :: (Address, USize, USize) -> A;
into<A> :: (A, Address, USize, USize) -> Unit;
copy<A> :: (A, USize, A, USize, USize) -> Unit;
fill<A, E> :: (A, USize, USize, E) -> Unit;
```

- `from(address, offset, length)`はhostの`length`要素を読み、coordinate `[0, length)`にそれを持つ新しい値を返す。
- `into(value, address, offset, length)`は`[offset, offset + length)`の要素をhostへ書く。
- `copy(destination, destinationOffset, source, sourceOffset, length)`はsourceのrunをdestinationのrunへ写す。
- `fill(value, offset, length, element)`はrunの各coordinateへ`element`を書く。

`copy`と`from`は同じ型の間だけで定義する。`from`のcontainer型はoperandから決まらないため、期待result型から推論し、
それがなければ`from<Buffer<UInt8>>(...)`のように明示する。`fill`だけは要素値をoperandに持つため、要素型`E`をfamily parameterに加え、
containerと要素値の両方から推論する。

## 意味

各operationは、runへslot操作を昇順に適用した場合と同じ意味を持つ。`copy`だけはsourceのrun全体を読んでから書くため、同じ値の
中でrunが重なっても開始時点のsourceを写す。書き込んだcoordinateはそれぞれ独立したresponsibilityを受け取り、`fill`はその数だけ
`element`を`Share`する。実装がbulk copyを使っても、観測できる結果とshare、dropの回数はこの分解と一致する。

`from`と`into`は`Representable`な要素だけを扱い、canonical layoutでhostと交換する。そのためlifecycle glueを呼ばない。
host側の範囲、permission、初期化は現行の`from`、`into`と同じ[未検査precondition](../../spec/memory.md#未検査precondition)である。

どのcoordinateを読めるか、どこへ書けるか、書いた後にcontainerの他のStateがどう変わるかは、各実装がpreconditionと方針として
定める。Bufferは`offset <= #buffer`を要求し、書いたrunの末尾までcountを延ばす。runの操作中にmal codeは実行されない。

## Poolのrun primitive

generic implementationのpatternはfamilyの型parameterしかbinderにできないため、二つのparameterを持つ`Pool<S, T>`は
`into<A>`のような一parameterのfamilyを`S`と`T`について汎用に実装できない。Poolは代わりに、familyの実装に使うrun primitiveを
提供する。

```mal
poolLoad<State, T> :: (Pool<State, T>, USize, Address, USize, USize) -> Unit;
poolStore<State, T> :: (Pool<State, T>, USize, USize, Address) -> Unit;
poolWriteRange<State, T> :: (Pool<State, T>, USize, USize, T) -> Unit;
poolCopyRange<State, T> :: (Pool<State, T>, USize, Pool<State, T>, USize, USize) -> Unit;
```

| primitive | 引数 | precondition | 必要な理由 |
|---|---|---|---|
| `poolLoad` | `(pool, offset, address, hostOffset, length)` | runがcapacity内、host側は`from`と同じ | 必須。malはAddressを読めない |
| `poolStore` | `(pool, offset, length, address)` | runが全てLive、host側は`into`と同じ | 必須。malはAddressへ書けない |
| `poolWriteRange` | `(pool, offset, length, value)` | runがcapacity内 | 性能。slot操作のloopで書ける |
| `poolCopyRange` | `(destination, destinationOffset, source, sourceOffset, length)` | destinationのrunがcapacity内、sourceのrunが全てLive | 性能。loopで書ける |

書き込むcoordinateはLiveになり、VacantならinitしLiveなら新しい値を成立させてから旧値をDropする。`poolCopyRange`は同じPool
identityの重なるrunでも開始時点のsourceを写す。`poolLoad`と`poolStore`は`Representable(T)`を要求し、host側のoffset計算を
表現できない場合は現行の`from`、`into`と同じくtrapする。share、dropの回数は[所有権primitive](ownership-primitives.md#派生operation)に
示す。primitiveの名前は、Poolの名前を`require`したfileだけへ導入する規則と合わせて未決定である。

## `*`

`Buffer<UInt8>`と`Symbol`の変換`*`はrun protocolに含めず、`Buffer<UInt8>`だけの操作とする。byte列に限った変換であり、
他のcontainerへ一般化する実例がないためである。実装はbyte Poolのstorageを`Symbol`と共有する
[representation](lifecycle-contract.md#runtime-representation)を使う。

## 他のcontainer

Buffer以外のcontainerも、自分のinvariantでPoolのrun primitiveの条件を満たせば同じfamilyを実装できる。例えばring bufferの
`into`は、折り返しの前後で二回`poolStore`を呼ぶ。利用者はcontainerの種類に関係なく同じ名前でhostと交換できる。

## 未決定事項

- `Representable(A)`をrequirementとして伝播させる規則。現行の[generics](../../spec/generics.md#requirements)ではgeneric本体が
  型parameterの要素に`from`と`into`を使えず、`from<Buffer<A>>`のgeneric implementationを書けない。`Storable(A)`と同じく
  requirementとしてcallerのspecializationまで持ち上げる必要がある。
- Pool primitiveの名前と、Poolの名前を`require`したfileだけへ導入する規則。
