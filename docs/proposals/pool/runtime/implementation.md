# compilerとruntimeの実装

Status: Exploratory support document

この文書は、[runtime contract](contract.md#runtime-representation)をcompilerとruntimeがどう分担して実装するかと、検証の段階を
管理する。

## compilerとruntimeの分担

compilerのexecution ownershipに新しく要るのは、operand effectの`Store`だけである。

- `Store`は、operandのresponsibilityをprimitiveが保持することを表す。`swap`、`slot`、`initAt`、`putAt`のvalue、`pool`、`swapMeta`、
  `setMeta`のMeta、writable successorのinputとstorageを移し得る`freeze`と`thaw`のinputが該当する。
- use planは`Store`を`Share`または`Consume`へlowerする。[D083](../../../history/decisions/active/D083.md)の保持解析は、`Store`へ渡る
  parameterをreturnやcaptureと同じく保持として扱い、Bufferの`put`のようなmal wrapperをowned native entryにする。
- IxPool handle、index、lengthは`Borrow`である。resultは全てownedである。

現行のBuffer operandは全て`Borrow`で、保存に必要なretainはruntimeが行う。これは`Store`を常にruntime内の`Share`として扱うことに
当たり、`Store`を導入するとlast useのvalueを`Consume`してruntime内のretainとcall site側のreleaseを省ける。

runtimeが型ごとに必要とするglueは、上の表で「primitive内」に数えたものだけである。`share<V>`はread、一括処理の`fill`と`copy`、
共有時のwritable successorとIxPoolの複製が、`drop<V>`はwriteの旧値とIxPoolの終了が使う。relocationとswapはcarrierを移動する
だけでglueを呼ばない。glueは失敗せず、I/O、host resourceの`close`、別IxPoolの更新など観測可能な作用を持たず、IxPool
終了時のdrop順はsourceから観測できない。

LLVM backendにはmanaged valueの型再帰的なretain/release、managed placeのinitialize/replace/vacate、Buffer elementごとの
callback生成が既にある。IxPoolはこのloweringの新しい利用者になり、別の型再帰を持たない。

### Ordinary externとの境界

現在のexternal opaque valueをmalのopaque型で包むだけではIxPoolにならない。external handleはcopyしてもreferentのlifetimeを
延長せず、mal側のdropもresourceを解放しないため、wrapperはuse-after-free、多重close、最後のaliasとstorage解放の対応を保証
できない。IxPoolの実装をextern-likeなruntime libraryへ置く場合も、IxPool handleはmal-controlledなEngram leafとして登録し、
compilerまたはtrusted extensionが次を供給する。

- operandとresultの`Borrow`、`Share`、`Consume`、`Drop` effect
- concreteな`Meta`と`V`のruntime layout、およびmaterializeした`share`と`drop` glue
- 最後のIxPool responsibilityでMetaと全Live slotをdropするhandle lifecycle
- 非`HostMappable`なruntime carrierとlifecycle glueを渡せるprivate ABI

### Managed Engramとの依存関係

本案の各部分は同じ段階を要求しない。

| 部分 | managed Engramへの依存 |
|---|---|
| file-local opaque identityとrepresentation view | frontendに閉じ、unmanaged representationだけならD055/D080に依存しない |
| IxPoolのMetaとslot | [D080](../../../history/decisions/active/D080.md)の`initialize`、`replace`、`vacate`と同じcarrier invariantをruntime storageへ適用する |
| operand effect `Store` | trusted metadataをuse planとD083のparameter保持解析へ接続する |
| writable successorのstorage再利用 | `Store`をowned native entryへ伝播し、runtimeのuniqueness検査に依存する |
| managed Metaまたは`Symbol` element | D055の`Share`、`Consume`、`Drop`と、型別glueをruntime callbackへmaterializeする境界に依存する |
| Bufferのmal実装 | IxPoolがmanaged elementを正しく扱えれば、Buffer固有のretain/release loweringには依存しない |
| plugin-defined Engram leaf | representation、lifecycle、artifact dependencyを登録する安定したtrusted extension contractが要る |

IxPoolをbuilt-in Engramとして先に実装し、`Storable(Meta)`と`Storable(V)`だけを受理することは、一般plugin ABIより前に検証できる。
ただしunmanaged scalarだけのIxPoolでは重複削減を確認できないため、`Symbol`を含むelementを扱う段階までに、型別`share`と`drop`を
通常値、IxPool callback、closure environment destructorから共有できる必要がある。

境界が安定した後、IxPool自体または新しいEngram leafをcompilerと同じversionへ静的に結合する
trusted crateへ移せるかを評価する。leafの登録には、layout、valid valueの構築、runtime representation、`share`、
`drop`、relocation、保持するchild Engram、runtime source選択と、operationごとの`Borrow`または`Store`と上の分解の宣言が要る。

## 占有tagの費用

BufferとVectorはas-ifで実装するため（[BufferとVector](../api/buffer-vector.md#bufferとvectorの対)）、占有tagを払うのはIxPoolを
直接使うcontainerだけである。比べる相手は、同じcontainerをBuffer上に番兵値とsum tagで書いた形である
（[Bufferの前提を外すと変わること](../containers/overview.md#bufferの前提を外すと変わること)）。IxPool側が払う差は次の三つに限られる。

- 占有tagの書き込み：`initAt`と`takeAt`ごとに一回。要素の書き込みに比べた割合は、要素が小さいほど大きい。
- 占有tagのmemory：slotごとに1 byteなら小さい要素ではstorageが倍近くになる。bitmapなら1/8で済むが、書き込みが読み出しと
  書き戻しになる。
- IxPool終了時の走査：`drop`がno-opでない要素型だけが払い、capacityまでtagを読む。

Buffer上の形は代わりに、slotごとのsum tag、構築時の`fill`、移動ごとの`Share`と`Drop`を払う。二つの試作はこの差を測れない。
C host試作はoperationごとにextern callを挟み、Buffer上のemulationは意味の参照でありcopyを含むためである。測定には
[検証の段階](#検証の段階)のstep 2の実装が要り、Map、Deque、heapを[`generic-map`](../../../../examples/generic-map/map.mal)のような
Buffer上の実装と比べる。tagの費用が目立つ場合は、tagの表現を見直す。

## 検証の段階

1. opaque identity、file-local representation view、別fileからの構築と分解の拒否をfrontend testで固定する。
2. backend内部にIxPoolの核と周辺operationを置き、unmanaged Metaとelementで実行する。
   zero-sizedなMetaとelement、capacity 0でもslot遷移とdrop回数が一致し、capacity overflowはtrapする。
3. `Symbol`とmanaged aggregateで、分解を直接実行するtest用runtimeとshare/drop回数と順序を比べる。relocation、同じvalueの
   書き戻し、同じBufferで範囲が重なる`copy`でDrop済みのreferentを読まず、IxPool終了時のlive allocationは0になる。
4. `Store`へ渡るparameterを持つmal wrapperがowned native entryになり、last-use argumentを`Consume`する。
5. IxPool上のBufferを現在のBufferとalias、range、overlap、trap semanticsで比べ、範囲と占有tagを検査するtest用runtimeで公開
   preconditionを満たすprogramがIxPool preconditionへ違反しないことを確かめる。canonical host copyはpaddingや非選択sum payloadへ
   依存せずround-tripする。これはas-ifで実装するBufferが参照実装と一致することの検査を兼ねる。
6. ImPool上のVectorで、shared時のcopyとlast-use時のstorage再利用を別々に測る。
7. 木やgeneration付きkeyのcontainerをcoordinateで実装し、coordinateの再利用と古いkeyの拒否を検査する。
8. semanticsと生成物のcostが妥当な場合だけ、predefined Bufferの置換とtrusted crate境界を別々に判断する。

compilerを変えない二つの試作が、step 5から7の一部を先取りした。結果は[試作で確かめたこと](../prototypes.md)に置く。
