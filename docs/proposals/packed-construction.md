# scoped capabilityによる`Packed`構築

Status: Draft proposal; non-normative

この文書は、要素数が事前に分からない計算からmal-ownedな`Packed<T>`を直接構築する案を定める。現在の言語規則は
[`Region`と`Packed`](../spec/packed.md)、canonical representationは[external memory](../spec/memory.md)、managed ownerの
実装規約は[managed value ownership](../implementation/ownership.md)、機能を置く境界は
[authority](../design/authority.md)を正とする。既存Packedを初期状態にして置換または追加する案は
[scoped capabilityによる`Packed`編集](packed-editing.md)で別に扱う。

## 目的

現在もexternal allocatorから得たstorageを`Region<T>`として初期化し、`<-region`で`Packed<T>`へadmitできる。しかしこの経路は
external allocationのfree contractを必要とし、完成時に別のmal-owned storageへcopyする。構築中storageを最初からMalのownerにすれば、
runtimeがcapacityを増やし、完成時に同じownerをimmutableな`Packed<T>`へ移せる。

末尾追加だけのproducerは列を構築できるが、親を子より先に置くtree、forward edge、cycleを持つgraphでは、後から確定したindexを
初期化済み要素へ書き戻せない。構築中だけ有効なcapabilityにappend、read、replaceを閉じることで、完成後の`Packed`をimmutableに
保ったままこれらを表現する。

## interface

source-levelで追加するbindingはpredefined generic intrinsic `pack`だけとする。構築操作は通常のfunction valueとしてcallbackへ渡す。

```mal
pack<T> ::
    (((T -> USize), (USize -> T), ((USize, T) -> Unit)) -> Unit)
    -> Packed<T>;
```

callbackの三つのparameterを、この文書では`new`、`get`、`put`と呼ぶ。

```mal
packedInt32 := pack<Int32>((new, get, put) -> {
    first := new(10i32);
    _ := new(20i32);
    put(first, get(first) + 1i32);
});
```

- `new(value)`はvalueを新しい末尾slotへ置き、その安定したindexを返す。
- `get(index)`は構築中の現在値を返す。
- `put(index, value)`は構築中の現在値を置換する。

単純な列生成では不要なcapabilityをpatternで捨てられる。

```mal
addRange :: ((USize -> USize), USize, USize) -> Unit :=
    (new, current, end) ->
        if (current == end)
        then ()
        else {
            _ := new(current);
            addRange(new, current + 1usize, end);
        };

values := pack<USize>((new, _, _) ->
    addRange(new, 0usize, 1000usize)
);
```

`new`がindexを返すため、独立したlength capabilityは設けない。現在のcountが必要な処理は最後に返されたindexから追跡する。
具体的な対象programでこの規則が不自然なら、length追加を別に評価する。

## indexed structureの構築

binary treeをflat record列に置く最小例では、親を仮のleafとして先に追加し、子のindexが確定した後で置換できる。

```mal
// kind: 0 = leaf, 1 = branch
TreeNode :: (Int32, UInt8, USize, USize);

tree := pack<TreeNode>((new, _, put) -> {
    root := new((1i32, 0u8, 0usize, 0usize));
    left := new((2i32, 0u8, 0usize, 0usize));
    right := new((3i32, 0u8, 0usize, 0usize));
    put(root, (1i32, 1u8, left, right));
});
```

子から親の順に追加できる処理は`put`を使わない。parent-first traversal、forward edge、cycle、複数の既存情報から要素を再計算する処理だけが
必要なcapabilityを使う。node挿入、tree結合、balance rotationは
[indexで結ぶ`Packed`構造](indexed-packed-structures.md)を参照する。

## 暫定意味論

`pack<T>(callback)`は次の順序で評価する。

1. callbackを一度評価する。
2. countが0で、Malがlifetime authorityを持つ空のbuilderを作る。
3. builderを閉じ込めた`new`、`get`、`put`をcallbackへ渡す。
4. `new(value)`はvalueを一度評価し、必要ならcapacityを増やし、新しい末尾slotへstoreしてそのindexを返す。
5. `get(index)`と`put(index, value)`は、それ以前に完了したoperationを観測する。
6. callbackが`Unit`で正常完了するとbuilderをfreezeし、count要素のowned `Packed<T>`を返す。

`get`と`put`はapplication開始時に`index < count`を未検査preconditionとして要求する。`put`はcountを変更せず、未初期化slotやholeを
作らない。再確保後もindexは安定するが、storageのAddressやCursorはsourceへ公開しない。growth factor、initial capacity、余剰capacityの
保持または縮小は、成功時に同じ値を作るimplementation detailとする。

count、`count * stride(T)`、owner allocation sizeがtargetで表現できない場合と、実際に必要となったallocationのfailureはtrapとする。
`Packed<Unit>`はcount-only representationでよい。

## 型とcapabilityの境界

現行仕様では`Packed<T>`は`Representable(T)`の場合だけwell-formedであり、`pack`も同じ制約を持つ。これによりelementはmanaged ownerを
含まず、growthは初期化済みbytesの移動、破棄はflat byte ownerの解放になる。recursive typeおよびmanaged elementを持つPackedへの拡張は
本proposalの採択条件にも、その直後の実装段階にも含めない。

capabilityはhelper、closure、recursive frameへ渡せるが、callbackの正常完了後には使えない。callback resultの`Unit`、Mal内部に留まる
function value、immutable capture、result binderのcapture規則により、capabilityを保持するrootはcallback終了までに終わる。mutable cellまたは
hostへ渡せるfunction carrierを導入するときは、scoped typeまたはsealed stateとして再検討する。

`put`は現在の`Representable(T)`ではbyte replacementである。将来managed elementを認める場合は、旧値のreleaseと新値のhandoffを
operationの意味へ加える必要がある。この拡張はflat indexed structureの利用結果を確認した後の独立した低優先度候補とする。

## loweringの候補

`pack`をresolve後のbinding identityで認識するcompiler intrinsicとする。backendはstableなbuilder stateを参照するcapability entryを生成し、
runtimeのreserve後にはdata pointerを再取得する。`new`、`get`、`put`はtyped valueとindexだけを運び、allocation identityを公開しない。

既知のlambdaではcallback inline、capability closure elimination、builder stateのstack配置、既知countのpreallocationをoptional optimizationに
できる。baselineはfirst-class functionとして同じobservable valueとoperation順序を作る。

## 検証境界

採択する場合は、empty、複数回growth、Unit、product/sum、再帰的callback、helperへ渡した各capability、allocation failure、countとbyte sizeの
overflow、正常完了後のlive allocation count 0を検査する。`get`のread-after-new、`put`のread-your-writes、同じindexへの複数回put、
growth前に得たindexのgrowth後の利用、範囲外preconditionもfocused testで固定する。

tree、DAG、cycleを持つgraphの構築はindex topologyを検査し、完成後のPackedが通常のimmutable operationだけを公開することを確認する。
