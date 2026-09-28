# Pool lifecycle contract

Status: Exploratory support document

この文書は、[Poolとopaque型によるcontainer基盤](README.md)が成立するために必要なstorage、ownership、failureの
低レイヤcontractを管理する。候補APIとcontainer algorithmは
[source sketch](container-examples.md)、現在の規範は[AddressとBuffer](../../spec/memory.md)と
[実行意味論](../../spec/execution.md)を正とする。

## Runtime representation

`Pool<State, T>`は`State`のcarrier、`T`のslot storage、各slotの`Vacant`または`Live`状態を一つのmanaged identityとして所有する。
初期profileでは`State`と`T`をruntime value representationで保持し、canonical memory layoutをPoolの内部表現にしない。
これにより`Symbol`を含むaggregateとplugin-defined Engram leafへ同じlifecycle planを適用できる。

canonical memoryはhostとの値交換に限る。Pool上のBufferが`from`または`into`を提供する場合、canonical elementとruntime slotを
fieldごとに変換する。現行Bufferと同等のbulk copyが必要なら、semantic parityを固定した後に型別fast pathとして評価し、Poolの
correctness contractへcanonical layoutを混ぜない。

`size<T> == 0`または`stride<T> == 0`でもslotは消滅しない。Poolはcoordinateごとの占有状態と、handle extensionを使う場合はgenerationを
別に保持する。element payloadのbyte数が0でも、capacity、`Live`/`Vacant`遷移、二重dropの拒否は通常の`T`と同じである。

## Ownership effect

Pool operationのfunction typeだけからowner successorを推論しない。built-in primitiveまたはtrusted plugin declarationが、各operandと
resultに次のeffectを付与し、execution ownershipがcall siteのlast-use情報と合わせて具体的な`Share`、`Consume`、`Drop`へlowerする。

| operation | lifecycle effect |
|---|---|
| `makePool(state, capacity)` | `state`を新しいPoolへShareまたはConsumeする |
| `poolState(pool)` | StateをPoolに残し、resultをShareする |
| `poolSetState(pool, state)` | 新StateをShareまたはConsumeした後、旧StateをDropする |
| `poolInitAt(pool, index, value)` | `value`をVacant slotへShareまたはConsumeする |
| `poolGetAt(pool, index)` | slotをLiveに保ち、resultをShareする |
| `poolPutAt(pool, index, value)` | 新valueをShareまたはConsumeした後、旧valueをDropする |
| `poolTakeAt(pool, index)` | slotのresponsibilityをresultへConsumeし、slotをVacantにする |
| `poolDropAt(pool, index)` | slotのresponsibilityをDropし、slotをVacantにする |
| `poolReserve(pool, capacity)` | carrierをrelocateするだけでShareまたはDropしない |
| Poolの終了 | Stateと全Live slotを一度ずつDropしてstorageを解放する |

`Share`と`Consume`の選択はsource operandがcall後も必要かで決まり、primitive名やC callback側で推測しない。primitive effectは
use planだけでなく、parameterをPool slotへ保持する通常のmal wrapperをowned native entryにできるようD083の保持解析にも入力する。
pluginが新しいPool相当operationを追加する場合は同じmetadataをtrusted boundaryで宣言する。

lifecycle glueは失敗せず、I/O、host resourceの`close`、別Poolの更新など観測可能な作用を持たない。Pool破棄時のStateとslotのdrop順は
sourceから観測できず、container algorithmはその順序へ依存しない。

## Primitive transitionとcontainer invariant

各Pool primitiveは一つの検査済み遷移として振る舞う。`reserve`は成功時に全Stateとslotを保存して指定されたlogical capacityへ
拡張し、allocation failureまたはcapacity overflowでは変更前のPoolを失わずtrapする。物理的なover-allocationは観測させない。
`replace`は新しいresponsibilityの成立前に旧valueを破棄しない。Vacant slotの`get`、
Live slotの`init`、範囲外coordinateをsumで返すかtrapするかはAPI決定事項だが、未初期化`T`を作ってはならない。

複数primitiveからなるBufferやMapのoperation全体はtransactionではない。container実装はallocationと、hashやequalityなど失敗または
中断し得る通常のcallを可能な限りmutation前に完了し、file-local opaque invariantをoperationのreturn時に回復する。trapはterminalであり、
trap後にsource executionへrollbackした状態を公開する必要はない。

