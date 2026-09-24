# EngramとExtern

Status: Accepted v0.6

## authority

Engramはsource-levelの型名ではなく、mal program内部で有効な値として構成されたものの総称である。`Unit`、numeric scalar、
`Symbol`、`Buffer`、product、sum、function valueはEngramであり、その構築、保持できる区別、identity、到達可能性、
lifetime authorityはmalに属する。domain上の意味は値だけから決まらず、その値を解釈するoperationとinvariantが与える。
実装がstatic storage、arena、region、tracing、reference countingのどれを使うかは、観測できない限りsource semanticsではない。

Externはmalの外部にあるstate、storage、resource、作用の領域である。`Address`はexternal storageへのcapabilityであり、
external opaque valueはhost resourceへのcopyable handleである。どちらもcopyしてもreferentのlifetimeを延長しない。
close、free、permission、alias、failureは個々のhost contractが定める。`Buffer`のidentityとstorageはmalに属するが、
elementとして保持した`Address`やexternal opaque valueのreferent authorityまでは取得しない。

EngramとExternはsource-level typeを二分する分類ではなく、意味とlifetimeのauthorityを分類する。Addressを含むproductや
closureの構造はmalが持つEngramだが、AddressのreferentはExternに残る。external opaque valueについても同じであり、
Engramへ包んでもresource ownershipは移らない。

## 境界のoperation

境界を通るoperationは三種類に分ける。

| operation | direction | meaning |
|---|---|---|
| admission | ExternからEngram | external representationを検査またはcopyし、新しいmal valueを構成する |
| observation | EngramからExtern | call中にborrowするか外部storageへcopyし、malのidentityとlifetimeを渡さない |
| capability transfer | 双方向 | `Address`またはexternal opaque valueを運び、referentのauthorityをExternに残す |

`from<T>`はadmission、`buffer.into`はobservationである。Bufferの`new`、`get`、`put`、`fill`、`copy`はmal-owned value内の通常の操作である。extern resultとparameterは
HostMappableなEngram leafのadmissionまたはobservationと、Addressやexternal opaque valueのcapability transferだけを行う。
`Symbol`と`Buffer`はextern signatureへ現れない。

backend adapterはraw host operationとmal valueの間に立つtrusted boundary codeである。adapterがruntime contextを使って
admission helperを呼ぶことは、ExternがEngramを生成することではない。adapterはmalへ構築を依頼し、完成した値を運ぶだけで、
contextやEngramのlifetime authorityを取得しない。

Addressのextern parameter/resultはcapability transferである。Addressを含むBufferのnew/get/put/fill/copyはreferentのauthorityを
移さず、copyableなcapability valueだけをBufferへ格納またはBufferから取得する。Address admissionは任意のbytesを
有効なcapabilityに変換せず、hostが提供したかBuffer operationが有効なAddress値から格納したpointer representationだけを復元できる。
external opaque valueもhostが有効性を支配し、malはhandle bitsからresourceを生成しない。

external opaque valueを外部storageへ保存し、後で復元する必要がある場合、hostはその型に固有の`extern`
operationを定義できる。保存表現、有効な値の範囲、復元後のresource lifetime、stale handleや多重解放の扱いは
Externのauthorityに残り、保存や復元によってreferentのlifetimeは延長されない。malのpredefined memory
primitiveがopaque valueの表現を定めないことは[external memory](memory.md#representable)に定める。

外部storageへEngramのdescriptor、managed pointer、rootを書いて後で復元する経路は提供しない。`buffer.into`が
書くのはelementのcanonical representationだけである。Externはmal内部のidentityを生成できず、malのlifetimeを延長できない。

## composition

productとsumはfieldごとに境界operationを再帰的に適用する。例えば`(UInt64, Address)`では第一fieldを値として運び、第二fieldの
capabilityをtransferする。aggregate carrier全体を一つのownership単位とはみなさない。非HostMappableなleafを含むaggregateは
境界へ出せない。

Bool、sum tag、opaque handleなど有効表現が限定される値をhostが返す場合、adapterはそのcontractを満たさなければならない。
function valueはhost境界を通せない。closureを渡すにはmal-owned code/environmentの保持期間と呼出権限が必要になり、
admission、observation、capability transferのいずれにも暗黙には分類できないためである。

## lifetime

Engramがいつ回収可能になるかはmalが決める。source programとhostが観測できるのは、到達可能な値の意味が保持され、
borrowがcall中有効であることだけである。Extern resourceのlifetimeはこの回収に連動しない。`malc`の
現在の回収方式は[implementation notes](../implementation/compiler.md)に記録し、言語contractには固定しない。

正確なextern signatureとtrusted範囲は[`extern`](extern.md)、C representationは
[C host ABI](c-host-abi.md)、memory operationは[external memory](memory.md)と[`Buffer`](memory.md)に定める。
