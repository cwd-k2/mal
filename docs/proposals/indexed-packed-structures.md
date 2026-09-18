# indexで結ぶ`Packed`構造

Status: Draft proposal; non-normative

この文書は、再帰的なdata structureを`Representable`なrecord列とindexで表し、構造の再帰をoperationの再帰として扱う方針を評価する。
構築capabilityは[`pack` proposal](packed-construction.md)、既存値からの置換と追加は
[`edit` proposal](packed-editing.md)、現在のoperationは[`Region`と`Packed`](../spec/packed.md)を正とする。

## 優先する方向

recursive typeおよびmanaged elementを持つ`Packed`は本proposalの前提にしない。まず現行の`Representable(T)`境界を保ち、次を組み合わせる。

- topologyを`USize` indexとして`Packed<NodeRecord>`へ格納する。
- 新しい構造は`pack(new, get, put)`で構築する。
- 既存構造へのnode追加または置換は`edit(new, get, put)`で行う。
- traversal、結合、挿入、rotationはindexを辿る通常の再帰関数として書く。

この方向でtree、DAG、cycleを持つgraph、AST、control-flow graphを表現できる。Packedをindirectionに使うrecursive aliasは
現在の実装順序から外し、flat encodingの具体的な負担が繰り返し確認された場合にだけ再評価する。

## tree representation

最小のfull binary treeは次のrecordで表せる。

```mal
// kind: 0 = leaf, 1 = branch
TreeNode :: (Int32, UInt8, USize, USize);
Tree :: Packed<TreeNode>;
```

rootをindex 0とし、branchの第三、第四要素をleft、right indexとする。`kind`の値、rootの位置、child bounds、acyclic性はprogramの
preconditionであり、`Packed`の型自体は保証しない。

## 再帰的な結合

同じshapeを持つ二本のtreeを加算して新しいtreeを作る。親を先に`new`で作り、子の構築後にlinkを`put`するため、source treeとresult treeの
物理indexが一致する必要はない。

```mal
NewNode :: TreeNode -> USize;
PutNode :: (USize, TreeNode) -> Unit;

mergeAt ::
    (Tree, USize, Tree, USize, NewNode, PutNode) -> USize :=
    (leftTree, leftIndex, rightTree, rightIndex, new, put) -> {
        (leftValue, kind, leftLeft, leftRight) := leftTree # leftIndex;
        (rightValue, _, rightLeft, rightRight) := rightTree # rightIndex;
        value := leftValue + rightValue;
        output := new((value, 0u8, 0usize, 0usize));

        if (kind == 0u8)
        then output
        else {
            outputLeft := mergeAt(
                leftTree, leftLeft, rightTree, rightLeft, new, put
            );
            outputRight := mergeAt(
                leftTree, leftRight, rightTree, rightRight, new, put
            );
            put(output, (value, 1u8, outputLeft, outputRight));
            output
        }
    };

mergeTrees :: (Tree, Tree) -> Tree :=
    (leftTree, rightTree) ->
        pack<TreeNode>((new, _, put) -> {
            _ := mergeAt(
                leftTree, 0usize, rightTree, 0usize, new, put
            );
            ()
        });
```

この例は左右の`kind`とshapeが一致することをpreconditionとする。shapeが異なる結合でも、欠けたchildをcopyするpolicyを
`mergeAt`へ加えれば同じcapabilityで構築できる。

## topologyを保つ`edit`

同じshapeなら、左treeのlinkを保ったまま値だけを更新できる。この用途ではnode追加がないため`pack`より`edit`が狭く、ownerが一意なら
storage再利用の余地もある。

```mal
GetNode :: USize -> TreeNode;
PutExistingNode :: (USize, TreeNode) -> Unit;

addIntoAt ::
    (Tree, USize, USize, GetNode, PutExistingNode) -> Unit :=
    (rightTree, leftIndex, rightIndex, get, put) -> {
        (leftValue, kind, leftLeft, leftRight) := get(leftIndex);
        (rightValue, _, rightLeft, rightRight) := rightTree # rightIndex;
        put(leftIndex, (
            leftValue + rightValue, kind, leftLeft, leftRight
        ));

        if (kind == 0u8)
        then ()
        else {
            addIntoAt(rightTree, leftLeft, rightLeft, get, put);
            addIntoAt(rightTree, leftRight, rightRight, get, put);
        }
    };

mergeSameShape :: (Tree, Tree) -> Tree :=
    (leftTree, rightTree) ->
        edit<TreeNode>(leftTree, (_, get, put) ->
            addIntoAt(rightTree, 0usize, 0usize, get, put)
        );
```

node挿入は`edit`の`new`で表せる。AVL rotationを含む完全な形は
[indexed treeの挿入とbalance](indexed-packed-tree-examples.md)に分ける。

## 得るものと負担

| 観点 | flat indexed structure |
|---|---|
| runtime representation | 現行のflat `Packed<Representable record>`を維持 |
| 構築 | `new/get/put`のscoped capabilityが必要 |
| traversal | recursive typeのpatternではなくindexを辿るoperationになる |
| ownership | element destructorや再帰owner graphを追加しない |
| sharingとcycle | 複数edgeやback-edgeを同じindexへ向けて表現可能 |
| validation | bounds、tag、root、acyclic性などをprogramが所有 |
| deep structure | 非tail traversalには明示的なindex stackが必要になる場合がある |

graphにcycleがある場合、単純な再帰traversalは停止しない。visited setまたはprogram固有のmarkingが必要になる。treeでも深さがnative stackの
上限を超える場合は、構造をflatにしただけでは解決せず、operationを明示的stackへ変換する必要がある。

## 低優先度の将来候補

`RecursiveType<T> :: [Unit, (T, Packed<RecursiveType<T>>)]`のようなguarded recursive aliasは技術的候補として残すが、現段階では
実装対象にも次段階のprototype対象にもしない。少なくとも次の要求がflat encodingで繰り返し問題になった場合にだけ再評価する。

- subtreeを独立した値としてAPI間で渡し、ownerを共有したい。
- generic traversalをindex recordのschemaから独立して書きたい。
- topology invariantを型構造として観測する必要がある。
- index remapとheader conventionのprogram負担が、型検査とmanaged element runtimeの追加costを上回る。

再評価時にはrecursive aliasだけでなく、managed elementの構築、index resultのownership、slice、growth時のmove、深さに依存しないreleaseを
一緒に解決する。これらは`pack`によるflat indexed structureの採択を妨げない独立段階とする。
