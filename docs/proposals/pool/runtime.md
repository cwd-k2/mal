# runtime contract

Status: Exploratory support document

この文書は、Pool案のprimitiveをtrusted layerがどう保持し、所有権をどう動かし、どの条件を誰が保証するかと、それを実装する
compilerとruntimeの分担を管理する。primitiveの一覧は[primitive一覧](primitives.md)、型形成条件は
[identity](identity.md#型形成条件)、現在の規範は[AddressとBuffer](../../spec/memory.md)、[実行意味論](../../spec/execution.md)、
[managed valueのownership](../../implementation/ownership.md)を正とする。

## Runtime representation

`IxPool<State, T>`は`State`のcarrier、`T`のslot storage、各slotの`Vacant`または`Live`状態を一つのmanaged identityとして所有する。
Stateはruntime value representationで保持する。slot storageのlayoutは要素型ごとに実装が選び、sourceとhostへ観測させない。
初期実装は[現行Buffer](../../implementation/ownership.md#bufferのelement)と同じく次を使い分ける。

- `Representable`な`T`はcanonical memory layoutで置く。host境界のcopyはbulk copyになり、lifecycle glueを持たない。
- `Symbol`を含む`T`はruntime value representationで置き、型別の`share`と`drop` glueを使う。plugin-defined Engram leafを含む
  要素も同じ側に置き、同じlifecycle planを適用する。

この選択はcorrectness contractではない。canonical layoutはhostとの値交換の形式であり、IxPool storageがそれと一致することを
sourceもhostも前提にしない。`IxPool<State, UInt8>`のslot storageはbyte列そのものになるため、`Symbol`のbyte ownerとstorageを
共有する特殊化も実装の選択として取れる。

`size<T> == 0`または`stride<T> == 0`でもslotは消滅しない。element payloadのbyte数が0でも、capacity、`Live`/`Vacant`遷移、
precondition、drop回数は通常の`T`と同じである。占有状態は`isLive`の結果とIxPool終了時にDropするslotの決定に使い、slot
operationごとの検査には使わない。

`reserve`は全Stateとslotを保存して指定されたlogical capacityへ拡張し、要求が現在のcapacity以下ならIxPoolを変更しない。
物理的なover-allocationは観測させない。storage sizeをtargetで表現できない場合とallocationに失敗した場合は、既存Engram
allocationと同じくtrapする。[trap](../../spec/execution.md#trap)はterminalなので、失敗後のIxPool状態を公開する規則は要らない。
IxPoolとmanaged valueは現在のC runtime contextと同じくthread-confinedであり、物理relocation中の一時状態は一つのprimitive内部へ
閉じる。

## 所有権

IxPoolのslotとStateは、どちらも`Vacant`または`Live`のplaceである。Liveなplaceはちょうど一つのresponsibilityを持つ。これは
[local slot](../../implementation/ownership.md#slotとoperation)のinitialize、vacate、replaceと同じ状態であり、違いはplaceが
IxPool identityの中にあり、実行時のcoordinateで選ばれることだけである。所有権を動かす遷移は次の二つだけである。

| 遷移 | place | responsibility |
|---|---|---|
| `init(place, value)` | Vacant → Live | operandのresponsibilityをplaceへ移す |
| `take(place)` | Live → Vacant | placeのresponsibilityをresultへ移す |

どちらもresponsibilityを移すだけで、型別の`share`も`drop`も呼ばない。`init`へ渡すresponsibilityは、call siteのexecution
ownershipが、operandを後で使うなら`Share`し、last useなら`Consume`して用意する。`take`のresultは通常のowned resultであり、
使われなくなった時点で`Drop`される。したがってsourceへ一般の`drop(value)`を公開する必要はない。

他のoperationは`init`と`take`の合成として定義する。primitiveとして残すのは性能のため、および`get`のように派生形の`init`が
共有storageのcopyを起こすのを避けるためであり、`share`と`drop`の回数と順序は分解と一致しなければならない。

```text
get(place)          = v := take(place); init(place, v); v      // vを二度使うのでinitのoperandはShare
put(place, value)   = old := take(place); init(place, value); drop(old)
drop(place)         = drop(take(place))
move(source, dest)  = init(dest, take(source))
state(pool)         = get(pool.state)
setState(pool, s)   = put(pool.state, s)
```

`put`の分解は、新valueをplaceへ成立させてから旧valueをDropする順序を与える。同じmanaged valueを読み出して書き戻しても、
旧valueのDropが新valueのreferentを解放しない。

| operation | 分解 | primitive内のshare | primitive内のdrop |
|---|---|---|---|
| `initAt`、`takeAt`、`moveAt` | `init`、`take`、`move` | なし | なし |
| `getAt`、`state` | `get` | 1 | なし |
| `putAt`、`setState` | `put` | なし | 旧value 1 |
| `dropAt` | `drop` | なし | 1 |
| IxPoolの終了 | Stateと全Live slotの`drop` | なし | Live place数 |
| `reserve` | 遷移なし。全carrierを移動するだけ | なし | なし |

Bufferの`fill`と`copy`は[参照実装](buffer-implementation.md#range-operation)のloopがこれらの遷移を呼ぶため、回数はその分解から
決まり、runtimeが一括処理で実装しても同じ回数にする。`from`、`into`、`*`は`Representable`な型か`UInt8`だけを扱い、`share`と
`drop`はno-opなので所有権解析へ入力を持たない。

この表はIxPoolのstorageが共有されていない場合の回数である。[freeze](primitives.md#freezeとthaw)がstorageをImPoolと共有する案を
採ると、共有中のIxPoolへの最初の書き込み、つまり`getAt`、`isLive`、`capacity`、`state`以外のoperationは、先にwritable
successorと同じ複製を行い、Stateと各Live slotを一回ずつ`Share`する。

### writable successor

ImPoolの各更新は、まずinputのwritable successorを作る。inputが唯一のresponsibilityならstorageをresultへ移し、共有中なら新しい
storageを作ってStateと各Live slotを`get`して`init`し、inputのresponsibilityをDropする。その後successorへ`put`または`init`を行って
返す。共有時の更新はflatなslot carrierと占有metadataを複製し、managed `T`のpayloadをdeep copyしないが、処理量はO(length)である。

## 未検査precondition

slot coordinateとslot状態に関する条件は、[Buffer](../../spec/memory.md#未検査precondition)と同じ未検査preconditionである。
runtimeはこれらを検査せず、違反時の実行結果を保証せず、trapへも写像しない。

| operation | precondition |
|---|---|
| `isLive(pool, index)` | `index < capacity(pool)` |
| `initAt(pool, index, value)` | `index < capacity(pool)`かつslotがVacant |
| `getAt`、`putAt`、`takeAt`、`dropAt` | `index < capacity(pool)`かつslotがLive |
| `moveAt(pool, source, destination)` | 両方が`capacity(pool)`未満、`source`がLive、`destination`がVacant |

`makeIxPool`、`state`、`setState`、`capacity`、`reserve`はpreconditionを持たない。

preconditionの責任は二段に分かれる。container利用者はcontainerが公開するprecondition、例えばBufferの`index < #buffer`を満たす。
container実装は、公開preconditionを満たすcallから到達する全IxPool callがIxPool preconditionを満たすことをfile-local invariantで
保証する。例えばBufferの`new`が`count == capacity`で`reserve`を忘れると、利用者が公開preconditionを守っても`initAt`が
`index < capacity`に違反する。これはBuffer実装の誤りである。公開preconditionを持たないoperation、例えばMapのlookupは、
`isLive`とStateで判定してからslot operationを呼ぶ。

IxPool preconditionへの違反は、Vacant carrierのread、同じresponsibilityの二重Drop、Live valueのleakを起こし得る。言語は
これを防がず、IxPoolを直接呼ぶcodeがmanaged valueのmemory safetyを担う。containerはopaque型でrepresentationを隠すと、その責任を
宣言元fileのinvariantへ集められる。
実装はtestやdebug buildで占有状態を検査してよい。

複数primitiveからなるcontainer operationはtransactionではなく、invariantはreturn時に回復すればよい。lifecycle glueはmal codeを
実行しない。`hash`や`equal`のようなoperation requirementは要素型の値しか受け取らず、malは再帰型を持たないため要素型の値は
それを要素とするcontainerを含めない。top-level initializerはIxPoolを作れない。したがってこれらから変更中のcontainerへ到達
できない。この議論は[`Storable`と`Stable`の分割案](identity.md#判定の分割案)でIxPoolを要素にできるようになっても変わらない。caller-suppliedなclosureを受け取るoperationはclosureが同じcontainerのaliasを
captureし得るため、呼び出し前にinvariantを回復する。

## compilerとruntimeの分担

compilerのexecution ownershipに新しく要るのは、operand effectの`Store`だけである。

- `Store`は、operandのresponsibilityをprimitiveが保持することを表す。`init`と`put`のvalue、`makeIxPool`と`setState`のState、
  writable successorのinputとstorageを移し得る`thaw`のinputが該当する。
- use planは`Store`を`Share`または`Consume`へlowerする。[D083](../../history/decisions/active/D083.md)の保持解析は、`Store`へ渡る
  parameterをreturnやcaptureと同じく保持として扱い、Bufferの`put`のようなmal wrapperをowned native entryにする。
- IxPool handle、index、lengthは`Borrow`である。resultは全てownedである。

現行のBuffer operandは全て`Borrow`で、保存に必要なretainはruntimeが行う。これは`Store`を常にruntime内の`Share`として扱うことに
当たり、`Store`を導入するとlast useのvalueを`Consume`してruntime内のretainとcall site側のreleaseを省ける。

runtimeが型ごとに必要とするglueは、上の表で「primitive内」に数えたものだけである。`share<T>`は`get`、一括処理の`fill`と`copy`、
共有時のwritable successorとIxPoolの複製が、`drop<T>`は`put`の旧value、`drop`、IxPoolの終了が使う。relocationと`init`、`take`はcarrierを
移動するだけでglueを呼ばない。glueは失敗せず、I/O、host resourceの`close`、別IxPoolの更新など観測可能な作用を持たず、IxPool
終了時のdrop順はsourceから観測できない。

LLVM backendにはmanaged valueの型再帰的なretain/release、managed placeのinitialize/replace/vacate、Buffer elementごとの
callback生成が既にある。IxPoolはこのloweringの新しい利用者になり、別の型再帰を持たない。

### Ordinary externとの境界

現在のexternal opaque valueをmalのopaque型で包むだけではIxPoolにならない。external handleはcopyしてもreferentのlifetimeを
延長せず、mal側のdropもresourceを解放しないため、wrapperはuse-after-free、多重close、最後のaliasとstorage解放の対応を保証
できない。IxPoolの実装をextern-likeなruntime libraryへ置く場合も、IxPool handleはmal-controlledなEngram leafとして登録し、
compilerまたはtrusted extensionが次を供給する。

- operandとresultの`Borrow`、`Share`、`Consume`、`Drop` effect
- concreteな`State`と`T`のruntime layout、およびmaterializeした`share`と`drop` glue
- 最後のIxPool responsibilityで全Live slotとStateをdropするhandle lifecycle
- 非`HostMappable`なruntime carrierとlifecycle glueを渡せるprivate ABI

### Managed Engramとの依存関係

本案の各部分は同じ段階を要求しない。

| 部分 | managed Engramへの依存 |
|---|---|
| file-local opaque identityとrepresentation view | frontendに閉じ、unmanaged representationだけならD055/D080に依存しない |
| IxPoolのStateとslot | [D080](../../history/decisions/active/D080.md)の`initialize`、`replace`、`vacate`と同じcarrier invariantをruntime storageへ適用する |
| operand effect `Store` | trusted metadataをuse planとD083のparameter保持解析へ接続する |
| writable successorのstorage再利用 | `Store`をowned native entryへ伝播し、runtimeのuniqueness検査に依存する |
| managed Stateまたは`Symbol` element | D055の`Share`、`Consume`、`Drop`と、型別glueをruntime callbackへmaterializeする境界に依存する |
| Bufferのmal実装 | IxPoolがmanaged elementを正しく扱えれば、Buffer固有のretain/release loweringには依存しない |
| plugin-defined Engram leaf | representation、lifecycle、artifact dependencyを登録する安定したtrusted extension contractが要る |

IxPoolをbuilt-in Engramとして先に実装し、`Storable(State)`と`Storable(T)`だけを受理することは、一般plugin ABIより前に検証できる。
ただしunmanaged scalarだけのIxPoolでは重複削減を確認できないため、`Symbol`を含むelementを扱う段階までに、型別`share`と`drop`を
通常値、IxPool callback、closure environment destructorから共有できる必要がある。

境界が安定した後、IxPool自体または新しいEngram leafをcompilerと同じversionへ
静的に結合するtrusted crateへ移せるかを評価する。leafの登録には、layout、valid valueの構築、runtime representation、`share`、
`drop`、relocation、保持するchild Engram、runtime source選択と、operationごとの`Borrow`または`Store`と上の分解の宣言が要る。

## 検証の段階

1. opaque identity、file-local representation view、別fileからの構築と分解の拒否をfrontend testで固定する。
2. backend内部にIxPoolのVacant/Live遷移を置き、unmanaged Stateとelementで`init`、`take`とその派生operationを実行する。
   zero-sizedなStateとelement、capacity 0でもslot遷移とdrop回数が一致し、capacity overflowはtrapする。
3. `Symbol`とmanaged aggregateで、分解を直接実行するtest用runtimeとshare/drop回数と順序を比べる。relocation、同じvalueの
   書き戻し、同じBufferで範囲が重なる`copy`でDrop済みのreferentを読まず、IxPool終了時のlive allocationは0になる。
4. `Store`へ渡るparameterを持つmal wrapperがowned native entryになり、last-use argumentを`Consume`する。
5. IxPool上のBufferを現在のBufferとalias、range、overlap、trap semanticsで比べ、占有状態を検査するtest用runtimeで公開
   preconditionを満たすprogramがIxPool preconditionへ違反しないことを確かめる。canonical host copyはpaddingや非選択sum payloadへ
   依存せずround-tripする。
6. ImPool上のimmutable arrayで、shared時のcopyとlast-use時のstorage再利用を別々に測る。
7. 木やgeneration付きkeyのcontainerをcoordinateで実装し、coordinateの再利用と古いkeyの拒否を検査する。
8. semanticsと生成物のcostが妥当な場合だけ、predefined Bufferの置換とtrusted crate境界を別々に判断する。

compilerを変えない二つの試作が、step 5から7の一部を先取りした。結果は[試作で確かめたこと](prototypes.md)に置く。
