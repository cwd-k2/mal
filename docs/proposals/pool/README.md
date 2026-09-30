# Poolとopaque型によるcontainer基盤

Status: Exploratory

この文書は、現在の`Buffer<T>`が持つtyped storage mechanismとdense sequence policyを分離し、少数のtrustedな
`Pool<State, T>` primitive上でcontainerをmal sourceとして定義する案を管理する。現行仕様は
[AddressとBuffer](../../spec/memory.md)、managed responsibilityは[D055](../../history/decisions/active/D055.md)と
[D083](../../history/decisions/active/D083.md)、carrier lifecycleの現在の実装境界は
[D080](../../history/decisions/active/D080.md)を正とする。本案はまだsource syntax、`Pool` API、plugin ABIを定めない。

## 目的

現在の`Buffer<T>`はmal-owned storageに加えて、共有mutable identity、denseな`[0, count)`、append、index access、growth、range
operationを一つの組み込み型として持つ。この形はArrayまたはVectorとして有用だが、slot map、deque、heap、tree、graph、arenaなどを
追加するたびにallocation、relocation、element lifecycleを別の組み込みruntimeへ複製するか、Bufferのindexとrow encodingへ
container policyを押し込む必要がある。

本案は次の層を分ける。

```text
Engram lifecycle lowering
        ↓
Pool<State, T>: shared state、typed carrier、slot lifecycle
        ↓
opaqueなmal型: Buffer<T>、Deque<T>、Map<K, V>、Tree<T>、Graph<T>
        ↓
通常のmal code: container固有の公開operationだけを使う
```

利用者へraw storage、uninitialized value、manual `drop`を公開せず、上位containerの実装者もstride、alignment、retain、releaseを
型ごとに再定義しないことを目標とする。

ここで表出するのはbackendの物理表現ではなく、backendが既に管理するtyped carrier、`Vacant`/`Live`遷移、relocation、
`Share`/`Consume`/`Drop`というsemantic mechanismである。layout、lifecycle correctness、allocation failureはtrusted layerに残し、
logical State、denseまたはsparseな利用、growth、free-list、ordering、hashing、snapshot policyをmal側が選ぶ。このpolicyを選べず
Bufferと同じdense sequenceしか作れないならPoolを独立させる意味はなく、逆にraw memory操作まで開くことも本案の目的ではない。

候補となるPool primitiveと、Bufferおよびopen-address Mapのmal表現は
[Pool-backed containerのsource sketch](container-examples.md)に分けて示す。storage layout、precondition、failure境界など、
この分離が成立するための低レイヤcontractは[Pool lifecycle contract](lifecycle-contract.md)、各operationを`init`と`take`へ
分解したownership effectは[所有権primitive](ownership-primitives.md)で管理する。mutable identityと
copy-on-write valueの参照管理はoptional extensionとして[Pool array ownership example](array-ownership.md)で比較する。
既存の`examples/`をPool上へ移したときにsource、ownership、precondition、costの何が変わるかは
[既存exampleで見るPool化の差分](current-examples.md)、現行Bufferの全operationをPoolとcompiler primitiveで書いた形は
[Pool上のBuffer実装](buffer-implementation.md)で示す。

## Poolのauthority

`Pool<State, T>`はmal-controlledなEngramであり、共有mutableな`State`、typed storage、全slotのlifetime authorityを持つ。
Poolのaliasは同じidentityを共有する。Poolはcontainerの挿入位置、順序、logical size、free-list policyを決めず、各slotを
backend上の`Vacant`または`Live<T>`として保持する。公開operationは少なくとも次の遷移へ閉じる。

```text
poolSetState(State)     旧Stateを終了し、新Stateを共有identityへ置く
poolReserve(capacity)   slotの状態を変えずallocationだけを拡張する
poolInitAt(index, T)    Vacant -> Live<T>
poolPutAt(index, T)     Live<T> -> Live<T>
poolTakeAt(index)       Live<T> -> Vacant、既存responsibilityをresultへ移す
poolDropAt(index)       Live<T> -> Vacant、既存responsibilityを終了する
Poolの破棄             全Live<T>を終了してからstorageを解放する
```

Poolは`State`、allocation済みslotまたはchunk、各slotの状態を同じidentity内に保持する。capacityは現在initできる
coordinateの上限としてcontainer実装から観測できるlogical capacityであり、物理配置、over-allocation、growth単位は観測させない。
`makePool`の第2引数はinitial logical capacityとする。
live slot数を内部で保持してよいが、sequence length、挿入位置、最大live coordinateを表すpublic contractにはしない。

Poolはauto-growせず、container実装が必要なcapacityとgrowth policyを決めて`reserve`する。現在値より大きい
`poolReserve(pool, capacity)`はlogical capacityを指定値と等しくし、runtimeが確保した余剰bytesを`poolCapacity`から観測させない。
allocation failureと表現できないsizeは既存Engram allocationと同じくtrapする。fixed capacity、geometric growth、load factor、
bucket数、vacant coordinateの選択は上位containerのpolicyとする。

