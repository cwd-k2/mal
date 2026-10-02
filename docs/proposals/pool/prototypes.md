# 試作で確かめたこと

Status: Exploratory support document

この文書は、compilerを変えずにPool案を動かした二つの試作と、その結果を管理する（2026-09-30から10-01）。試作のsourceはrepositoryに含めず、
結論と、その結論が依拠する条件だけを記録する。IxPool APIは[Pool primitive](api/pool.md)、試作から抜き出したcodeは
[列のcontainer](containers/sequences.md)と[keyで引くcontainer](containers/keyed.md)を正とする。

## 二つの試作

- C host試作：占有tagを検査するC hostがIxPoolのstorageを持ち、要素型ごとの核をoperation familyでhostへ渡す。preconditionへの
  違反はtrapになる。要素はhostへ渡せる型に限られ、IxPoolの寿命はないため明示的に解放する。`Symbol`もextern境界を通らないため、
  同じoperationを持つhost側のtextで代用した。
- Buffer上のemulation：現在の言語だけで、`Buffer<M>`のMetaと`Buffer<Slot<V>>`のslotとしてIxPoolの核を実装した。ImPool、
  `freeze`と`thaw`、`Host<A>`もここに置き、preconditionへの違反は戻らない。

二つの試作は、周辺のoperationとstack、binary heap、Map、Deque、SlotMap、木を一つのsourceで共有して動く。containerはIxPoolの
核と周辺にしか依存せず、二つの試作は一つの意味の二つの実装である。[検証の段階](runtime/implementation.md#検証の段階)のstep 5と7の一部に当たる。

ImPoolはBuffer上のemulationだけで実装した。malはexternal handleをhostへ知らせずにcopyするため、C hostはstorageが一意かを
知れない。emulationも参照数を観測できないため、更新のたびにcopyする。意味はstorageを再利用する場合と同じである。

核はpoolのconstructor `F`をkeyに持つoperation familyとして書き、IxPoolとImPoolで一つの名前を共有した。更新はどれもpoolを
返し、IxPoolは同じidentityを、ImPoolはsuccessorを返す。周辺は`F`の上に一度だけ書け、IxPool上のcontainerとImPool上の
Vectorが同じ周辺を使った。呼び出しは`meta<IxPool>(map)`のようにconstructorを明示し、これはopaqueのどの層として見るかの
指定も兼ねる。更新がpoolを返すため、更新で終わる`Unit`のblockには末尾の`()`が要った。この形を書く過程で、kind多相な型parameterを
扱うcompilerの不具合を四つ見つけて直し、本体が求めるkindをspecializationで検査する規則を
[D093](../../history/decisions/active/D093.md)として決めた。

## Bufferの参照実装

[Buffer実装](containers/buffer.md)をBufferと別名の`Buf<T>`としてIxPoolの上に書き、predefinedな`Buffer<UInt64>`と比べた。
同じ擬似乱数列で`new`、`put`、`fill`、重なる範囲の`copy`、別の値からの`copy`を1000回、aliasを通して両方へ適用し、各操作の後に
長さと全要素が一致することを二つの試作で確かめた。`fill`と`copy`は参照実装と同じくslot操作のloopで書き、`copy`はoffsetの大小で
loopの向きを選ぶ。同じBufferで前後どちらへ重なる`copy`も、範囲の一括primitiveなしに現行Bufferと一致した。overflowのtrapは
`trap` primitiveがないため比べていない。

## ImPoolとfreezeとthaw

Buffer上のemulationで、[Vector](containers/sequences.md#vector)の`Vector<T>`をImPoolの上に書いた。更新は前の値を変えず、`freeze`の
後のIxPoolへの書き込みも、`thaw`したIxPoolへの書き込みも、値と他のIxPoolから観測されないことを確かめた。`Symbol`を要素に
しても、valgrindで全allocationの解放とerror 0を確かめた。emulationのImPoolはBufferを含むため`Storable`にならず、
`Vector<Vector<T>>`は確かめていない。要素を`takeAt`で取り出して更新し、戻す`vectorUpdate`は、入れ子の値を一意に保ったまま
更新する経路として動いた。

## slot遷移とcontainer

- 要素型ごとの実装はstorageの作成、`peek`、`swap`、`meta`、`swapMeta`だけで足り、周辺は全て核の上の通常のgeneric関数として
  二つの試作で同じsourceに書けた。
- 核はLiveとVacantのpreconditionを持たない。C host試作でVacantとLiveのどちらのslotで`swap`しても、trapせずに古い値の側を返した。
- `takeAt`により、tombstoneのないMap削除、同じidentityでのrehash、Dequeのring展開、heapの穴を動かすsift、木とSlotMapの
  slot再利用を、要素をcopyせずに書けた。
- C host試作の検査は、containerの`grow`忘れを範囲のpreconditionへの違反として、利用者のprecondition違反を周辺が仮定する
  slot状態への違反としてtrapへ変えた。messageはIxPoolの条件を示し、container operationを示さない。
- Buffer上のemulationで`Symbol`を要素にしたMap、Deque、heap、SlotMapを動かし、valgrindで全allocationの解放とerror 0を
  確かめた。`takeAt`と`initAt`による移動は、managed valueのresponsibilityを一つに保った。

## `peek`とstorageの共有

C host試作ではbyte IxPoolのstorageを`Symbol`相当のtextと共有した。snapshotはO(1)で、以後どちらかへ書いた側だけがcopyした。
逆向きの変換は、textが一つの葉ならstorageを貸し、それ以外は一度だけflattenした。

読み出しを書き込みの組で派生させると、読むだけで共有storageのcopyが起き、読み出しだけのscenarioでcopy-on-writeのcopyが二回から
三回に増え、探索ごとのhost callも倍になった。`peek`は書き込まず、copyは書いた側の二回だけである。[区分](api/pool.md#区分)が`peek`を
計算量の核に置くのはこのためである。

## `Host<A>`とbyte列のfreezeとthaw

Buffer上のemulationで、`Buf<T>`の`from`、`into`、`*`の二方向を`Host<A>`、`freeze`、byte列の`freeze`と`thaw`の上に書き、host storageと
`Symbol`についてpredefinedなBufferと比べた。結果は一致し、`Host<A>`と`Symbol`は作った後のVecへの書き込みを観測しなかった。

generic codeはmemory intrinsicへ届かないため、`admit`と`observe`は要素型ごとのimplementationを持つoperation familyになった。
runtimeのprimitiveとして要素型ごとに実装するという位置づけと一致する。malはAddressをoffsetできないため、emulationの`observe`は
offsetより前の範囲を読み戻して書き直しており、本物のprimitiveには要らない追加のpreconditionを持つ。

## ropeとflatな`Symbol`

同じtextの操作を、AVL木のropeと、現在の`Symbol`のflatなbyte ownerの方式で比べた。flatな方式では、consumingな`+`が一意な
ownerをその場で伸ばす。1.4 MBの行の反転、split、比較、書き出しはflatが4〜12倍速く、ropeが勝ったのは大きなtextの中央への
挿入の反復だけだった。IxPoolとのstorage共有とcopy-on-writeはどちらでも成り立つため、`Symbol`の表現はflatのままでよく、
ropeはIxPool上の別containerとして持つ方が合う。

## IxPoolとBufferの非対称

IxPoolはBufferの上に、BufferはIxPoolの上に、どちらも意味の上では書ける。Buffer上のIxPoolはslotごとのsum tagと、`takeAt`ごとの
`Share`と`Drop`を払うが、IxPool上のBufferが払うのは占有tagの更新とIxPool終了時の走査だけである。IxPoolをprimitiveに
するのはこの非対称のためである。
Vacantを末尾だけに限ったdense primitiveへ`[Unit, T]`を載せる形とは意味が同じであり、残る差はこれらのcostだけである。
その大きさは測っていない。

## 試作から出た言語とcompilerの変更

- phantomな型parameterにしか現れない型argumentを推論できなかった。[D092](../../history/decisions/active/D092.md)で、
  constructorでない場合に推論する規則へ改めた。
- phantomなopaque argumentを持つgeneric codeの拒否、同じfileのopaque層をinferenceが見ない問題、diagnosticでopaque applicationの
  `<`が欠ける問題、operation implementationの欠落がkeyを示さない問題を修正した。

## containerの試作
試作では、六つの例を次の条件で動かし、全IxPoolの解放まで確認した。

- stack：2000個をpushし、逆順にpopする。
- Deque：両端へpushして折り返しを作り、両端からpopした後、折り返したまま成長させても順序が保たれることを確かめる。
- binary heap：擬似乱数の2000個をpushし、popの結果が減少しないことと個数を確かめる。
- SlotMap：2000個を挿入し、3個に1個を削除した後、同数を挿入し直す。削除した`SlotKey<T>`はmissingになり、残した`SlotKey<T>`と新しい`SlotKey<T>`は
  値を返す。
- Map：2000個を挿入し、100個を置き換え、1000個を削除した後、全keyの存在と値を確かめる。rehashを含む。
- 木：擬似乱数のkeyを2000個挿入し、半分を削除して、残りの存在、削除したkeyの不在、中順の単調性を確かめる。削除したkeyを
  入れ直してもcapacityは増えない。
