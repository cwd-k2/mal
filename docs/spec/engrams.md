# EngramとExtern

Status: Accepted v0.6 profile

## authority

Engramはsource-levelの型名ではなく、mal program内部に刻まれた意味を持つものの総称である。`Unit`、numeric scalar、
`Symbol`、`Packed`、product、sum、function valueはEngramであり、その構築、有効な値の範囲、identity、到達可能性、
lifetime authorityはmalに属する。実装がstatic storage、arena、region、tracing、reference countingのどれを使うかは、
観測できない限りsource semanticsではない。

Externはmalの外部にあるstate、storage、resource、作用の領域である。`Address`、`Cursor`、`Region`はexternal storageへのcapabilityであり、
external opaque valueはhost resourceへのcopyable handleである。いずれもcopyしてもreferentのlifetimeを延長しない。
close、free、permission、alias、failureは個々のhost contractが定める。

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

Cursor load、RegionからPackedへのtransfer、extern resultはadmissionである。Cursor/Region storeとextern parameterはobservationである。`Symbol`
parameterのdataはcall中だけborrowされ、hostはreturn後に保持しない。

backend adapterはraw host operationとmal valueの間に立つtrusted boundary codeである。adapterがruntime contextを使って
admission helperを呼ぶことは、ExternがEngramを生成することではない。adapterはmalへ構築を依頼し、完成した値を運ぶだけで、
contextやEngramのlifetime authorityを取得しない。

Addressのextern parameter/resultとAddressを含むCursor load/storeはcapability transferである。Address loadは任意のbytesを
有効なcapabilityに変換せず、hostまたはCursor storeが書いた有効なpointer representationだけを復元できる。
external opaque valueもhostが有効性を支配し、malはhandle bitsからresourceを生成しない。

external opaque valueを外部storageへ保存し、後で復元する必要がある場合、hostはその型に固有の`extern`
operationを定義できる。保存表現、有効な値の範囲、復元後のresource lifetime、stale handleや多重解放の扱いは
Externのauthorityに残り、保存や復元によってreferentのlifetimeは延長されない。malのpredefined memory
primitiveがopaque valueの表現を定めないことは[external memory](memory.md#representable)に定める。

外部storageへEngramのdescriptor、managed pointer、rootを書いて後で復元する経路は提供しない。PackedからRegionへのtransferが
書くのはelementのcanonical representationだけである。Externはmal内部のidentityを生成できず、malのlifetimeを延長できない。

## composition

productとsumはfieldごとに境界operationを再帰的に適用する。例えば`(Symbol, Address)`をhostへ渡すと、第一fieldは
observation、第二fieldはcapability transferになる。hostから返す場合は第一fieldをadmitし、第二fieldのcapabilityを
importする。aggregate carrier全体を一つのownership単位とはみなさない。

Bool、sum tag、opaque handleなど有効表現が限定される値をhostが返す場合、adapterはそのcontractを満たさなければならない。
function valueはhost境界を通せない。closureを渡すにはmal-owned code/environmentの保持期間と呼出権限が必要になり、
admission、observation、capability transferのいずれにも暗黙には分類できないためである。

## lifetime

Engramがいつ回収可能になるかはmalが決める。source programとhostが観測できるのは、到達可能な値の意味が保持され、
borrowがcall中有効であることだけである。Extern resourceのlifetimeはこの回収に連動しない。reference compilerの
現在の回収方式は[implementation notes](../implementation/compiler.md)に記録し、言語contractには固定しない。

正確なextern signatureとtrusted範囲は[`extern`](extern.md)、reference C representationは
[C host ABI](c-host-abi.md)、memory operationは[external memory](memory.md)と[`Region`と`Packed`](packed.md)に定める。
