# 試作から引き継ぐ証拠

Status: Historical experiments rebased on the pointer/IxPool proposal

既存試作は`Header`を内蔵し、slotを`Vacant | Live(V)`としてread/exchangeする旧IxPool案を対象にした。
現在のproposalは`*A` pointerを基礎とし、`IxPool<A>.at(index)`がfirst-classなpointerを返す。このため試作結果を
そのまま現APIの実装証明とは扱わず、再利用できるmechanismと未検証の境界を分ける。

## 現行runtime kernel

`runtime/c11/pool.c`と
[`runtime_pool.rs`](../../../crates/mal-compiler/tests/driver/runtime_pool.rs)は、source constructからまだ
選択されず`mal.h`にも公開しない内部kernelである。次を直接検査している。

- stableなshared ownerと、growthで交換可能なbacking allocation
- bitmap occupancyと、Vacant payloadを値として読まないこと
- growth後もcoordinate、occupancy、Live carrierが保存されること
- zero-stride payload、capacityとallocation sizeのoverflow trap
- trivial carrierとOwned childを同じspecialization後layoutへ置くこと
- readのShare、exchangeとrelocationのMove、終了時のLive carrierのDrop
- `-O2 -flto`とAddressSanitizer/UBSanで同じharnessが通ること

この証拠はIxPool owner、occupancy、growth、Aのlifecycle glueへ再利用できる。旧kernelのHeader carrierは
container metadata用のstandalone `*H`へ移し、kernelから外す。

## 未検証のpointer境界

既存試作は次を検証していない。

- standalone `*A`のVacant/Liveとload / store / take
- `at(pool, i)`がowner + coordinateのfirst-class pointer carrierを返すこと
- 同じpoolとindexから得た複数pointerが状態を共有観測すること
- growth後も以前のpointer carrierが有効なこと
- 元のIxPool handleよりpointerが長生きし、最後のpointerまでownerを生かすこと
- pointerやIxPool handleを別placeへ保存したときのcycle境界
- metadata pointerとIxPoolという二identityのcost、またはidentityを保ったas-if co-allocation

従ってruntime kernelが動くことは、現在のsource APIが既に実装済みであることを意味しない。再開位置は
[compilerとruntimeの実装](runtime/implementation.md#実装を再開する位置)を正とする。

## source semantic referenceから残る証拠

`.scratch/pool-prototype/`は現行Buffer上で旧Pool APIをemulateしたhistorical test corpusである。API名と
snapshot部分は現proposalの参照実装ではないが、次のalgorithmic evidenceは残る。

- Buffer相当はcountとindexed storageへ分けて書け、alias、`new`、`get`、`put`、`fill`、重なる`copy`を
  predefined Bufferと照合できた。
- Map、Deque、binary heap、SlotMap、treeは、途中をVacantにできるcoordinate空間と、値を複製しない
  Live→Vacant→LiveのMoveを使って実装できた。
- `Symbol`などのmanaged valueを移す経路は、Share/Dropを余分に増やさず最後に全allocationを解放できた。
- Buffer上のsum-tag emulationでもcontainer algorithmは成立するため、IxPoolをprimitiveにする根拠は
  表現不能性ではなくoccupancyとMoveのcostにある。

旧`takeAt`は現在の`take(pool.at(i))`、旧`initAt`はVacant placeへのstore、旧`getAt`はloadへ対応する。
旧`peek` / `swap`が`Slot<A>`値を返す部分と、Header operationは対応APIを持たない。

## snapshot実験の扱い

旧試作は`ImPool`、`freeze`、`thaw`、Vector候補もBuffer上のcopyで検査した。しかしfirst-classなpointerを
導入すると、outstanding pointerがあるfreezeの意味を新たに決める必要がある。旧snapshot結果はpersistent
containerの一般的な実装可能性を示すhistorical evidenceに留め、今回の採択対象やruntime contractにはしない。

## occupancyの測定

direct Cとsafe Rustでseparate byte、bitmap、inline sumを比較した結果、bitmapは最小memoryで、randomな
word payloadでは両言語で最速だった。byte payloadのRustでは三方式が2%以内で、sequential accessの差も
小さい。詳細は[Pool占有tagの表現比較](../../history/performance/pool-occupancy.md)に置く。

この測定は初期IxPool kernelにbitmapを選ぶ根拠にはなるが、first-class pointer carrier、owner count、`at`の
escape costを含まない。dense Bufferはoccupancyをinvariantから除去できるため、全Bufferをbitmap付き表現へ
置き換える根拠にもならない。

## 次のtest matrix

1. standalone pointerでtrivial、Owned、zero-sized Aの状態遷移とDrop回数を検査する。
2. pool-derived pointerをgrowth前後とIxPool handle終了後に使う。
3. 同じcoordinateの複数pointerと異なるpoolの同じindexを区別する。
4. pointer/IxPool handleの入れ子を通常値、Buffer element、別placeで検査する。
5. IxPool上のBuffer、Map、Deque、heap、SlotMap、treeを新APIへ移植し、flat formと共有descriptor formのalias観測を
   それぞれ検査する。
6. hand-lowered lower boundと比較し、occupancy、carrier escape、二identityを別々に測る。

## 周辺で得たcompiler上の知見

旧generic operation familyの試作では、phantom型argumentの推論、opaque層、kind検査、diagnosticの問題を
見つけ、[D092](../../history/decisions/active/D092.md)と
[D093](../../history/decisions/active/D093.md)へ反映した。この知見はpointer/IxPoolのgeneric operationにも
有効だが、旧constructor-polymorphicなIxPool/ImPool共通APIは本proposalへ持ち込まない。
