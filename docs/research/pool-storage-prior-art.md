# Pool storage、allocation、copy-on-writeの関連事例

Status: Research note

この文書は[Pool proposal](../proposals/pool/README.md)のcore contractを定めず、C/LLVMのallocation object、Rustのallocatorと
collection、Haskellのmutable/immutable array、optionalなwritable successor、key、persistent containerを判断するための外部事例を
整理する。これらの機構をmalへそのまま移すのではなく、raw storage、typed place、authority、container policyが各systemのどこに
置かれるかを比較する。

## 比較する層

同じ「配列」や「allocation」という語でも、各systemで担う層が異なる。

| 層 | C / LLVM | Rust | Haskell / GHC | Pool案 |
|---|---|---|---|---|
| raw allocation | `malloc`、LLVM allocated object | `Allocator`と`Layout` | GC heap、array primop | C runtime内部のmechanism |
| typed storage | C object、LLVM load/store | `RawVec<T>`、`MaybeUninit<T>` | `Array#`、`MutableArray# s` | runtime payloadとoccupancy |
| mutation authority | pointerと外部contract | `&mut`、move、unsafe contract | `State# s`、`ST s`、`IO` | IxPool handleが指すidentity |
| immutable snapshot | conventionまたはcopy | owned value、`Arc`による共有 | immutable array | ImPool snapshot value |
| lifetime responsibility | callerの`free` | ownerと`Drop` | GC | compilerの`Share`、`Consume`、`Drop` |
| container policy | library code | `Vec`、`HashMap` | array library、container library | malで書くopaque container |

この比較ではPoolはallocatorの代替ではない。typed placeとそのlifecycleを持ち、raw allocationより上、container invariantより下にある。

## CとLLVMのallocation object

Cの`malloc`は指定byte数のuninitialized storageを返し、失敗をnullで表す。返ったpointerをどの型とlifetimeで使い、いつ`free`するかは
callerのcontractである。Poolはこのpointerと解放責任をsourceへ出さず、runtimeがEngram representationとして閉じて扱う。

