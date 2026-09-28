# indexで結ぶBuffer構造

Status: Exploratory

`Buffer<T>`の要素に`USize`座標を格納すると、recursive typeを導入せずにtree、DAG、graphを表現できる。
Bufferは物理的なcarrierであり、座標の意味やtopologyの不変条件はdomain operationが所有する。

```mal
TreeNode :: (Int32, UInt8, USize, USize);
Tree :: Buffer<TreeNode>;
```

この例では`kind`がleafとbranchを区別し、branchの後半2 fieldだけをchild座標として解釈する。型だけではroot、
座標の範囲、acyclic性を証明しない。constructorとvalidatorがそれらを確立し、traversalは検証済みという
preconditionの下で座標を読む。

Buffer値をbindingや引数へ渡すcopyは同じmutable identityへのaliasを作る。したがって、更新operationは次のいずれを行うかを明記する。

- 既存carrierをその場で変更し、すべてのaliasに変更を公開する。
- `make`で作った新しいBufferへ`copy`で全rowを代入し、独立したsnapshotを返す。

rowを並べ替えるoperationは、carrier相対の全座標も同時にremapしなければならない。要素追加だけなら既存indexは安定する。
この性質を使う実行例は[`buffer-tree`](../../examples/buffer-tree/README.md)、relationを複数の方法で解釈する例は
[`relation-views`](../../examples/relation-views/README.md)を参照する。

## predefined Index

座標が参照する要素型をsource上へ残すため、`Buffer<T>`を補完するpredefined transparent aliasとして`Index<T>`を導入する案がある。
意味上の擬似定義は次のとおりであり、source fileへ宣言を置く必要はない。`Index`はほかのpredefined typeと同じく
top-levelで再宣言できない。

```mal
Index<T> :: USize;

TreeNode :: (Int32, UInt8, Index<TreeNode>, Index<TreeNode>);
Tree :: Buffer<TreeNode>;
```

`Index`の`T`はaliasの表現に寄与しないphantom parameterである。aliasはnominal identityを作らないため、
`Index<TreeNode>`、`Index<OtherRow>`、`USize`はすべて同じ型であり、既存の整数operationとBuffer operationをそのまま使う。
このnotationは座標の意図を示し、type argumentのdefinition navigationとhoverに参照先を残すが、異なるrow型や異なるBufferの
indexの混同、範囲、carrierとの対応を型検査で保証しない。それらは従来どおりdomain operationとinvariantが所有する。

Buffer operationのsource上のsignatureでは、要素位置を`Index<T>`、個数とcapacityを`USize`で表す。

```text
make<T>(USize)                                  -> Buffer<T>
#Buffer<T>                                      -> USize
Buffer<T>.new(T)                                -> Index<T>
Buffer<T>.get(Index<T>)                         -> T
Buffer<T>.put(Index<T>, T)                      -> Unit
Buffer<T>.fill(Index<T>, USize, T)              -> Unit
Buffer<T>.copy(Index<T>, Buffer<T>, Index<T>, USize) -> Unit
```

`Index<T>`はtransparentなので、既存の`USize` argumentとresultの意味や互換性は変わらない。区別はAPIとsourceが、座標、個数、
capacityの役割を表すためだけに使う。`from<T>`と`buffer.into`のoffsetはBufferの座標ではなくexternal storage上の要素offsetなので、
引き続き`USize`で表す。

型parameterがalias右辺に現れないかどうかはdeclarationから決める。canonical typeを構成するときはphantom parameterに対応する
type argumentをrepresentation dependencyとして展開しない。上の`TreeNode`はsource上では自身を参照するが、canonical typeは
`(Int32, UInt8, USize, USize)`であり、recursive value typeや新しいruntime indirectionを導入しない。実際の表現に寄与するalias cycleは
引き続き拒否する。

editorはchecked canonical typeから`USize`だけを表示するのではなく、現在のalias表示方針どおりsource spellingの
`Index<TreeNode>`を保持する。型aliasのhoverは右辺を一段だけ表示し、phantom argumentから`TreeNode`の展開へ進まない。
表示量と再訪は既存のbounded expansionに従う。

この案を実装する場合は、少なくとも次を確認する。

- phantom argumentだけを通る自己参照を受理し、同じaliasのproduct、sum、function、`Buffer`位置を通るcycleは拒否する。
- `Index<A>`と`Index<B>`を`USize`として比較、演算、Bufferへ格納でき、既存のBuffer program、runtime representation、ownershipを変更しない。
- phantom argument内のtype referenceがdefinitionとrenameの対象になり、hoverが再帰展開せず`Index<TreeNode>`を表示する。
- parameterが別のalias applicationのargumentとして右辺に現れる場合はphantomとみなさず、推移的な消去を暗黙に導入しない。

外部resourceに属するnode、recoverable allocation、個別releaseが必要な構造には`Address`とextern contractを使う。
その場合、malはAddressのreferentを知らず、`from<T>`と`buffer.into`によるcopy可能な範囲だけをC hostとのcopy primitiveが提供する。