value operandをslotへ保存するoperationはowner successorを持ち、call後もsourceが必要なら`Share`、ownedなlast useなら`Consume`
になる。単一slot operationを、常にborrowed valueを受けてruntime内でretainする現在のBuffer contractへ固定しない。各operationの
effectと遷移順序は[所有権primitive](ownership-primitives.md)が所有する。動的なlengthを持つ`fill`、`copy`、
Pool破棄はprogram非依存runtimeで行うなら型別のshareまたはdrop callbackを必要とし、reserveによるrelocationだけがどちらも行わない。

slot coordinateとLive/Vacant状態に関する条件は、Bufferのindexと同じ[未検査precondition](lifecycle-contract.md#未検査precondition)である。
container利用者は公開preconditionを守り、container実装は自分のinvariantでPool preconditionを満たす。Poolは違反を検査もtrapもしない。
sourceから`Vacant` carrierを値として取得するoperation、任意addressへの`init`、manual `drop`、raw element pointerは提供しない。
公開Pool APIの正確な名前と、live slot iterationをどの層が持つかは未決定である。

## Core外のextension

初期Pool APIはcoordinateによるslot accessだけで成立し、generation、key、copy-on-writeを含めない。slot map、tree、graphが
外部へ参照を返す場合は、Poolをretainしない`Storable`なkeyを[Pool key extension](pool-keys.md)として追加する。slotを指す
`SlotKey`は既存のPoolをownerにでき、Pool自体を指す`PoolKey`はownerとなる`Arena`を別に要する。

identity-bearing keyが存在すると、copy-on-writeがstorageを再利用したか複製したかでkeyの有効性が変わり得る。したがって
[array ownership example](array-ownership.md)のwritable-successor profileはkeyを発行せず、両extensionを同一profileへ
自動合成しない。`Pool<State, T>`自身をStateまたはelementへ格納することも初期profileでは認めない。

## file-local opaque型

Poolを安全なcontainerへ閉じ込めるには、transparent aliasとは別に
[file-local opaque type](../../spec/types.md#file-local-opaque-type)を使う。syntaxを次に示す。

```mal
opaque Buffer<T> :: _BufferRepresentation<T>;
```

この宣言は`Buffer<T>`へcanonical declaration identityを与える。hidden representationを観察できるauthorityは宣言元source fileだけが
持ち、`require`先へは移らない。同じfileの通常のmal bindingは、専用のpack/open構文なしに`Buffer<T>`をPool operationへ渡し、
`Pool<USize, T>`を`Buffer<T>`が期待される位置から返せる。operation側へ`opaque` markerを付けない。

```mal
makeBuffer<T> :: USize -> Buffer<T> := ...;
new<T> :: (Buffer<T>, T) -> USize := ...;
get<T> :: (Buffer<T>, USize) -> T := ...;
put<T> :: (Buffer<T>, USize, T) -> Unit := ...;
```

opaque型は宣言元fileでもcanonical typeとしてrepresentationと同一にせず、generic specializationとoperation familyのkeyに
declaration identityを残す。宣言元fileのtype checkingだけがrepresentation viewを使い、同じrepresentationを持つ二つのopaque型を
混同しない。このviewはallocation、copy、新しいEngram identityを作らず、同じcarrier responsibilityを受け渡す。

mutable metadataはPoolの`State`として共有identity側に置く。例えばBufferは`Pool<USize, T>`をhidden representationとし、Stateを
logical countとして使える。Mapはsize、使用bucket数、rehash thresholdなどを別のStateに持てる。opaque alias自体ではなく、
`reserve`とslot lifecycleだけを提供するPool上へ各fileが異なるState invariantとgrowth policyを実装することが分離の目的である。

## Bufferと上位container

Poolがallocationとelement lifecycleを所有すれば、Bufferはdense sequence policyとしてmalで実装できる。`new`はgrowth policyを適用して末尾slotをinitし、
`get`は独立したresult responsibilityを作り、`put`はlive slotをreplaceする。`fill`、overlapping `copy`、snapshot conversionは
Poolの安全なoperationを組み合わせ、現在の評価順、alias、count semanticsを保つ。

同じ基盤から、generational slot map、deque、priority queue、hash table、tree、graphを別々のopaque型として定義できる。
element equality、hash、orderingはPoolやpluginへ埋め込まず、通常のfunction引数または
[operation family](../../spec/operation-families.md)のrequirementとして上位algorithmが要求する。

standard Bufferのcore operationをdownstream sourceが同じidentityのまま上書きする仕組みは導入しない。実装はmalで書けるが、
一つのopaque型のrepresentation invariantと公開operationは宣言元fileが所有する。別policyは別のopaque型として定義する。

## Managed Engramとの依存関係

本案の各部分は同じ段階を要求しない。

| 部分 | managed Engramへの依存 |
|---|---|
| file-local opaque identityとrepresentation view | frontendのsyntax、resolve、type checkingに閉じ、unmanaged representationだけならD055/D080に依存しない |
| `Pool<State, T>`のstateとslot | D080の`initialize`、`replace`、`vacate`と同じcarrier invariantをruntime storageへ適用する |
| Pool primitiveのownership effect | function typeだけではowner successorを表せないため、trusted metadataをuse planとD083のparameter保持解析へ接続する必要がある |
| writable-successor fast path | owner-successor effectをD083のowned native entryへ伝播できればlast-use argumentをConsumeできるが、storage再利用はruntime uniqueness検査に依存する |
| managed Stateまたは`Symbol` element | D055の`Share`、`Consume`、`Drop`と、型別lifecycle glueをruntime callbackへmaterializeする境界に依存する |
| Bufferのmal実装 | Poolがmanaged elementを正しく扱えれば、Buffer固有のretain/release loweringには依存しない |
| plugin-defined Engram leaf | representation、lifecycle、artifact dependencyを登録する安定したtrusted extension contractが別途必要になる |

したがって、`Pool`をbuilt-in Engramとして先に実装し、既存compilerが知る`Storable(State)`と`Storable(T)`だけを受理することは、一般plugin ABIより前に
検証できる。ただしunmanaged scalarだけへ制限したPoolでは本案の重複削減を十分に確認できない。`Symbol`を含むmanaged elementを
扱う段階までに、[managed valueのownership](../../implementation/ownership.md)が定める型別`share`と`drop`を通常値、Pool callback、
closure environment destructorから共有できる必要がある。

## Plugin境界

pluginはPool container algorithmの必須実装場所ではない。初期段階ではcompilerとruntimeが`Pool<State, T>`を組み込み、Bufferその他をmalで
実装できる。境界が安定した後、Pool自体または新しいEngram leafをcompilerと同じversionへ静的に結合するtrusted crateへ移せるかを
評価する。

新しいEngram leafの登録には、layoutだけでなくvalid valueの構築、runtime representation、`share`、`drop`、relocation、保持する
child Engram、cycleを作らない根拠、`Storable`、`Representable`、`HostMappable`、runtime source選択が必要である。`Storable`を
登録するleafは、storage内のShareが安全で、後のmutationをaliasから観測させず、container edgeによるowner cycleを作らないことも示す。
shared mutable leafは初期profileで`Storable`にしない。任意のsource codeを
destructorとして登録せず、dropは失敗せずI/OやExtern resourceの`close`のような観測可能な作用を持たない。

この条件により、上位containerと通常のmal codeはplugin型を他の値と同じlifecycle規則で扱える。pluginが提供するExtern resourceの
lifetimeはEngram回収へ結合せず、従来どおり明示したhost operationが所有する。

## 段階的な検証

1. opaque identity、file-local representation view、別fileからの構築と分解の拒否をfrontend testで固定する。
2. backend内部にPoolのVacant/Live遷移を置き、unmanaged Stateとelementでreserve、init、put、take、dropを実行する。
3. `Symbol`とmanaged aggregateでshare/drop回数、relocation、同じvalueの書き戻し、Pool終了時のlive allocation 0を検査する。
4. Pool上に実験的なdense containerをmalで実装し、現在のBufferとalias、range、overlap、trap semanticsを比較する。
5. handleを持たないimmutable arrayでwritable successorを検証し、shared時のcopyとlast-use時のstorage再利用を別々に測る。
6. slot mapまたはtreeをcoordinateで実装し、stable handleが実際に必要ならidentity、generation、stale/cross-Pool rejectionを独立して検査する。
7. semanticsと生成物のcostが妥当な場合だけ、predefined Bufferの置換とtrusted crate境界を別々に判断する。

## 非目標

- raw allocation、pointer arithmetic、manual `init`、manual `drop`、`free`を一般mal codeへ公開しない。
- Poolだけを理由にborrow checker、linear type、user-defined finalizer、tracing GCを導入しない。
- `Representable(T)`やpublic C carrierを`Storable(T)`から自動的に導かない。
- plugin ABI、dynamic loading、package system、friend fileをPoolの初期実装条件にしない。
- 現在のBufferを、semantic parityと代表的なperformance測定なしに置き換えない。

## 未決定事項

- [Pool key extension](pool-keys.md#未決定事項)のgeneration幅、iteration、Arena State、key equality。
- live slot iteration、dense storage、bulk relocationのどこまでをcore外のextensionとして追加するか。
- 任意coordinateのVacantを持つPoolと、Vacantを末尾だけに限ったdense primitiveへ`[Unit, T]` elementを載せる形の比較。前者は
  占有metadataとPool終了時の走査を、後者はslotごとのsum tagと空値の書き込みを払う。
- opaque型のdiagnosticと、public APIがrepresentationを返せる範囲。
- `from`、`into`、`*buffer`、`*symbol`はcore Pool APIだけで書けない。[Buffer実装](buffer-implementation.md)が仮定する
  range、host、Symbol primitiveを採るか、これらをBuffer固有のpredefined operationとして残すかを決める。
- immutableな`Array<T>`を`Storable`にするため、更新がsuccessorを返す
  [`ValuePool<State, T>`](array-ownership.md#storableなimmutable-array)をidentity共有のPoolと別の型として持つか。
- Pool callbackを既存Buffer callbackから一般化するか、共通lifecycle planを先に抽出するか。
- plugin crateのversion、reproducible build、artifact cache、runtime source選択のcontract。
