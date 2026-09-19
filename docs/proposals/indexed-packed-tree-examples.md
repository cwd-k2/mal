# indexed treeの挿入とbalance

Status: Draft example; non-normative

この文書は[indexで結ぶ`Packed`構造](indexed-packed-structures.md)に対し、node追加とAVL rotationを
`pack`と`edit`が渡す`Buffer`の`new/get/put`で書く具体例を示す。APIの型と意味は
[`Packed`構築と編集](../spec/packed.md#scoped-constructionとediting)を正とし、ここでは
tree固有のalgorithmとpreconditionだけを扱う。

## representation

index 0をheader兼null sentinelとし、nodeはindex 1以降に置く。headerの`left` fieldがroot indexを保持する。

```mal
// Node fields: key, height, left, right
// Header:      0,   0,      root, 0
AvlEntry :: (Int32, USize, USize, USize);
AvlTree :: Packed<AvlEntry>;

```

`0usize` linkはchildなしを表す。nodeのheightはleafで1、nullで0とする。完成したtreeはheaderを必ず持ち、すべてのchild indexが
同じPacked内を指し、各nodeについて左右のheight差が1以下であることをprogram invariantとする。

## heightとrotation

```mal
heightOf :: (USize, Buffer<AvlEntry>) -> USize :=
    (index, buffer) ->
        if (index == 0usize)
        then 0usize
        else {
            (_, height, _, _) := buffer.get(index);
            height
        };

refreshHeight :: (USize, Buffer<AvlEntry>) -> Unit :=
    (index, buffer) -> {
        (key, _, left, right) := buffer.get(index);
        leftHeight := heightOf(left, buffer);
        rightHeight := heightOf(right, buffer);
        height := if (leftHeight > rightHeight)
            then leftHeight + 1usize
            else rightHeight + 1usize;
        buffer.put(index, (key, height, left, right));
    };

rotateRight :: (USize, Buffer<AvlEntry>) -> USize :=
    (root, buffer) -> {
        (rootKey, rootHeight, pivot, rootRight) := buffer.get(root);
        (pivotKey, pivotHeight, pivotLeft, pivotRight) := buffer.get(pivot);

        buffer.put(root, (rootKey, rootHeight, pivotRight, rootRight));
        refreshHeight(root, buffer);
        buffer.put(pivot, (pivotKey, pivotHeight, pivotLeft, root));
        refreshHeight(pivot, buffer);
        pivot
    };

rotateLeft :: (USize, Buffer<AvlEntry>) -> USize :=
    (root, buffer) -> {
        (rootKey, rootHeight, rootLeft, pivot) := buffer.get(root);
        (pivotKey, pivotHeight, pivotLeft, pivotRight) := buffer.get(pivot);

        buffer.put(root, (rootKey, rootHeight, rootLeft, pivotLeft));
        refreshHeight(root, buffer);
        buffer.put(pivot, (pivotKey, pivotHeight, root, pivotRight));
        refreshHeight(pivot, buffer);
        pivot
    };
```

rotationはrecordのindex自体を移動しない。edgeを書き換え、新しいsubtree rootのindexを返す。`rootHeight`と`pivotHeight`は
record全体を置換するため一時的に保持し、直後の`refreshHeight`で再計算する。

## rebalanceと挿入

```mal
rebalance :: (USize, Int32, Buffer<AvlEntry>) -> USize :=
    (index, insertedKey, buffer) -> {
        (key, height, left, right) := buffer.get(index);
        leftHeight := heightOf(left, buffer);
        rightHeight := heightOf(right, buffer);

        if (leftHeight > rightHeight + 1usize)
        then {
            (leftKey, _, _, _) := buffer.get(left);
            if (insertedKey < leftKey)
            then rotateRight(index, buffer)
            else {
                newLeft := rotateLeft(left, buffer);
                buffer.put(index, (key, height, newLeft, right));
                rotateRight(index, buffer)
            }
        }
        else if (rightHeight > leftHeight + 1usize)
        then {
            (rightKey, _, _, _) := buffer.get(right);
            if (insertedKey > rightKey)
            then rotateLeft(index, buffer)
            else {
                newRight := rotateRight(right, buffer);
                buffer.put(index, (key, height, left, newRight));
                rotateLeft(index, buffer)
            }
        }
        else index
    };

insertAt ::
    (USize, Int32, Buffer<AvlEntry>) -> USize :=
    (index, insertedKey, buffer) -> {
        if (index == 0usize)
        then buffer.new((insertedKey, 1usize, 0usize, 0usize))
        else {
            (key, height, left, right) := buffer.get(index);

            if (insertedKey == key)
            then index
            else if (insertedKey < key)
            then {
                newLeft := insertAt(left, insertedKey, buffer);
                buffer.put(index, (key, height, newLeft, right));
                refreshHeight(index, buffer);
                rebalance(index, insertedKey, buffer)
            }
            else {
                newRight := insertAt(right, insertedKey, buffer);
                buffer.put(index, (key, height, left, newRight));
                refreshHeight(index, buffer);
                rebalance(index, insertedKey, buffer)
            }
        }
    };
```

再帰callはsubtree root indexを返す。rotationが起きなければ同じindex、起きればpivot indexになるため、callerは常に返されたindexを
parentへ書き戻す。構造の再帰を型ではなくこのoperation contractが担う。

## 空treeからbalanceする例

```mal
balancedExample :: Unit -> AvlTree := () ->
    pack<AvlEntry>((buffer) -> {
        header := buffer.new((0i32, 0usize, 0usize, 0usize));

        root1 := insertAt(0usize, 30i32, buffer);
        root2 := insertAt(root1, 20i32, buffer);
        root3 := insertAt(root2, 10i32, buffer);
        root4 := insertAt(root3, 25i32, buffer);
        root5 := insertAt(root4, 40i32, buffer);
        root6 := insertAt(root5, 50i32, buffer);

        buffer.put(header, (0i32, 0usize, root6, 0usize));
    });
```

`30, 20, 10`の挿入でright rotationが起き、その後の挿入でも必要なrotationを行う。完成時のlogical topologyは次になる。

```text
        30
       /  \
     20    40
    /  \     \
   10  25     50
```

nodeの物理順序は`new`の実行順のままであり、rootがどのindexかはheaderだけが決める。

## 既存treeへのnode追加

`edit`はsourceの要素を初期状態とし、`new`で末尾へnodeを追加できる。既存indexをremapする必要はない。

```mal
insertTree :: (AvlTree, Int32) -> AvlTree :=
    (source, key) ->
        edit<AvlEntry>(source, (buffer) -> {
            (_, _, oldRoot, _) := buffer.get(0usize);
            newRoot := insertAt(oldRoot, key, buffer);
            buffer.put(0usize, (0i32, 0usize, newRoot, 0usize));
        });
```

sourceは不変である。correctness baselineは最初の`new`または`put`でsourceをcopyする。source responsibilityがresultだけへ移り、ownerが一意で
capacityに余裕がある場合は同じstorageをresultへ移して追加できるが、これはoptional optimizationでありobservable semanticsには含めない。

同じnode数のrotationやkey更新では`new`を使わず、`get/put`だけでよい。末尾の未使用nodeは完成後の`/`で落とせるが、中間の
unreachable nodeを除いてindexを詰め直す処理は`pack`で再構築する。
