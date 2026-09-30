# Pool lifecycle contract

Status: Exploratory support document

この文書は、[Poolとopaque型によるcontainer基盤](README.md)が成立するために必要なstorage、precondition、failureの
低レイヤcontractを管理する。候補APIとcontainer algorithmは
[source sketch](container-examples.md)、現在の規範は[AddressとBuffer](../../spec/memory.md)と
[実行意味論](../../spec/execution.md)を正とする。各operationのownership effectと型別glueの必要箇所は
[所有権primitive](ownership-primitives.md)、型形成条件は[identity](identity.md#型形成条件)、representation viewの権限は
[README](README.md#file-local-opaque型)が所有する。

## Runtime representation

`Pool<State, T>`は`State`のcarrier、`T`のslot storage、各slotの`Vacant`または`Live`状態を一つのmanaged identityとして所有する。
初期profileでは`State`と`T`をruntime value representationで保持し、canonical memory layoutをPoolのcontractにしない。
これにより`Symbol`を含むaggregateとplugin-defined Engram leafへ同じlifecycle planを適用できる。

canonical memoryはhostとの値交換に限る。Pool上のBufferが`from`または`into`を提供する場合、canonical elementとruntime slotを
fieldごとに変換する。numeric scalarのようにruntime representationとcanonical layoutが一致する`T`では、実装がbulk copyを使ってよい。
ただし一致はcorrectness contractではなく、Pool storageのlayoutをsourceやhostへ観測させない。

`size<T> == 0`または`stride<T> == 0`でもslotは消滅しない。Poolはcoordinateごとの占有状態と、handle extensionを使う場合はgenerationを
別に保持する。element payloadのbyte数が0でも、capacity、`Live`/`Vacant`遷移、precondition、drop回数は通常の`T`と同じである。
占有状態は`poolIsLive`の結果とPool終了時にDropするslotの決定に使い、slot operationごとの検査には使わない。

## Primitive transitionとcontainer invariant

各Pool primitiveは、preconditionを満たすcallで一つの遷移として振る舞う。`reserve`は全Stateとslotを保存して指定された
logical capacityへ拡張し、要求が現在のcapacity以下ならPoolを変更しない。物理的なover-allocationは観測させない。
storage sizeをtargetで表現できない場合とallocationに失敗した場合は、既存Engram allocationと同じくtrapする。
[trap](../../spec/execution.md#trap)はterminalなので、失敗後のPool状態をsourceやlifecycle callbackへ公開する規則は要らない。

複数primitiveからなるBufferやMapのoperation全体はtransactionではなく、file-local invariantはcontainer operationのreturn時に
回復すればよい。途中状態を観測できるのは、container実装が途中で呼ぶfunctionだけである。lifecycle glueはmal codeを実行せず、
`hash`や`equal`のようなoperation requirementは`Storable`な引数しか受け取らず、top-level initializerはPoolを作れないため、
これらから変更中のcontainerへ到達できない。caller-suppliedなclosureを受け取るcontainer operationはclosureが同じcontainerの
aliasをcaptureし得るため、呼び出し前にinvariantを回復する。

Poolとmanaged valueは現在のC runtime contextと同じくthread-confinedであり、atomic containerやcross-thread synchronizationを提供しない。
物理relocation中の一時状態は一つのprimitive内部へ閉じ、mal callbackやplugin callbackへ公開しない。

## 未検査precondition

slot coordinateとslot状態に関する条件は、[Buffer](../../spec/memory.md#未検査precondition)と同じ未検査preconditionである。
runtimeはこれらを検査せず、違反時の実行結果を保証せず、trapへも写像しない。

| operation | precondition |
|---|---|
| `poolIsLive(pool, index)` | `index < poolCapacity(pool)` |
| `poolInitAt(pool, index, value)` | `index < poolCapacity(pool)`かつslotがVacant |
| `poolGetAt`、`poolPutAt`、`poolTakeAt`、`poolDropAt` | `index < poolCapacity(pool)`かつslotがLive |

`makePool`、`poolState`、`poolSetState`、`poolCapacity`、`poolReserve`はpreconditionを持たず、表現できないsizeと
allocation failureだけがtrapになる。

preconditionの責任は二段に分かれる。container利用者はcontainerが公開するprecondition、例えばBufferの`index < #buffer`を満たす。
container実装は、公開preconditionを満たすcallから到達する全Pool callがPool preconditionを満たすことをfile-local invariantで
保証する。利用者が公開preconditionを守ってもPool preconditionへ違反するなら、それはcontainer実装の誤りである。公開preconditionを
持たないoperation、例えばMapのlookupは、`poolIsLive`とStateで判定してからslot operationを呼ぶ。

Pool preconditionへの違反は、Vacant carrierのread、同じresponsibilityの二重Drop、Live valueのleakを起こし得る。したがって
Poolを直接呼ぶfileはmanaged valueのmemory safetyをinvariantとして担い、opaque型はその責任を宣言元fileへ閉じ込める。
実装はtestやdebug buildで占有状態を検査してよいが、その検査結果をsemanticsにしない。

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
必要なretainを行う。Poolではoperand effect`Store`を追加し、use planとparameter保持解析の両方へ入力する必要がある。

LLVM backendにはmanaged valueの型再帰的なretain/release、managed placeのinitialize/replace/vacate、Buffer elementごとのcallback生成が
既にある。Poolはこのloweringの新しい利用者になり、別の型再帰を持たない。frontendの`Storable`は現在もclosed judgmentであり、Poolまたは
plugin leafの導入時に暗黙に拡張しない。

## 実装前に固定する検査

- zero-sized Stateとelement、capacity 0でもslot遷移とdrop回数が一致し、capacity overflowはtrapする。
- aliasしたPoolでStateとslotの更新が同じidentityへ見え、opaqueなrepresentation viewを通してもidentityが変わらない。
- managed StateとelementについてShare、Consume、take、replace、同じvalueの書き戻し、Pool終了後のlive allocationが期待値と一致する。
- 占有状態を検査するtest用runtimeで、公開preconditionを満たすBufferとMapのprogramがPool preconditionへ違反しない。
- canonical host copyがpaddingや非選択sum payloadへ依存せず、runtime representationとの差を越えてround-tripする。
