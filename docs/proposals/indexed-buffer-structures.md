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

Bufferのcopyは同じmutable identityへのaliasを作る。したがって、更新operationは次のいずれを行うかを明記する。

- 既存carrierをその場で変更し、すべてのaliasに変更を公開する。
- 新しいBufferへ全rowをcopyし、独立したsnapshotを返す。

rowを並べ替えるoperationは、carrier相対の全座標も同時にremapしなければならない。要素追加だけなら既存indexは安定する。
この性質を使う実行例は[`buffer-tree`](../../examples/buffer-tree/README.md)、relationを複数の方法で解釈する例は
[`relation-views`](../../examples/relation-views/README.md)を参照する。

外部resourceに属するnode、recoverable allocation、個別releaseが必要な構造には`Address`とextern contractを使う。
その場合、malはAddressのreferentを知らず、`from<T>`と`buffer.into`によるcopy可能な範囲だけをC host profileが提供する。
