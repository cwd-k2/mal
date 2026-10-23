# 既存Bufferとsnapshotを分ける境界

Status: Exploratory support document; rebased on mal v0.7

この文書はpointer/IxPool案と、採択済みの`Buffer`および`Symbol`変換を同時に決めないための境界を記録する。
現行言語の規則は[`memory`](../../../spec/memory.md)を正とする。

## 説明上のdense container

pointerとIxPoolだけでdenseな可変列を説明すると、参照構造は次になる。

```mal
opaque Dense<A> :: (*USize, IxPool<A>);
```

`*USize`が論理長を指し、IxPoolがcapacityとindexed placesを持つ。invariantは
`0 <= load(count) <= capacity`、`[0, load(count))`がLive、残りがVacantである。

この分解は基礎の表現力を確認する例であり、predefined `Buffer<A>`のsource type、operation、C ABI、物理表現を
変更する提案ではない。dense containerはoccupancyをlengthから導き、logical identityとlifecycleを保ったままmetadataと
storageを一allocationへco-allocateできる。

## 既存prefix `*`との分離

現行v0.7は次をsnapshot変換として定める。

```text
*Buffer<UInt8> -> Symbol
*Symbol        -> Buffer<UInt8>
```

pointer案の`*A`はtype位置だけに現れ、readは`load`、writeは`store`で表す。従ってexpression位置の既存prefix
`*`とはgrammar contextで分離され、pointer dereferenceとしてoverloadしない。Buffer/Symbol変換の意味や移行は
pointer coreでは変更しない。

## snapshotを分離する理由

IxPoolから得たpointerはpool handleとは独立に残り、同じcoordinateの変更を観測できる。この状態でfreezeを
導入すると、既存pointerをmutable側に残してcopyするか、copy-on-writeへ参加させるか、outstanding pointerを
拒否する追加authorityが必要になる。どれもpointer/IxPoolの最小核からは決まらない。

初回proposalは`ImPool`、Vector、freeze、thawを含めない。immutable sequenceが必要なら、copy条件とhandle要素の
shallow/deep観測を明記した独立の型として評価する。

## 導入順

1. `*A`、load、store、takeのidentityとlifecycleを採択する。
2. `at`が`*A`を返すIxPoolとgrowth時の安定性を実装する。
3. pointer/IxPoolだけで代表containerを書き、表現力と費用を測る。
4. その後にBuffer内部表現とsnapshotを別々に判断する。
