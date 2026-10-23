# pointerとIxPoolによるcontainer基盤

Status: Exploratory; rebased on mal v0.7

このproposalは、address arithmeticを持たない共有locationをpointer型`*A`としてstorage semanticsの基礎に置き、
`IxPool<A>`をindexから`*A`へ到達するmanaged identityとして定める。現在の`Buffer<A>`が一つの組み込み型として
持つ可変identity、占有状態、growth、dense sequence policyを分け、Map、Deque、heap、木などのcontainerを
mal sourceで記述できるようにする。

現行仕様は[`Buffer`](../../spec/memory.md)、managed responsibilityは
[D055](../../history/decisions/active/D055.md)と[D083](../../history/decisions/active/D083.md)を正とする。
このdirectoryは未採択の設計案であり、現在の言語仕様ではない。

## 中心モデル

```text
*A
  Aを置く、addressを持たない一つの共有locationへのpointer
  pointee stateはVacantまたはLive(A)

IxPool<A>
  grow可能な有限coordinate空間を持つshared identity
  pool.at(index)で、そのidentityに属する*Aへ到達する

opaque container
  *HeaderとIxPool<Element>へrelationとinvariantを加える
```

pointerをcopyしても同じplaceへ到達し、どのaliasからのstoreとtakeも共有観測する。`IxPool`をcopyすると
同じcoordinate空間へ到達する。同じpool identityとindexから得たpointerは同じplaceを指し、`grow`でphysical
storageが移動しても変わらない。

VacantとLiveはpointeeの状態であり、`Slot<A> = [Unit, A]`というsource valueを格納しない。runtimeはoccupancyを
bitmapなどで保持してよいが、sourceから物理表現を観測させない。`load(p)`と`take(p)`はLiveをpreconditionとし、
`store(p, value)`はVacantを初期化し、Liveを置換する。

## surfaceの核

```mal
p := place(initial);
value := load(p);
store(p, replacement);

storage := pool<A>();
grow(storage, requested);
element := load(storage.at(index));
store(storage.at(index), replacement);
```

pointer自体は通常のvalueであり、binding、parameter、result、aggregate、storableなplaceへ置ける。評価位置へ
現れただけではdereferenceしない。`load(p)`はAをShareして返し、`take(p)`はAをMoveしてVacantにする。
`store(p, value)`はvalueをStoreし、旧pointeeがLiveならDropする。pointer用のexpression記号、implicit
dereference、general lvalue、assignment operatorは導入しない。

## 文書の構成

- `model/`
  - [位置付けと根本モデル](model/foundations.md)：pointer、IxPool、containerのauthorityとminimality
  - [意味論](model/semantics.md)：暗黙store、pointee state、IxPoolの到達性identity、operation law
  - [responsibility](model/responsibility.md)：状態遷移とShare、Move、Drop
  - [identityと型判定](model/identity.md)：alias、`Storable`、lifecycle、extern admission
- `api/`
  - [pointerとIxPool API](api/pool.md)：source syntax、operation、precondition
  - [Bufferとsnapshot候補](api/buffer-vector.md)：既存機能を初回判断から分ける境界
- `runtime/`
  - [runtime contract](runtime/contract.md)：logical place、pool identity、backing allocation、lifecycle
  - [compilerとruntimeの実装](runtime/implementation.md)：lowering、carrier、検証順序
- `containers/`
  - [container overview](containers/overview.md)：metadata pointerとIxPoolを使う共通形
  - [Buffer相当](containers/buffer.md)：dense prefixを持つ参照構造
  - [列のcontainer](containers/sequences.md)：stack、Deque、heap
  - [keyで引くcontainer](containers/keyed.md)：Map、SlotMap、木
- [試作から引き継ぐ証拠](prototypes.md)：旧Header/Slot kernelから残る証拠と再検証が必要な境界

## 推奨する最小採択単位

最初のlanguage changeは次に切る。

```text
pointer type                           *A

vacant<A>()                           -> *A
place<A>(A)                           -> *A
isLive<A>(*A)                         -> Bool
load<A>(*A)                           -> A
take<A>(*A)                           -> A
store<A>(*A, A)                       -> Unit

pool<A>()                             -> IxPool<A>
grow<A>(IxPool<A>, USize)             -> Unit
capacity<A>(IxPool<A>)                -> USize
at<A>(IxPool<A>, USize)               -> *A
```

constructor名はproposal内の作業名である。`load`と`take`はpointeeがLive、`at`はindexがcapacity未満で
あることを未検査preconditionとする。`store`はVacantとLiveの両方で定義し、Liveなら旧値をDropする。`grow`は
Vacantなplaceを末尾へ追加し、既存pointerの指すlogical placeを保存する。

`*A`と`IxPool<A>`は初回案では`Storable(A)`を要求する。これは現在のreference-counted回収で型から見えない
owner back-edgeを導入しないための採択境界であり、pointerがexclusiveまたはborrowedであることを意味しない。

次の機構はこの最小surfaceだけから意味が決まらないため、初回採択では定義しない。この一覧は言語からの恒久的な
除外を定めるものではなく、必要になった機構は固有のauthority、lifecycle、failureを持つ後続proposalとして判断する。

- pointer arithmetic、pointer ordering、pointer equality、integer/address conversion、明示的な`free`
- pointee projection、pointerでguardされたrecursive type、weak pointer
- immutable pool snapshot、`ImPool`、Vector、`freeze`、`thaw`
- allocator parameter、arena、recoverable allocation failure
- pointerまたはIxPoolのextern signature admission
- range transfer、Live pointer iterator、checked APIとunchecked APIの二重化

container metadataはIxPoolへ組み込まず、通常のplaceへ置く。例えばMapのfile-local opaque representationは
`(*Count, IxPool<Entry>)`にも`*(Count, IxPool<Entry>)`にもできる。前者は二carrierを直接共有するためpool identityを
固定し、後者は共有descriptorを通じてpool handle自体を交換できる。内部pointerを公開しない限り、このproposalは
どちらをcontainer一般の表現としても固定しない。

opaque containerが内部の`*Element`を返すと、利用者へ`load`だけでなく`store`と`take`のauthorityも渡り、container
invariantを迂回できる。従って内部pointerは、意図してそのauthorityを公開するAPIでない限りopaque境界からescape
させない。metadataとstorageを同じallocationへ置く最適化は可能だが、二つのlogical identityとそれぞれのlifecycleは
保つ。

## 既存prefix `*`との境界

現行v0.7はexpression位置のprefix `*`をBuffer/Symbolのsnapshot変換として定めている。本proposalが追加する
`*A`はtype位置だけのpointer constructorであり、pointer dereferenceには使わない。従って二つはgrammar contextで
区別でき、Buffer/Symbol変換の移行をpointer coreの採択条件にはしない。

## 後続判断

- `*A` carrierをownerとcoordinateの二wordにするか、standalone pointerを別表現にするか。
- pointerでguardされたrecursive opaque typeを認めるか。認める場合もstrong cycleの自動回収は要件とせず、
  利用者が到達可能な間に`take`などでcycleを切る。
- `*A`をfunctionを含む任意のruntime typeへ広げるか。
- metadata placeとIxPoolがcontainerごとに払うallocationとretain/release costを許容できるか。
- IxPool由来のpointerが残る場合にpool全体を保持するcostを許容できるか。
- snapshotが必要になったとき、outstanding pointer aliasをcopy gateへどう数えるか。
- pointerとIxPoolをextern signatureへ認める実需があるか。
