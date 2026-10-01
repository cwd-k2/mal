# 試作で確かめたこと

Status: Exploratory support document

この文書は、compilerを変えずにPool案を動かした二つの試作と、その結果を管理する（2026-09-30）。試作のsourceはrepositoryに含めず、
結論と、その結論が依拠する条件だけを記録する。IxPool APIは[primitive一覧](primitives.md)、試作から抜き出したcodeは
[collection例](collection-examples.md)を正とする。

## 二つの試作

- C host試作：占有状態を検査するC hostがIxPoolのstorageを持ち、operation familyが要素型ごとにhostを呼ぶ。preconditionへの違反は
  trapになる。要素はhostへ渡せる型に限られ、IxPoolの寿命はないため明示的に解放する。`Symbol`もextern境界を通らないため、
  同じoperationを持つhost側のtextで代用した。
- Buffer上のemulation：現在の言語だけで、`Buffer<S>`のStateと`Buffer<[Unit, T]>`のslotとしてIxPool APIを実装した。Vacantは
  `Unit`のvariantで表し、preconditionへの違反は戻らない。

どちらでも、stack、binary heap、Map、Deque、SlotMap、木の同じsourceが変更なしに動いた。containerはIxPool APIにしか依存せず、
二つの試作は一つの意味の二つの実装である。[段階的な検証](README.md#段階的な検証)のstep 4と6の一部に当たり、`Store`による
所有権の効果と性能は対象外である。

ImPoolはBuffer上のemulationだけで実装した。malはexternal handleをhostへ知らせずにcopyするため、C hostはstorageが一意かを
知れない。emulationも参照数を観測できないため、更新のたびにcopyする。意味はstorageを再利用する場合と同じである。

## Bufferの参照実装

[Buffer実装](buffer-implementation.md)をBufferと別名の`Vec<T>`としてIxPoolの上に書き、predefinedな`Buffer<UInt64>`と比べた。
同じ擬似乱数列で`new`、`put`、`fill`、重なる範囲の`copy`、別の値からの`copy`を1000回、aliasを通して両方へ適用し、各操作の後に
長さと全要素が一致することを二つの試作で確かめた。`fill`と`copy`は参照実装と同じくslot操作のloopで書き、`copy`はoffsetの大小で
loopの向きを選ぶ。同じBufferで前後どちらへ重なる`copy`も、範囲の一括primitiveなしに現行Bufferと一致した。overflowのtrapは
`trap` primitiveがないため比べていない。

## ImPoolとfreezeとthaw

Buffer上のemulationで、[array ownership](array-ownership.md)の`Array<T>`をImPoolの上に書いた。更新は前の値を変えず、`freeze`の
後のIxPoolへの書き込みも、`thaw`したIxPoolへの書き込みも、値と他のIxPoolから観測されないことを確かめた。`Symbol`を要素に
しても、valgrindで全allocationの解放とerror 0を確かめた。emulationのImPoolはBufferを含むため`Storable`にならず、
`Array<Array<T>>`は確かめていない。

## slot遷移とcontainer

- 要素型ごとの実装は`initAt`、`takeAt`、Stateだけで足り、`putAt`、`dropAt`、`moveAt`は通常のgeneric関数として書けた。
- `takeAt`により、tombstoneのないMap削除、同じidentityでのrehash、Dequeのring展開、heapの穴を動かすsift、木とSlotMapの
  slot再利用を、要素をcopyせずに書けた。
- C host試作の検査は、containerの`reserve`忘れと利用者のprecondition違反の両方をtrapへ変えた。messageはIxPoolのpreconditionを
  示し、container operationを示さない。
- Buffer上のemulationで`Symbol`を要素にしたMap、Deque、heap、SlotMapを動かし、valgrindで全allocationの解放とerror 0を
  確かめた。`takeAt`と`initAt`による移動は、managed valueのresponsibilityを一つに保った。

## `get`とstorageの共有

C host試作ではbyte IxPoolのstorageを`Symbol`相当のtextと共有した。snapshotはO(1)で、以後どちらかへ書いた側だけがcopyした。
逆向きの変換は、textが一つの葉ならstorageを貸し、それ以外は一度だけflattenした。

`getAt`を`takeAt`と`initAt`で派生させると、読むだけで`initAt`が書き込みとして共有storageのcopyを起こし、探索ごとのhost callも
倍になった。[primitive一覧](primitives.md#slot-primitive)が`getAt`をprimitiveに残すのはこのためである。

## ropeとflatな`Symbol`

同じtextの操作を、AVL木のropeと、現在の`Symbol`のflatなbyte ownerの方式で比べた。flatな方式では、consumingな`+`が一意な
ownerをその場で伸ばす。1.4 MBの行の反転、split、比較、書き出しはflatが4〜12倍速く、ropeが勝ったのは大きなtextの中央への
挿入の反復だけだった。IxPoolとのstorage共有とcopy-on-writeはどちらでも成り立つため、`Symbol`の表現はflatのままでよく、
ropeはIxPool上の別containerとして持つ方が合う。

## IxPoolとBufferの非対称

IxPoolはBufferの上に、BufferはIxPoolの上に、どちらも意味の上では書ける。Buffer上のIxPoolはslotごとのsum tagと、`takeAt`ごとの
`Share`と`Drop`を払うが、IxPool上のBufferは追加のcostを持たない。IxPoolをprimitiveにするのはこの非対称のためである。
Vacantを末尾だけに限ったdense primitiveへ`[Unit, T]`を載せる形とは意味が同じであり、残る差はこれらのcostだけである。
その大きさは測っていない。

## 試作から出た言語とcompilerの変更

- phantomな型parameterにしか現れない型argumentを推論できなかった。[D092](../../history/decisions/active/D092.md)で、
  constructorでない場合に推論する規則へ改めた。
- phantomなopaque argumentを持つgeneric codeの拒否、同じfileのopaque層をinferenceが見ない問題、diagnosticでopaque applicationの
  `<`が欠ける問題、operation implementationの欠落がkeyを示さない問題を修正した。
