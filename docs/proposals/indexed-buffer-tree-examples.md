# Bufferで表すindexed treeの更新

Status: Exploratory

この文書は[indexで結ぶBuffer構造](indexed-buffer-structures.md)に対する更新方針を示す。完全なAVL実装を
surface syntaxの固定例として保存せず、現在のBuffer semanticsから必要な設計判断だけを記録する。

## その場の更新

appendで既存rowのindexは変わらない。先にchildを追加し、その後parent rowを`put`で置換できる。

```mal
appendLeaf :: (Tree, Int32) -> USize := (tree, value) ->
    tree.new((value, 0u8, 0usize, 0usize));

setChildren :: (Tree, USize, USize, USize) -> Unit := (tree, parent, left, right) -> {
    (value, _, _, _) := tree.get(parent);
    tree.put(parent, (value, 1u8, left, right));
};
```

この更新はtreeの全aliasから観測できる。rotationも関係するrowを`get`し、計算したrowを`put`するtransactionとして
記述する。途中状態をcallbackやexternへ渡さず、必要なrowを読み終えてから書き戻すと不変条件の境界が明確になる。

## 独立した結果

元のtreeを残すoperationは、明示的にcarrierをcopyしてから更新する。Buffer assignmentだけではsnapshotにならない。
copy helperはsourceのcountと順序を保存し、各要素を一度ずつ`get`してresultへ`new`する。copy完了後のresultには
元Bufferへのmanaged edgeがないため、その後の変更は独立する。

## 検証

tree validatorは少なくともroot coordinate、child bounds、kind、acyclic性を検査する。balance treeならheightも
domain invariantとして照合する。`Buffer<TreeNode>`という型はこれらを保証しないため、外部入力やhost storageから
`from<TreeNode>`で受けた値は検証前にtreeとして扱わない。
