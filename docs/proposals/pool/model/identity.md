# identityと型判定

Status: Exploratory support document; rebased on mal v0.7

この文書は、共通の[Pool state](semantics.md#pool-state)に対するsource carrierの観測則と、Poolに関係する`Storable`、
lifecycle、extern admissionの境界を管理する。Pool全体のauthorityは[根本モデル](foundations.md#authority)、responsibilityの遷移は
[runtime contract](../runtime/contract.md#responsibility)、ImPoolのAPIは[Pool primitive](../api/pool.md#impool)を正とする。
現行`Storable`とruntime carrierは[`Buffer`](../../../spec/memory.md)、extern境界は
[`extern`](../../../spec/extern.md)と[C ABI](../../../spec/c-host-abi.md)を正とする。

## 判定が答える問い

型をplaceへ保存できること、carrierのShareとDropが必要なこと、extern signatureへ出せることは別の問いである。

| 判定または分類 | 問い | 所有する層 |
|---|---|---|
| `Storable(T)` | typed Engram placeが`T`のcarrierを保持し、lifecycleを完結できるか | languageとstorage mechanism |
| stability | `T`からMal-owned shared mutable identityの変更を観測できるか | source authority |
| `Lifecycle(T)` | carrierのShareとDropにglueが必要か | execution ownershipとbackend |
| extern admission | closed concrete runtime carrierをC signatureへ出せるか | languageとC runtime extension ABI |

mal v0.7には`Representable`、`HostMappable`、source-level `Address`、canonical host memory boundaryはない。extern Cは同じartifactの
runtime carrierを直接扱い、managed parameterをborrowし、managed resultをowned moveとして返す。したがってPoolの型形成をportableな
memory representationへ結び付けない。

stabilityはこのproposalでcompiler judgmentにしない。structural snapshot、Map key、serializationでは安定させる対象が異なる。
複数のAPIが同じ推移的条件を要求した時点でだけ共通judgmentを検討する。

## handleもvalueである

IxPool、ImPool、external opaque carrier、scalarはすべてsource valueである。分類するのはvalueかどうかではなく、そのcarrierから何へ
到達し、どの変更を観測できるかである。

| source carrier | 到達先 | carrierを別bindingへ渡した後の観測 |
|---|---|---|
| IxPool handle | Mal-owned shared identity | どのhandleからの更新も他のhandleから観測する |
| ImPool snapshot | 保存したPool state | 更新はsuccessorを返し、以前のHeader、capacity、slot carrierは変わらない |
| external opaque carrier | Extern-owned referent | referentの観測はextern contractに従い、carrierはlifetimeを延長しない |
| plain data | carrier自身 | operationが返す新しいvalueだけが異なる |

IxPool handleをplaceへ保存することは、handle carrierへのresponsibilityと、identityへ到達するauthorityを保存することである。同じhandleを
複数のplaceへ保存すれば、それぞれのcarrier responsibilityが同じidentityを生かし、変更を共有観測する。exclusive authorityを移すのではない。

```text
place A ─┐
place B ─┼─ handle ─▶ identity i
local  ──┘
```

## `Storable`の原理

`Storable(T)`はvalue semanticsの分類ではなく、typed placeのlifecycle contractである。placeは次を型だけから実装できなければならない。

- 有効なcarrierを保持し、readでは必要なresponsibilityをShareする。
- exchangeではresponsibilityをcarrierとともに移す。
- replaceとplace終了では保持したresponsibilityをDropする。
- physical relocationでsource上のauthorityと観測を変えない。
- 現在の回収方式で、型から見えないowner back-edgeをstorageへ導入しない。

現行Buffer規則へPoolを加えた到達形は次である。

```text
Storable(Unit | numeric scalar | ByteSize | USize | Symbol)
Storable(external opaque type)
Storable((A...))                 if all Storable(A)
Storable([A...])                 if the sum has at least two variants and all Storable(A)
Storable(Buffer<A>)              if Storable(A)
Storable(IxPool<H, V>)           if Storable(H) and Storable(V)
Storable(ImPool<H, V>)           if Storable(H) and Storable(V)
not Storable(function)
not Storable(empty sum)
```

transparent aliasは展開後、file-local opaque型はhidden representationから判定する。external opaque carrierを保存してもresourceの
lifetimeは延長せず、closeなどを暗黙に実行しない。`Buffer<Buffer<T>>`とexternal opaque elementはD096、D097で既に採択済みである。
Pool handleは同じ再帰へ加え、Pool専用のstorage admissionを作らない。

read、fill、copyが返すhandleは同じidentityを指す。functionを除く理由はmutable identityではなく、closure environmentの型に
capture edgeが現れず、containerをcaptureしたclosureを同じcontainerへ保存するとowner cycleを型から検出できないためである。
BufferやPoolのhandle nestingは、recursive value typeがなくfile-local opaque representationの再帰も拒否されるため、このhidden
back-edgeを導入しない。

## lifecycleとruntime carrier

specialization後の各型は現行compilerと同じ分類を持つ。

```text
Lifecycle(T) = Trivial
             | Owned(share glue, drop glue)
```

`Storable`はplaceへ入れられるかを答え、`Lifecycle`はoperationをどうlowerするかを答える。scalarとexternal opaque carrierは
`Storable`かつ`Trivial`、Symbol、Buffer、Pool handleは`Storable`かつ`Owned`である。functionは`Owned`だが`Storable`でない。
Headerとslot payloadはspecialization後のruntime carrier layoutで置き、product field、sum tagとpayload、paddingをLLVM moduleと
型別glueが同じtarget planから扱う。portable encodingや別のcanonical layoutは定めない。

## structural snapshot

ImPoolが保存するのはHeader、capacity、各slotのcarrierである。successorを作っても旧snapshotから得るcarrierは置換されない。
carrierがhandleなら、そのhandleが指すreferentの変更までは固定しない。

```text
s0.slot[0] = handle i
s1 = slot(s0, 0, handle j)

peek(s0, 0) = handle i
peek(s1, 0) = handle j
```

この後identity `i`が別のhandleから変更されても、`s0`のslot 0は依然として`handle i`である。変わったのは保存したcarrierでなく、
carrierから到達するreferentである。したがってIxPoolとImPoolの型形成条件は同じであり、ImPoolに推移的immutabilityを要求しない。

```text
IxPool<H, V> is well-formed if Storable(H) and Storable(V)
ImPool<H, V> is well-formed if Storable(H) and Storable(V)
```

`freeze`と`thaw`もcarrierを保存するshallowな変換であり、handle referentをcloneしない。serialization、content-addressed key、
deep snapshotは、それぞれが必要とする別の条件と明示的なencodeを持つ。

## extern admission

D098ではextern parameterとresultにfunctionを含まないclosed concrete runtime carrierを認めるが、これは新しいsource typeを自動的に
externへ加える規則ではない。最初のPool採択ではIxPool、ImPool、Vectorをextern signatureへ認めず、Pool kernelも`mal.h`のpublic
operationにしない。Cと値を交換するcontainer APIは、既にadmittedな`Buffer`、`Symbol`、aggregate、external opaque typeで表す。

後続decisionでPool carrierを認める場合は、次を一緒に追加する。

- generated C headerにLLVMと同じtarget runtime carrierを出す。
- managed parameterをborrow、managed resultをowned moveとし、保持には型別share、終了にはdropを使う。
- Poolのstable objectとslot storageを`mal.h` operationから正しいlifecycleで操作する。
- exact-match artifact、thread confinement、growth後のpointer再取得、contract違反後は保証しないというD098の規則を引き継ぐ。

この拡張にもraw pointer、canonical host representation、generic extern、callback/reentryは導入しない。別artifactや永続storageとの交換は
`Buffer<UInt8>`または`Symbol`へ明示的にencode、decodeするextern operationが所有する。

## minimality監査

- PoolとBufferの型形成は一つの`Storable`を共有する。
- handle nestingに別の`Placeable`や`Managed` judgmentを追加しない。
- ImPoolはstructural snapshotなので`Stable`を要求しない。
- runtime layoutとlifecycleは現行のtarget carrier planと`Lifecycle`を再利用する。
- extern admissionはplace admissionから独立させ、初回採択のsurfaceを増やさない。
- closure cycleはfunctionのstorage admissionとして残し、mutable identity一般を除外しない。

## 入れ子

| 外側 | 内側 | 保存されるもの | 内側の更新 |
|---|---|---|---|
| IxPool上のcontainer | IxPool handle | handle authorityとresponsibility | 全aliasから観測する |
| ImPool上のcontainer | IxPool handle | snapshot内のhandle carrier | referentの変更は全handleから観測する |
| IxPool上のcontainer | ImPool snapshot | snapshot carrier | successorを外側へ書き戻す |
| ImPool上のcontainer | ImPool snapshot | snapshot carrier | successorを外側のsuccessorへ書き戻す |
| 一つのIxPool | coordinate | scalar coordinate | 同じPool identity内を直接更新する |

snapshot内のhandleを取り出してreferentを変更しても外側snapshotのslotは変わらない。外側の構造まで更新する場合だけsuccessorを作る。
