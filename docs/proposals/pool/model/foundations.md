# Poolの位置付けと根本モデル

Status: Exploratory support document; rebased on mal v0.7

この文書は、Poolがmal全体のどのauthorityを担い、なぜsystem全体のminimalityを改善するかを定める。
Poolの状態とoperationの形式的な核は[意味論](semantics.md)、responsibilityの移動は
[responsibility](responsibility.md)、物理表現は[runtime contract](../runtime/contract.md)を正とする。

## 中心命題

Poolはallocatorでもcontainerでもない。Engram authorityの下にある有限個のtyped placeを、shared identityへのhandle valueまたは
structural snapshot valueとして扱うmechanismである。

```text
Engram-owned finite places
    + indexed coordinates
    + place read and exchange
    + handle or snapshot observation
    = Pool
```

allocation、relocation、reference count、occupancy bitmapはこの意味を実装するmechanismであり、Poolのsource semanticsではない。
順序、hash、free list、root、key、Liveなcoordinateの形はcontainerのrelationとinvariantであり、Poolの意味ではない。

## 言語全体での位置

Poolは純粋なcore calculusへ加えるdata constructorではない。純粋な値とapplicationだけから共有mutable identityは導けないため、
PoolはEngram storage primitiveの層に属する。

```text
value and control core
    variable, lambda, application, product, sum

Engram authority
    managed lifetime, reusable carrier, shared identity

Pool state algebra
    distinguished Header place, finite indexed Slot places

source carrier
    IxPool: handle value for a shared mutable identity
    ImPool: snapshot value and successor

container policy written in mal
    Buffer, Vector, Map, Deque, heap, tree

implementation mechanism
    execution responsibility, LLVM layout, C runtime allocation
```

現在の`Buffer`は既にmal-owned lifetime、共有mutable identity、growableなindexed storage、managed elementのlifecycleを持つ。
Poolは新しい種類のauthorityを追加するのではなく、`Buffer`がstorage mechanismとdense sequence policyを一つにしている状態を
分解する。

```text
Buffer
  = indexed mutable storage
  + Live prefix [0, count)
  + count and growth policy
  + sequence and range operations

IxPool
  = indexed mutable places
  + shared identity
  + coordinate-space extension
  + place read and exchange
```

この分解により、Map、Deque、heap、木ごとに独自のexternal allocatorやBuffer上の番兵規約を発明せず、共通mechanismの上へ
relationとinvariantだけを書ける。

## authority

Pool carrierとreferentはEngramに属する。C runtimeがstorageを確保しても、そのallocation functionや返したpointerへExternの
authorityが移るわけではない。

| 問い | authority |
|---|---|
| validなPool stateとSlotを定める | language semantics |
| IxPool identityを構成する | Pool primitive |
| identityの変更を観測、実行する | IxPool handle |
| carrierのlifetimeを終了する | Engram responsibility |
| ImPoolの旧snapshotを保存しsuccessorを作る | ImPool operation |
| coordinateにcontainer上の意味を与える | opaque containerのoperationとinvariant |
| backing storageを確保、移動、解放する | trusted runtime mechanism |
| target上のlayoutとtyped lifecycle glueを作る | LLVM backend |
| allocation failureを処理する | 既存Engram allocationと同じlanguage trap |
| PoolをExtern resourceとして保持または解放する | 認めない。Poolはmal-owned Engramである |

この分類では、Pool constructionはextern operationではない。closure environmentや現行Bufferのstorageと同じく、観測不能な
Engram representationをruntimeが用意する。extern CがPool carrierを扱えるよう将来拡張しても、それはD098と同じruntime
extensionとしてmal-owned identityを操作するのであり、PoolをExtern-owned resourceへ変えない。host固有のallocator選択、
recoverable failure、external resource identityはPoolの意味へ現れない。

## authority、responsibility、representation

PoolをRustのownershipへ寄せすぎるとsource valueまでlinearになり、HaskellのGCへ寄せすぎると更新の費用を制御する手掛かりが消える。
本案は、三つの問いを別の層へ置く。

