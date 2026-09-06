# EngramとExtern

Status: Current v0.5 profile

## authority

Engramはsource-levelの型名ではなく、mal program内部に刻まれた意味を持つものの総称である。`Unit`、numeric scalar、
`Symbol`、product、sum、function valueはEngramであり、その構築、有効な値の範囲、identity、到達可能性、
lifetime authorityはmalに属する。実装がstatic storage、arena、region、tracing、reference countingのどれを使うかは、
観測できない限りsource semanticsではない。

Externはmalの外部にあるstate、storage、resource、作用の領域である。`Ptr`はexternal storageへのcapabilityであり、
external opaque valueはhost resourceへのcopyable handleである。いずれもcopyしてもreferentのlifetimeを延長しない。
close、free、permission、alias、failureは個々のhost contractが定める。

EngramとExternはsource-level typeを二分する分類ではなく、意味とlifetimeのauthorityを分類する。`Ptr`を含むproductや
closureの構造はmalが持つEngramだが、`Ptr`のreferentはExternに残る。external opaque valueについても同じであり、
Engramへ包んでもresource ownershipは移らない。

## 境界のoperation

境界を通るoperationは三種類に分ける。

| operation | direction | meaning |
|---|---|---|
| admission | ExternからEngram | external representationを検査またはcopyし、新しいmal valueを構成する |
| observation | EngramからExtern | call中にborrowするか外部storageへcopyし、malのidentityとlifetimeを渡さない |
| capability transfer | 双方向 | `Ptr`またはexternal opaque valueを運び、referentのauthorityをExternに残す |

numeric scalarの`loadT`とextern result、`loadSymbol(pointer, length)`はadmissionである。`loadSymbol`はbytesを
mal-controlled storageへcopyする。scalarや`Symbol`のstoreとextern parameterはobservationである。`Symbol`
parameterのdataはcall中だけborrowされ、hostはreturn後に保持しない。

`Ptr`のextern parameter/resultと`loadPtr`/`storePtr`はcapability transferである。`loadPtr`は任意のbytesを
有効なcapabilityに変換せず、hostまたは`storePtr`が書いた有効なpointer representationだけを復元できる。
external opaque valueもhostが有効性を支配し、malはhandle bitsからresourceを生成しない。

外部storageへEngramのdescriptor、managed pointer、rootを書いて後で復元する経路は提供しない。`storeSymbol`が
書くのはbytesだけである。Externはmal内部のidentityを生成できず、malのlifetimeを延長できない。

## composition

productとsumはfieldごとに境界operationを再帰的に適用する。例えば`(Symbol, Ptr)`をhostへ渡すと、第一fieldは
observation、第二fieldはcapability transferになる。hostから返す場合は第一fieldをadmitし、第二fieldのcapabilityを
importする。aggregate carrier全体を一つのownership単位とはみなさない。

Bool、sum tag、opaque handleなど有効表現が限定される値をhostが返す場合、adapterはそのcontractを満たさなければならない。
function valueはv0.5の境界を通せない。closureを渡すにはmal-owned code/environmentの保持期間と呼出権限が必要になり、
admission、observation、capability transferのいずれにも暗黙には分類できないためである。

## lifetime

Engramがいつ回収可能になるかはmalが決める。source programとhostが観測できるのは、到達可能な値の意味が保持され、
borrowがcall中有効であることだけである。Extern resourceのlifetimeはこの回収に連動しない。reference compilerの
現在の回収方式は[implementation notes](../implementation/compiler.md)に記録し、言語contractには固定しない。

正確なextern signatureとtrusted範囲は[`extern`](extern.md)、reference C representationは
[C host ABI](c-host-abi.md)、memory operationは[memory primitive](memory.md)に定める。
