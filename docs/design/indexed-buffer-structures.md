# indexで結ぶBuffer構造

Status: Current v0.6 design guidance

この文書はBuffer上に座標構造を作る際のmodeling方針を記録する。現在の型規則とBuffer operation signatureは
[型](../spec/types.md)と[AddressとBuffer](../spec/memory.md)を正とする。

`Buffer<T>`の要素に`USize`座標を格納すると、recursive typeを導入せずにtree、DAG、graphを表現できる。
Bufferは物理的なcarrierであり、座標の意味やtopologyの不変条件はdomain operationが所有する。

座標の役割をsource上へ残す場合は、その意味を所有するdomainがtransparent aliasを宣言する。

```mal
NodeCoordinate :: USize;
TreeNode :: (Int32, UInt8, NodeCoordinate, NodeCoordinate);
Tree :: Buffer<TreeNode>;
```

`NodeCoordinate`は`USize`と同じcanonical typeであり、nominal identityを作らない。この名前はfieldとoperationの意図を示すが、
異なるBufferの座標の混同、範囲、root、acyclic性を型検査で保証しない。constructorとvalidatorがそれらを確立し、traversalは
検証済みというpreconditionの下で座標を読む。要素型との対応を表示したいdomainは、一般のphantom generic aliasとして
`Coordinate<A> :: USize`を宣言できるが、そのtype argumentも同じ不変条件を証明しない。

Bufferのpredefined operationは要素位置、個数、capacityを`USize`で受け渡す。

```text
make<T>(USize)                             -> Buffer<T>
#Buffer<T>                                 -> USize
Buffer<T>.new(T)                           -> USize
Buffer<T>.get(USize)                       -> T
Buffer<T>.put(USize, T)                    -> Unit
Buffer<T>.fill(USize, USize, T)            -> Unit
Buffer<T>.copy(USize, Buffer<T>, USize, USize) -> Unit
```

domain aliasはtransparentなので、これらのoperationと整数operationへそのまま渡せる。count、capacity、length、外部storageの
offsetのように座標とは異なる量へdomainの座標名を付けず、`USize`のまま表す。

Buffer値をbindingや引数へ渡すcopyは同じmutable identityへのaliasを作る。したがって、更新operationは次のいずれを行うかを明記する。

- 既存carrierをその場で変更し、すべてのaliasに変更を公開する。
- `make`で作った新しいBufferへ`copy`で全rowを代入し、独立したsnapshotを返す。

rowを並べ替えるoperationは、carrier相対の全座標も同時にremapしなければならない。要素追加だけなら既存indexは安定する。
この性質とdomain aliasを使う実行例は[`buffer-tree`](../../examples/buffer-tree/README.md)、relationを複数の方法で解釈する例は
[`relation-views`](../../examples/relation-views/README.md)を参照する。

外部resourceに属するnode、recoverable allocation、個別releaseが必要な構造には`Address`とextern contractを使う。
その場合、malはAddressのreferentを知らず、`from<T>`と`buffer.into`によるcopy可能な範囲だけをC hostとのcopy primitiveが提供する。
