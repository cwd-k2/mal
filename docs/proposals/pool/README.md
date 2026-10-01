# Poolとopaque型によるcontainer基盤

Status: Exploratory

この文書は、現在の`Buffer<T>`が持つtyped storage mechanismとdense sequence policyを分離し、少数のtrustedな
`IxPool<State, T>` primitive上でcontainerをmal sourceとして定義する案を管理する。現行仕様は
[AddressとBuffer](../../spec/memory.md)、managed responsibilityは[D055](../../history/decisions/active/D055.md)と
[D083](../../history/decisions/active/D083.md)、carrier lifecycleの現在の実装境界は
[D080](../../history/decisions/active/D080.md)を正とする。本案はまだsource syntax、`IxPool` API、plugin ABIを定めない。

## 目的

現在の`Buffer<T>`はmal-owned storageに加えて、共有mutable identity、denseな`[0, count)`、append、index access、growth、range
operationを一つの組み込み型として持つ。この形はArrayまたはVectorとして有用だが、slot map、deque、heap、tree、graph、arenaなどを
追加するたびにallocation、relocation、element lifecycleを別の組み込みruntimeへ複製するか、Bufferのindexとrow encodingへ
container policyを押し込む必要がある。

本案は、malが所有する可変storageのmodelとしてIxPoolを置き、BufferとMap、Deque、heap、treeなどをその上のopaque型として
malで定義する。利用者へraw storage、uninitialized value、manual `drop`を公開せず、container実装者もstride、alignment、retain、
releaseを型ごとに再定義しない。layout、lifecycle correctness、allocation failureはtrusted layerに残し、logical State、denseまたは
sparseな利用、growth、free list、ordering、hashing、snapshot policyをmal側が選ぶ。このpolicyを選べずBufferと同じdense sequenceしか
作れないならIxPoolを独立させる意味はなく、逆にraw memory操作まで開くことも本案の目的ではない。

## 設計の軸

本案が決める事項は次の六つの軸に分かれ、それぞれを一つの文書が所有する。

