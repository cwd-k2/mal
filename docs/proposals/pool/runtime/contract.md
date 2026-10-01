# runtime contract

Status: Exploratory support document

この文書は、Poolをtrusted layerがどう保持し、所有権をどう動かし、どの条件を誰が保証するかを管理する。primitiveの一覧は
[Pool primitive](../api/pool.md)、型形成条件は[identity](../model/identity.md#型形成条件)、現在の規範は[AddressとBuffer](../../../spec/memory.md)、
[実行意味論](../../../spec/execution.md)、[managed valueのownership](../../../implementation/ownership.md)を正とする。

## Runtime representation

`IxPool<Meta, V>`は`Meta`のcarrier、`n`、`V`のslot storage、各slotの占有tagを一つのmanaged identityとして所有する。Metaは
runtime value representationで保持する。`Slot<V>`のtagはslotの外の占有tagとして持ち、slot storageにはLiveの値だけを置く。
slot storageのlayoutは要素型ごとに実装が選び、sourceとhostへ観測させない。初期実装は
[現行Buffer](../../../implementation/ownership.md#bufferのelement)と同じく次を使い分ける。

- `Representable`な`V`はcanonical memory layoutで置く。host境界のcopyはbulk copyになり、lifecycle glueを持たない。
- `Symbol`を含む`V`はruntime value representationで置き、型別の`share`と`drop` glueを使う。plugin-defined Engram leafを含む
  要素も同じ側に置き、同じlifecycle planを適用する。

この選択はcorrectness contractではない。canonical layoutはhostとの値交換の形式であり、IxPool storageがそれと一致することを
sourceもhostも前提にしない。`IxPool<Meta, UInt8>`のslot storageはbyte列そのものになるため、`Symbol`のbyte ownerとstorageを
共有する特殊化も実装の選択として取れる。

`Host<A>`はcanonical layoutの密な列を一つの不変なstorageとして持ち、占有tagもlifecycle glueも持たない。`Host<UInt8>`のstorageは
`Symbol`のbyte ownerと同じ形を取れる。

`size<V> == 0`または`stride<V> == 0`でもslotは消滅しない。element payloadのbyte数が0でも、`n`、VacantとLiveの遷移、
precondition、drop回数は通常の`V`と同じである。占有tagは`peek`と`isLive`の結果、`slot`で旧値をDropするかの判定、IxPool終了時に
Dropするslotの決定に使う。LiveかVacantかを仮定する周辺operationはtagを検査しない。

`grow(pool, k)`はMetaと全slotを保存して`n`を`k`だけ広げ、増えたslotのtagをVacantにする。物理的なover-allocationは観測させない。
`n + k`またはstorage sizeをtargetで表現できない場合とallocationに失敗した場合は、既存Engram allocationと同じくtrapする。
[trap](../../../spec/execution.md#trap)はterminalなので、失敗後のIxPool状態を公開する規則は要らない。IxPoolとmanaged valueは現在の
C runtime contextと同じくthread-confinedであり、物理relocation中の一時状態は一つのprimitive内部へ閉じる。

## 所有権

IxPoolのMetaと各slotはplaceであり、常に値を一つ持つ。値のresponsibilityはplaceが持ち、Vacantの`Unit`はresponsibilityを
持たない。これは[local slot](../../../implementation/ownership.md#slotとoperation)のinitialize、vacate、replaceと同じ状態であり、違いは
placeがIxPool identityの中にあり、実行時のcoordinateで選ばれることだけである。placeに対する操作は次の三つの規則で動き、
Metaとslotで同じである。核はreadとswapであり、writeはswapから導く。

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
| `getAt`、`meta` | read | 1 | なし |
| `isLive`、`capacity` | tagまたは`n`を読む | なし | なし |
| `slot`、`dropAt` | slotのwrite | なし | 旧値がLiveなら1 |
| `initAt` | 旧値がVacantのwrite | なし | なし |
| `putAt`、`setMeta` | 旧値が値を持つwrite | なし | 1 |
| `swap`、`swapMeta`、`takeAt`、`moveAt` | swap | なし | なし |
| `grow` | 遷移なし。全carrierを移動するだけ | なし | なし |
| IxPoolの終了 | Metaと全Live slotのDrop | なし | 1とLive slot数 |

Bufferの`fill`と`copy`は[参照実装](../containers/buffer.md#range-operation)のloopがこれらのoperationを呼ぶため、回数はその分解から
決まり、runtimeが一括処理で実装しても同じ回数にする。`Host<A>`のoperationは`Representable`な型か`UInt8`だけを扱い、`share`と
`drop`はno-opなので所有権解析へ入力を持たない。

この表はIxPoolのstorageが共有されていない場合の回数である。[freeze](../api/pool.md#freezeとthaw)がstorageをImPoolと共有する案を
採ると、共有中のIxPoolへの最初の書き込み、つまり読み出し以外のoperationは、先にwritable successorと同じ複製を行い、Metaと
各Live slotを一回ずつ`Share`する。

### writable successor

ImPoolの各更新は、まずinputのwritable successorを作る。inputが唯一のresponsibilityならstorageをresultへ移し、共有中なら新しい
storageを作ってMetaと各Live slotをreadして置き、inputのresponsibilityをDropする。その後successorへ更新を行って返す。共有時の
更新はflatなslot carrierと占有tagを複製し、managed `V`のpayloadをdeep copyしないが、処理量はO(n)である。

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

`pool`、`grow`、`capacity`、`meta`、`swapMeta`、`setMeta`はpreconditionを持たない。

preconditionの責任は二段に分かれる。container利用者はcontainerが公開するprecondition、例えばBufferの`index < #buffer`を満たす。
container実装は、公開preconditionを満たすcallから到達する全IxPool callがIxPool preconditionを満たすことをfile-local invariantで
保証する。例えばBufferの`new`が`count == capacity`で`grow`を忘れると、利用者が公開preconditionを守っても`initAt`が
`index < capacity`に違反する。これはBuffer実装の誤りである。公開preconditionを持たないoperation、例えばMapのlookupは、
`peek`の結果やMetaで判定してからslot operationを呼ぶ。

IxPool preconditionへの違反は、範囲外のstorageへのaccessを起こし得る。周辺operationのLiveとVacantの条件への違反は、Vacant
carrierのread、同じresponsibilityの二重Drop、Live valueのleakを起こし得る。言語はこれを防がず、IxPoolを直接呼ぶcodeがmanaged
valueのmemory safetyを担う。containerはopaque型でrepresentationを隠すと、その責任を宣言元fileのinvariantへ集められる。実装は
testやdebug buildで範囲と占有tagを検査してよい。

複数primitiveからなるcontainer operationはtransactionではなく、invariantはreturn時に回復すればよい。lifecycle glueはmal codeを
実行しない。`hash`や`equal`のようなoperation requirementは要素型の値しか受け取らず、malは再帰型を持たないため要素型の値は
それを要素とするcontainerを含めない。top-level initializerはIxPoolを作れない。したがってこれらから変更中のcontainerへ到達
できない。この議論は[`Storable`と`Stable`の分割案](../model/identity.md#判定の分割案)でIxPoolを要素にできるようになっても変わらない。
caller-suppliedなclosureを受け取るoperationはclosureが同じcontainerのaliasをcaptureし得るため、呼び出し前にinvariantを回復する。