| 層 | 問い | Poolでの答え |
|---|---|---|
| authority | 同じcarrierを別bindingへ渡した後の変更を誰が観測できるか | IxPool handleとImPool snapshotの観測則 |
| responsibility | carrierを誰が次へ渡し、いつ終了するか | compilerが`Share`、`Consume`、`Drop`へlowerする |
| representation | allocationを共有、移動、複製できるか | runtimeとbackendが観測不能な範囲で選ぶ |

source valueは通常のmal valueとして再利用できる。再利用を許すsource semanticsと、実行時responsibilityをaffineに移すloweringは
矛盾しない。call後にもvalueを使うならcompilerが`Share`し、last useなら`Consume`できる。物理storageが一意かはさらに別の問いであり、
`Consume`はstorage再利用の許可を与えるが、一意性を保証しない。

この分離により、Rustの`Box<T>`に似た「一つのresponsibilityがallocation上の`T`を保持する」状態はsource typeでなくlowering上の
事実として現れる。Boxのようなexclusive ownerをPool authorityへ加える必要はない。runtimeはIxPoolにもImPoolにも一時的に単一ownerの
representationを使えるが、IxPoolの変更は別handleから観測され、ImPoolの旧snapshotの構造は変更されないというsource semanticsを
変えない。

すべてのPool carrierはsource valueである。違いはvalueであるかどうかではなく、そのvalueがstateへ到達するauthorityにある。

```text
IxPool<H, V> = handle value for identity i carrying PoolState<H, V>
ImPool<H, V> = snapshot value representing PoolState<H, V>
```

区別はcarrierを別のbindingへ渡した後の観測で定める。

```text
y := x

IxPool: update(x)      の後、observe(y)は更新後のstateを返す
ImPool: x2 := update(x)の後、observe(y)は更新前、observe(x2)は更新後のstateを返す
```

IxPool handleの複製は同じidentityを共有する。ImPoolの更新はsuccessor snapshotを返し、更新前のHeader、capacity、slot carrierは
変わらない。これは二つのstorage algebraではなく、一つのPool state algebraに対する二つの観測則である。

IxPool handleまたはImPool snapshotへのresponsibilityを誰が持つかは`Share`、`Consume`、`Drop`で定まり、place内の値へのresponsibilityは
read、exchange、writeのloweringで移る。IxPoolがidentityを共有することは、各handleが独立したresponsibilityを持つことと矛盾しない。
ImPoolがsnapshot valueであることも、物理storageやslotに保存したhandleのreferentをdeep copyすることを意味しない。

handleもsource valueなのでtyped placeへ保存できる。保存されるのはhandle carrierへのresponsibilityと、同じidentityへ到達する
authorityである。IxPoolやBufferのhandleを複数のslotへ保存すれば、各slotは同じidentityの変更を共有観測する。このaliasは
handle semanticsそのものであり、value storageの例外ではない。

ImPoolのsnapshotはcarrierについてstructuralである。旧snapshotが`handle i`を持つslotはsuccessor作成後も`handle i`を返すが、
identity `i`を別のhandleから変更すれば、そのreferentの変更は旧snapshotから得たhandleでも観測する。外側の構造が変わらないことと、
到達可能な全stateが推移的に不変であることを同一視しない。

ImPoolのstorage再利用は次のas-if ruleに従う。

> 更新前のsnapshotに対する今後のPool operationによる構造観測と区別できない場合に限り、実装はstorageをsuccessorへ再利用できる。

inputを`Consume`でき、runtimeが区別可能なaliasを持たないと確認できることは、この条件の十分条件である。reference countはその確認に
使えるrepresentation上のwitnessの一つにすぎず、ImPoolの意味ではない。

## RustとHaskellの間に置くもの

Rustはmutation authorityとallocation responsibilityをsource ownershipとborrowへ強く結び付ける。Haskellはmutation authorityを
`ST`や`IO`へ置き、allocation responsibilityをGCの下へ隠す。malは両者の中間に新しいownership systemを作るのではなく、既存の
分担を組み合わせる。