| 軸 | 問い | IxPoolでの答え | 所有する文書 |
|---|---|---|---|
| 所有権 | 誰がresponsibilityを持つか | `init`と`take`、operand effect `Store` | [所有権primitive](ownership-primitives.md) |
| lifecycle | carrierをいつ確保、移動、解放するか | `reserve`、IxPool終了、型別glue | [lifecycle contract](lifecycle-contract.md) |
| identity | 変更を誰が観測し、何を`Storable`にできるか | IxPoolとImPool | [identity](identity.md) |
| 妥当性 | どのplaceがLiveで、誰がそれを保証するか | 未検査preconditionとcontainer invariant | [lifecycle contract](lifecycle-contract.md#未検査precondition) |
| 権限 | 誰がIxPoolへ直接触れるか | file-local opaque型の宣言元file | 本書と[file-local opaque type](../../spec/types.md#file-local-opaque-type) |
| 表現 | どのbitで保持し、hostとどう交換するか | runtime representationとdata primitive | [lifecycle contract](lifecycle-contract.md#runtime-representation) |

妥当性と権限は対になる。IxPoolのpreconditionを未検査にできるのは、IxPoolを直接呼ぶfileをopaque型で一つに閉じ込め、そのfileの
invariantでpreconditionを満たせるからである。

## 位置づけ

本案のstorageの中心は、変更の扱いが異なる二つのprimitiveの対である。

- `IxPool<State, T>`：identityを共有し、その場で書き換える。coordinateで引くplaceの線形空間と、明示的な占有状態を持つ。
- `ImPool<State, T>`：identityを持たず、更新のたびにsuccessorを返す。入力が唯一のresponsibilityならstorageを再利用し、
  共有中ならcopyする。

ImPoolはIxPoolの上に書けない。storageを再利用できるかは参照数で決まり、malは参照数をsourceへ見せないためである。また、
その場で書き換えるAPIを持たない別の型でなければ、保持した値が変わらないことを型で保証できない。現在のruntimeのBufferと
`Symbol`は、この対をbyte列に特化した形に当たる。Bufferは共有されるidentityであり、`Symbol`の連結は一意なownerならその場で
伸ばし、共有中ならcopyする。

containerは、この対の上にmalで定義する。Buffer、Map、Deque、heap、木などはIxPool上に、immutable arrayは
ImPool上に置き、各containerが引き方と占有状態の形を決める。BufferはIxPoolの上に追加のcostなしに書けるが、逆はslotごとの
sum tagと移動ごとの`Share`と`Drop`を払う。heapやopen addressing Mapのように位置で引きながら空きを持つ構造はIxPoolだけが
直接表すため、仕組みの層ではIxPoolがBufferより基本的である。未検査のpreconditionはIxPoolだけが持ち、containerの利用者は
それに触れない。

## 文書の構成

API、語彙、contractは次の文書が所有する。

- [primitive一覧](primitives.md)：memoryの層、coordinateの線形性、slot、sequence、runの語彙、IxPoolとBufferの責務、全primitive
- [IxPool上のcontainer](containers.md)：代表的なcontainerの比較、Bufferの前提を外すと変わること、IxPoolの輪郭

次の文書は、個別のcontainerの設計、規則を確かめる例、試作の結果である。

- [Buffer実装](buffer-implementation.md)：現行Bufferのhost交換以外のoperationをIxPoolの上に書いた参照実装
- [collection例](collection-examples.md)：stack、binary heap、open addressing Map、SlotMap、木
- [既存exampleとの差分](current-examples.md)：`examples/`をIxPool上へ移したときのsource、precondition、costの変化
- [array ownership](array-ownership.md)：IxPool上のmutable arrayとImPool上のimmutable arrayの比較
- [試作で確かめたこと](prototypes.md)：C host試作とBuffer上のemulationの結果

## file-local opaque型

IxPoolを安全なcontainerへ閉じ込めるには、transparent aliasとは別に
[file-local opaque type](../../spec/types.md#file-local-opaque-type)を使う。

```mal
opaque Buffer<T> :: IxPool<USize, T>;
```

hidden representationを観察できるのは宣言元source fileだけであり、この権限は`require`先へ移らない。同じfileの通常のmal bindingは、
専用のpack/open構文なしに`Buffer<T>`をIxPool operationへ渡し、IxPoolを`Buffer<T>`が期待される位置から返せる。このviewは
allocation、copy、新しいEngram identityを作らず、同じcarrier responsibilityを受け渡す。opaque型はgeneric specializationと
operation familyのkeyにdeclaration identityを残し、同じrepresentationを持つ二つのopaque型を混同しない。

mutable metadataはIxPoolの`State`として共有identity側に置く。BufferはStateをcountとして使い、Mapは要素数を、Dequeは`(head, count)`を
Stateに持つ。各fileは、`reserve`とslot遷移だけを提供するIxPool上に、自分のState invariantとgrowth policyを実装する。

## Bufferと上位container

Bufferは組み込み型ではなく、IxPool上のpreludeのopaque型になる。仕様上のBufferは[Buffer実装](buffer-implementation.md)の参照実装で
意味を定め、実装は同じ結果になる限り専用runtimeを使ってよい。hostと`Symbol`との交換はBufferだけが持つprimitiveとして残り、
IxPoolはAddressに触れない。Bufferが提供する語彙とIxPoolとの責務分担は
[primitive一覧](primitives.md#ixpoolとbufferの責務)が所有する。

element equality、hash、orderingはIxPoolやpluginへ埋め込まず、通常のfunction引数または
[operation family](../../spec/operation-families.md)のrequirementとして上位algorithmが要求する。standard Bufferのcore operationを
downstream sourceが同じidentityのまま上書きする仕組みは導入しない。一つのopaque型のrepresentation invariantと公開operationは
宣言元fileが所有し、別policyは別のopaque型として定義する。

## Managed Engramとの依存関係

本案の各部分は同じ段階を要求しない。

| 部分 | managed Engramへの依存 |
|---|---|
| file-local opaque identityとrepresentation view | frontendのsyntax、resolve、type checkingに閉じ、unmanaged representationだけならD055/D080に依存しない |
| `IxPool<State, T>`のstateとslot | D080の`initialize`、`replace`、`vacate`と同じcarrier invariantをruntime storageへ適用する |
| operand effect `Store` | function typeだけではowner successorを表せないため、trusted metadataをuse planとD083のparameter保持解析へ接続する必要がある |
| writable-successor fast path | `Store`をD083のowned native entryへ伝播できればlast-use argumentをConsumeできるが、storage再利用はruntime uniqueness検査に依存する |
| managed Stateまたは`Symbol` element | D055の`Share`、`Consume`、`Drop`と、型別lifecycle glueをruntime callbackへmaterializeする境界に依存する |
| Bufferのmal実装 | IxPoolがmanaged elementを正しく扱えれば、Buffer固有のretain/release loweringには依存しない |
| plugin-defined Engram leaf | representation、lifecycle、artifact dependencyを登録する安定したtrusted extension contractが別途必要になる |

したがって、`IxPool`をbuilt-in Engramとして先に実装し、既存compilerが知る`Storable(State)`と`Storable(T)`だけを受理することは、
一般plugin ABIより前に検証できる。ただしunmanaged scalarだけへ制限したIxPoolでは本案の重複削減を十分に確認できない。`Symbol`を
含むmanaged elementを扱う段階までに、[managed valueのownership](../../implementation/ownership.md)が定める型別`share`と`drop`を
通常値、IxPool callback、closure environment destructorから共有できる必要がある。

## Plugin境界

pluginはIxPool container algorithmの必須実装場所ではない。初期段階ではcompilerとruntimeが`IxPool<State, T>`を組み込み、Bufferその他を
malで実装できる。境界が安定した後、IxPool自体または新しいEngram leafをcompilerと同じversionへ静的に結合するtrusted crateへ
移せるかを評価する。

新しいEngram leafの登録には、layoutだけでなくvalid valueの構築、runtime representation、`share`、`drop`、relocation、保持する
child Engram、runtime source選択が必要である。operationごとのoperand effectは[所有権primitive](ownership-primitives.md)、
`Storable`などのpropertyを登録する条件は[identity](identity.md#型形成条件)に従う。任意のsource codeをdestructorとして登録せず、
dropは失敗せずI/OやExtern resourceの`close`のような観測可能な作用を持たない。pluginが提供するExtern resourceのlifetimeは
Engram回収へ結合せず、従来どおり明示したhost operationが所有する。

## 段階的な検証

1. opaque identity、file-local representation view、別fileからの構築と分解の拒否をfrontend testで固定する。
2. backend内部にIxPoolのVacant/Live遷移を置き、unmanaged Stateとelementで`init`、`take`とその派生operationを実行する。
3. `Symbol`とmanaged aggregateでshare/drop回数、relocation、同じvalueの書き戻し、IxPool終了時のlive allocation 0を検査する。
4. IxPool上に実験的なdense containerをmalで実装し、現在のBufferとalias、range、overlap、trap semanticsを比較する。
5. ImPool上のimmutable arrayで更新を検証し、shared時のcopyとlast-use時のstorage再利用を別々に測る。
6. 木やgeneration付きhandleのcontainerをcoordinateで実装し、coordinateの再利用と古いhandleの拒否を検査する。
7. semanticsと生成物のcostが妥当な場合だけ、predefined Bufferの置換とtrusted crate境界を別々に判断する。

compilerを変えない二つの試作が、step 4と6の一部を先取りした。結果は[試作で確かめたこと](prototypes.md)に置く。

## 非目標

- raw allocation、pointer arithmetic、manual `init`、manual `drop`、`free`を一般mal codeへ公開しない。
- IxPoolだけを理由にborrow checker、linear type、user-defined finalizer、tracing GCを導入しない。
- `Representable(T)`やpublic C carrierを`Storable(T)`から自動的に導かない。
- plugin ABI、dynamic loading、package system、friend fileをIxPoolの初期実装条件にしない。
- 現在のBufferを、semantic parityと代表的なperformance測定なしに置き換えない。

## 未決定事項

- IxPool primitiveの名前と、IxPoolの名前を`require`したfileだけへ導入する規則。
- [測定後の候補](primitives.md#測定後の候補)の`moveRange`、IxPoolとhostの直接の交換、ImPoolの範囲の写しを足すか。
- live slot iterationをcoreに持つか、core外のextensionにするか、containerに任せるか。
- 任意coordinateのVacantを持つIxPoolと、Vacantを末尾だけに限ったdense primitiveへ`[Unit, T]`を載せる形の比較。意味は同じであり
  （[試作](prototypes.md#ixpoolとbufferの非対称)）、占有metadataとIxPool終了時の走査に対する、slotごとのsum tagと移動ごとの
  `Share`と`Drop`のcostは測っていない。
- IxPoolを使えるfileを、どのfileにも開くか、標準libraryとtrustedなcodeだけに限るか。後者では利用者はBufferと標準の
  containerだけを見る。
- [`Storable`を保持の可否に絞り、値の意味が変わらないことを`Stable`へ分ける案](identity.md#判定の分割案)。採ると
  `Buffer<Buffer<T>>`やIxPoolの入れ子を書ける。[D075](../../history/decisions/active/D075.md)の見直しを伴う。
- [ImPool](identity.md#impool)をIxPoolと対のprimitiveとして持つか。持つ場合のAPIと、uniqueness検査をruntimeへ置く範囲。
- [freezeとthaw](identity.md#freezeとthaw)でstorageを共有するか。共有するとIxPoolへの書き込みのたびに共有中かの確認が入る。
- opaque型のdiagnosticと、public APIがrepresentationを返せる範囲。
- IxPool callbackを既存Buffer callbackから一般化するか、共通lifecycle planを先に抽出するか。
- plugin crateのversion、reproducible build、artifact cache、runtime source選択のcontract。
