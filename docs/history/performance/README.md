# 性能測定履歴

Status: Historical records

このdirectoryは再現条件と日付を伴う測定結果を保存する。ここにある結果はすべてhistorical recordであり、文中の
「現在」「現行」は測定時点を指す。現在の最適化課題と採用条件は
[generated program最適化policy](../../development/generated-program-optimization.md)、通常の検証commandは
[test policy](../../development/testing.md)を正とする。

| 調べたい対象 | 記録 | 位置づけ |
|---|---|---|
| backend世代と生成program | [backend](backend/) | 退役したgenerated Cと、その後のLLVM backendを分離した時系列記録 |
| 現在のexample corpus | [example corpus](examples.md) | baseline / productionと採択済みclosure optimizationの比較 |
| recursionとloop combinator | [loop combinator](loop-combinators.md) | native recursion、stack bound、persistent parameterの調査 |
| genericsとmanaged container | [generics](generics.md) | HKT、monad、nested BufferのC/Rust比較と抽象消去の境界 |
| managed value | [managed Engram](managed-engrams.md) | C backend当時のownership costと回帰条件 |
| Buffer element storage | [Buffer](buffer.md) | LLVM backendのcanonical elementとruntime-owned handleのC比較 |
| C host boundary | [C host ABI](c-host-abi.md) | headerとadapter生成の測定 |
| compiler自身 | [compiler compile-time](compiler.md) | frontend、lowering、editor queryの規模と深度 |

新しい測定は対象を所有する既存文書へ日付順に追加する。実装世代を跨ぐ記録は`backend/`、program集合を横断する
最新比較は`examples.md`へ置き、activeな作業計画をこのdirectoryへ書かない。
