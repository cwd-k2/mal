# Pool storage、copy-on-write、persistent vectorの関連事例

Status: Research note

この文書は[Pool proposal](../proposals/pool/README.md)のcore contractを定めず、optionalなwritable successor、key、
persistent containerを判断するための外部事例を整理する。

## Copy-on-writeとuniqueness

Rustの[`Arc::make_mut`](https://doc.rust-lang.org/std/sync/struct.Arc.html#method.make_mut)は、同じallocationへの別のstrong `Arc`があれば
inner valueをcloneし、なければ同じallocationへのmutable accessを与える。uniquenessはoperation内部で判定し、reference countを
container利用者の分岐条件として返さない。この形は、malのoperationを「参照数を返す」ものではなく「既存aliasから独立した
writable successorを返す」ものとして定義する根拠になる。

Swiftの[`Array`](https://developer.apple.com/documentation/swift/array)もvariable-size collectionのstorageをcopy-on-writeで共有する。
library実装向けの
[`isKnownUniquelyReferenced`](https://developer.apple.com/documentation/swift/isknownuniquelyreferenced%28_%3A%29-98zpp)は、strong referenceが
一つと判明した場合だけtrueを返す。判明しない場合にcopyへfallbackできるため、uniqueness最適化とvalue semanticsを分離できる。

malではruntime reference countをsource semanticsにせず、owned responsibilityを受け取ったwritable-successor operationだけが
storage再利用を試みる。borrowed call、一時Share、別の回収方式では保守的にcopyしてよく、observableなArray valueは同じである。

## Weak pointerとkey

`Arc::make_mut`はstrong aliasがなくweak pointerだけが残る場合、inner valueをcloneせずweak pointerを元allocationから切り離す。
これはCOWとnonowning identityを合成する際にも追加ruleが必要なことを示す。Poolのkeyがidentityやgenerationを持つ場合、
storage再利用とcopyのどちらを選んだかがkeyの有効性へ現れてはならない。Pool案での扱いは
[identity](../proposals/pool/identity.md#keyとの合成)に置く。

## Persistent indexed sequence

[RRB Vector](https://www.cs.purdue.edu/homes/rompf/papers/stucki-icfp15.pdf)は、wide treeとstructural sharingを使い、flat arrayの
全要素copyとは異なるimmutable indexed sequenceを構成する。random accessにはtree traversalが入り、split、concatenate、updateなど
別のoperation setとのtrade-offを持つ。

これはflatなPoolのwritable successorをpersistent vector一般の解決にしない根拠になる。chunkまたはtree nodeを共有するArrayは、
node間参照、root ownership、使われなくなったnodeの回収を別途設計し、最小Poolのslot lifecycleへ混ぜない。
