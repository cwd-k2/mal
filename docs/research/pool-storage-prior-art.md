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
| structural snapshot | conventionまたはcopy | owned container、共有handleを要素にできる | immutable arrayもreferenceを要素にできる | ImPool snapshot value |
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
Box-likeな実装事実として現れ得る。しかしIxPool valueはshared identityへのhandleであり、ImPool valueはstructural snapshotである。
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

## value containerにhandleを保存する言語

### Rust

Rustのpointerはfirst-class valueであり、data structureへ保存できる
（[Rust Reference: pointer types](https://doc.rust-lang.org/reference/types/pointer.html)）。`Rc<T>`はmultiple ownershipを明示し、
`RefCell<T>`はshared referenceを経由するinterior mutationをruntime borrow checkで提供する
（[`Rc<T>`](https://doc.rust-lang.org/book/ch15-04-rc.html)、
[`RefCell<T>`](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)）。したがって`Vec<Rc<RefCell<T>>>`のようなcontainerは、
外側の要素列と、要素handleが指すidentityを別のauthorityとして持つ。

Rustの[`Freeze`](https://doc.rust-lang.org/stable/core/marker/trait.Freeze.html)は型の内部に`UnsafeCell`を含まないことを表す実験的な
compiler traitだが、indirection先までは追わない。pointer carrierがFreezeでもreferentの変更不能性を意味しない。この境界は、
ImPoolのsnapshotがslot carrierを保存してもhandle referentまで固定しないことの直接の比較になる。

Rustは`Rc<RefCell<T>>`などの組合せでreference cycleを構成でき、そのcycleは解放されないことを明示している
（[Reference Cycles Can Leak Memory](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html)）。これはhandleを保存できることと、
cycleを回収できることが独立したcontractである例である。malではclosure captureが型へ現れないためfunction storageを引き続き
除外できる一方、再帰型を持たないcontainer handleの入れ子まで同じ理由で除外する必要はない。

### Haskell

Haskellの`STRef s a`、`IORef a`、mutable arrayはhandle valueであり、list、tuple、別のarrayなどの通常のdataへ要素として置ける。
`STArray s i e`もelement `e`をmutable handleから除外しない
（[`Data.Array.ST`](https://hackage.haskell.org/package/array/docs/Data-Array-ST.html)）。
`runST :: (forall s. ST s a) -> a`はstate parameter `s`を含むhandleのescapeを防ぐが、`ST s`内でhandleをdata structureへ保存することは
防がない（[`Control.Monad.ST`](https://hackage.haskell.org/package/base/docs/Control-Monad-ST.html)）。

したがってHaskellも、data constructorが保存するcarrierと、carrierを使って実行するeffectを分ける。malはIxPool handleのescapeを
禁止せず既存Bufferと同じ共有観測を保つが、handleをvalueとしてcontainerへ保存する点は同じである。

### Swift

SwiftのArrayはvalue typeでcopy-on-writeを使うが、class instanceを要素にできる。Arrayの一方で要素referenceを置換しても他方の
Arrayは変わらない一方、二つのArrayが同じclass instanceを指す間はinstance propertyの変更を両方から観測する
（[Swift Array: Modifying Copies of Arrays](https://developer.apple.com/documentation/swift/array#Modifying-Copies-of-Arrays)）。
[Swift Language Guide](https://docs.swift.org/swift-book/LanguageGuide/ClassesAndStructures.html)も、structureとArrayをvalue type、classを
reference typeとして区別する。

これはshallow copyという実装上の妥協ではない。Array valueが保存するelement value自体がreferenceだからである。ImPoolも同様に、
slot carrierの列をstructural snapshotとして保存し、carrierが持つauthorityをdeep copyしない。

## RustとHaskellの間での分解

Rustではsource ownershipがmutation authorityとlifetime responsibilityの両方を表す。Haskellではstate tokenがmutation authorityを
表し、GCがlifetime responsibilityを引き受ける。malはsource valueの再利用可能性を保ちながら、compilerが実行時responsibilityだけを
affineに移す。Poolの型はauthority、compilerはresponsibility、runtimeはrepresentation uniquenessを受け持つ。

この分解により、IxPoolで構築して`freeze`する経路はaliasがなければRustのmoveに近いO(1)のstorage transferになり、aliasがあれば
slot carrierのstructural snapshotをcopyで保つ。handleをslotに含む場合もreferentをcloneしない。どちらになったかはprogramの
意味へ現れない。coordinateも同様に、
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