```text
Haskellに近い部分  source valueは再利用でき、ImPoolはstructural snapshot semanticsを持つ
Rustに近い部分     backendへ渡すresponsibilityはaffineに移動できる
mal固有の境界       authorityは型とoperation、responsibilityはcompiler、物理一意性はruntime
```

IxPoolで組み立てて`freeze`する形は`ST`に似るが、identityのescapeを型で禁止しない。escapeしたhandleがあればsnapshotのためにcopyし、
なければstorageを移せる。違反を不正なprogramにせず、同じ意味の高い費用へ落とす。Rustのinterior referenceの代わりにcoordinateを
使うのも同じ選択であり、sourceへborrow lifetimeを加えずにgrowthとrelocationを許す。

比較に使える軸は、source typeを増やす分類ではなく、責務の混同を検出するための四つである。

| 抽象 | authority | shape | occupancy | addressability |
|---|---|---|---|---|
| C allocationとpointer | pointerを解釈する外部contract | byte extent | 規定しない | pointer arithmetic |
| Rust `Box<T>` | exclusive owner | single | always Live | borrowable address |
| IxPool | handleからshared identityを観測 | extensible indexed places | Vacant / Live | coordinate |
| ImPool | structural snapshot | extensible indexed state | Vacant / Live | coordinate |

この表から、C allocationはPoolのbacking mechanismだけを提供し、typed place、occupancy、authorityを定めないことが分かる。
BoxとPoolの共通部分は「typed valueをmanaged lifetimeで保持する」ことだけである。Boxのsingle place、exclusive source owner、
stableなderefをPoolへ持ち込まず、Poolの最小核はindexed place、occupancy、handleとsnapshotの観測則に限る。

## minimality

[minimality](../../../design/minimality.md)が小さくする対象はprimitive数だけではなく、language surface、static/dynamic semantics、
runtime、backend、extern contract、暗黙のcost、利用者が調べるAPIの合計である。Poolは次の理由でこの合計を減らす。

- shared mutable identityを新設せず、現行Bufferが既に持つEngram authorityを一般化する。
- vacancyを新しいuninitialized value categoryにせず、既存のsum `Slot<V> = [Unit, V]`で表す。
- valueの移動をlinear source valueにせず、placeのexchangeと既存responsibility規則で表す。
- `Storable`をplace lifecycleの一つの判定にし、handle用の`Managed`や`Placeable`を追加しない。
- ImPoolへ推移的な`Stable` judgmentを要求せず、handleを含むslot carrierの構造だけをsnapshotとして保存する。
- allocator、pointer、layout、reference countをsourceへ公開しない。
- container固有のrelationとinvariantをPoolへ固定しない。
- 初回採択ではextern surfaceを増やさず、C連携を現行BufferとSymbolのdirect runtime carrierへ閉じる。
- Buffer、Map、Deque、heap、木のstorage lifecycleを一つのtyped mechanismから導く。

Poolの核が小さいかは、operationを削れるかだけでなく、削った結果をどこへ移すかで判断する。例えばslot readはexchange二回でも
表せるが、ImPoolやfreeze後に読み出しだけでcopyを起こすため、`peek`を除くと計算量contractが別の場所へ漏れる。Headerを外へ
分けると、malには可変product fieldがないため、containerごとに別identityとの同期規約が要る。これらはsurface上のoperation数を
減らしてもsystemを小さくしない。

反対に、次はPoolへ入れない。

- allocator selection、allocation region、recoverable allocation failure
- physical reserve、alignment、stride、AoSまたはSoA
- iteration、ordering、hash、key validity、free list
- checked APIとunchecked APIの二重のoperation集合
- persistent treeやchunkの構造共有policy

これらは既存のauthority、container relation、または別の要求が現れた時点で所有する層へ置く。

## CとLLVMとの境界

各systemがallocation、typed storage、mutation authorityを置く層の資料は
[Pool storageの関連事例](../../../research/pool-storage-prior-art.md)にまとめる。ここではその違いからPoolが採る境界だけを定める。