LLVMは`alloca`、認識されたheap allocation、globalなどが作るmemory regionをallocated objectとして扱う。allocated objectは
provenance、size、lifetimeを持ち、認識されたobjectのsizeは後から変わらない。`realloc`相当のoperationは、同じaddressを返した場合も
新しいobjectを作って旧objectを無効にする（[LLVM Language Reference](https://llvm.org/docs/LangRef.html#allocated-objects)）。

したがってIxPoolのidentityをpayload allocationのpointer identityとしてlowerできない。stableなmanaged objectからcurrent backing
allocationを指し、growth後にpointerを再取得する形なら、Pool identityとLLVM provenanceを分離できる。Live/Vacantもallocated objectの
lifetimeではなく、Malの`Slot<V>`をoccupancyとpayloadへlowerした状態である。

## RustのAllocator、初期化、collection

Rustの[`Allocator`](https://doc.rust-lang.org/std/alloc/trait.Allocator.html)は、`Layout`で指定したraw blockのallocate、grow、shrink、
deallocateを扱う。blockがどのallocatorに由来するか、allocator equivalence、alignment、operation後のpointer validityがcontractに
含まれる。要素型、初期化済み要素数、container invariantは`Allocator`の責務ではない。

`Vec<T>`は概念上、allocationとcapacityを持つ`RawVec<T>`に、初期化済みprefixのlengthを加える。未初期化storageを`T`として
扱わない境界には`MaybeUninit<T>`やunsafeなpointer operationが使われる。Pool案はuninitializedな`V`をsourceへ持たず、
`Slot<V> = [Unit, V]`をoccupancyとpayloadへlowerする。Bufferのdense prefixはPoolではなくBuffer invariantに残す。

Rustのborrowはgrowthで無効になり得るinterior referenceを制約する。Poolはslot pointerをsourceへ出さず、coordinateをPool identityに
相対的に解釈するため、growthとrelocationのためのsource-level borrow lifetimeを必要としない。

Rustのallocatorをcontainerの型parameterにできることは、Poolにもallocator parameterが必要であることを意味しない。allocator policy、
recoverable failure、allocation regionをprogramが選ぶ要求がなければ、runtime detailとして閉じる方がcontractは少ない。

### `Box<T>`とsource ownership

Rustの[`Box<T>`](https://doc.rust-lang.org/std/boxed/struct.Box.html)は、一つの`T`をheap allocationに置き、そのallocationを所有する
source valueである。`Box`のmove、borrow、`Drop`はexclusive owner、addressableな一つのLive value、deallocation responsibilityを
一つの型へ結び付ける。raw pointerへの変換ではcleanup responsibilityもcallerへ移る。

Pool案はこの結合をsourceへ導入しない。一つのresponsibilityが一つのallocationを保持する状態は、compilerとruntimeでは
Box-likeな実装事実として現れ得る。しかしIxPool valueはshared identityへのhandleであり、ImPool valueはimmutable snapshotである。
物理ownerが一つだからIxPoolをexclusiveと解釈したり、ImPoolを破壊的に変更したりしない。Poolに必要なのはsingle objectへのderefでも
stable addressでもなく、有限個のtyped placeをcoordinateで選ぶことだからである。

## Haskellのarrayとstate

GHCはimmutableな`Array#`と`MutableArray# s`を分け、mutable operationを`State# s`のthreadingとして表す
（[GHC primitive types](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/primitives.html)）。`ST s`ではstate parameterにより
mutable identityをscope外へ逃がさず、`IO`ではworld stateの中でidentityを扱う。

IxPoolとImPoolはmutable/immutable arrayの対に似るが、IxPoolはaliasがscopeを越えて変更を共有観測できるため、`STArray`より
`IOArray`に近い。malはstrict evaluationと既存Bufferのshared identityを持つので、Poolのために`State# s`に対応するsource valueを
導入しない。operationの評価順とidentity authorityで変更を説明する。

HaskellのGCとmalのmanaged responsibilityは回収mechanismが異なる。どちらもarray valueの意味をphysical copyや解放時刻から
分離できるが、mal backendはmanaged elementの`Share`、`Consume`、`Drop`を明示的に計画する。

GHCの[Linear Types](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/linear_types.html)はargumentの一回消費を型で表せる。
Pool案はuniquenessをsource authorityにせず、ImPoolのobservable snapshot semanticsを保ったままstorageを再利用する実装条件に置く。

## RustとHaskellの間での分解

Rustではsource ownershipがmutation authorityとlifetime responsibilityの両方を表す。Haskellではstate tokenがmutation authorityを
表し、GCがlifetime responsibilityを引き受ける。malはsource valueの再利用可能性を保ちながら、compilerが実行時responsibilityだけを
affineに移す。Poolの型はauthority、compilerはresponsibility、runtimeはrepresentation uniquenessを受け持つ。

この分解により、IxPoolで構築して`freeze`する経路はaliasがなければRustのmoveに近いO(1)のstorage transferになり、aliasがあれば
Haskellのimmutable valueと同じsnapshot semanticsをcopyで保つ。どちらになったかはprogramの意味へ現れない。coordinateも同様に、
Rustのinterior borrowを公開せず、Haskellのarray indexのようにrelocationから独立している。

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
storage再利用とcopyのどちらを選んだかがkeyの有効性へ現れてはならない。Pool案のImPoolはkeyを発行しないため、この合成を
持たない（[ImPool](../proposals/pool/api/pool.md#impool)）。

## Persistent indexed sequence

[RRB Vector](https://www.cs.purdue.edu/homes/rompf/papers/stucki-icfp15.pdf)は、wide treeとstructural sharingを使い、flat arrayの
全要素copyとは異なるimmutable indexed sequenceを構成する。random accessにはtree traversalが入り、split、concatenate、updateなど
別のoperation setとのtrade-offを持つ。

これはflatなPoolのwritable successorをpersistent vector一般の解決にしない根拠になる。chunkまたはtree nodeを共有するArrayは、
node間参照、root ownership、使われなくなったnodeの回収を別途設計し、最小Poolのslot lifecycleへ混ぜない。