Poolとmanaged valueは現在のC runtime contextと同じくthread-confinedであり、atomic containerやcross-thread synchronizationを提供しない。
物理relocation中の一時状態は一つのprimitive内部へ閉じ、mal callbackやplugin callbackへ公開しない。

## Propertyとopaque boundary

初期profileではPoolの形成に現在のclosed judgmentである`Storable(State)`と`Storable(T)`を要求する。Pool自体は`Storable`でも
`Representable`でも`HostMappable`でもなく、
Poolを包むopaque型がこれらのpropertyを新たに宣言してhidden representationの制約を迂回することもできない。opaque型のrequirementsと
lifecycleはcompilerがhidden representationから導く。

将来plugin leafを`Storable`へ追加する場合も、layoutとdropだけから自動導出しない。storage内のShareが安全であること、値carrierの
aliasが後のmal-owned mutationを観測しないこと、container edgeからowner cycleを作らないことを登録時に検査する。shared mutableな
Pool、Buffer、function、external opaque valueを除外する現在の制約を、opaque wrapperやplugin registrationで迂回させない。

opaqueのfile-local representation viewはPoolのallocation、copy、share、新しいidentityを暗黙に発生させない。ownership上は同じcarrier
responsibilityの受け渡しであり、必要なShareまたはConsumeはopaque境界の周囲にある通常のvalue useが決める。

Poolの`initAt`と`dropAt`は一般的なmanual memory operationではない。対象はPoolが所有する検査済みslotに限られ、raw address、
uninitialized carrier、取り出し後にも残る`T`へのreferenceをsourceへ公開しない。この制限により、低レイヤcontainer authorへslot lifecycleを
開きながら、一般の`drop(value)`やarbitrary placement initializationを導入せずに済む。

## Ordinary externとの境界

現在のexternal opaque valueをmalのopaque型で包むだけではPoolにならない。external handleはcopyしてもreferentのlifetimeを延長せず、
mal側のdropもresourceを解放しないため、wrapperはuse-after-free、多重close、最後のaliasとstorage解放の対応を保証できない。
`Borrow<T>`もcall中の有効性だけを表し、hostが`T`をslotへ保持する根拠にはならない。

Poolの実装自体をextern-likeなruntime libraryへ置くことはできる。その場合もPool handleはmal-controlledなEngram leafとして登録し、
compilerまたはtrusted extensionが次を供給する。

- operandとresultの`Borrow`、`Share`、`Consume`、`Drop` effect
- concreteな`State`と`T`のruntime layout、およびmaterializeした`share`と`drop` glue
- 最後のPool responsibilityで全Live slotとStateをdropするhandle lifecycle
- 非`HostMappable`なruntime carrierとlifecycle glueを渡せるprivate ABI

したがって、Pool algorithmをcompilerへ組み込む必要はないが、source-levelの通常の`extern` declarationだけへ縮退させることもできない。
sourceへ明示的な`retain`や`borrow`を公開するより、ownership planのeffectと型別glueをtrusted runtime callへ接続する形を初期候補とする。

## 現在のcompilerとの差分

現在の`execution/ownership`は[D083](../../history/decisions/active/D083.md)のowned native entryを持ち、last-use argumentをcalleeへ
Consumeできる。一方、primitive、host、memory、Buffer operationのoperandは現在すべてBorrowとして列挙され、Buffer runtimeが保存に
必要なretainを行う。Poolでは`init`、`put`、`take`、writable successorのoperand/result relationをmetadataにし、use planとparameter保持解析の
両方へ入力する必要がある。

LLVM backendにはmanaged valueの型再帰的なretain/release、managed placeのinitialize/replace/vacate、Buffer elementごとのcallback生成が
既にある。Poolはこのloweringの新しい利用者になり、別の型再帰を持たない。frontendの`Storable`は現在もclosed judgmentであり、Poolまたは
plugin leafの導入時に暗黙に拡張しない。

## 実装前に固定する検査

- zero-sized Stateとelement、capacity 0、capacity overflowでもslot遷移とdrop回数が一致する。
- aliasしたPoolでStateとslotの更新が同じidentityへ見え、opaqueなrepresentation viewを通してもidentityが変わらない。
- managed StateとelementについてShare、Consume、take、replace、Pool終了後のlive allocationが期待値と一致する。
- allocation failure時に旧capacity、State、全slotが保存され、lifecycle callbackは呼ばれない。
- canonical host copyがpaddingや非選択sum payloadへ依存せず、runtime representationとの差を越えてround-tripする。
