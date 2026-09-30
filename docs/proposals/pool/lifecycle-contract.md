# IxPool lifecycle contract

Status: Exploratory support document

この文書は、[Poolとopaque型によるcontainer基盤](README.md)が成立するために必要なstorage、precondition、failureの
低レイヤcontractを管理する。候補APIは[primitive一覧](primitives.md)、container algorithmは
[collection例](collection-examples.md)、現在の規範は[AddressとBuffer](../../spec/memory.md)と
[実行意味論](../../spec/execution.md)を正とする。各operationのownership effectと型別glueの必要箇所は
[所有権primitive](ownership-primitives.md)、型形成条件は[identity](identity.md#型形成条件)、representation viewの権限は
[README](README.md#file-local-opaque型)が所有する。

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

`size<T> == 0`または`stride<T> == 0`でもslotは消滅しない。IxPoolはcoordinateごとの占有状態を持ち、[IdPool](idpool.md)を載せる場合はgenerationも
別に保持する。element payloadのbyte数が0でも、capacity、`Live`/`Vacant`遷移、precondition、drop回数は通常の`T`と同じである。
占有状態は`isLive`の結果とIxPool終了時にDropするslotの決定に使い、slot operationごとの検査には使わない。

## Primitive transitionとcontainer invariant

各IxPool primitiveは、preconditionを満たすcallで一つの遷移として振る舞う。`reserve`は全Stateとslotを保存して指定された
logical capacityへ拡張し、要求が現在のcapacity以下ならIxPoolを変更しない。物理的なover-allocationは観測させない。
storage sizeをtargetで表現できない場合とallocationに失敗した場合は、既存Engram allocationと同じくtrapする。
[trap](../../spec/execution.md#trap)はterminalなので、失敗後のIxPool状態をsourceやlifecycle callbackへ公開する規則は要らない。

複数primitiveからなるBufferやMapのoperation全体はtransactionではなく、file-local invariantはcontainer operationのreturn時に
回復すればよい。途中状態を観測できるのは、container実装が途中で呼ぶfunctionだけである。lifecycle glueはmal codeを実行せず、
`hash`や`equal`のようなoperation requirementは`Storable`な引数しか受け取らず、top-level initializerはIxPoolを作れないため、
これらから変更中のcontainerへ到達できない。caller-suppliedなclosureを受け取るcontainer operationはclosureが同じcontainerの
aliasをcaptureし得るため、呼び出し前にinvariantを回復する。

IxPoolとmanaged valueは現在のC runtime contextと同じくthread-confinedであり、atomic containerやcross-thread synchronizationを提供しない。
物理relocation中の一時状態は一つのprimitive内部へ閉じ、mal callbackやplugin callbackへ公開しない。

## 未検査precondition

slot coordinateとslot状態に関する条件は、[Buffer](../../spec/memory.md#未検査precondition)と同じ未検査preconditionである。
runtimeはこれらを検査せず、違反時の実行結果を保証せず、trapへも写像しない。

| operation | precondition |
|---|---|
| `isLive(pool, index)` | `index < capacity(pool)` |
| `initAt(pool, index, value)` | `index < capacity(pool)`かつslotがVacant |
| `getAt`、`putAt`、`takeAt`、`dropAt` | `index < capacity(pool)`かつslotがLive |

`makeIxPool`、`state`、`setState`、`capacity`、`reserve`はpreconditionを持たず、表現できないsizeと
allocation failureだけがtrapになる。

preconditionの責任は二段に分かれる。container利用者はcontainerが公開するprecondition、例えばBufferの`index < #buffer`を満たす。
container実装は、公開preconditionを満たすcallから到達する全IxPool callがIxPool preconditionを満たすことをfile-local invariantで
保証する。利用者が公開preconditionを守ってもIxPool preconditionへ違反するなら、それはcontainer実装の誤りである。公開preconditionを
持たないoperation、例えばMapのlookupは、`isLive`とStateで判定してからslot operationを呼ぶ。

IxPool preconditionへの違反は、Vacant carrierのread、同じresponsibilityの二重Drop、Live valueのleakを起こし得る。したがって
IxPoolを直接呼ぶfileはmanaged valueのmemory safetyをinvariantとして担い、opaque型はその責任を宣言元fileへ閉じ込める。
実装はtestやdebug buildで占有状態を検査してよいが、その検査結果をsemanticsにしない。

## Ordinary externとの境界

現在のexternal opaque valueをmalのopaque型で包むだけではIxPoolにならない。external handleはcopyしてもreferentのlifetimeを延長せず、
mal側のdropもresourceを解放しないため、wrapperはuse-after-free、多重close、最後のaliasとstorage解放の対応を保証できない。
`Borrow<T>`もcall中の有効性だけを表し、hostが`T`をslotへ保持する根拠にはならない。

IxPoolの実装自体をextern-likeなruntime libraryへ置くことはできる。その場合もIxPool handleはmal-controlledなEngram leafとして登録し、
compilerまたはtrusted extensionが次を供給する。

- operandとresultの`Borrow`、`Share`、`Consume`、`Drop` effect
- concreteな`State`と`T`のruntime layout、およびmaterializeした`share`と`drop` glue
- 最後のIxPool responsibilityで全Live slotとStateをdropするhandle lifecycle
- 非`HostMappable`なruntime carrierとlifecycle glueを渡せるprivate ABI

したがって、IxPool algorithmをcompilerへ組み込む必要はないが、source-levelの通常の`extern` declarationだけへ縮退させることもできない。
sourceへ明示的な`retain`や`borrow`を公開するより、ownership planのeffectと型別glueをtrusted runtime callへ接続する形を初期候補とする。

## 現在のcompilerとの差分

現在の`execution/ownership`は[D083](../../history/decisions/active/D083.md)のowned native entryを持ち、last-use argumentをcalleeへ
Consumeできる。一方、primitive、host、memory、Buffer operationのoperandは現在すべてBorrowとして列挙され、Buffer runtimeが保存に
必要なretainを行う。IxPoolで追加する`Store`は[所有権primitive](ownership-primitives.md#compilerとruntimeの分担)に示す。

LLVM backendにはmanaged valueの型再帰的なretain/release、managed placeのinitialize/replace/vacate、Buffer elementごとのcallback生成が
既にある。IxPoolはこのloweringの新しい利用者になり、別の型再帰を持たない。frontendの`Storable`は現在もclosed judgmentであり、IxPoolまたは
plugin leafの導入時に暗黙に拡張しない。

## 実装前に固定する検査

- zero-sized Stateとelement、capacity 0でもslot遷移とdrop回数が一致し、capacity overflowはtrapする。
- aliasしたIxPoolでStateとslotの更新が同じidentityへ見え、opaqueなrepresentation viewを通してもidentityが変わらない。
- managed StateとelementについてShare、Consume、take、replace、同じvalueの書き戻し、IxPool終了後のlive allocationが期待値と一致する。
- 占有状態を検査するtest用runtimeで、公開preconditionを満たすBufferとMapのprogramがIxPool preconditionへ違反しない。
- canonical host copyがpaddingや非選択sum payloadへ依存せず、runtime representationとの差を越えてround-tripする。
