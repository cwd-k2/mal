# reference compiler compile-time評価

Status: Current measurement record

この文書はreference compiler自身とsemantic editor queryの性能測定方法、baseline、採否判断を所有する。
生成programのruntime評価は[generated C performance](performance.md)、通常の検証は[test policy](testing.md)を正とする。

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

LSPは最初のsemantic requestで得たanalysisをopen documentのversionと共にmemoryへ保持し、後続requestごとに
`editor::analyze`を繰り返さない。full document change時に破棄し、次のsemantic requestで再計算する。diagnosticだけを
必要とするchangeでは追加のsemantic indexを構築せず、invalid sourceではsemantic resultを保持しない。これはincremental
compilationではなく、同一versionのimmutable resultの再利用である。

parse後の各frontend stageに支配的かつ不要な処理は観測されなかった。generated C側は
[generated C performance記録](performance.md)の再検討条件を満たす新しいhotspotがないため変更しない。