Cの`malloc`が返すものはuninitializedなbyte storageであり、pointerと明示的な解放責任を伴う。LLVMのallocated objectは
provenance、size、lifetimeを持ち、`realloc`相当のoperationは同じaddressを返しても新しいallocated objectを作って旧objectを
無効にする（[LLVM Language Reference](https://llvm.org/docs/LangRef.html#allocated-objects)）。

したがって次のidentityを同一視しない。

```text
Pool semantic identity
!= managed Pool runtime object
!= current backing allocation
!= slot coordinate
!= value stored in a slot
```

`grow`はPool identityと既存coordinateを保存するが、backing allocationやslot addressを保存しない。runtimeは新しいC allocationへ
carrierを移してよく、LLVM backendはgrowth後にbacking pointerを再取得する。slot pointerをprimitive invocationの外へ出さないことで、
Rustのinterior referenceを調停するようなborrow/lifetime mechanismを追加せずにrelocationできる。

Live/VacantもCまたはLLVMのobject lifetimeではない。`Slot<V>`というMal valueの区別であり、runtimeはoccupancy tagとpayload storageへ
lowerできる。`llvm.lifetime.start`と`llvm.lifetime.end`をslot stateの意味として使わない。

## 他systemとの比較から採る境界

[関連事例](../../../research/pool-storage-prior-art.md)との比較から、Pool案は次の境界を採る。

- Rustの`Allocator`に当たるraw block policyはC runtimeへ閉じ、Poolをallocator parameter付きの型にしない。
- Rustの`MaybeUninit<T>`に当たるsource categoryを加えず、Vacantを既存のsum `Unit + V`で表す。
- Rustのinterior referenceに当たるslot pointerを発行せず、coordinateをPool identityに相対化してgrowth後も使う。
- Rustのpointer、Haskellのreference、Swiftのclass referenceと同様にhandleをcontainerへ保存できるvalueとする。
- Swift Arrayと同様にsnapshotをelement carrierについてstructuralとし、handle referentのdeep immutabilityを含めない。
- Haskellのmutable/immutable arrayと同様にIxPoolとImPoolを分けるが、`State# s`に当たるsource valueは加えない。
- Haskellの`ST s`のようにIxPool identityのescapeを禁止せず、既存Bufferと同じ共有観測を保つ。
- RustやLinear Haskellのuniquenessをsource authorityにせず、compilerのresponsibility移動とruntimeの観測不能なstorage再利用に分ける。

この選択は他systemの機能を省略した一覧ではない。malが既に持つstrict evaluation、shared Buffer identity、thread-confinedなEngram、
execution responsibilityを再利用し、新しい独立contractを作らないための境界である。

## 採択を判断する条件

Poolを採択するには、operationが動くことだけでなく次を満たす必要がある。

1. Buffer固有のstorage contractをPool state algebraとBuffer invariantへ完全に分解できる。
2. IxPoolとImPoolが同じstateとoperation lawを共有し、違いをhandleとsnapshotの観測則だけで説明できる。
3. handleを含むStorable valueのread、exchange、終了が既存responsibility規則から導ける。
4. Pool identityをC/LLVM allocation identityへ依存させず、growth後のpointer再取得をlowering contractにできる。
5. 初回採択ではPoolをextern signatureへ出さず、C連携を現行`Buffer`と`Symbol`のruntime carrierへ閉じられる。将来Poolを
   admissionする場合もD098のdirect carrier contractを拡張する独立判断にできる。
6. containerのinvariantをPoolへ取り込まず、opaque型の宣言元へ置ける。
7. Buffer上のemulationより増えるoccupancy costと、減るShare、Drop、番兵costを測定できる。
8. source authority、compiler responsibility、runtime representationのどの層も、下位層の一意性やlayoutを上位層の意味として
   逆輸入しない。
9. 現行BufferとPoolが同じ`Storable`を使い、handle nestingだけを禁止する歴史的なelement制限を残さない。

この条件を満たさない場合、Poolという型を追加するだけではminimalityの改善にならない。
