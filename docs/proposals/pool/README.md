# Poolとopaque型によるcontainer基盤

Status: Exploratory

この文書は、現在の`Buffer<T>`が持つtyped storage mechanismとdense sequence policyを分離し、少数のtrustedな
`Pool<State, T>` primitive上でcontainerをmal sourceとして定義する案を管理する。現行仕様は
[AddressとBuffer](../../spec/memory.md)、managed responsibilityは[D055](../../history/decisions/active/D055.md)と
[D083](../../history/decisions/active/D083.md)、carrier lifecycleの現在の実装境界は
[D080](../../history/decisions/active/D080.md)を正とする。本案はまだsource syntax、`Pool` API、plugin ABIを定めない。

## 目的

現在の`Buffer<T>`はmal-owned storageに加えて、共有mutable identity、denseな`[0, count)`、append、index access、growth、range
operationを一つの組み込み型として持つ。この形はArrayまたはVectorとして有用だが、slot map、deque、heap、tree、graph、arenaなどを
追加するたびにallocation、relocation、element lifecycleを別の組み込みruntimeへ複製するか、Bufferのindexとrow encodingへ
container policyを押し込む必要がある。

本案は、malが所有する可変storageのmodelとしてPoolを置き、BufferとMap、Deque、heap、treeなどをその上のopaque型として
malで定義する。利用者へraw storage、uninitialized value、manual `drop`を公開せず、container実装者もstride、alignment、retain、
releaseを型ごとに再定義しない。layout、lifecycle correctness、allocation failureはtrusted layerに残し、logical State、denseまたは
sparseな利用、growth、free list、ordering、hashing、snapshot policyをmal側が選ぶ。このpolicyを選べずBufferと同じdense sequenceしか
作れないならPoolを独立させる意味はなく、逆にraw memory操作まで開くことも本案の目的ではない。

## 設計の軸

本案が決める事項は次の六つの軸に分かれ、それぞれを一つの文書が所有する。

