# Pool primitive

Status: Exploratory support document

この文書は、IxPoolとImPoolのprimitiveを、意味と計算量を定める核と、核で意味を定めたうえで定数倍の費用のために持つ周辺に
分けて管理する。核の導出は[Poolの意味論](../model/semantics.md#最小核の導出)、所有権の効果とpreconditionの責任は[runtime contract](../runtime/contract.md)、
BufferとVectorは[語彙の分担](buffer-vector.md#語彙の分担)を正とする。名前は仮のものである。

## 区分

各operationを、導けるかと、導いたときに何が変わるかで分類する。

| 区分 | 判断の基準 | 例 |
|---|---|---|
| 意味論の核 | malの他の操作では表せない | `swap`、`meta`、`swapMeta`、`grow`、Vectorの`from` |
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
slotだけがVacantを取り得るのは、`grow`が値を渡さずにplaceを作るためである（[Poolの意味論](../model/semantics.md#metaとslot)）。

各placeの核は、読み出しと交換の二つである。意味の上では、slotは`swap`だけで閉じ、`peek`も`swap`で書ける。`peek`を計算量の核に
置くのは、読み出しを書き込みにしないためである。書き込みにすると、共有中のImPoolや`freeze`したstorageを読むたびにO(n)の
copyが起きる。Metaには交換の間に置いておける値がないため、`meta`は`swapMeta`から導けない
（[最小核の導出](../model/semantics.md#最小核の導出)）。

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

## ImPool

`ImPool<Meta, V>`は、更新するたびにsuccessorを返す、identityを持たないPoolであり、IxPoolと対になるprimitiveの候補である。
状態はIxPoolと同じ`(m, n, slots)`を値として持ち、核と周辺はIxPoolと同じ名前、意味、preconditionを持つ。違いは、IxPoolで
`Unit`を返す更新がsuccessorを返し、値を返す更新がsuccessorとの組を返すことだけである。

次の表は、IxPoolとImPoolのsignatureを対で並べる。`IxPool`、`ImPool`はそれぞれ`IxPool<Meta, V>`、`ImPool<Meta, V>`を略す。

| operation | IxPool | ImPool |
|---|---|---|
| `pool` | `Meta -> IxPool` | `Meta -> ImPool` |
| `grow` | `(IxPool, USize) -> Unit` | `(ImPool, USize) -> ImPool` |
| `capacity` | `IxPool -> USize` | `ImPool -> USize` |
| `peek` | `(IxPool, USize) -> Slot<V>` | `(ImPool, USize) -> Slot<V>` |
| `swap` | `(IxPool, USize, Slot<V>) -> Slot<V>` | `(ImPool, USize, Slot<V>) -> (ImPool, Slot<V>)` |
| `meta` | `IxPool -> Meta` | `ImPool -> Meta` |
| `swapMeta` | `(IxPool, Meta) -> Meta` | `(ImPool, Meta) -> (ImPool, Meta)` |
| `slot` | `(IxPool, USize, Slot<V>) -> Unit` | `(ImPool, USize, Slot<V>) -> ImPool` |
| `setMeta` | `(IxPool, Meta) -> Unit` | `(ImPool, Meta) -> ImPool` |
| `isLive` | `(IxPool, USize) -> Bool` | `(ImPool, USize) -> Bool` |
| `getAt` | `(IxPool, USize) -> V` | `(ImPool, USize) -> V` |
| `initAt` | `(IxPool, USize, V) -> Unit` | `(ImPool, USize, V) -> ImPool` |
| `takeAt` | `(IxPool, USize) -> V` | `(ImPool, USize) -> (ImPool, V)` |
| `putAt` | `(IxPool, USize, V) -> Unit` | `(ImPool, USize, V) -> ImPool` |
| `dropAt` | `(IxPool, USize) -> Unit` | `(ImPool, USize) -> ImPool` |
| `moveAt` | `(IxPool, USize, USize) -> Unit` | `(ImPool, USize, USize) -> ImPool` |

二つの型のoperationを同じ名前で書くには、constructorをkeyに持つ[operation family](../../../spec/operation-families.md)が要る。
familyは一つのsignatureを持つため、表の形のままでは一つにまとまらない。[試作](../prototypes.md#二つの試作)は、IxPoolの更新も
同じidentityを返す形に揃えて一つのfamilyにし、新しいidentityを作る`pool`だけはconstructorごとに分けた。どちらの形を採るかは
[README](../README.md#未決定事項)の未決定事項である。

ImPoolのMetaは、意味の上では`(Meta, ImPool<Unit, V>)`というproductと同じである。値はproductごと更新できるため、
IxPoolと違ってMetaをPoolに置く必要はなく、ここではIxPoolとの対応のために持つ。

各更新はinputを`Store`で受け取り、内部で[writable successor](../runtime/contract.md#writable-successor)を作ってから変更して返す。inputが唯一の
responsibilityならstorageを再利用し、共有中ならcopyする。この判断は参照数に依存し、malの他の操作では表せないため、核の
更新`grow`、`swap`、`swapMeta`は意味論の核である。

更新前のvalueをsourceから変更する手段がないため、storage内でShareしても後のmutationを観測しない。

更新を`Unit`を返す操作とsuccessor取得の二つへ分けると、Shareしたsuccessorを更新しないことが未検査preconditionになり、違反は
別のvalueの変更として現れる。更新自体がsuccessorを返す形なら、`Storable`の健全性をcontainer実装のinvariantへ依存させない。

### 更新の費用

ImPoolの更新がstorageを再利用するかは、inputが唯一のresponsibilityかで決まる。意味は変わらないが費用はO(1)とO(n)に分かれる
ため、どこでcopyが起きるかを費用の約束として定める。copyの原因は次の三つである。

| 原因 | sourceからの見え方 | 扱い |
|---|---|---|
| 更新前の値を後で使う | 同じ関数の中で見える | 意味が求めるcopyであり、避けられない |
| 同じ値を別のplaceにも保存している | 保存した場所が離れていると見えない | 入れ子の値は`takeAt`で取り出して更新し、戻す |
| 呼び出し経路がresponsibilityを借りる | 見えない | `Store`をmal wrapperへ伝播して起こさない |

三つ目を実装の自由として残すと、同じsourceの費用がcompilerの解析の精度で変わる。本案は、last useのinputを`Store`へ渡す
呼び出しは`Consume`になることを約束し、`Store`へ渡るparameterを持つmal wrapperをowned native entryにする
（[compilerとruntimeの分担](../runtime/implementation.md#compilerとruntimeの分担)）。残る非局所的なcopyは二つ目だけであり、
入れ子の値を一意に保つ書き方は[Vector](../containers/sequences.md#vector)の`vectorUpdate`と同じ`takeAt`と`initAt`の組である。
SwiftのArrayも同じ費用の約束を持つ（[関連事例](../../../research/pool-storage-prior-art.md)）。

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
のloopで定まり、区分は計算量の核である。primitiveにするのはO(n)のcopyを避けられる場合があるためであり、`thaw`は入力が唯一の
responsibilityならstorageを移し、共有中ならcopyする。`freeze`の実装には次の三つがある。

| 方式 | `freeze`の費用 | IxPoolへの書き込みの費用 |
|---|---|---|
| 常にcopyする | O(n) | 追加なし |
| storageを共有し、IxPool側の後の書き込みでcopyする | O(1) | 毎回、共有中かを確認する |
| inputがlast useで他のaliasがなければstorageを移し、それ以外はcopyする | 移せればO(1)、それ以外はO(n) | 追加なし |

二つ目は、全要素型のIxPoolの全書き込みに確認を課す。現在のBufferは`Symbol`とbyte ownerを共有するため同じ確認を持つが、
IxPoolを安い可変primitiveとする前提と衝突する。三つ目は`thaw`と対称であり、組み立ててから公開する主な用途、例えばBufferを
`*`で`Symbol`にして捨てる形ではcopyも確認も起きない。copyが残るのは、`freeze`した後もIxPoolを使い続ける場合である。そのとき
二つ目はIxPoolへ次に書くまでcopyを遅らせ、書かなければcopyしない。本案は三つ目を採り、`freeze`のinputを`Store`で受け取る。

## 測定後の候補

次の操作は意味を核のloopで書ける。loopのcostが測定で問題になった場合に追加を検討する。

- IxPoolの`moveRange`：範囲の`takeAt`と`initAt`を一括で行い、`Share`も`Drop`もしない。Dequeの成長、Mapのrehash、詰め直しが使う。
- ImPoolの範囲の写し：Vectorのsliceと連結。`Symbol`の`+`、`/`、`%`をbyte列以外へ広げたものに当たる。
