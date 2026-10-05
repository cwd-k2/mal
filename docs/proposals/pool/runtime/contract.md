# runtime contract

Status: Exploratory support document; rebased on mal v0.7

この文書は、Pool stateをtrusted layerがどう保持し、semantic identityを物理allocationからどう分け、responsibilityをどう動かし、
どの条件を誰が保証するかを管理する。Poolのauthorityは[位置付けと根本モデル](../model/foundations.md)、operation lawは
[意味論](../model/semantics.md#operation-law)、primitiveの一覧は[Pool primitive](../api/pool.md)、型形成条件は
[identity](../model/identity.md#storableの原理)、現在の規範は[`Buffer`](../../../spec/memory.md)、
[実行意味論](../../../spec/execution.md)、[managed valueのownership](../../../implementation/ownership.md)を正とする。

## authority boundary

IxPoolとImPoolはmal-controlledなEngramである。runtimeがC allocatorを使うことはimplementation mechanismであり、Pool constructionを
extern operation、allocator capability、またはExtern-owned resourceにしない。Poolのlifetimeはmanaged
responsibilityが支配し、C pointerの保持や明示的な`free`をsource contractへ出さない。

Poolはallocation policyを選ばない。recoverable allocation failure、program固有のarena、hostと共有するmutable storageが必要な場合は、
Poolへallocator parameterを足すのではなく、そのauthorityを所有する別のextern contractとして検討する。現在のPool allocation failureと
size overflowは既存Engram allocationと同じterminalなtrapである。

## semantic identityとallocation object

実装は次を別のものとして扱う。

```text
IxPool semantic identity
managed Pool runtime object
Header carrier
occupancy storage
payload backing allocation
slot coordinate
slotに保持したvalue
```

IxPool handleが共有するのはsemantic identityであり、payloadのaddressやC/LLVM allocation objectではない。初期実装はstableなmanaged
Pool objectからcurrent backing allocationを指す。`grow`は同じsemantic identityと既存coordinateを保存しながらbacking allocationを
置き換えてよい。

LLVMが認識するallocation objectは大きさを変えない。Cの`realloc`またはallocate-and-moveを使う場合も、growth成功後のbacking
storageは新しいallocation objectとして扱い、古いpointerを同じaddressへ再配置された場合にも再利用しない。backendとruntimeは次を守る。

- slot payloadへのpointerをPool primitive invocationの外へ返さない。
- growthし得るcallの後はactive backing pointerをmanaged Pool objectから再取得する。
- cached pointer、capacity、occupancy viewをgrowthとwritable-successor copyで失効させる。
- coordinate `i`は物理addressではなくPool identityに相対的な値として解釈する。
- `llvm.lifetime.start`と`llvm.lifetime.end`をLive/Vacantの意味として使わない。

Live/VacantはLLVM object lifetimeではなく`Slot<V>`のvariantである。runtimeはoccupancyとpayloadを別々に保持できるが、Vacant payloadを
`V`としてloadせず、LiveからVacantになるときだけ保持していた`V`のresponsibilityを移動またはDropする。

## Runtime representation

`IxPool<Header, V>`は`Header`のcarrier、logical capacity `n`、`V`のslot storage、各slotの占有tagを一つのmanaged identityとして
所有する。この「一つ」はsemantic identityとlifetimeを指し、単一のC allocationや連続layoutを要求しない。Headerはruntime value
representationで保持する。`Slot<V>`のtagはslotの外の占有tagとして持ち、slot storageにはLiveの値だけを置く。

`ImPool<Header, V>`は同じ論理fieldをmanaged snapshot carrierとして持つ。runtime objectやbacking allocationに実装上のidentityがあっても、
ImPool自身のshared mutable identityをsourceへ公開しない。複数のsnapshotが同じphysical storageを共有してよく、更新はwritable
successorを介して以前のsnapshotのHeader、capacity、slot carrierから独立させる。保存したhandle carrierのreferentは共有してよい。

Headerとslot storageは、現行Bufferと同じspecialization後のruntime carrier layoutを使う。primitive、external opaque carrier、
product、sum、Symbol、Buffer、Pool handleのsize、alignment、field offset、sum tagとpayloadをtarget layoutから決め、lifecycleは
`Trivial | Owned(share, drop)`を独立に選ぶ。paddingとinactive sum payloadをportableな値表現として読まない。

初回採択ではPool carrierをextern signatureや`mal.h`へ出さない。将来admitする場合もgenerated C headerとLLVM moduleが同じruntime
carrier planを共有し、D098のborrow、move、share、dropを適用する。別のcanonical memory representationは作らない。
`IxPool<Header, UInt8>`と`Symbol`のbyte owner間でstorageを移す特殊化は、source semanticsとlifecycleを保つas-if実装に限る。

`size<V> == 0`または`stride<V> == 0`でもslotは消滅しない。element payloadのbyte数が0でも、`n`、VacantとLiveの遷移、
precondition、drop回数は通常の`V`と同じである。占有tagは`peek`と`isLive`の結果、`slot`で旧値をDropするかの判定、IxPool終了時に
Dropするslotの決定に使う。LiveかVacantかを仮定する周辺operationはtagを検査しない。

`grow(pool, k)`はHeaderと全slotを保存してlogical capacity `n`を`k`だけ広げ、増えたslotのtagをVacantにする。物理的な
over-allocationとphysical capacityは観測させない。
`n + k`またはstorage sizeをtargetで表現できない場合とallocationに失敗した場合は、既存Engram allocationと同じくtrapする。
[trap](../../../spec/execution.md#trap)はterminalなので、失敗後のIxPool状態を公開する規則は要らない。IxPoolとmanaged valueは現在の
C runtime contextと同じくthread-confinedであり、物理relocation中の一時状態は一つのprimitive内部へ閉じる。

### BufferとVectorの表現

現行runtimeのBuffer（`MalBuffer`）は、IxPool上のBufferをas-ifで実装したものとして説明できる。

| 現行runtime | Pool上の意味 |
|---|---|
| `count` | Header |
| inlineまたはflat storageの物理容量 | slot数`n`。`make`のcapacityは確保量の要求であり、sourceから観測できない |
| 占有tagを持たない | invariant `[0, count)`がLiveからslot状態が決まる |
| managed要素の`retain`と`release` callback | 要素型ごとのlifecycle glue |
| 終了時に`[0, count)`をrelease | IxPool終了時のLive slotのDrop。tagを走査しない |
| `put`が新しいresponsibilityを移してから旧値をrelease | placeのwrite |
| `fill`、`copy`がcountを範囲末尾まで延ばす | 参照実装の`initAt`、`putAt`のloopとHeaderの更新 |
| extern Cからの操作 | 現行Buffer carrierを`mal.h`のstorage contractで直接扱う。Poolを経由しない |

pointer一個分以下の初期Buffer storageはstable object内に置き、それより大きい初期storageとgrowth後のstorageは`Symbol`と同じ形の
flatなbyte ownerに置く。inline storageはgrowth時にflat ownerへ移る。このsmall-buffer表現は、semantic identityとbacking storageを
概念として分けることが別allocationを要求しない例である。`Symbol`は`(owner, data, length)`という不変のviewであり、
Vectorも要素型について一般化した`(owner, data, count)`で表せる。element carrierの列は不変なので、`slice`はcopyせずviewとして作ってよい。
その代わり、sliceは`Symbol`と同じく元のstorage全体を生かし続ける。実装上は`Symbol`と`Vector<UInt8>`でowner/view表現を共有できるが、
source-levelの型同一性は要求しない。Vectorも採択するならruntime carrierの密な列として置き、Owned elementのsnapshot copyでは
Shareする。別artifactとの交換形式はVector representationでなく明示的なcodecが定める。

`freeze`と`thaw`はこの二つの表現の間でbyte ownerを受け渡す。inputがlast useで、Bufferのidentityとbyte ownerに区別可能なaliasが
ないとruntimeが確認できる
とき、`freeze`はflat ownerをviewへ移し、`thaw`はviewが先頭から全体を覆うflatなownerを新しいBufferへ移す。inline storageを含む
それ以外はcopyする
（[freezeとthaw](../api/pool.md#freezeとthaw)）。byte列の`*`の両方向がこの規則の最初の例であり、現行runtimeは`*`の意味を変えずにこの形で実装している
（[managed valueのownership](../../../implementation/ownership.md)）。

## responsibility

IxPoolのHeaderと各slotはplaceであり、常に値を一つ持つ。値のresponsibilityはplaceが持ち、Vacantの`Unit`はresponsibilityを
持たない。これは[local slot](../../../implementation/ownership.md#slotとoperation)のinitialize、vacate、replaceと同じ状態であり、違いは
placeがIxPool identityの中にあり、実行時のcoordinateで選ばれることだけである。placeに対する操作は次の三つの規則で動き、
Headerとslotで同じである。核はreadとswapであり、writeはswapから導く。

| 規則 | place | responsibility |
|---|---|---|
| read | 変わらない | placeの値を`Share`してresultにする |
| swap | operandの値になる | operandのresponsibilityをplaceへ移し、旧値のresponsibilityをresultへ移す |
| write | operandの値になる | swapの結果をDropする。operandのresponsibilityをplaceへ移し、旧値をDropする |

swapはresponsibilityを移すだけで、型別の`share`も`drop`も呼ばない。writeはswapの結果をDropしたものであり、新しい値をplaceへ
置いてから旧値をDropする順序を持つ。同じmanaged valueを読み出して書き戻しても、旧値のDropが新しい値のreferentを解放しない。
operandのresponsibilityは、call siteのexecution ownershipが、operandを後で使うなら`Share`し、last useなら`Consume`して用意する。
swapのresultは通常のowned resultであり、使われなくなった時点で`Drop`される。

| operation | 規則 | primitive内のshare | primitive内のdrop |
|---|---|---|---|
| `peek` | slotのread | Liveなら1 | なし |
| `getAt`、`header` | read | 1 | なし |
| `isLive`、`capacity` | tagまたは`n`を読む | なし | なし |
| `slot`、`dropAt` | slotのwrite | なし | 旧値がLiveなら1 |
| `initAt` | 旧値がVacantのwrite | なし | なし |
| `putAt`、`setHeader` | 旧値が値を持つwrite | なし | 1 |
| `swap`、`swapHeader`、`takeAt`、`moveAt` | swap | なし | なし |
| `grow` | 既存carrierをrelocateし、新slotをVacantにする | なし | なし |
| IxPoolの終了 | Headerと全Live slotのDrop | なし | 1とLive slot数 |

Bufferの`fill`と`copy`は[参照実装](../containers/buffer.md#range-operation)のloopがこれらのoperationを呼ぶため、回数はその分解から
決まり、runtimeが一括処理で実装しても同じ回数にする。C runtime extensionからBufferを操作する場合も`mal_storage(T)`が同じ
ShareとDropを供給する。`symbol`は`UInt8`だけを扱うためelement lifecycle glueを必要としない。

### writable successor

ImPoolの各更新は、まずinputのwritable successorを作る。更新前のsnapshotに対する今後の構造観測と区別できない場合はstorageを
resultへ移し、区別できるreferenceがあれば新しいstorageを作ってHeaderと各Live slotをreadして置く。その後successorへ更新を行って返す。
共有時の更新はflatなslot carrierと占有tagを複製する。managed carrierはShareするだけでreferentをdeep copyしないが、処理量は
O(n)である。

source valueは更新callの後にも再利用でき、その場合compilerはcallへ渡すresponsibilityを`Share`する。last useなら`Consume`できるが、
これは物理storageの一意性を主張せず、inputのresponsibilityをsuccessorへ移してよいというpermissionだけを与える。

`Consume`に加え、runtime representationへの区別可能なreferenceが一つであることはstorageを移せる十分条件である。reference countは
このrepresentation uniquenessを確かめるwitnessの一つであり、source semanticsでも唯一の実装でもない。別の回収方式やより強い
compiler proofを使ってもobservableなImPool structural snapshotとこのcopy boundを保てればよい。

Rustの`Box<T>`に相当する単一のphysical ownerは、この層では現れ得るがsource authorityではない。backendはPool object、backing
allocation、またはpayload carrierを一つのresponsibilityで保持できる。IxPool handleとImPool snapshotの観測則は、その時点の
物理owner数から推論しない。

## 未検査precondition

slot coordinateとslot状態に関する条件は、[Buffer](../../../spec/memory.md#未検査precondition)と同じ未検査preconditionである。
runtimeはこれらを検査せず、違反時の実行結果を保証せず、trapへも写像しない。核の条件は範囲だけであり、周辺operationの一部が
LiveかVacantかを加える。

| operation | precondition |
|---|---|
| `peek`、`slot`、`swap`、`isLive`、`dropAt` | `index < capacity(pool)` |
| `initAt` | `index < capacity(pool)`かつslotがVacant |
| `getAt`、`putAt`、`takeAt` | `index < capacity(pool)`かつslotがLive |
| `moveAt(pool, source, destination)` | 両方が`capacity(pool)`未満、`source`がLive、`destination`がVacant |

`pool`、`grow`、`capacity`、`header`、`swapHeader`、`setHeader`はpreconditionを持たない。

preconditionの責任は二段に分かれる。container利用者はcontainerが公開するprecondition、例えばBufferの`index < #buffer`を満たす。
container実装は、公開preconditionを満たすcallから到達する全IxPool callがIxPool preconditionを満たすことをfile-local invariantで
保証する。例えばBufferの`new`が`count == capacity`で`grow`を忘れると、利用者が公開preconditionを守っても`initAt`が
`index < capacity`に違反する。これはBuffer実装の誤りである。公開preconditionを持たないoperation、例えばMapのlookupは、
`peek`の結果やHeaderで判定してからslot operationを呼ぶ。

IxPool preconditionへの違反は、範囲外のstorageへのaccessを起こし得る。周辺operationのLiveとVacantの条件への違反は、Vacant
carrierのread、同じresponsibilityの二重Drop、Live valueのleakを起こし得る。言語はこれを防がず、IxPoolを直接呼ぶcodeがmanaged
valueのmemory safetyを担う。containerはopaque型でrepresentationを隠すと、その責任を宣言元fileのinvariantへ集められる。実装は
testやdebug buildで範囲と占有tagを検査してよい。

複数primitiveからなるcontainer operationはtransactionではなく、invariantはreturn時に回復すればよい。lifecycle glueはmal codeを
実行しないため、primitive内部の一時状態へ再入しない。handle nestingだけでも、表現に寄与する再帰型とfunction storageがないため、
要素からそれを保持する同じcontainerへのowner back-edgeは作れない。ただし`hash`、`equal`、比較はhandle referentを変更して
container固有のkey invariantを壊し得る。caller-supplied closureは同じcontainerをcaptureして再入し得るため、containerは呼び出す前に
公開invariantを回復するか、operation contractで再入を禁止しなければならない。これらはStorable lifecycleでなくcontainer algorithmの
callback boundaryである。
