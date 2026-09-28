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
[Pool-backed containerのsource sketch](container-examples.md)に分けて示す。storage layout、ownership effect、failure境界など、
この分離が成立するための低レイヤcontractは[Pool lifecycle contract](lifecycle-contract.md)で管理する。mutable identityと
copy-on-write valueの参照管理はoptional extensionとして[Pool array ownership example](array-ownership.md)で比較する。

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
`makePool`の値はinitial logical capacityとする。
live slot数を内部で保持してよいが、sequence length、挿入位置、最大live coordinateを表すpublic contractにはしない。

Poolはauto-growせず、container実装が必要なcapacityとgrowth policyを決めて`reserve`する。現在値より大きい
`poolReserve(pool, capacity)`が成功するとlogical capacityは指定値と等しくなり、runtimeが確保した余剰bytesを
`poolCapacity`から観測させない。`reserve`のallocation failureは
既存Engram allocationと同じ規則で扱い、元のState、capacity、live slotを保つ。fixed capacity、geometric growth、load factor、
bucket数、vacant coordinateの選択は上位containerのpolicyとする。

`poolInitAt`と`poolPutAt`のvalue operandはPool slotというowner successorを持つ。execution ownershipは、source responsibilityがcall後も
必要なら`Share`し、ownedなlast useなら`Consume`してslotへ移す。単一slot operationは常にborrowed valueを受けてruntime内でretainする
現在のBuffer contractへ固定しない。`poolGetAt`はslotを残すためresultを`Share`し、`poolTakeAt`はslotのresponsibilityをresultへ`Consume`する。
`poolDropAt`はresultを作らずslotを`Drop`する。`poolState`と`poolSetState`もStateの同じownership規則に従う。

この規則はretain callbackを全面的に不要にはしない。動的なlengthを持つ`fill`と`copy`は実行時に複数のowner successorを作り、
source slotを残すcopyは各destination分を`Share`する。これらをprogram非依存runtimeで行うなら型別share callbackが必要である。
Pool破棄と`dropAt`にはdrop callbackが必要であり、reserveによるrelocationだけがshare/dropを行わない。

growthによるrelocationはlogical coordinateを保ち、`share`または`drop`を発生させない。新しい値を成立させる前に
置換対象を破棄せず、途中のallocation failureで元のlive valueを失わない。allocation failureとtarget sizeで表現できないcapacityは、
既存Engram allocationと同じfatal resource failureまたはtrap規則に従う候補とする。

sourceから`Vacant` carrierを値として取得するoperation、任意addressへの`init`、manual `drop`、raw element pointerは提供しない。
公開Pool APIの正確な名前、Vacantまたは範囲外coordinateをsumで返すかtrapするか、live slot iterationをどの層が持つかは未決定である。

## Core外のextension

初期Pool APIはcoordinateによるslot accessだけで成立し、generation、stable handle、copy-on-writeを含めない。stable handleは
slot map、tree、graphが外部へkeyを返す実例から、Pool identity、slot generation、stale key、Pool破棄後の表現を独立して決める。
handleがPoolをretainするとelementから同じPoolへのowner cycleを作れるため、nonowning keyとliveなPool operandを組み合わせる案を
初期候補とする。

identity-bearing handleが存在すると、copy-on-writeがstorageを再利用したか複製したかでhandleの有効性が変わり得る。したがって
[array ownership example](array-ownership.md)のwritable-successor profileはstable handleを発行せず、両extensionを同一profileへ
自動合成しない。`Pool<State, T>`自身をStateまたはelementへ格納することも初期profileでは認めない。

## file-local opaque型

Poolを安全なcontainerへ閉じ込めるには、transparent aliasとは別にnominalなopaque型が必要になる。候補syntaxを次に示す。

```mal
opaque Buffer<T> :: _BufferRepresentation<T>;
```

この宣言は`Buffer<T>`へnominal identityを与える。representationをpackまたはopenできるauthorityは宣言元source fileだけが持ち、
`require`先へは移らない。同じfileの通常のmal bindingはauthorityを使って実装でき、operation側へ`opaque` markerを付けない。

```mal
makeBuffer<T> :: USize -> Buffer<T> := ...;
new<T> :: (Buffer<T>, T) -> USize := ...;
get<T> :: (Buffer<T>, USize) -> T := ...;
put<T> :: (Buffer<T>, USize, T) -> Unit := ...;
```

opaque型は宣言元fileでもrepresentationと型同一にしない。同じrepresentationを持つ二つのopaque型を混同しないよう、pack/openは
型identityを指定する明示的な構文またはchecker operationとする。pack/openはallocation、copy、新しいEngram identityを作らず、
同じrepresentation responsibilityをnominal boundary越しに受け渡す。具体的なsyntaxと、public signatureからrepresentationを
意図的に返すことを許すかは未決定である。

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
| file-local opaque identity | frontendのsyntax、resolve、type checkingに閉じ、unmanaged representationだけならD055/D080に依存しない |
| `Pool<State, T>`のstateとslot | D080の`initialize`、`replace`、`vacate`と同じcarrier invariantをruntime storageへ適用する |
| Pool primitiveのownership effect | function typeだけではowner successorを表せないため、trusted metadataをuse planとD083のparameter保持解析へ接続する必要がある |
| writable-successor fast path | owner-successor effectをD083のowned native entryへ伝播できればlast-use argumentをConsumeできるが、storage再利用はruntime uniqueness検査に依存する |
| managed Stateまたは`Symbol` element | D055の`Share`、`Consume`、`Drop`と、型別lifecycle glueをruntime callbackへmaterializeする境界に依存する |
| Bufferのmal実装 | Poolがmanaged elementを正しく扱えれば、Buffer固有のretain/release loweringには依存しない |
| plugin-defined Engram leaf | representation、lifecycle、artifact dependencyを登録する安定したtrusted extension contractが別途必要になる |

したがって、`Pool`をbuilt-in Engramとして先に実装し、既存compilerが知る`Storable(State)`と`Storable(T)`だけを受理することは、一般plugin ABIより前に
検証できる。ただしunmanaged scalarだけへ制限したPoolでは本案の重複削減を十分に確認できない。`Symbol`を含むmanaged elementを
扱う段階までに、[Engram lifecycle proposal](../engram-lifecycle-foundation.md)が示す型別`share`と`drop`を通常値、Pool callback、
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

1. opaque identity、file-local pack/open、別fileからの構築と分解の拒否をfrontend testで固定する。
2. backend内部にPoolのVacant/Live遷移を置き、unmanaged Stateとelementでreserve、init、put、take、dropを実行する。
3. `Symbol`とmanaged aggregateでshare/drop回数、relocation、allocation failure後の元value保持、Pool終了時のlive allocation 0を検査する。
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

- optionalなPool handleのsource型、nonowning identity、generation幅、failure resultとtrapの境界。
- live slot iteration、dense storage、bulk relocationのどこまでをcore外のextensionとして追加するか。
- opaque型のpack/open構文と、public APIがrepresentationを返せる範囲。
- Bufferのcanonical host copyに対するruntime-layout Poolの性能と、追加fast pathの要否。
- Pool callbackを既存Buffer callbackから一般化するか、共通lifecycle planを先に抽出するか。
- plugin crateのversion、reproducible build、artifact cache、runtime source選択のcontract。
