# IxPool APIとprimitive一覧

Status: Exploratory support document

この文書は、IxPoolとBufferの責務分担と、Pool案が仮定するprimitiveの一覧を管理する。slot primitiveのpreconditionは
[lifecycle contract](lifecycle-contract.md#未検査precondition)、Bufferのoperationの意味とpreconditionは
[AddressとBuffer](../../spec/memory.md)、所有権の効果は[所有権primitive](ownership-primitives.md)を正とする。
名前は仮のものであり、IxPoolの名前を`require`したfileだけへ導入する規則と合わせて決める。

## 層

Pool案のmemoryは次の層からなる。

- Engram lifecycle：`Share`、`Consume`、`Drop`と回収により、すべてのmanaged valueの寿命を自動で管理する。Poolを使わない
  `Symbol`、closure environment、productとsumもここに属する。
- IxPoolとImPool：malが所有するstorageのprimitiveの対である。IxPoolはidentityを共有してその場で書き換え、ImPoolは
  identityを持たず更新のたびにsuccessorを返す。どちらもState、slot、各slotのLiveとVacantを持つ。
- container：IxPoolまたはImPoolの上にmalで定義する。BufferはIxPool上の、占有状態とcapacityを自動で管理するsequenceであり、
  一つのLiveなrun `[0, count)`だけを持つ。Map、Deque、heap、木も同じくIxPool上にあり、immutable arrayは
  ImPool上にある。
- `Symbol`：不変のrunであり、ImPoolをbyte列に特化した既存の値に当たる。`Buffer<UInt8>`とstorageを共有して相互に変換できる。
- `Address`：hostのstorageであり、Bufferの`from`と`into`を通してだけ交換する。

IxPool、ImPool、containerはどれもEngramであり、寿命の管理に差はない。Bufferが自動で管理するのは、slotの占有状態と
capacityである。

IxPoolとImPoolの対は、一つのLiveなrunへ特化した型と、byte列へ特化した既存の型でも同じ形を取る。

| | identityを共有する | 値 |
|---|---|---|
| primitive | IxPool | ImPool |
| 一つのLiveなrun | Buffer（`[0, count)`） | Array（`[0, length)`） |
| byte列 | `Buffer<UInt8>` | `Symbol` |

`Symbol`は意味の上ではImPoolの上のbyte列のrunに、`#`、`+`、`/`、`%`、`==`を加えたものである。ImPoolで再定義はせず、
storageを共有するsliceのview、static storageのliteral、占有metadataのないdenseなbyte列という専用の表現を保つ。BufferとIxPoolの
関係と同じく、意味はImPoolの上で説明し、実装は同じ結果になる限り専用でよい。

## coordinateの線形性

IxPoolのslotは`0`から`capacity - 1`までの`USize` coordinateで選び、coordinate空間は順序を持ち途中に抜けがない。この線形性により
`[offset, offset + length)`という区間、つまりrunが意味を持つ。runの意味はIxPool storageの物理配置に依存しない。実装は
`Representable`な要素をcanonical layoutで連続に置くことを選べ、その場合Bufferのrunの操作はbulk copyになる。

coordinate空間が線形でも、占有状態には穴があり得る。Bufferは、Liveなcoordinateの集合が0から始まる一つの区間であることを
invariantにして、runの操作が占有状態を問わずに済むようにする。他のcontainerがこの線形空間をどう使うかは
[IxPool上のcontainer](containers.md)で比較する。

## IxPoolとBufferの責務

memory操作は、抽象する単位で三つの語彙に分かれ、IxPoolとBufferが分け持つ。

| 語彙 | 抽象する単位 | 操作 | 持つ型 |
|---|---|---|---|
| slot | 一つのcoordinateのLiveまたはVacant | `initAt`、`takeAt`、`getAt`、`reserve`、State | IxPool |
| sequence | 一つのLiveなrun `[0, count)` | `make`、`new`、`get`、`put`、`#`、growth policy | Buffer |
| run | 連続した要素範囲の一括の転送 | `fill`、`copy`、`from`、`into`、`*` | Buffer |

IxPoolはslotを抽象し、hostともrunとも関わらない。Bufferは一つのLiveなrunを抽象し、runの転送とhostとの交換を一手に引き受ける。
IxPoolはslotを空ける語彙を、Bufferは占有状態を気にせずrunを扱う語彙を持ち、互いに相手の持たない語彙を補う。

IxPoolがrunの語彙を持たないのは、runを本当に必要とするcontainerがBufferだけだからである。Map、木のLiveな集合は区間に
ならず、Dequeが成長時に必要とするのは値を写す`copy`ではなく移す操作である（[container](containers.md#runの語彙)）。
hostとの交換もBufferへ集めると、hostのdataがmalのstorageへ入る入口が`from`の一つになり、低い層のIxPoolがAddressの権限を
持たずに済む。他のcontainerはBufferを経由してhostと交換する。

Bufferは言語の組み込み型ではなく、IxPool上のpreludeのopaque型である。sequenceの語彙と`fill`、`copy`は
[Buffer実装](buffer-implementation.md)がIxPoolの上に書く参照実装で意味を定め、`from`、`into`、`*`はBufferのprimitiveとして
runtimeが持つ。

## 区分

各primitiveを次のどれかに分類する。

- 核：所有権を動かす遷移。他の操作はこれらへ分解できる。
- 必須：malの他の操作では表せない。
- 維持：意味は他の操作へ分解できるが、分解するとstorageの共有を壊すためprimitiveに残す。
- 性能：他の操作のloopで書けるが、call数や分岐を減らすためprimitiveにする。
- 派生：他のprimitiveで書く通常の関数。性能のためprimitiveにしてもよく、その場合も意味は分解と同じである。

## slot primitive

```mal
IxPool<State, T>

makeIxPool<State, T> :: (State, USize) -> IxPool<State, T>;
state<State, T> :: IxPool<State, T> -> State;
setState<State, T> :: (IxPool<State, T>, State) -> Unit;
capacity<State, T> :: IxPool<State, T> -> USize;
reserve<State, T> :: (IxPool<State, T>, USize) -> Unit;
isLive<State, T> :: (IxPool<State, T>, USize) -> Bool;
initAt<State, T> :: (IxPool<State, T>, USize, T) -> Unit;
takeAt<State, T> :: (IxPool<State, T>, USize) -> T;
getAt<State, T> :: (IxPool<State, T>, USize) -> T;
putAt<State, T> :: (IxPool<State, T>, USize, T) -> Unit;
dropAt<State, T> :: (IxPool<State, T>, USize) -> Unit;
```

| primitive | 区分 | 役割 |
|---|---|---|
| `makeIxPool` | 必須 | 初期StateとlogicalなcapacityでIxPoolを作る |
| `state`、`setState` | 必須 | 共有identityのStateを読み、置き換える |
| `capacity`、`reserve` | 必須 | logical capacityを読み、増やす。IxPoolは自動でgrowthしない |
| `isLive` | 必須 | coordinateの占有状態を読む |
| `initAt`、`takeAt` | 核 | Vacant → Live、Live → Vacant |
| `getAt` | 維持 | Liveなslotの値を`Share`する |
| `putAt`、`dropAt` | 派生 | `takeAt`して`initAt`、`takeAt`して結果を捨てる |

IxPoolの形成は`Storable(State)`と`Storable(T)`を要求する。IxPoolのcopyは同じState、capacity、slotを持つidentityを共有する。

## Buffer primitive

Bufferのoperationのうち、IxPoolの上に書けないものだけをruntimeのprimitiveとする。型、意味、preconditionは現行の
[AddressとBuffer](../../spec/memory.md)のままであり、本案は変えない。

| operation | 区分 | 理由 |
|---|---|---|
| `from<A>(address, offset, length)` | 必須 | malはAddressを読めない |
| `buffer.into(address, offset, length)` | 必須 | malはAddressへ書けない |
| `*buffer`（`Buffer<UInt8>`から`Symbol`） | 必須 | bytes列から`Symbol`を作る操作がmalにない。byte列へ特化した`freeze`に当たる |
| `*symbol`（`Symbol`から`Buffer<UInt8>`） | 性能 | `symbol # index`のloopでも書ける。byte列へ特化した`thaw`に当たる |
| `fill`、`copy` | 派生 | 参照実装はslot操作のloopであり、runtimeは同じ結果になる一括処理で実装してよい |

runtimeはこれらをBufferのrepresentation、つまりIxPoolとState=countの上で実装する。`*`はbyte IxPoolのstorageを`Symbol`と
共有してよく、書き込みはcopy-on-writeにする（[representation](lifecycle-contract.md#runtime-representation)）。

## 制御

```mal
trap :: Symbol -> [];
```

[primitive `trap`案](../primitive-trap.md)のprimitiveであり、Bufferなどのcontainerが現行runtimeと同じoverflow trapをmalで
起こすのに使う。Pool案はこの採択に依存する。

## ImPool primitive（候補）

ImPoolはIxPoolと対になるprimitiveの候補であり、意味は[identity](identity.md#impool)が所有する。読み出しはIxPoolと同じ意味を持ち、
更新は入力を`Store`で受け取ってsuccessorを返す。

```mal
ImPool<State, T>

makeImPool<State, T> :: (State, USize) -> ImPool<State, T>;
imState<State, T> :: ImPool<State, T> -> State;
imCapacity<State, T> :: ImPool<State, T> -> USize;
imIsLive<State, T> :: (ImPool<State, T>, USize) -> Bool;
imGetAt<State, T> :: (ImPool<State, T>, USize) -> T;
imSetState<State, T> :: (ImPool<State, T>, State) -> ImPool<State, T>;
imReserve<State, T> :: (ImPool<State, T>, USize) -> ImPool<State, T>;
imInitAt<State, T> :: (ImPool<State, T>, USize, T) -> ImPool<State, T>;
imPutAt<State, T> :: (ImPool<State, T>, USize, T) -> ImPool<State, T>;
imDropAt<State, T> :: (ImPool<State, T>, USize) -> ImPool<State, T>;
```

更新ごとの再利用かcopyかの判断は参照数に依存するため、malの他の操作では表せず、更新はすべて必須のprimitiveである。
preconditionは同名のIxPool primitiveと同じであり、`Storable`と`Stable`の扱いは[判定の分割](identity.md#判定の分割案)で扱う。

対の間の変換として次を置く。意味は[identity](identity.md#freezeとthaw)が所有する。

```mal
freeze<State, T> :: IxPool<State, T> -> ImPool<State, T>;
thaw<State, T> :: ImPool<State, T> -> IxPool<State, T>;
```

| primitive | 区分 | 役割 |
|---|---|---|
| `freeze` | 性能 | IxPoolの今のStateとslotを値にする。slotを一つずつcopyするloopでも書けるが、storageをO(1)で共有する |
| `thaw` | 性能 | ImPoolと同じ中身を持つ新しいidentityのIxPoolを作る。loopでも書けるが、一意ならstorageを移す |

## primitiveでないもの

検査付きのhandleを返すslot map（[IdPool](collection-examples.md#idpool)）は仕様に含めない。generationとfree listを含めて
IxPoolの上に書けるため、必要なcontainerが自分で持つ。

## 測定後の候補

次の操作は意味をslot操作のloopで書けるため、初期のprimitiveに含めない。loopのcostが測定で問題になった場合に追加を検討する。

- IxPoolの`moveRange`：範囲の`takeAt`と`initAt`を一括で行い、`Share`も`Drop`もしない。Dequeの成長、Mapのrehash、詰め直しが使う。
- IxPoolとhostの直接の交換：Bufferを経由する一段のcopyを省く。ring bufferのように大量のI/Oを自前で行うcontainerが使う。
  hostの権限をIxPoolへ広げるため、IxPoolを開くfileの範囲と合わせて判断する。
- ImPoolの範囲の写し：immutable arrayのsliceと連結。`Symbol`の`+`、`/`、`%`をbyte列以外へ広げたものに当たる。
