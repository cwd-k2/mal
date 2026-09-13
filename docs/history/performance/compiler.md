# reference compiler compile-time評価

Status: Historical measurement record

この文書はreference compiler自身とsemantic editor queryの性能測定方法、baseline、採否判断を所有する。
生成programのruntime評価は[generated C performance](generated-c.md)、通常の検証は
[test policy](../../development/testing.md)を正とする。

## 測定方法

`compiler/benches/pipeline.rs`は250個のtype alias、500個の定数、251個のfunctionを持つ45,904 byteのsourceを
memory上で生成する。parse、resolve、check、各lowering、C emissionを独立して測定し、frontend全体とsemantic queryも
測定する。各項目は100 ms以上の反復を7 sample行い、`ns/iter`の中央値と最小・最大を表示する。

```nu
cargo bench --manifest-path compiler/Cargo.toml --bench pipeline
```

絶対時間をCIの合否条件にしない。同じoptimized binary、同じworkload、同じmachineで変更前後を比較し、結果と
language behaviorのtestを分けて扱う。

## 2026-09-05 lossless lexing調査

環境はx86_64 NixOS development environment、Rust 1.97.1。調査前の`lex`はformatter用の`lex_lossless`へ
委譲していたため、通常のcompile pathでもtoken、whitespace、line commentの全lexemeを構築してから捨てていた。
同一benchmark runnerでこの経路と、通常`lex`ではlossless記録を行わない経路を交互に測定した。

| Boundary | 変更前 median | 変更後 median | 変化 |
|---|---:|---:|---:|
| `lex` | 928,501 ns | 199,448 ns | -78.5% |
| `parse_tokens` | 332,473 ns | 331,395 ns | -0.3% |
| `parse` | 1,345,790 ns | 552,815 ns | -58.9% |
| `resolve` | 453,153 ns | 456,440 ns | +0.7% |
| `check` | 443,035 ns | 442,605 ns | -0.1% |
| `pipeline::check` | 2,429,949 ns | 1,478,958 ns | -39.1% |
| `editor::analyze` | 3,604,467 ns | 2,710,857 ns | -24.8% |

変化しないはずのstageが概ね同水準であり、改善がlossless bookkeepingを含む境界に集中したため、この変更を採用する。
`lex_lossless`は従来どおり全lexemeを返し、formatterのliteral/comment保存contractは変えない。

LSPはdiagnostic生成で成功したfrontend analysisをopen documentのversionと共にmemoryへ保持し、最初のsemantic requestで
そのresolved/checked programからsemantic indexを構築する。後続requestでは同じindexを再利用する。full document change時に
analysisとindexを破棄する。diagnosticだけを必要とするchangeではsemantic indexを構築せず、invalid sourceではどちらも
保持しない。これはincremental compilationではなく、同一versionのimmutable resultの再利用である。

parse後の各frontend stageに支配的かつ不要な処理は観測されなかった。generated C側は
[generated C performance記録](generated-c.md)の再検討条件を満たす新しいhotspotがないため変更しない。

## 2026-09-05に確認した入力深度

一つのblockで、直前のbindingを参照する単純なbindingを512個直列に並べて当時の`emit-c`まで処理すると、
reference compilerがRust threadのstack overflowでabortすることを確認した。通常のdiagnostic経路を通らない
compiler processの異常終了なので、この記録では入力規模の性能問題ではなくrobustness defectとして分類した。

解消条件は、どのstageの再帰がsource上のbinding数に比例して深くなるかを切り分け、十分に大きい直列blockを
正常にcompileするかstructured diagnosticとして拒否し、processをabortしないregression testを置くこととした。
この調査はgenerated programのSymbol表現とは独立に行う。

## 2026-09-13 左結合operator列

同一の`Int32`加算を左結合で並べたsourceでは、Pratt parserがloopで構文を読む一方、resolve、check、core loweringが
左spineをhost再帰で辿っていた。変更前のoptimized `malc check`は1,200項でstack overflowによりabortした。

resolveとcheckを反復走査へ変更し、core境界で中間結果をsource順の`let`列へ変換した。4,096項について
parse、resolve、check、core、ANF、closure conversion、execution planning、LLVM emissionまでをdebug test threadで完走する
regression testを置いた。これは括弧による明示的な構文nestの上限緩和ではなく、平坦に記述できるoperator列を内部treeの
形だけで制限しないための変更である。

## 2026-09-13 sum layout

LLVM表現がsumのtagに続けて全variant型をfieldとして並べていたため、値sizeが最大payloadではなく全payloadの総和になり、
共有された同型variantを持つaliasではlayout計算とLLVM type文字列も重複していた。C host表現は既にunionを使用しており、
implementation文書が定める表現とも一致していなかった。

LLVM表現をtagと最大payload長のbyte regionへ変更し、全variant offsetを同じpayload先頭へ写した。layout計算は共有型nodeを
memoizeする。`[UInt8, UInt64]`の64-bit target上の内部sizeは従来の16 byteから12 byteとなり、同じ直前型を二variantに持つ
64段のsum DAGは256 byte、LLVM type文字列1,500 byte未満として計算できることを回帰テストにした。

同じDAGをextern signatureに使う場合、当初のC shim生成は各辺からmarshalling planとsum helperを再構成していたため、
内部layoutを共有しても生成量が指数的に増加した。planを共有node identityでinternし、read/write helperも方向別に一度だけ
生成するよう変更した。直前の型を二variantに持つ16段のextern sumについて、生成shimを250,000 byte未満かつwrite helper
16個とする回帰テストを置いた。
