# BufferとVector

Status: Exploratory support document

この文書は、一つの密な列をshared mutable identityとして扱う`Buffer`と、structural snapshotとして扱う`Vector`の意味、
operationの分担、現行Bufferからの移行境界を管理する。Pool primitiveの区分は[Pool primitive](pool.md#区分)、現行Bufferの
規範的な意味とpreconditionは[AddressとBuffer](../../../spec/memory.md)、Vectorの参照実装は
[列のcontainer](../containers/sequences.md#vector)を正とする。

現行仕様にはVector、IxPool、ImPoolは存在しない。本書は採択済みBuffer APIを直ちに変更する決定ではなく、Pool案を採択するときに
別々に判断すべき意味論、実装、互換性を分離する。

## BufferとVectorの対

BufferとVectorは、どちらも`[0, n)`の全coordinateがcarrierを持つ密な有限列である。Bufferでは`n`をcount、Vectorではlengthと呼ぶ。
違うのは列を更新したときの観測則である。

| 型 | source valueが運ぶもの | 更新 | 旧valueからの観測 |
|---|---|---|---|
| `Buffer<A>` | shared mutable identityへのhandle | referentの列を変更する | 同じidentityへの全handleから変更を観測する |
| `Vector<A>` | 列構造のsnapshot | 変更後のsuccessorを返す | 旧snapshotのlengthとelement carrierは変わらない |

参照実装では同じinvariantをIxPoolとImPoolへ置く。

```mal
opaque Buffer<A> :: IxPool<USize, A>;    // Metaはcount、[0, count)がLive
opaque Vector<A> :: ImPool<USize, A>;    // Metaはlength、[0, length)がLive
```

この対応は表現同一性を要求しない。BufferとVectorはpreludeのopaque型として意味を参照実装で定め、runtimeは観測できる結果が同じなら
占有tagのない密なstorage、bulk operation、last-use時のstorage移動を使える。capacityとspare storageはopaque representationであり、
length、element carrier、更新の独立性だけがsequence semanticsに現れる。

Vectorのsnapshotはelement carrierについてstructuralであり、推移的なdeep immutabilityではない。handle elementを含むVectorの
successorを作っても旧Vectorのslot carrierは置換されないが、両方に保存された同じhandleからreferentの変更は共有観測される。
これは[ImPoolのstructural snapshot](../model/identity.md#structural-snapshot)を密な列へ限定した規則である。

## 語彙の分担

memory操作は、抽象する単位で次の語彙に分かれる。

| 語彙 | 抽象する単位 | 操作 | 所有する層 |
|---|---|---|---|
| slot | 一つのcoordinateのplace | `peek`、`swap`、`grow`、Meta | IxPool / ImPool |
| mutable sequence | 一つのLiveなrun `[0, count)` | construction、append、read、replace、length、growth policy | Buffer |
| snapshot sequence | 確定したcarrierのrun `[0, length)` | construction、append、read、successor、slice、length | Vector |
| mutable run | 既存identity内の連続範囲 | `fill`、`copy` | Buffer |
| canonical memory boundary | host storageと密な値列の交換 | admission、observation | Vectorを正規形とする候補 |

IxPoolとImPoolはslotを抽象し、dense run、順序、host storageを知らない。BufferとVectorは同じdense sequence invariantを持つため、
slot stateを利用者へ公開しない。Mapや木のLive集合は連続区間にならず、Dequeの成長に必要なのは値を複製する`copy`ではなく
place間の移動なので、run operationをPoolへ上げない。

canonical memoryとの交換はPoolの責務ではない。交換対象はVacantを含み得るPool stateでなく、全positionに値を持つ密な列だからである。
その正規形をVectorに置くと、admissionは新しいsnapshot valueの構成、observationはsnapshot valueのcopyとして説明できる。

ただし、これはauthority上Vectorだけがhostと交換できるという意味ではない。現行`from`はExternから読んだcarrierで新しい
mal-owned Buffer identityを構成し、現行`buffer.into`はBufferを変更せず観測するため、どちらもEngram/Extern境界を破らない。
Vectorを正規形にすることと、現行Buffer APIを削除することは別のdecisionである。

## `freeze`と`thaw`

BufferとVectorの間の変換は列構造についてshallowである。

```text
freeze(Buffer<A>) -> Vector<A>
thaw(Vector<A>)   -> Buffer<A>
```

`freeze(buffer)`は呼出時のcountと`[0, count)`のelement carrierを持つVectorを返す。以後のBufferのappend、replace、fill、copyは
Vectorのlengthとcarrierを変えない。`thaw(vector)`は同じlengthとcarrierで初期化した新しいBuffer identityを返し、以後のBuffer変更は
Vectorを変えない。どちらもoperandをconsumeせず、handle elementのreferentをcloneしない。同じVectorを複数回thawした結果は別々の
外側Buffer identityを持つが、要素として保存した同じhandleのreferentは共有する。

実装は通常copyし、last useとruntime上の一意性を利用できる場合はstorageを移動または再利用してよい。Vector同士のsliceは不変な
storageを共有できるが、liveなBuffer aliasとVectorの間でstorageを共有してBufferの全書き込みにcopy確認を課す方式は採らない。
sourceへuniqueness、consume、物理capacityを公開しない。

## operationの対応

次の表は意味上の対応であり、Vector側の名前は未決定である。特に現行`from`と同じ名前をresult typeだけでBuffer版とVector版へ
overloadできるとは仮定しない。

| 役割 | Buffer | Vector |
|---|---|---|
| 空を作る | `make<A>(capacity)` | empty construction（名前未定） |
| 末尾に足す | `buffer.new(value)` | `vectorAppend(vector, value)`（仮） |
| 読む | `buffer.get(index)` | `vector.get(index)`（仮） |
| 書き換える | `buffer.put(index, value)` | `vectorSet(vector, index, value)`（仮） |
| 長さ | `#buffer` | `#vector`（仮） |
| 範囲を埋める | `buffer.fill(offset, count, value)` | 必須でない。successorを返すlibrary operationにはできる |
| 範囲を移す／写す | `buffer.copy(offset, source, sourceOffset, count)` | `slice(vector, offset, length)`（仮） |
| hostから読む | 現行`from<A>`、またはVector admission後の`thaw` | Vector admission primitive（名前未定） |
| hostへ書く | 現行`buffer.into`、または`freeze`と`slice`後のobservation | Vector observation primitive（名前未定） |
| `Symbol`へ | 現行`*buffer`、または`symbol(freeze(buffer))` | `symbol(vector)`（仮） |
| `Symbol`から | 現行`*symbol` | `freeze(*symbol)`で導出できる |
| 相手への変換 | `freeze(buffer)` | `thaw(vector)` |

Vectorのempty construction、append、read、set、length、sliceはImPool operationからmalで定義でき、trusted primitiveを必要としない。
host storageのdereferenceと`Symbol`の専用表現の構築だけはmalで書けないため、Vectorを採択するならtrusted operationになる。

## Buffer

Pool案の正規形では、BufferはIxPoolへdense sequence invariantとgrowth policyを加えたpreludeのopaque型である。参照実装は
[Buffer実装](../containers/buffer.md)が定める。`make`、`new`、`get`、`put`、`fill`、`copy`、`#`の意味、評価順、alias、
未検査precondition、overflow trapは現行仕様から変えない。elementの型形成は既に
[place lifecycleとしての`Storable`](../model/identity.md#storableの原理)へ揃い、Buffer handleを要素にできる。Pool採択時にはPool handleも
同じ規則で要素に加える。

| operation | 参照実装上の位置 |
|---|---|
| `make`、`new`、`get`、`put`、`#` | IxPoolのslot operationとMetaにdense invariantを加える |
| `fill`、`copy` | slot operationのloop。runtimeは同じcarrier lifecycleになるbulk operationを使える |
| `*buffer` | `symbol(freeze(buffer))`と同じsnapshot |
| `*symbol` | Symbolのbyte列からVectorを構成して`thaw`したものと同じmutable copy |
| 現行`from` | Vector admissionを`thaw`したものと同じ新しいBuffer identity |
| 現行`buffer.into` | 対象rangeをstructural snapshotとして観測し、Vector observationと同じcanonical copyを行う |

この対応は、現行`from`、`buffer.into`、`*`を実装から直ちに除くことを要求しない。採択時には既存名をcompatibility operationとして
残す、deprecated wrapperにする、破壊的に置き換える、のいずれかを別decisionで選ぶ。wrapperを残してもruntimeはfreeze、slice、thawの
中間objectを作らず、一段のallocationまたはbulk copyへlowerできる。

containerが現行runtimeと同じoverflow trapをmalで起こすには、[primitive `trap`案](../../primitive-trap.md)の
`trap :: Symbol -> []`を使う。Bufferをmalの参照実装へ移す判断は、この依存も含む。

## Vector

VectorはImPoolへdense sequence invariantを加えたpreludeのopaque型である。更新は外側の列構造を変えずsuccessorを返す。
elementがhandleでも`Storable`なら保存できるが、hostとの交換は`Representable`なelementに限るため、Mal-owned handle authorityを
canonical memoryへ出さない。

host boundaryの意味は、公開名を決める前に次の形で固定できる。

```text
vector admission : (Address, USize, USize) -> Vector<A>
vector observation : (Vector<A>, Address, USize) -> Unit
symbol : Vector<UInt8> -> Symbol
```

| operation | 意味 | precondition |
|---|---|---|
| admission | host storageの`[offset, offset + length)`をcopyした、長さ`length`のVectorを返す | `Representable(A)`。対象rangeがreadable、初期化済みで、各要素がvalid canonical representationを持つ |
| observation | Vector全体をhost storageの`[destinationOffset, destinationOffset + length)`へcopyする | `Representable(A)`。対象rangeが`length`要素分writable |
| `symbol(vector)` | `[0, length)`と同じbyte列の`Symbol`を返す | elementは`UInt8` |

runtimeは`Representable`なelement carrierをcanonical layoutで連続に置き、admissionとobservationをbulk copyで実装してよい。
この配置はsourceから観測できない。host側のextent、permission、initialization、representationと、offset、length、allocation sizeの
overflowは、現行の[C host copy boundary](../../../spec/memory.md#c-host-copy-boundary)と同じcontractを引き継ぐ。

`Symbol`は意味の上ではimmutableなbyte sequenceだが、Vector一般のaliasではない。text operation、literal、専用表現を持つ既存型であり、
`symbol`と`freeze(*symbol)`が二つの型の値を変換する。実装上`Symbol`を`Vector<UInt8>`と同じowner/view表現にしてもよいが、
source-levelの型同一性は別に判断する。

## 設計、実装、移行の順序

BufferとVectorを揃える順序には三種類ある。

1. 設計では、現行Bufferを互換性の基準にし、Buffer/Vectorのsequence semanticsと変換をPool採択前に確定する。これによりIxPoolと
   ImPoolの差を、container固有のAPIでなくhandleとsnapshotの観測則として検査できる。
2. 実装では、型別lifecycle、IxPool、ImPoolを先に作り、その上へBufferとVectorの参照実装を載せる。Vectorを一時的な別built-inとして
   先行実装し、後でImPoolへ載せ替える必要はない。
3. 移行では、拡張`Storable`を現行Bufferで先に検証できる。PoolとVectorの意味と費用が確認できた後にだけ、predefined Bufferの
   実装置換とhost operationの公開surfaceを判断する。

この順序により、Bufferの互換性、Vectorの新しい価値、Poolのminimalityを一つのall-or-nothingな変更にしない。

## 採択前に残る判断

- Vectorのempty construction、append、read、set、length、sliceの公開名。既存Buffer名とoperation familyで共有するか。
- Vectorのhost admissionとobservationの公開名。現行`from`とresult typeだけでoverloadしない。
- 現行Bufferの`from`、`into`、`*`を恒久的なcompatibility operationとして残すか。
- Vector observationを常に全体copyとし、部分範囲は`slice`へ分けるか、range引数を直接持たせるか。
- `Symbol`をsource-levelでも`Vector<UInt8>`のopaque specializationとみなすか、変換可能な別型のまま保つか。
- VectorをPoolと同時にpublic preludeへ採択するか、ImPoolの検証用参照containerとして先に確定して公開採択を分けるか。
