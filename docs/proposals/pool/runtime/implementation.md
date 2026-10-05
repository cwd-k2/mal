# compilerとruntimeの実装

Status: Exploratory support document; [v0.7 rebase notice](../README.md) applies

この文書は、[runtime contract](contract.md#runtime-representation)をcompilerとruntimeがどう分担して実装するかと、検証の段階を
管理する。Poolのsource semantics、C runtime object、LLVM allocation objectを分ける規則は
[semantic identityとallocation object](contract.md#semantic-identityとallocation-object)を正とする。

## compilerとruntimeの分担

Poolがexecution ownershipで使うoperand effectは`Store`である。このeffectはPool導入に先立って、現行Bufferの
`new`と`put`のvalue operandで実装済みである。

source functionをaffineにするのではなく、type checking後のuse graphをresponsibility planへelaborateする。source valueは再利用でき、
planだけが各edgeを`Borrow`、`Share`、`Consume`、`Drop`として扱う。したがってRustの`Box`に似たexclusive ownerが生成物に現れても、
それはsource typeやPool authorityの追加ではない。

- `Store`は、operandのresponsibilityをprimitiveが保持することを表す。`swap`、`slot`、`initAt`、`putAt`のvalue、`pool`、`swapMeta`、
  `setMeta`のMeta、writable successorのinputとstorageを移し得る`freeze`と`thaw`のinputが該当する。
- use planは`Store`を`Share`または`Consume`へlowerする。`Store`はsource operandを必ず消費する意味ではなく、calleeまたはresultが
  carrierを保持する可能性を示す。[D083](../../../history/decisions/active/D083.md)の保持解析は、`Store`へ渡る
  parameterをreturnやcaptureと同じく保持として扱い、Bufferの`put`のようなmal wrapperをowned native entryにする。
- IxPool handle、index、lengthは`Borrow`である。resultは全てownedである。

現行Bufferの`new`と`put`は`Store`をuse planの`Share`または`Consume`へlowerし、owned responsibilityを
storageへ渡す。`fill`は一のoperandから複数elementを作り得るため`Borrow`のままであり、runtimeが各elementを
`Share`する。Bufferで検証したこの分解をPool operationにも使い、第二のresponsibility plannerを作らない。

runtimeが型ごとに必要とするglueは、上の表で「primitive内」に数えたものだけである。`share<V>`はread、一括処理の`fill`と`copy`、
共有時のwritable successorとIxPoolの複製が、`drop<V>`はwriteの旧値とIxPoolの終了が使う。relocationとswapはcarrierを移動する
だけでglueを呼ばない。BufferやPool handleを含む型も同じ再帰的なglueを使う。glueは失敗せず、I/O、host resourceの`close`、
別IxPoolの更新など観測可能な作用を持たず、IxPool
終了時のdrop順はsourceから観測できない。

LLVM backendにはmanaged valueの型再帰的なretain/release、managed placeのinitialize/replace/vacate、Buffer elementごとの
callback生成が既にある。IxPoolはこのloweringの新しい利用者になり、別の型再帰を持たない。

## 現行Bufferから分離する実装境界

現行BufferをPool導入の起点として読むと、変更は一枚岩ではない。既にあるmechanismと、現在の分類では表せない部分を分ける。

| 現行箇所 | 既にあるもの | 導入前に必要な整理 |
|---|---|---|
| frontend `types/properties` | place lifecycleとしての`Storable`、nested Bufferとexternal opaque carrierのadmission、requirementの再帰 | Pool handleを同じ再帰へ加える。functionとempty sumは拒否を保つ |
| execution ownership | product、sum、Symbol、Buffer、functionを再帰する`Lifecycle = Trivial | Owned`分類と、Buffer `new` / `put`の`Store` | Pool handleを同じ分類へ加え、Pool operationのuseとdropを計画する |
| LLVM value lifetime | Buffer handleを含むproductとsumの再帰的retain/release | nested Bufferのelement callbackから同じ処理を再利用する。Pool handle追加時も別の型再帰を作らない |
| LLVM Buffer storage | `Canonical(layout)`と`Runtime(layout, Lifecycle)`。external opaqueはTrivial、nested BufferはOwnedで動作する | Pool loweringから同じrepresentation/lifecycle分解を使う |
| C Buffer runtime | plain storageと、element retain/release callbackを持つstorage。nested Bufferは既存managed pathを使う | Trivialなruntime-value elementにはcallbackを課さない |
| Buffer host operation | canonical layoutだけを扱う`from`と`into` | `Representable`制限を保ち、Storable拡張から独立させる |

C runtimeの`_managed`はcallbackを受け取るprivate ABI variantの名前であり、languageの`Storable`またはcompilerの
`Runtime`全体を分類する語ではない。nested Bufferは`Owned` lifecycleなのでこのvariantを使うが、external opaqueの
`Runtime(_, Trivial)`はruntime-value representationを使ってもcallbackを必要としない。このABI名をsource-level judgmentへ
逆輸入しない。

特にstorage分類は、canonical representationとlifecycleを一つの二択へ押し込まない。導入時の概念形は次である。

```text
ElementStorage(T) = Canonical(layout)
                  | Runtime(layout, Lifecycle(T))

Lifecycle(T) = Trivial
             | Owned(share glue, drop glue)
```

`Buffer<Buffer<T>>`は`Runtime(_, Owned)`となり、[D096](../../../history/decisions/active/D096.md)でfrontend admissionと
positive runtime testまで採択した。external opaque carrierは`Runtime(_, Trivial)`となり、
[D097](../../../history/decisions/active/D097.md)でfrontend、LLVM lowering、C runtimeを通すpositive testとともに採択した。

この分解を先に現行Bufferへ適用すると、Pool固有のoccupancy、Meta、`Store`を導入する前に、`Storable`とlifecycle planの境界を
検証できる。`Canonical`を`Runtime(_, Trivial)`へ統合する必要はない。前者はhost bulk copyと既存の最適化を所有し、後者は
canonical memoryへ出せないruntime carrierを保持する。

型再帰的なmanaged判定はrepresentationと独立した`Lifecycle`として名前を与え、execution ownershipとBuffer storageが同じ分類を
消費するようにした。残る導入では、このBufferで確かめた`Lifecycle`と`ElementStorage`をIxPoolとImPoolのMetaおよびslotへ使う。
Pool用に第三の型分類や別のglue再帰を作らない。

D096では既存の`RuntimeOwned` mechanismだけでnested Bufferを通せたため、利用者が一つしかない段階で抽象的な`Lifecycle` data typeを
先行追加しなかった。[D097](../../../history/decisions/active/D097.md)でrepresentationとlifecycleを直交させ、その直後に二つのconsumerが
生じた時点で共通分類を抽出した。これによりPool固有のstate machineはlifecycle分類から独立して導入できる。
現行実装のallocation、動的instruction、LLVMのalias証明の限界は
[Buffer生成物とownership cost](../../../history/performance/buffer.md)で測定した。そこで最初に観測した小容量Bufferの二重allocationと、
同一slotへの書き戻しが消えないことは、nested identityの意味論的なcostではない。前者はstable object内のsmall-buffer storage、
後者はtyped operationの恒等変換で削減できた。`Store`とmove-based `swap`の
採用判断では、現行`get` / `put`のownership trafficからこれらの実装costを分けて比較する。

[genericsとmanaged containerの生成物](../../../history/performance/generics.md)では、operation family、generic control、
`extend<Focus>`の高階呼出しがspecializationとLTOでdictionaryもclosure allocationも残さず消える一方、
関数を値として返す`State`だけが反復ごとにclosure environmentを確保した。これはPool storageの不足ではなく、
specialized producer-consumer間のclosure deforestationというcompiler課題である。IxPoolの非公開kernelをclosure arenaや
汎用allocatorへ広げず、Pool loweringが所有するのはslot、occupancy、growth、carrier lifecycleに限る。

## lowering boundary

Pool operationはsource primitive、LLVM instruction、C runtime functionを一対一に対応させない。各層は次を所有する。

| 層 | 入力 | 所有する判断 | 出力 |
|---|---|---|---|
| type checking | concreteな`Meta`と`V` | 型形成、`Storable`、operation signature | checked Pool operation |
| execution ownership | typed operandとcontrol | `Borrow`、`Share`、`Consume`、`Drop`、`Store` | responsibility plan |
| LLVM source layout | concrete typeとtarget data layout | payloadのsize、alignment、stride | typed layout |
| LLVM Pool lowering | operationとresponsibility plan | typed load/store、glue、runtime call、pointer再取得 | LLVM IR |
| C runtime | layoutとprogram固有glueを伴うprivate ABI | allocation、growth、occupancy、reference count | backing mechanism |

runtimeは`V`の意味を再解釈せず、compilerが渡したlayoutと`share`/`drop` glueだけを使う。LLVM backendはreference countやphysical
capacityからsource上のLive set、container count、last useを推論しない。execution ownershipはphysical layoutやallocation strategyを
知らない。つまりauthorityはtype checkingとoperation、responsibilityはexecution ownership、representation uniquenessはruntimeという
一方向の境界を保つ。

### C runtime object

初期実装は概念上、stableなPool objectと交換可能なbacking storageを分ける。

```text
Pool object
    managed reference count
    logical capacity
    physical capacity
    Header carrier
    occupancy representation
    current payload pointer
    payload layout and lifecycle glue
```

このfield一覧はprivate ABIの要求を示す模式であり、固定layoutではない。Header、occupancy、payloadを別allocationに分けても、
small payloadをstable objectの末尾またはunionへinline化しても、zero-sized payloadをallocationなしで表してもよい。logical backingを
object metadataから分けることは、常に別のC allocationへ置くという意味ではない。sourceから観測できるのはIxPoolではsemantic identity、
ImPoolではstructural stateと、両者に共通するoperation lawだけである。現行Bufferはpointer一個分以下の初期storageをobjectへinline化し、
growth時にflat ownerへ昇格する。この境界を先に検証している。

現在の非公開kernelは`runtime/c11/pool.c`にあり、source constructからはまだ選択されない。stableなreference-counted objectが
Header carrierと、bitmapおよびpayloadを一つにした交換可能なbacking allocationを所有する。Headerは大きさによらずstable objectの
末尾へ整列して置き、objectと同じlifetimeのためだけに別allocationを作らない。logical capacityと倍増するphysical capacityは分離し、
physical capacity内の`grow`はallocationもpayload relocationも行わない。physical growthは新backingを完成させ、occupancyとlogical
capacity分のpayload byteを一括で移してからpointerと両capacityをcommitする。Vacant payloadは解釈せず、occupancyだけがLive carrierの存在を決める。
direct C boundary testはaliasから同じidentityを観測できること、Header交換、Vacant/Live交換、zero-stride、
growth前後のcoordinate保存を検査する。managed variantは既存Bufferと同じ`retain(context, carrier)`と`release(carrier)`の
callback型をHeaderとpayloadで共有し、readだけがShare、exchangeとrelocationはMove、最後のPool handleだけがHeaderと全Live slotを
Dropすることも同じboundary testで検査する。source APIとLLVM loweringはまだ接続しない。

growthは次のtransactionとして実装する。

1. logical capacity、payload size、occupancy sizeのoverflowを判定する。
2. 必要なら新しいbacking allocationを確保する。失敗は既存Engramと同じtrapにする。
3. Live payload carrierとoccupancyを新storageへ移す。relocationは`share`と`drop`を呼ばない。
4. Pool objectのcurrent pointerとphysical capacityをcommitする。
5. 古いbacking allocationを解放する。
6. logical capacityを更新し、新しいcoordinateをVacantとして観測可能にする。

trapはterminalなので、allocation失敗時のrecoverable Pool stateをsourceへ定めない。runtime内部でcommit前のcleanupを行うことは
implementation correctnessであり、source transaction semanticsを追加することではない。

### LLVM lowering

LLVM loweringはPool carrierをmanaged Pool objectへのpointerとして運び、slot accessごとにcurrent payload pointerを取得する。
同じbasic region内でpointerやcapacityをcacheしてよいが、次をclobber boundaryとする。

- `grow`と、physical relocationを行い得るPool operation。
- ImPoolのwritable successorを複製または移動し得るoperation。
- callbackまたはnative Mal callが同じIxPool handleへ到達し得る境界。

growth前に得たslot pointerへgrowth後に`getelementptr`、load、storeを行わない。Cの`realloc`が同じaddressを返しても、LLVM上は旧objectの
pointerを新objectへ持ち越さない。既存Bufferがgrowth後にactive dataを再取得する規則と同じlowering helperを共有する。

Slot payloadのinitialize、replace、vacateは既存managed place helperを使う。occupancy tagを先にLiveへして未初期化payloadを公開したり、
payloadをDropした後もLiveとしてcallbackから観測できる順序にしない。lifecycle glueはmal codeを実行しないため、primitive内部の一時的な
invariant破れはcallbackへ再入しない。

LLVMの`llvm.lifetime.start/end`はPool slotのLive/Vacantへ対応させない。これらはallocation object、特にstack objectのoptimizer向け
lifetimeであり、managed valueのvariant、responsibility、destructor boundaryを表さない。必要ならbackend内部のlocal temporaryにだけ
通常の規則で使う。

### backendを越えて保存するfact

後段が表現から意味を逆推論しないよう、次のfactは所有stageから順方向に渡す。

- type checkerからconcreteなPool constructor、Meta型、element型、周辺operationのprecondition。
- executionから各operandのresponsibility effectとresultのowner destination。
- source layoutからpayload layoutとzero-sizedかどうか。
- Pool loweringからruntime call後に再取得すべきcached view。

occupancy bitmapのbit、null pointer、physical capacity、reference countをsemantic factの代用にしない。

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
| writable successorのstorage再利用 | `Store`をowned native entryへ伝播し、runtimeのrepresentation uniqueness検査に依存する |
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

Buffer上の形は代わりに、slotごとのsum tag、構築時の`fill`、移動ごとの`Share`と`Drop`を払う。二つの試作はhost callまたはcopyを
挟むため直接比較に使えないが、IxPool kernelと同じVacant/Live反転を行うdirect Cとsafe Rustの測定では、separate byte、bitmap、
inline sumのうちbitmapが最小のmemoryだった。randomなword payloadでは両言語で最速、byte payloadではCで最速、Rustでは三方式が
2%以内であり、sequential accessでも差が小さかった
（[Pool占有tagの表現比較](../../../history/performance/pool-occupancy.md)）。初期kernelはbitmapを使い、step 3以降でmanaged payload、
growth、終了時の走査を含めて再測定する。その際はMap、Deque、heapを
[`generic-map`](../../../../examples/generic-map/map.mal)のようなBuffer上の実装と比べる。

## 検証の段階

1. opaque identity、file-local representation view、別fileからの構築と分解の拒否をfrontend testで固定する。
2. [D096](../../../history/decisions/active/D096.md)の`Buffer<Buffer<T>>`と
   [D097](../../../history/decisions/active/D097.md)のexternal opaque carrierを、通常値とBuffer element callbackが共有する
   `Lifecycle(T) = Trivial | Owned(share, drop)`で検証する。
3. backend内部にIxPoolの核と周辺operationを置き、unmanaged Metaとelementで実行する。
   初期occupancyは測定済みのbitmapとし、zero-sizedなMetaとelement、capacity 0でもslot遷移とdrop回数が一致し、capacity overflowはtrapする。
4. `Symbol`、nested Buffer、nested Pool handleとmanaged aggregateで、分解を直接実行するtest用runtimeとshare/drop回数と順序を比べる。relocation、同じvalueの
   書き戻し、同じBufferで範囲が重なる`copy`でDrop済みのreferentを読まず、IxPool終了時のlive allocationは0になる。
5. Bufferで検証済みの`Store`をPool primitiveへ付与する。`Store`へ渡るparameterを持つmal wrapperがowned native entryになり、
   last-use argumentを`Consume`する。
6. IxPool上のBufferを現在のBufferとalias、range、overlap、trap semanticsで比べ、範囲と占有tagを検査するtest用runtimeで公開
   preconditionを満たすprogramがIxPool preconditionへ違反しないことを確かめる。canonical host copyはpaddingや非選択sum payloadへ
   依存せずround-tripする。これはas-ifで実装するBufferが参照実装と一致することの検査を兼ねる。
7. ImPool上のVectorで、shared時のcopyとlast-use時のstorage再利用を別々に測る。handle elementを持つsnapshotでは外側の置換が
   独立し、内側referentの変更が共有観測されることも検査する。
8. 木やgeneration付きkeyのcontainerをcoordinateで実装し、coordinateの再利用と古いkeyの拒否を検査する。
9. semanticsと生成物のcostが妥当な場合だけ、predefined Bufferの置換、Vectorのpublic採択、既存host operationの互換性、
   trusted crate境界を別々に判断する。

compilerを変えない二つの試作が、step 6から8の一部を先取りした。結果は[試作で確かめたこと](../prototypes.md)に置く。

## 実装を再開する位置

step 1と2に必要だったopaque identity、nested Buffer、external opaque element、`ElementStorage`の分離は現行実装とactive decisionへ
反映済みである。step 3と4のruntime側についても、非公開C kernelがbitmap occupancy、stable identity、owner末尾のHeader、growth、
trivial/managed payloadのreadとexchange、終了時drop、overflow trapを実行し、直接harnessで検証している。growthはlogical payloadを
一括relocationし、occupancyだけをLive authorityとして保つ。

未実装なのは、このkernelをMalのsource semanticsへ接続する境界である。次回は次の順に再開する。

1. [推奨する最小採択単位](../README.md#推奨する最小採択単位)の`IxPool<Header, V>`とprimitive `trap`について、source rule、
   operation signature、未検査preconditionをspec decisionとして確定する。`ImPool`、Vector、allocator parameterは含めない。
2. predefined typeとoperationをfrontend、formatter、editor/LSPへ一つの集合として追加し、`Storable(Header)`と`Storable(V)`、
   opaque representation authority、unsupported syntaxの拒否をfocused testで固定する。
3. checked IxPool operationをLLVM lowering、runtime requirement収集、`Lifecycle` callbackへ接続する。handleの通常値とslot/closure内保存が
   同じshare/drop authorityを使い、`grow`後にcached backing addressを使わないことをartifact testで検査する。
4. `Store` effectとowned native entryを接続し、last-useのIxPool argumentだけを`Consume`できることをbaseline/productionの
   observable一致とexact validatorで確かめる。
5. IxPool上のmal製Bufferを現行Bufferと比較する。semantic parityのC/Rust referenceに加えてhand-lowered lower boundを別枠で測り、
   occupancy、allocation policy、generic erasureのcostを混ぜない。

この順序より前に非公開kernelを汎用allocator、closure arena、public plugin ABIへ拡張しない。既存kernelはsource contractが採択される
まで実装可能性とlifecycleのprobeであり、IxPoolが既に言語機能として実装済みであることを意味しない。
