# primitive一覧

Status: Exploratory support document

この文書は、IxPoolとBufferの責務分担と、Pool案が仮定するprimitiveの一覧と意味を管理する。slot primitiveのpreconditionと
所有権の効果は[runtime contract](runtime.md)、Bufferのoperationの意味とpreconditionは[AddressとBuffer](../../spec/memory.md)を
正とする。名前は仮のものである。

## 区分

各primitiveを次のどれかに分類する。

- 核：所有権を動かす遷移。他の操作はこれらへ分解できる。
- 必須：malの他の操作では表せない。
- 維持：意味は他の操作へ分解できるが、分解するとstorageの共有を壊すためprimitiveに残す。
- 性能：他の操作のloopで書けるが、call数や分岐を減らすためprimitiveにする。
- 派生：他のprimitiveで書く通常の関数。性能のためprimitiveにしてもよく、その場合も意味は分解と同じである。

## IxPool

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

slotは`0`から`capacity - 1`までの`USize` coordinateで選び、coordinate空間は順序を持ち途中に抜けがない。この線形性により
`[offset, offset + length)`という区間、つまりrunが意味を持つ。runの意味はstorageの物理配置に依存しないが、実装は
`Representable`な要素をcanonical layoutで連続に置くことを選べ、その場合Bufferのrunの操作はbulk copyになる。coordinate空間が
線形でも、占有状態には穴があり得る。

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

## Buffer

Bufferは言語の組み込み型ではなく、IxPool上のpreludeのopaque型である。sequenceの語彙と`fill`、`copy`は
[Buffer実装](buffer-implementation.md)の参照実装で意味を定め、IxPoolの上に書けないものだけをruntimeのprimitiveとする。
型、意味、preconditionは現行の[AddressとBuffer](../../spec/memory.md)のままであり、本案は変えない。

| operation | 区分 | 理由 |
|---|---|---|
| `from<A>(address, offset, length)` | 必須 | malはAddressを読めない |
| `buffer.into(address, offset, length)` | 必須 | malはAddressへ書けない |
| `*buffer`（`Buffer<UInt8>`から`Symbol`） | 必須 | bytes列から`Symbol`を作る操作がmalにない。byte列へ特化した`freeze`に当たる |
| `*symbol`（`Symbol`から`Buffer<UInt8>`） | 性能 | `symbol # index`のloopでも書ける。byte列へ特化した`thaw`に当たる |
| `fill`、`copy` | 派生 | 参照実装はslot操作のloopであり、runtimeは同じ結果になる一括処理で実装してよい |

runtimeはこれらをBufferのrepresentation、つまりIxPoolとState=countの上で実装する。`*`はbyte IxPoolのstorageを`Symbol`と
共有してよく、書き込みはcopy-on-writeにする（[representation](runtime.md#runtime-representation)）。

containerが現行runtimeと同じoverflow trapをmalで起こすには、[primitive `trap`案](../primitive-trap.md)の
`trap :: Symbol -> []`を使う。Pool案はこの採択に依存する。

## ImPool

`ImPool<State, T>`は、更新するたびにsuccessorを返す、identityを持たないPoolであり、IxPoolと対になるprimitiveの候補である。
読み出しはIxPoolと同じ意味を持ち、preconditionは同名のIxPool primitiveと同じである。

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

各更新はinputを`Store`で受け取り、内部で[writable successor](runtime.md#writable-successor)を作ってから変更して返す。inputが唯一の
responsibilityならstorageを再利用し、共有中ならcopyする。この判断は参照数に依存し、malの他の操作では表せないため、更新は
すべて必須のprimitiveである。

- 更新前のvalueをsourceから変更する手段がないため、storage内でShareしても後のmutationを観測しない。
- 同じvalueを別のslotへ保存したoperandはShareされてuniquenessが成り立たず、以後の更新はcopyへfallbackする。

更新を`Unit`を返す操作とsuccessor取得の二つへ分けると、Shareしたsuccessorを更新しないことが未検査preconditionになり、違反は
別のvalueの変更として現れる。更新自体がsuccessorを返す形なら、`Storable`の健全性をcontainer実装のinvariantへ依存させない。

`Symbol`は意味の上ではImPoolの上のbyte列のrunに、`#`、`+`、`/`、`%`、`==`を加えたものである。ImPoolで再定義はせず、
storageを共有するsliceのview、static storageのliteral、占有metadataのないdenseなbyte列という専用の表現を保つ。BufferとIxPoolの
関係と同じく、意味はImPoolの上で説明し、実装は同じ結果になる限り専用でよい。

## freezeとthaw

```mal
freeze<State, T> :: IxPool<State, T> -> ImPool<State, T>;
thaw<State, T> :: ImPool<State, T> -> IxPool<State, T>;
```

- `freeze(pool)`は、呼び出し時点のStateとslotを持つImPoolを返す。元のIxPoolはidentityを保ったまま使い続けられ、以後の変更は
  返した値から観測されない。
- `thaw(value)`は、同じStateとslotを持つ新しいidentityのIxPoolを返す。返したIxPoolへの変更は、元の値からも、同じ値から
  thawした別のIxPoolからも観測されない。

可変なIxPoolで効率よく組み立ててから値として公開すること、値から編集用の可変なcopyを作ることに使う。意味は要素を一つずつ
copyするloopで定まるため、区分は性能である。primitiveにするのはstorageを共有するためであり、`freeze`はIxPoolのstorageを
ImPoolと共有して、IxPool側への後の書き込みでcopyする。`thaw`は入力が唯一のresponsibilityならstorageを移し、共有中なら
書き込みでcopyする。共有を許すと、IxPoolへの書き込みのたびにstorageが共有中かの確認が一回入る。現在のBufferも`Symbol`と
byte ownerを共有するため同じ確認を持つが、全要素型のIxPoolへ広げるか、`freeze`を常にcopyにして確認を省くかは未決定である。

## primitiveでないもの

検査付きのkeyを返す[SlotMap](collection-examples.md#slotmap)は仕様に含めない。generationとfree listを含めて
IxPoolの上に書けるため、必要なcontainerが自分で持つ。

## 測定後の候補

次の操作は意味をslot操作のloopで書けるため、初期のprimitiveに含めない。loopのcostが測定で問題になった場合に追加を検討する。

- IxPoolの`moveRange`：範囲の`takeAt`と`initAt`を一括で行い、`Share`も`Drop`もしない。Dequeの成長、Mapのrehash、詰め直しが使う。
- IxPoolとhostの直接の交換：Bufferを経由する一段のcopyを省く。ring bufferのように大量のI/Oを自前で行うcontainerが使う。
  hostのdataがmalのstorageへ入る入口が`from`の外にも増える。
- ImPoolの範囲の写し：immutable arrayのsliceと連結。`Symbol`の`+`、`/`、`%`をbyte列以外へ広げたものに当たる。