| 軸 | 問い | Poolでの答え | 所有する文書 |
|---|---|---|---|
| 所有権 | 誰がresponsibilityを持つか | `init`と`take`、operand effect `Store` | [所有権primitive](ownership-primitives.md) |
| lifecycle | carrierをいつ確保、移動、解放するか | `reserve`、Pool終了、型別glue | [lifecycle contract](lifecycle-contract.md) |
| identity | 変更を誰が観測し、何を`Storable`にできるか | 共有Pool、`ValuePool`、key | [identity](identity.md)、[key extension](pool-keys.md) |
| 妥当性 | どのplaceがLiveで、誰がそれを保証するか | 未検査preconditionとcontainer invariant | [lifecycle contract](lifecycle-contract.md#未検査precondition) |
| 権限 | 誰がPoolへ直接触れるか | file-local opaque型の宣言元file | 本書と[file-local opaque type](../../spec/types.md#file-local-opaque-type) |
| 表現 | どのbitで保持し、hostとどう交換するか | runtime representationとdata primitive | [lifecycle contract](lifecycle-contract.md#runtime-representation) |

妥当性と権限は対になる。Poolのpreconditionを未検査にできるのは、Poolを直接呼ぶfileをopaque型で一つに閉じ込め、そのfileの
invariantでpreconditionを満たせるからである。

## 文書の構成

API、語彙、containerは次の文書が所有する。

- [primitive一覧](primitives.md)：memoryの層、coordinateの線形性、slot、run、sequenceの語彙、PoolとBufferの責務、全primitive
- [run protocol](run-protocol.md)：`from`、`into`、`copy`、`fill`のfamilyとrun primitiveの意味
- [Pool上のcontainer](containers.md)：代表的なcontainerの比較、Bufferの前提を外すと変わること、Poolの輪郭

次の文書は、規則を具体的なcodeで確かめる例と結果である。

- [Buffer実装](buffer-implementation.md)：現行Bufferの全operationをPoolとcompiler primitiveで書いた形
- [collection例](collection-examples.md)：stack、binary heap、open addressing Map、slot map、木
- [既存exampleとの差分](current-examples.md)：`examples/`をPool上へ移したときのsource、precondition、costの変化
- [array ownership](array-ownership.md)：mutable arrayとcopy-on-write immutable arrayの比較
- [試作で確かめたこと](prototypes.md)：C host試作とBuffer上のemulationの結果

## file-local opaque型

Poolを安全なcontainerへ閉じ込めるには、transparent aliasとは別に
[file-local opaque type](../../spec/types.md#file-local-opaque-type)を使う。

```mal
opaque Buffer<T> :: Pool<USize, T>;
```

hidden representationを観察できるのは宣言元source fileだけであり、この権限は`require`先へ移らない。同じfileの通常のmal bindingは、
専用のpack/open構文なしに`Buffer<T>`をPool operationへ渡し、Poolを`Buffer<T>`が期待される位置から返せる。このviewは
allocation、copy、新しいEngram identityを作らず、同じcarrier responsibilityを受け渡す。opaque型はgeneric specializationと
operation familyのkeyにdeclaration identityを残し、同じrepresentationを持つ二つのopaque型を混同しない。

mutable metadataはPoolの`State`として共有identity側に置く。BufferはStateをcountとして使い、Mapは要素数を、Dequeは`(head, count)`を
Stateに持つ。各fileは、`reserve`とslot遷移だけを提供するPool上に、自分のState invariantとgrowth policyを実装する。

## Bufferと上位container

Bufferは組み込み型ではなく、Pool上のpreludeのopaque型になる。仕様上のBufferは[Buffer実装](buffer-implementation.md)の参照実装で
意味を定め、実装は同じ結果になる限り専用runtimeを使ってよい。Bufferが提供する語彙とPoolとの責務分担は
[primitive一覧](primitives.md#poolとbufferの責務)が所有する。

element equality、hash、orderingはPoolやpluginへ埋め込まず、通常のfunction引数または
[operation family](../../spec/operation-families.md)のrequirementとして上位algorithmが要求する。standard Bufferのcore operationを
downstream sourceが同じidentityのまま上書きする仕組みは導入しない。一つのopaque型のrepresentation invariantと公開operationは
宣言元fileが所有し、別policyは別のopaque型として定義する。

## Managed Engramとの依存関係

本案の各部分は同じ段階を要求しない。

| 部分 | managed Engramへの依存 |
|---|---|
| file-local opaque identityとrepresentation view | frontendのsyntax、resolve、type checkingに閉じ、unmanaged representationだけならD055/D080に依存しない |
| `Pool<State, T>`のstateとslot | D080の`initialize`、`replace`、`vacate`と同じcarrier invariantをruntime storageへ適用する |
| operand effect `Store` | function typeだけではowner successorを表せないため、trusted metadataをuse planとD083のparameter保持解析へ接続する必要がある |
| writable-successor fast path | `Store`をD083のowned native entryへ伝播できればlast-use argumentをConsumeできるが、storage再利用はruntime uniqueness検査に依存する |
| managed Stateまたは`Symbol` element | D055の`Share`、`Consume`、`Drop`と、型別lifecycle glueをruntime callbackへmaterializeする境界に依存する |
| Bufferのmal実装 | Poolがmanaged elementを正しく扱えれば、Buffer固有のretain/release loweringには依存しない |
| plugin-defined Engram leaf | representation、lifecycle、artifact dependencyを登録する安定したtrusted extension contractが別途必要になる |

したがって、`Pool`をbuilt-in Engramとして先に実装し、既存compilerが知る`Storable(State)`と`Storable(T)`だけを受理することは、
一般plugin ABIより前に検証できる。ただしunmanaged scalarだけへ制限したPoolでは本案の重複削減を十分に確認できない。`Symbol`を
含むmanaged elementを扱う段階までに、[managed valueのownership](../../implementation/ownership.md)が定める型別`share`と`drop`を
通常値、Pool callback、closure environment destructorから共有できる必要がある。

## Plugin境界

pluginはPool container algorithmの必須実装場所ではない。初期段階ではcompilerとruntimeが`Pool<State, T>`を組み込み、Bufferその他を
malで実装できる。境界が安定した後、Pool自体または新しいEngram leafをcompilerと同じversionへ静的に結合するtrusted crateへ
移せるかを評価する。

新しいEngram leafの登録には、layoutだけでなくvalid valueの構築、runtime representation、`share`、`drop`、relocation、保持する
child Engram、runtime source選択が必要である。operationごとのoperand effectは[所有権primitive](ownership-primitives.md)、
`Storable`などのpropertyを登録する条件は[identity](identity.md#型形成条件)に従う。任意のsource codeをdestructorとして登録せず、
dropは失敗せずI/OやExtern resourceの`close`のような観測可能な作用を持たない。pluginが提供するExtern resourceのlifetimeは
Engram回収へ結合せず、従来どおり明示したhost operationが所有する。

## 段階的な検証

1. opaque identity、file-local representation view、別fileからの構築と分解の拒否をfrontend testで固定する。
2. backend内部にPoolのVacant/Live遷移を置き、unmanaged Stateとelementで`init`、`take`とその派生operationを実行する。
3. `Symbol`とmanaged aggregateでshare/drop回数、relocation、同じvalueの書き戻し、Pool終了時のlive allocation 0を検査する。
4. Pool上に実験的なdense containerをmalで実装し、現在のBufferとalias、range、overlap、trap semanticsを比較する。
5. keyを持たないimmutable arrayでwritable successorを検証し、shared時のcopyとlast-use時のstorage再利用を別々に測る。
6. slot mapまたはtreeをcoordinateで実装し、keyが実際に必要ならidentity、generation、stale/cross-Pool rejectionを独立して検査する。
7. semanticsと生成物のcostが妥当な場合だけ、predefined Bufferの置換とtrusted crate境界を別々に判断する。

compilerを変えない二つの試作が、step 4と6の一部を先取りした。結果は[試作で確かめたこと](prototypes.md)に置く。

## 非目標

- raw allocation、pointer arithmetic、manual `init`、manual `drop`、`free`を一般mal codeへ公開しない。
- Poolだけを理由にborrow checker、linear type、user-defined finalizer、tracing GCを導入しない。
- `Representable(T)`やpublic C carrierを`Storable(T)`から自動的に導かない。
- plugin ABI、dynamic loading、package system、friend fileをPoolの初期実装条件にしない。
- 現在のBufferを、semantic parityと代表的なperformance測定なしに置き換えない。

## 未決定事項

- Pool primitiveの名前と、Poolの名前を`require`したfileだけへ導入する規則。
- [run protocol](run-protocol.md#未決定事項)が必要とする`Representable`のrequirement伝播。
- live slot iterationをcoreに持つか、core外のextensionにするか、containerに任せるか。
- 任意coordinateのVacantを持つPoolと、Vacantを末尾だけに限ったdense primitiveへ`[Unit, T]`を載せる形の比較。意味は同じであり
  （[試作](prototypes.md#poolとbufferの非対称)）、占有metadataとPool終了時の走査に対する、slotごとのsum tagと移動ごとの
  `Share`と`Drop`のcostは測っていない。
- [Pool key extension](pool-keys.md#未決定事項)のgeneration幅、iteration、Arena State、key equality。
- immutableな`Array<T>`を`Storable`にする[`ValuePool<State, T>`](identity.md#valuepool)を、identityを共有するPoolと別の型として持つか。
- opaque型のdiagnosticと、public APIがrepresentationを返せる範囲。
- Pool callbackを既存Buffer callbackから一般化するか、共通lifecycle planを先に抽出するか。
- plugin crateのversion、reproducible build、artifact cache、runtime source選択のcontract。
