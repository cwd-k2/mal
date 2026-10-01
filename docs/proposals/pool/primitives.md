# primitive一覧

Status: Exploratory support document

この文書は、Pool案のprimitiveを、意味と計算量を定める核と、核で意味を定めたうえで定数倍の費用のために持つ周辺に分けて
管理する。核の導出と図示は[slotモデル](slot-model.md)、所有権の効果とpreconditionの責任は[runtime contract](runtime.md)、Bufferの
operationの意味とpreconditionは[AddressとBuffer](../../spec/memory.md)を正とする。名前は仮のものである。

## 区分

各operationを、導けるかと、導いたときに何が変わるかで分類する。

| 区分 | 判断の基準 | 例 |
|---|---|---|
| 意味論の核 | malの他の操作では表せない | `swap`、`meta`、`swapMeta`、`grow`、`from` |
| 計算量の核 | 意味は他の操作で書けるが、書くと計算量が変わる | `peek`、`freeze`、`thaw` |
| 定数倍の周辺 | 意味は他の操作で書け、差は`Share`、`Drop`、tagの分岐、call数 | `getAt`、`takeAt`、`initAt` |
| 派生 | 費用も含めて他の操作と同じ | `slot`、`setMeta`、`moveAt` |

意味論の核と計算量の核をあわせて核、定数倍の周辺と派生をあわせて周辺と呼ぶ。定数倍の周辺は測定によって足し引きでき、
核は足し引きするとprogramの意味か計算量が変わる。

## IxPool

### 核

```mal
IxPool<Meta, V>
Slot<V> :: [Unit, V];

pool<Meta, V> :: Meta -> IxPool<Meta, V>;
grow<Meta, V> :: (IxPool<Meta, V>, USize) -> Unit;
capacity<Meta, V> :: IxPool<Meta, V> -> USize;
peek<Meta, V> :: (IxPool<Meta, V>, USize) -> Slot<V>;
swap<Meta, V> :: (IxPool<Meta, V>, USize, Slot<V>) -> Slot<V>;
meta<Meta, V> :: IxPool<Meta, V> -> Meta;
swapMeta<Meta, V> :: (IxPool<Meta, V>, Meta) -> Meta;
```

IxPoolは`(m, n, slots)`を一つの共有identityとして持つ。`m`は`Meta`の値、`n`はcoordinate空間の大きさ、`slots`は`[0, n)`の
各coordinateに`Slot<V>`の値を割り当てる。`Slot<V>`の第一項をVacant、第二項をLiveと呼ぶ。

| primitive | 区分 | 意味 | precondition |
|---|---|---|---|
| `pool(m)` | 意味論の核 | `(m, 0, ∅)`を持つ新しいidentityを返す | なし |
| `grow(pool, k)` | 意味論の核 | `n := n + k`とし、増えたslotをVacantにする | なし |
| `capacity(pool)` | 意味論の核 | `n`を返す | なし |
| `peek(pool, i)` | 計算量の核 | `slots[i]`を返す | `i < n` |
| `swap(pool, i, s)` | 意味論の核 | `slots[i] := s`とし、古い`slots[i]`を返す | `i < n` |
| `meta(pool)` | 意味論の核 | `m`を返す | なし |
| `swapMeta(pool, m')` | 意味論の核 | `m := m'`とし、古い`m`を返す | なし |

