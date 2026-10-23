# pointerとIxPoolのAPI

Status: Exploratory support document; rebased on mal v0.7

この文書は、[意味論](../model/semantics.md)をsource surfaceへ写すworking proposalである。`*A`はCの
spellingを借りるが、数値address、arithmetic、castを持たないmanaged pointerである。

## pointer type

```mal
read<A> :: *A -> A := p -> load(p);
```

`*A`はAを置ける一つのshared placeへのpointer型である。type grammarではpointerをatomic type constructorとして扱う。

```text
atomicType ::= ... | "*" atomicType

*A -> B       pointer Aを受け取るfunction
*(A -> B)     functionをpointeeにするpointer。初回案ではStorable制約により形成できない
**A           pointerをpointeeにするpointer
```

初回案では`*A`の形成に`Storable(A)`を要求する。pointer carrier自体も`Storable(*A)`であり、別のplaceや
IxPoolへ保存できる。`*`を追加するのはtype grammarだけで、pointer用のprefix expressionは追加しない。

## pointer operation

```mal
vacant<A> :: () -> *A;
place<A>  :: A -> *A;
isLive<A> :: *A -> Bool;
load<A>   :: *A -> A;
take<A>   :: *A -> A;
store<A>  :: (*A, A) -> Unit;
```

| operation | 意味 | precondition |
|---|---|---|
| `vacant()` | Vacantな新しいplaceへのpointerを返す | なし |
| `place(a)` | `a`をStoreして保持する新しいplaceへのpointerを返す | なし |
| `isLive(p)` | pointeeがLiveか返す | なし |
| `load(p)` | pointeeをShareして返す | Live |
| `take(p)` | pointeeをMoveして返し、Vacantにする | Live |
| `store(p, a)` | `a`をStoreし、旧値があればDropする | なし |

pointerは通常のvalueであり、評価位置へ現れただけではdereferenceしない。

```mal
q := p;              // pointer copy。q :: *A
value := load(p);    // Share read。value :: A
moved := take(p);    // Move read。pointeeはVacantになる
store(p, value);     // Vacantなら初期化、Liveなら置換
```

implicit dereference、address-of、general lvalue、assignment operatorは導入しない。pointer expressionとvalue expressionは
通常のfunction application規則によりsource順に一度ずつ評価される。

`Slot<A>`のような`Vacant | Live(A)`値も公開しない。状態は`isLive`、copying readは`load`、consuming readは
`take`、writeは`store`へ分ける。

次は意味を追加しない周辺operationとして定義できる。

```mal
clear<A> :: *A -> Unit := p -> { take(p); () };
transfer<A> :: (*A, *A) -> Unit := (from, to) -> store(to, take(from));
```

Live同士の`swap`やVacant専用storeをprimitiveにするかは生成コードを測って決める。`move`はsourceの
pointer operation名にせず、execution ownership上のMoveと、上の明示的な`take` + `store`を区別する。

## IxPool

```mal
IxPool<A>;

pool<A>     :: () -> IxPool<A>;
grow<A>     :: (IxPool<A>, USize) -> Unit;
capacity<A> :: IxPool<A> -> USize;
at<A>       :: (IxPool<A>, USize) -> *A;
```

receiver-first formでは次のように使う。

```mal
p := pool.at(i);                 // p :: *A
value := load(pool.at(i));
store(pool.at(i), value);
```

| operation | 意味 | precondition |
|---|---|---|
| `pool()` | capacity 0の新しいpool identityを返す | なし |
| `grow(p, k)` | capacityを`k`増やし、新coordinateをVacantにする | なし。表現不能sizeとallocation failureはtrap |
| `capacity(p)` | 現在のcoordinate数を返す | なし |
| `at(p, i)` | coordinate `i`のplaceへのpointerを返す | `i < capacity(p)` |

IxPoolは縮まない。`take(pool.at(i))`はcoordinateを削除せずVacantにし、後の`store`で再利用できる。同じpool
identityとindexへの`at`は同じlogical placeを指す。growthで物理配置が変わっても、既に得たpointerは有効である。

IxPoolはcontainer metadataを内蔵しない。長さを持つcontainerの参照構造は次になる。

```mal
opaque Dense<A> :: (*USize, IxPool<A>);
```

metadata pointerと要素poolは二つのmanaged identityになり得る。opaque境界の外から区別できない特殊化では
compiler/runtimeが同じallocationへco-allocateしてよいが、二つのlogical identityとlifecycleは保つ。APIは
co-allocationを前提にしない。

## 最小surfaceで未定義のもの

次はpointer/IxPoolのcore operationでは意味を定めない。将来の追加を禁止する一覧ではなく、別の観測や
authorityを必要とするため、個別に設計する境界である。

- pointerの数値化、arithmetic、ordering、equality
- pointerに対する`+`、`-`、`#`、index syntax
- Cの`&`に相当する、既存bindingやaggregate fieldからpointerを作るoperation
- pointerのimplicit dereference、pointer用prefix expression、store用operator
- `free`、IxPool shrink、coordinateの無効化
- `Header`を型引数に持つIxPool
- `Slot<A>`を返すread/exchange API
- immutable `ImPool`、`freeze`、`thaw`

source上でpointerが指せるplaceは`vacant`、`place`、IxPoolの`at`などstorage primitiveが明示的に作ったものだけで
ある。local bindingそのものをaddressable storageへ変えない。
