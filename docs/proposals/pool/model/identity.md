# identityと型判定

Status: Exploratory support document

この文書は、共通の[Pool state](semantics.md#pool-state)に対するsource carrierの観測則と、Poolに関係する
`Storable`、`Representable`、`HostMappable`の境界を管理する。Pool全体のauthorityは
[根本モデル](foundations.md#authority)、responsibilityの遷移は
[runtime contract](../runtime/contract.md#responsibility)、ImPoolのAPIは[Pool primitive](../api/pool.md#impool)を正とする。
現行仕様の`Storable`は[AddressとBuffer](../../../spec/memory.md#storable)に定める。Buffer handleのadmissionは
[D096](../../../history/decisions/active/D096.md)で先に採択した。本書に残る拡張はexternal opaque carrierとPool carrierであり、
それぞれ実装可能性とPool採択に対応する後続decisionで現行仕様を更新する。

## 判定が答える問い

型をplaceへ保存できること、保存後に変更を観測できないこと、hostと表現を交換できることは別の問いである。

| 判定または分類 | 問い | 所有する層 |
|---|---|---|
| `Storable(T)` | typed Engram placeが`T`のcarrierを保持し、lifecycleを完結できるか | languageとstorage mechanism |
| stability | `T`からMal-owned shared mutable identityの変更を観測できるか | source authority |
| `Representable(T)` | canonical memory representationとの値copyが定義されるか | admissionとobservation |
| `HostMappable(T)` | public extern signatureに使うC carrierとbridge contractがあるか | extern boundary |
| lifecycle plan | carrierのShareとDropにglueが必要か | compilerとbackend |

stabilityはこのproposalで新しいcompiler judgmentにしない。structural snapshot、Map key、serializationなど、何を安定させるかで
必要な条件が異なるためである。将来、複数のAPIが同じ推移的条件を要求した時点で`Stable(T)`というjudgmentへまとめられる。

`Representable`と`HostMappable`は`Storable`へ統合しない。external opaque typeはpublic C carrierを持つがcanonical memory copyの
対象ではない。`Symbol`はEngram placeへ保存できるが、どちらのhost境界にも直接は出ない。`Address`は三つを満たせるが、referentの
extent、permission、変更、lifetimeはExtern authorityに残る。

## handleもvalueである

IxPool、ImPool、Address、scalarはすべてsource valueである。分類するのはvalueかどうかではなく、そのcarrierから何へ到達し、
どの変更を観測できるかである。

| source carrier | 到達先 | carrierを別bindingへ渡した後の観測 |
|---|---|---|
| IxPool handle | Mal-owned shared identity | どのhandleからの更新も他のhandleから観測する |
| ImPool snapshot | 保存したPool state | 更新はsuccessorを返し、以前のMeta、capacity、slot carrierは変わらない |
| `Address` capability | Extern-owned referent | referentの観測はextern contractに従い、carrierはlifetimeを延長しない |
| plain data | carrier自身 | operationが返す新しいvalueだけが異なる |

IxPool handleをplaceへ保存することは、handle carrierへのresponsibilityと、identityへ到達するauthorityを保存することである。
同じhandleを複数のplaceへ保存すれば、それぞれのcarrier responsibilityが同じidentityを生かし、変更を共有観測する。exclusive
authorityを移すのではない。

```text
place A ─┐
place B ─┼─ handle ─▶ identity i
local  ──┘
```

このaliasはIxPoolの意味そのものであり、storageから除外すべき異常状態ではない。

## `Storable`の原理

`Storable(T)`はvalue semanticsの分類ではなく、typed placeのlifecycle contractである。placeは次を型だけから実装できなければ
ならない。

- `T`の有効なcarrierを保持し、readでは必要なresponsibilityをShareする。
- exchangeではresponsibilityをcarrierとともに移す。
- replaceとplace終了では保持したresponsibilityをDropする。
- physical relocationでsource上のauthorityと観測を変えない。
- 現在の回収方式で、型から見えないowner back-edgeをstorageへ導入しない。

現在のBuffer規則とPool proposalを合わせた到達形は次になる。

```text
Storable(Unit | numeric scalar | Address | ByteSize | USize | Symbol)
Storable(external opaque type)
Storable((A...))                 if all Storable(A)
Storable([A...])                 if the sum has at least two variants and all Storable(A)
Storable(Buffer<A>)              if Storable(A)
Storable(IxPool<M, V>)           if Storable(M) and Storable(V)
Storable(ImPool<M, V>)           if Storable(M) and Storable(V)
not Storable(function)
not Storable(empty sum)
```

transparent aliasは展開後、file-local opaque型はhidden representationから判定する。external opaque carrierを保存してもhost
resourceのlifetimeは延長しない。これは`Address`を保存する場合と同じであり、保存されたcarrierの有効性は元のextern contractに従う。

`Buffer<Buffer<T>>`は既にこの規則でwell-formedである。IxPool handleを要素にするIxPool、mutable containerをpayloadに持つMapも、
Pool採択後は同じ規則でwell-formedになる。
readが返すhandleは同じidentityを指し、`fill`と`copy`もhandle carrierのshallow copyになる。この観測はIxPoolと現行Bufferの
handle semanticsから直接決まり、特別なnested-container semanticsを追加しない。

functionを除く理由はmutable identityではなく、closure environmentの型にcaptureが現れないことである。containerにclosureを保存し、
そのclosureが同じcontainerをcaptureすると、型構造から検出できないowner cycleを作れる。BufferやPoolのhandle nestingは、表現に寄与する
再帰型がなくfile-local opaque representationの再帰も拒否されるため、owner edgeの型構造が真に小さくなる。D096はD075が一緒に
扱っていたhandle aliasとclosure cycleをこの理由で分離した。

将来、cycleを回収するmechanismまたはcapture effectを導入すればfunctionの`Storable`を再検討できる。これはhandle nestingの採択条件
ではない。plugin-defined Engram leafは自動的にStorableにしない。trusted extensionがlayout、Share、Drop、relocationと、保持するMal owner edgeを
宣言し、同じreclaimability invariantを満たす場合だけconformanceを与える。external opaque typeはMal owner edgeを持たないcopyable
carrierなので、このplugin条件とは別である。

## `Managed`へ改名しない理由

`Storable`を`Managed`へ単に改名しない。現在のimplementationでmanaged valueとは、ownerを持ちShareとDropのglueを必要とする
carrierを指す。scalarと`Address`はStorableだがmanagedでなく、functionはmanagedだがStorableでない。Bufferは両方だが、これだけで
二つの判定は同一にならない。

実装は型ごとに次のlifecycle planを持てばよい。

```text
Lifecycle(T) = Trivial
             | Owned(share glue, drop glue)
```

`Storable`はplaceへ入れられるかを答え、lifecycle planはそのplace operationをどうlowerするかを答える。`Managed`は後者の説明語に
留める。もし実装上の曖昧さが残るなら、`managed type`を`owned carrier`または`needs-drop type`へ改称する方が境界に合う。

## structural snapshot

ImPoolが保存するのはMeta、capacity、各slotのcarrierである。successorを作っても、旧snapshotから得るcarrierは置換されない。
carrierがhandleなら、そのhandleが指すreferentの変更までは固定しない。

```text
s0.slot[0] = handle i
s1 = slot(s0, 0, handle j)

peek(s0, 0) = handle i
peek(s1, 0) = handle j
```

この後identity `i`が別のhandleから変更されても、`s0`のslot 0は依然として`handle i`である。変わったのは保存したcarrierではなく、
carrierから到達するreferentである。この区別をstructural snapshotと呼ぶ。

Swift Arrayもclass referenceを要素にでき、Arrayのcopy後に一方の要素を別referenceへ置換しても他方のArrayは変わらないが、両方が
同じclass instanceを指す間はinstanceの変更を共有観測する。Rustの内部的な`Freeze`判定もindirection先まで追わない。Poolの
structural snapshotはこの境界と同じであり、deep copyまたは推移的不変性を暗黙に約束しない
（[関連事例](../../../research/pool-storage-prior-art.md#value-containerにhandleを保存する言語)）。

したがってIxPoolとImPoolの型形成条件は同じである。

```text
IxPool<M, V> is well-formed if Storable(M) and Storable(V)
ImPool<M, V> is well-formed if Storable(M) and Storable(V)
```

`freeze`と`thaw`もcarrierを保存するshallowな変換であり、handle referentをcloneしない。transitive snapshot、serialization、
content-addressed keyなど、到達先まで変化しないことが必要なAPIは別途stabilityを要求する。

## stabilityを要求する場所

将来の`Stable(T)`を定めるなら、少なくとも何に対する安定性かを名前またはcontractに含める必要がある。候補となる最小の定義は
次である。

```text
Stable(T) = Storable(T)
            and TからMal-owned shared mutable identityを観測できない
```

この定義では`Address`とexternal opaque typeはMal-owned identityについてStableでも、Extern referentが不変とは限らない。
serializationやMap keyには、extern capabilityを含まないこと、equalityとhashが時間で変わらないことなど、さらに別の条件が要る。
したがって`Stable`だけでそれらのAPIを一般化しない。

| 型 | Storable案 | Mal-owned identityに対するstability | Representable | HostMappable | lifecycle plan |
|---|---:|---:|---:|---:|---|
| `Unit`、numeric scalar、`ByteSize`、`USize` | ○ | ○ | ○ | ○ | Trivial |
| `Address` | ○ | ○ | ○ | ○ | Trivial |
| `Symbol` | ○ | ○ | × | × | Owned |
| external opaque | ○ | ○ | × | ○ | Trivial |
| `Buffer<A>` | `A`がStorableなら○ | × | × | × | Owned |
| `IxPool<M, V>` | `M`と`V`がStorableなら○ | × | × | × | Owned |
| `ImPool<M, V>` | `M`と`V`がStorableなら○ | `M`と`V`がstableなら○ | × | × | Owned |
| function | × | capture次第 | × | × | Owned |

`Representable(T)`は引き続き`Storable(T)`の一部であり、canonical memory copyだけを意味する。`HostMappable(T)`は独立である。

## minimality監査

この整理で新しく必要なのは`Stable` judgmentではなく、現行`Storable`の意味をplace lifecycleへ純化する変更である。独立contractの数は
次のように保つ。

- PoolとBufferの型形成は一つの`Storable`を共有する。
- handle nestingに別の`Placeable`や`Managed`を追加しない。
- ImPoolはstructural snapshotなので`Stable`を要求しない。
- canonical memory copyとpublic extern ABIは既存の`Representable`と`HostMappable`を使う。
- closure cycleはfunctionのstorage admissionとして個別に残し、mutable identity一般を除外しない。
- deep snapshotやkey stabilityの要求が複数のAPIで一致するまで`Stable`をcompiler judgmentにしない。

Buffer handleのelement restrictionは[D096](../../../history/decisions/active/D096.md)でPool採択前に外し、place lifecycleとしての
`Storable`へ揃えた。external opaque carrierも[D097](../../../history/decisions/active/D097.md)で`Runtime(_, Trivial)` storageとともに採択した。これによりIxPoolだけがhandleを
保存できる二つのstorage modelは避けられた。

## 入れ子

拡張後はcarrierの種類にかかわらず同じplace lawを使う。

| 外側 | 内側 | 保存されるもの | 内側の更新 |
|---|---|---|---|
| IxPool上のcontainer | IxPool handle | handle authorityとresponsibility | 全aliasから観測する |
| ImPool上のcontainer | IxPool handle | snapshot内のhandle carrier | referentの変更は全handleから観測する |
| IxPool上のcontainer | ImPool snapshot | snapshot carrier | successorを外側へ書き戻す |
| ImPool上のcontainer | ImPool snapshot | snapshot carrier | successorを外側のsuccessorへ書き戻す |
| 一つのIxPool | coordinate | scalar coordinate | 同じPool identity内を直接更新する |

snapshot内のhandleを取り出してreferentを変更しても外側snapshotのslotは変わらない。外側の構造まで更新する場合だけsuccessorを作る。
この違いをAPI documentationとexampleで明示し、「immutable container」が「到達可能な全stateのdeep immutability」を意味すると読ませない。
