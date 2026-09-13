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

formatterのcontrol layout分類にも同じleft spineを再帰走査する経路が残っていたため、expressionとblockを一つの明示work
stackで巡回する方式へ変更した。4,096項の加算列を`format`し、全4,095 operatorを保持する回帰テストを追加した。

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

## 2026-09-13 alias dependency

aliasのsource構文は一宣言ごとに平坦でも、後方宣言を参照する長いdependency chainはcanonical type展開時のRust再帰に
変換されていた。型式とalias参照を同じ明示work stackで評価し、active alias集合でcycleを検出する方式へ変更した。externの
function partsとaggregate metadataをsource型から回収する走査も同じく反復化し、aggregateの終端sourceをalias IDごとに
memoizeした。4,096本のalias chainの先にproductとextern functionを置くdebug testは、metadata memoize前の約1.9秒から
約0.06秒となった。これはD045の再帰的なsource構文上限には数えない。

共有されたproduct型を型不一致のdiagnosticへ表示すると、canonical representationはDAGでも従来の再帰的`type_name`が各辺を
展開し、文字列量が指数的に増えた。表示を明示stackへ移し、4,096 byteを超えるcanonical type名をellipsisで省略した。
64段の共有product DAGから作る表示が4,099 byte以下で終了することを回帰テストにした。

同じ形の共有productは表示だけでなく実値のfield数も指数的に増えるため、型検査へ64 nested level、65,536 storage componentの
表現上限を置いた。17段の二重productと、平坦な65本のaliasが構成する深いproductをbackend生成前にdiagnosticとして拒否し、
共有sumは64段でも受理する回帰テストで、物理的な重複とDAG走査上の重複を区別した。判断は
[D046](../decisions/D046.md)に記録する。

## 2026-09-13 decimal float coefficient

decimal exponentが有効桁数を相殺するliteralでは値のdecimal orderが小さくても、全係数と10の冪を多倍長整数へ変換する従来の
処理量が入力桁数に対して二次的に増えた。binary64の隣接値間のmidpointは分母が最大`2^1075`で有限十進展開を持つため、
先頭1,100有効桁と省略部分のnonzero有無があれば全rounding boundaryとの大小を保存できる。多倍長演算前にこの表現へ縮約し、
100,001桁の係数と、midpointから1,200桁先で初めて大きくなる値を正しく丸める回帰テストを置いた。

## 2026-09-13 C representation interning

C `TypeRegistry`はaggregateを追加するたびに既存の全型と構造比較し、型名の解決でもvectorを線形探索していた。postorderで既に
確定した子representation identityからproduct/sum keyを作り、同じkeyを一つのindexへinternする方式へ変更した。これにより
transparent aliasから独立に構成された同型DAGも同じC representationを使い、収集と名前解決は型DAGのnode・edge数に比例する。
独立に構成した64段の同型sum DAG二つが64個のaggregateだけを登録する回帰テストを置いた。

## 2026-09-13 execution identity lookup

tail-forwarder判定はapplication siteごとにcallee functionを線形探索し、common control判定はregion functionごとにentryを
線形探索していた。LLVM frame emissionもsiteごとにframe tagの位置を探索していた。それぞれplan構成時にfunction identity、
function entry、frame siteからtagへのmapを一度作り、site処理を定数時間のlookupへ変更した。semantic validationは従来の
execution optimization、call plan、recursive controlのfocused testで同じdecisionを確認した。

indirect applicationのpossible targetは従来siteごとに全internal functionのparameter/result型を比較していた。canonical type
DAGをmemoizeしながら反復的に構造fingerprintへ変換し、functionをsignatureごとのtarget groupへ一度だけ分類する方式へ変更した。
siteは対応groupだけを引き、fingerprint collision時には構造比較してcorrectnessを保つ。独立なtransparent aliasから作った
同型signature二つが同じindirect target集合へ入る回帰テストを置いた。
