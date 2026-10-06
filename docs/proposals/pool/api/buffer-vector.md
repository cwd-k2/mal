# BufferとVector

Status: Exploratory support document; rebased on mal v0.7

この文書は、一つの密な列をshared mutable identityとして扱う`Buffer`と、structural snapshotとして扱う`Vector`の意味、operationの
分担、現行Bufferからの移行境界を管理する。Pool primitiveは[Pool primitive](pool.md#区分)、現行Bufferの規範は
[`Buffer`](../../../spec/memory.md)、Vectorの参照実装は[列のcontainer](../containers/sequences.md#vector)を正とする。

現行仕様にはVector、IxPool、ImPoolは存在しない。本書は採択済みBuffer APIを直ちに変更せず、Pool採択後に検討する第二段階を示す。

## BufferとVectorの対

```mal
opaque Buffer<A> :: IxPool<USize, A>;    // Headerはcount、[0, count)がLive
opaque Vector<A> :: ImPool<USize, A>;    // Headerはlength、[0, length)がLive
```

これはsource semanticsの分解であり、既存Bufferを必ずIxPool objectと同じ物理表現へ置き換える要求ではない。implementationは公開された
意味、precondition、lifecycle、cost gateを保つ限り、BufferとVectorを専用runtime operationへas-if loweringできる。

両者はdense sequence invariantを共有する。Buffer valueはidentityへのhandleで、更新を全aliasが観測する。Vector valueはstructural
snapshotで、更新はsuccessorを返す。どちらもelement carrierについてshallowであり、elementがhandleなら内側referentをdeep copyしない。

## 語彙の分担

| 層 | 意味 | primitiveまたはinvariant | owner |
|---|---|---|---|
| slot | 一つのcoordinateのplace | `peek`、`swap`、`grow`、Header | IxPool / ImPool |
| dense sequence | `[0, length)`がLiveで順序を持つ | count、append、index、range | Buffer / Vector |
| mutable identity | 更新をaliasから共有観測する | handle semantics | Buffer / IxPool |
| structural snapshot | 以前のcarrier列を保存する | successor semantics | Vector / ImPool |
| runtime extension | 同じartifactのcarrierをCで操作する | borrow、move、share、drop | `extern`とC ABI |
| external encoding | 別artifactやresourceとbytesを交換する | 明示的なcodec contract | 個々のextern operation |

Poolはdense run、順序、encoding、I/Oを知らない。D098により、extern Cとの交換にも別のcanonical memory boundaryはない。Cが現行Bufferや
Symbolを受け取り、`mal.h` operationで直接観測・変更できるため、Vector導入の理由をhost copyの正規形にはしない。

## `freeze`と`thaw`

`freeze(buffer)`は呼出時点のelement carrier列を持つVectorを返す。以後のBuffer更新はVectorから観測されない。`thaw(vector)`は同じ列を
持つ新しいBuffer identityを返し、その変更は元Vectorへ現れない。element carrierがhandleなら内側identityは共有する。

storageをcopyするかmoveするかは[Pool primitive](pool.md#freezeとthaw)のas-if ruleに従う。last useとruntime上の一意性を証明できる
場合はownerを移せるが、source valueの再利用可能性を変えない。

## operationの対応

| 意味 | Buffer | Vector候補 |
|---|---|---|
| 空列 | `make<A>(capacity)` | `vector<A>()`または`freeze(make<A>(0))` |
| 長さ | `#buffer` | `#vector` |
| index read | `buffer.get(index)` | `vector.get(index)` |
| append | `buffer.new(value)` | `vector.push(value)`がsuccessorを返す |
| replace | `buffer.put(index, value)` | `vector.put(index, value)`がsuccessorを返す |
| range fill/copy | in-place mutation | successorを返すrange operation |
| mutableからsnapshot | `freeze(buffer)` | — |
| snapshotからmutable | — | `thaw(vector)` |
| bytesからSymbol | `*buffer` | `symbol(vector)`を採る場合も同じsnapshot semantics |
| Symbolからbytes | `*symbol` | `freeze(*symbol)`で導出できる |

Vector名と個々のoperation名は未決定である。特にexternとの交換用operationは追加しない。I/Oやcodecは現行BufferまたはSymbolを受け取る
program固有externで表せる。

## Buffer

現行Bufferの型形成、runtime carrier layout、未検査precondition、overflow trap、Symbol conversionは変更しない。Pool上の参照実装は
次の分解を検証するために使う。

| 現行Buffer operation | Pool上の意味 |
|---|---|
| `make`、`new`、`get`、`put`、`#` | slot operationとHeaderにdense invariantを加える |
| `fill`、`copy` | shallowなslot read/exchangeのrange loop |
| `*buffer` | 現在のbyte列から独立したSymbol snapshot |
| `*symbol` | byte列で初期化した新しいmutable identity |

C runtime extensionはBuffer carrierを既に直接扱える。`mal_storageof(T)`とBuffer operationはspecialization後のruntime layoutと
`Lifecycle(T)`を使い、nested Bufferやexternal opaque elementも処理する。IxPool上の実装へ置換する場合、このextern-visible behaviorと
generated C/LLVM layout agreementもcompatibility gateになる。

## Vector

Vectorはflatなpersistent sequenceの候補であり、serialization formatではない。`Storable(A)`ならhandleを含むelementも保存できる。
runtime representationはspecialization後のcarrier layoutを使い、snapshot copyではOwned elementをShareする。paddingやinactive sum
payloadを外部formatとして公開しない。

最初のIxPool採択にVectorを含めない。第二段階で採る場合も、まずmal内のsource semantics、更新費用、`freeze` / `thaw`を採択する。
extern signatureへのadmissionはさらに独立した判断とし、認めるならD098のdirect carrier contractへ次を追加する。

- generated headerとLLVMが同じVector carrier layoutを使う。
- parameterはborrow、resultはowned move、Cが保持する値はshare/dropする。
- Cからの更新は旧snapshotを変えずsuccessorを返すruntime operationだけを使う。
- exact ABI match、thread confinement、contract違反後の非保証を引き継ぐ。

raw pointer、`Address`、canonical host layout、generic externは追加しない。portableなbyte交換は明示的なcodecがBufferまたはSymbolを使う。

## 設計、実装、移行の順序

1. 現行Bufferをcompatibility基準にし、IxPool上の参照実装がsequence semanticsとpreconditionを再現することを確認する。
2. IxPool kernelをsource loweringへ接続し、runtime carrier layout、Owned element lifecycle、growth、drop、C runtime extensionを含む現行
   Bufferとのcostと生成物を比較する。
3. Buffer置換の価値がある場合だけas-if loweringを変更する。source APIとextern-visible Buffer ABIの変更は別decisionにする。
4. ImPoolを採択した後にVectorと`freeze` / `thaw`のsource APIを判断する。
5. Vectorをextern signatureへ出す実需がある場合だけC ABIを拡張する。

## 採択前に残る判断

- Vectorをpublic型にするか、ImPool上のlibrary opaque型に留めるか。
- Vector更新の公開名と、range operationをprimitiveにするcost threshold。
- `symbol(vector)`を専用operationにするか、`thaw`と現行`*Buffer<UInt8>`から導くか。
- BufferをIxPool上へ置換したときに現行runtimeのrange operationとC extension APIを同等以上のcostで保てるか。
- Vectorをextern signatureへadmitする必要があるか。