IxPoolの形成は`Storable(Meta)`と`Storable(V)`を要求する。IxPoolのcopyは同じidentityを共有し、Meta、`n`、slotへの変更を
すべてのaliasが観測する。`n`はPool自身の構造であり、containerが選ぶ値を置くMetaとは別に持つ。MetaとslotはどちらもIxPool
identityの中のplaceであり、Metaは常に`Meta`の値を、slotは常に`Slot<V>`の値を持つ。
slotだけがVacantを取り得るのは、`grow`が値を渡さずにplaceを作るためである（[slotモデル](slot-model.md#metaとslot)）。

各placeの核は、読み出しと交換の二つである。意味の上では、slotは`swap`だけで閉じ、`peek`も`swap`で書ける。`peek`を計算量の核に
置くのは、読み出しを書き込みにしないためである。書き込みにすると、共有中のImPoolや`freeze`したstorageを読むたびにO(n)の
copyが起きる。Metaには交換の間に置いておける値がないため、`meta`は`swapMeta`から導けない
（[最小核の導出](slot-model.md#最小核の導出)）。

coordinate空間は順序を持ち途中に抜けがない。この線形性により`[offset, offset + length)`という区間、つまりrunが意味を持つ。
runの意味はstorageの物理配置に依存しないが、実装は`Representable`な要素をcanonical layoutで連続に置くことを選べ、その場合
Bufferのrunの操作はbulk copyになる。coordinate空間が線形でも、Liveなslotの集合には穴があり得る。

### 周辺

周辺のoperationは、核の合成で意味を定める。一部は核にない未検査preconditionを加え、それを満たすcallでは核の合成と同じ結果を
少ない費用で返す。

```mal
vacant<V> :: Unit -> Slot<V> := () -> [empty, full] => { empty(); };
live<V> :: V -> Slot<V> := (value) -> [empty, full] => { full(value); };

slot<Meta, V> :: (IxPool<Meta, V>, USize, Slot<V>) -> Unit;
setMeta<Meta, V> :: (IxPool<Meta, V>, Meta) -> Unit;
isLive<Meta, V> :: (IxPool<Meta, V>, USize) -> Bool;
getAt<Meta, V> :: (IxPool<Meta, V>, USize) -> V;
initAt<Meta, V> :: (IxPool<Meta, V>, USize, V) -> Unit;
takeAt<Meta, V> :: (IxPool<Meta, V>, USize) -> V;
putAt<Meta, V> :: (IxPool<Meta, V>, USize, V) -> Unit;
dropAt<Meta, V> :: (IxPool<Meta, V>, USize) -> Unit;
moveAt<Meta, V> :: (IxPool<Meta, V>, USize, USize) -> Unit;
```

| operation | 区分 | 核による意味 | 追加のprecondition | primitiveにする理由 |
|---|---|---|---|---|
| `vacant`、`live` | 派生 | `Slot<V>`の各項を作る | なし | — |
| `slot(pool, i, s)` | 派生 | `swap(pool, i, s)`の結果を捨てる | なし | — |
| `setMeta(pool, m)` | 派生 | `swapMeta(pool, m)`の結果を捨てる | なし | — |
| `isLive(pool, i)` | 定数倍の周辺 | `peek(pool, i)`がLiveか | なし | payloadを`Share`しない |
| `getAt(pool, i)` | 定数倍の周辺 | `peek(pool, i)`のLiveの値 | Live | tagの分岐を省く |
| `initAt(pool, i, v)` | 定数倍の周辺 | `slot(pool, i, live(v))` | Vacant | 古い値の`Drop`の判定を省く |
| `takeAt(pool, i)` | 定数倍の周辺 | `swap(pool, i, vacant())`のLiveの値 | Live | tagの分岐を省く |
| `putAt(pool, i, v)` | 定数倍の周辺 | `slot(pool, i, live(v))` | Live | tagの分岐を省く |
| `dropAt(pool, i)` | 派生 | `slot(pool, i, vacant())` | なし | — |
| `moveAt(pool, a, b)` | 派生 | `initAt(pool, b, takeAt(pool, a))` | `a`がLive、`b`がVacant | — |

全operationは核と同じ`i < n`も要求する。核だけを使うcontainerは、LiveとVacantの一致を`Slot<V>`の除去で扱い、範囲以外の
preconditionを持たない。

## IxPoolとBufferの責務

memory操作は、抽象する単位で三つの語彙に分かれ、IxPoolとBufferが分け持つ。

| 語彙 | 抽象する単位 | 操作 | 持つ型 |
|---|---|---|---|
| slot | 一つのcoordinateのplace | `peek`、`swap`、`grow`、Meta | IxPool |
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
| `from<A>(address, offset, length)` | 意味論の核 | malはAddressを読めない |
| `buffer.into(address, offset, length)` | 意味論の核 | malはAddressへ書けない |
| `*buffer`（`Buffer<UInt8>`から`Symbol`） | 意味論の核 | bytes列から`Symbol`を作る操作がmalにない。byte列へ特化した`freeze`に当たる |
| `*symbol`（`Symbol`から`Buffer<UInt8>`） | 定数倍の周辺 | `symbol # index`のloopでも書ける。byte列へ特化した`thaw`に当たる |
| `fill`、`copy` | 派生 | 参照実装はslot操作のloopであり、runtimeは同じ結果になる一括処理で実装してよい |

runtimeはこれらをBufferのrepresentation、つまりIxPoolとMeta=countの上で実装する。`*`はbyte IxPoolのstorageを`Symbol`と
共有してよく、書き込みはcopy-on-writeにする（[representation](runtime.md#runtime-representation)）。

containerが現行runtimeと同じoverflow trapをmalで起こすには、[primitive `trap`案](../primitive-trap.md)の
`trap :: Symbol -> []`を使う。Pool案はこの採択に依存する。

## ImPool

`ImPool<Meta, V>`は、更新するたびにsuccessorを返す、identityを持たないPoolであり、IxPoolと対になるprimitiveの候補である。
状態はIxPoolと同じ`(m, n, slots)`を値として持ち、核と周辺はIxPoolと同じ名前、意味、preconditionを持つ。違いは、IxPoolで
`Unit`を返す更新がsuccessorを返し、値を返す更新がsuccessorとの組を返すことだけである。

```mal
ImPool<Meta, V>

pool<Meta, V> :: Meta -> ImPool<Meta, V>;
grow<Meta, V> :: (ImPool<Meta, V>, USize) -> ImPool<Meta, V>;
capacity<Meta, V> :: ImPool<Meta, V> -> USize;
peek<Meta, V> :: (ImPool<Meta, V>, USize) -> Slot<V>;
swap<Meta, V> :: (ImPool<Meta, V>, USize, Slot<V>) -> (ImPool<Meta, V>, Slot<V>);
meta<Meta, V> :: ImPool<Meta, V> -> Meta;
swapMeta<Meta, V> :: (ImPool<Meta, V>, Meta) -> (ImPool<Meta, V>, Meta);

slot<Meta, V> :: (ImPool<Meta, V>, USize, Slot<V>) -> ImPool<Meta, V>;
setMeta<Meta, V> :: (ImPool<Meta, V>, Meta) -> ImPool<Meta, V>;
takeAt<Meta, V> :: (ImPool<Meta, V>, USize) -> (ImPool<Meta, V>, V);
```

`isLive`、`getAt`はIxPoolと同じ型で読み、`initAt`、`putAt`、`dropAt`、`moveAt`は`ImPool<Meta, V>`を返す。
ImPoolのMetaは、意味の上では`(Meta, ImPool<Unit, V>)`というproductと同じである。値はproductごと更新できるため、
IxPoolと違ってMetaをPoolに置く必要はなく、ここではIxPoolとの対応のために持つ。

各更新はinputを`Store`で受け取り、内部で[writable successor](runtime.md#writable-successor)を作ってから変更して返す。inputが唯一の
responsibilityならstorageを再利用し、共有中ならcopyする。この判断は参照数に依存し、malの他の操作では表せないため、核の
更新`grow`、`swap`、`swapMeta`は意味論の核である。

- 更新前のvalueをsourceから変更する手段がないため、storage内でShareしても後のmutationを観測しない。
- 同じvalueを別のslotへ保存したoperandはShareされてuniquenessが成り立たず、以後の更新はcopyへfallbackする。

更新を`Unit`を返す操作とsuccessor取得の二つへ分けると、Shareしたsuccessorを更新しないことが未検査preconditionになり、違反は
別のvalueの変更として現れる。更新自体がsuccessorを返す形なら、`Storable`の健全性をcontainer実装のinvariantへ依存させない。

`Symbol`は意味の上ではImPoolの上のbyte列のrunに、`#`、`+`、`/`、`%`、`==`を加えたものである。ImPoolで再定義はせず、
storageを共有するsliceのview、static storageのliteral、占有tagのないdenseなbyte列という専用の表現を保つ。BufferとIxPoolの
関係と同じく、意味はImPoolの上で説明し、実装は同じ結果になる限り専用でよい。

## freezeとthaw

```mal
freeze<Meta, V> :: IxPool<Meta, V> -> ImPool<Meta, V>;
thaw<Meta, V> :: ImPool<Meta, V> -> IxPool<Meta, V>;
```

- `freeze(pool)`は、呼び出し時点のMeta、`n`、slotを持つImPoolを返す。元のIxPoolはidentityを保ったまま使い続けられ、以後の
  変更は返した値から観測されない。
- `thaw(value)`は、同じMeta、`n`、slotを持つ新しいidentityのIxPoolを返す。返したIxPoolへの変更は、元の値からも、同じ値から
  thawした別のIxPoolからも観測されない。

可変なIxPoolで効率よく組み立ててから値として公開すること、値から編集用の可変なcopyを作ることに使う。意味は核の`peek`と`swap`
のloopで定まり、区分は計算量の核である。primitiveにするのはstorageを共有してO(n)のcopyを避けるためであり、`freeze`はIxPoolのstorageを
ImPoolと共有して、IxPool側への後の書き込みでcopyする。`thaw`は入力が唯一のresponsibilityならstorageを移し、共有中なら
書き込みでcopyする。共有を許すと、IxPoolへの書き込みのたびにstorageが共有中かの確認が一回入る。現在のBufferも`Symbol`と
byte ownerを共有するため同じ確認を持つが、全要素型のIxPoolへ広げるか、`freeze`を常にcopyにして確認を省くかは未決定である。

## 測定後の候補

次の操作は意味を核のloopで書ける。loopのcostが測定で問題になった場合に追加を検討する。

- IxPoolの`moveRange`：範囲の`takeAt`と`initAt`を一括で行い、`Share`も`Drop`もしない。Dequeの成長、Mapのrehash、詰め直しが使う。
- IxPoolとhostの直接の交換：Bufferを経由する一段のcopyを省く。ring bufferのように大量のI/Oを自前で行うcontainerが使う。
  hostのdataがmalのstorageへ入る入口が`from`の外にも増える。
- ImPoolの範囲の写し：immutable arrayのsliceと連結。`Symbol`の`+`、`/`、`%`をbyte列以外へ広げたものに当たる。
