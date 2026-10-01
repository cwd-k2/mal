# run protocol

Status: Exploratory support document

この文書は、containerの連続したcoordinate範囲を扱う公開語彙、run protocolを管理する。IxPoolのslot遷移は
[所有権primitive](ownership-primitives.md)、slot preconditionは[lifecycle contract](lifecycle-contract.md)、Bufferによる実装は
[Buffer実装](buffer-implementation.md)を正とする。

run protocolは、IxPoolの仕組みとcontainerごとのinvariantをつなぐrunの語彙である。slot、run、sequenceの三つの語彙と
IxPoolとBufferの責務分担は[primitive一覧](primitives.md#ixpoolとbufferの責務)が所有する。

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

## IxPoolのrun primitive

generic implementationのpatternはfamilyの型parameterしかbinderにできないため、二つのparameterを持つ`IxPool<S, T>`は
`into<A>`のような一parameterのfamilyを`S`と`T`について汎用に実装できない。IxPoolは代わりに、familyの実装に使うrun primitiveを
提供する。

run primitiveの型と区分は[primitive一覧](primitives.md#run-primitive)に置き、この文書はpreconditionと意味を所有する。

| primitive | 読むrun | 書くrun |
|---|---|---|
| `load` | hostの範囲。現行の`from`と同じ条件 | IxPoolのrunがcapacity内 |
| `store` | IxPoolのrunが全てLive | hostの範囲。現行の`into`と同じ条件 |
| `writeRange` | なし | IxPoolのrunがcapacity内 |
| `copyRange` | sourceのrunが全てLive | destinationのrunがcapacity内 |

書き込むcoordinateはLiveになり、VacantならinitしLiveなら新しい値を成立させてから旧値をDropする。`copyRange`は同じIxPool
identityの重なるrunでも開始時点のsourceを写す。`load`と`store`は`Representable(T)`を要求し、host側のoffset計算を
表現できない場合は現行の`from`、`into`と同じくtrapする。share、dropの回数は[所有権primitive](ownership-primitives.md#派生operation)に
示す。

## `*`

`Buffer<UInt8>`と`Symbol`の変換`*`はrun protocolに含めず、`Buffer<UInt8>`だけの操作とする。runを写す操作ではなく、
IxPoolとImPoolの間の[freezeとthaw](identity.md#freezeとthaw)をbyte列へ特化した変換だからである。実装はbyte IxPoolのstorageを`Symbol`と共有する
[representation](lifecycle-contract.md#runtime-representation)を使う。

## 他のcontainer

Buffer以外のcontainerも、自分のinvariantでIxPoolのrun primitiveの条件を満たせば同じfamilyを実装できる。例えばring bufferの
`into`は、折り返しの前後で二回`store`を呼ぶ。利用者はcontainerの種類に関係なく同じ名前でhostと交換できる。

## 未決定事項

- `Representable(A)`をrequirementとして伝播させる規則。現行の[generics](../../spec/generics.md#requirements)ではgeneric本体が
  型parameterの要素に`from`と`into`を使えず、`from<Buffer<A>>`のgeneric implementationを書けない。`Storable(A)`と同じく
  requirementとしてcallerのspecializationまで持ち上げる必要がある。
- IxPool primitiveの名前と、IxPoolの名前を`require`したfileだけへ導入する規則。
