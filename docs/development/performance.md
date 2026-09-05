# generated C performance評価

Status: Current M7 record

この文書はreference compilerの性能調査方法、2026-09-05時点のbaseline、M7で得た結果と判断を定める。
言語の意味は[`spec/`](../spec/)、active gateと完了条件は
[implementation roadmap](../implementation/roadmap.md)、通常の検証commandは[test policy](testing.md)を正とする。

wall-clock値はconformanceではなく、同じ環境内で変更前後を比較するための観測値である。時間そのものをCI testへ
固定しない。behavior、generated Cの構造、同一machineでの反復比率を分けて検証する。

## 調査の発端

競技programmingのlocal corpusを、indexed storageとalgorithmをmal側に置いて実装した。behavior caseと
maximum-order smokeがpublic `malc build`経路で成功した。新しいcollection primitiveは必要なく、
`Ptr`、byte `offset`、numeric scalar load/storeで次を表現できた。

- CSR、binary lifting、residual network、Union-Find
- monotonic queue、binary heap、subset DP
- convex hull、NTT

この結果はmemory mechanismの不足よりgenerated codeのcostを次に調べる根拠になる。branch-heavyなheap操作と
規則的なnumeric loopはC optimizerへの反応が異なるため、両方を代表workloadにする。

## 2026-09-05 baseline

環境はrepositoryのpinned NixOS development environment、Clang 21.1.8。maximum-order inputをstdinから読み、
Hyperfineをshellなし、2回以上のwarmup、10回以上の反復で実行した。三つの実行形式は次の通り。

| Variant | 内容 |
|---|---|
| `solution-before` | 調査開始時のpublic `malc build`。driverはC optimization optionを渡していなかった |
| `generated-o2` | `malc emit-c`の出力を`clang -std=c11 -O2`でcompile |
| `baseline` | mal版と同じalgorithmの単純なC実装を`clang -O2`でcompile |

三者のstdoutが一致することを確認してから測定した。
対象の3 workloadはfloatを使わない。public buildへ`-O2`を採用する判断には、別途strict float optionを同時に指定した
native testが必要である。

| Workload | Maximum-order shape | `solution-before` | `generated-o2` | `baseline` |
|---|---|---:|---:|---:|
| short DP | monotonic queueを使うbounded DP | 4.3 ms | 4.3 ms | 3.0 ms |
| branch-heavy heap | direction-state shortest pathのlocal maximum-order input | 875.2 ms | 390.1 ms | 238.5 ms |
| numeric transform | NTTのlocal maximum-order input | 465.9 ms | 395.1 ms | 382.1 ms |

short DPは実行時間が短くprocess起動とinputの比率が大きいため、厳密なoptimization gateには使わない。残る二つの
比率を主な比較に使い、絶対時間はmachine間で比較しない。

`-O2` executableのtext sectionはshort DPが6215 bytes対2915 bytes、branch-heavy heapが7523 bytes対3978 bytes、
numeric transformが7866 bytes対4653 bytesで、いずれも左がgenerated C、右がdirect Cである。code sizeだけを
原因とはみなさないが、runtime check、tagged control flow、specialized product typeが残る量の補助指標にはなる。

public `build`へstrict float optionと同時に`-O2`を採用した後、`CC=clang`を明示して同じmaximum-order inputを
warmup 3回、10回反復で再測定した。public build対direct Cの比率はbranch-heavy heapが約1.62、numeric transformが
約1.03だった。絶対時間はmachineの状態で変動したため、初回tableと混ぜず比率だけを現在の比較値とする。

localのsource、input、expected output、direct C、generated C、Hyperfine JSONは`.scratch/`に置き、Git管理しない。
競技programming由来の問題文、固有名、source、sample、input、expected outputをrepositoryへ昇格しない。compiler
regressionとして追跡する必要が生じた場合は、名称、設定、source、入出力を独立に設計したsynthetic programで、
観測したgenerated-C構造だけを再現する。元問題の縮小やdataの置換はsyntheticとはみなさない。

## generated Cで確認済みの事実

self tail callは`mal_tail_entry`と`goto`へlowerされ、C stackを消費する再帰callにはなっていない。regular loopが
`-O2`後にdirect Cの約1.03倍まで近づくことは、このloweringが有効な根拠である。
top-level functionのC宣言・定義にはsource binding名のcommentがあり、numericなinternal symbolとsource上の責務を
対応付けられる。local lambdaはtop-level binding名を持たないため、このcommentの対象外である。

一方、未最適化のgenerated Cには次が明示的に現れる。

- source productに対応するC structと、call siteでのaggregate初期化
- Boolに対応するtagged sum、tag検査の`switch`
- ANFの各primitive stepに対応するtemporary
- `Ptr` accessorを含むsmall static function call

Clang `-O2`はこれらの多くをinline、scalar replacement、dead-code eliminationできる。numeric transformの残差は小さいが、
branch-heavyなheap workloadは
`-O2`後もdirect Cの約1.64倍であり、branch、heap entry、accessorが組み合わさるhot pathにはlowering上の差が残る。
short DPの`-O2`前後が同程度であることだけから、特定のoptimizationが無効だとは判断しない。

`-O2 -pg`の関数別計測では、directionごとのtransitionが約72%、それを呼ぶloop本体が約22%を占め、
inputとallocationはsampling粒度未満だった。Clangのoptimization reportではscalar memory helperとwrap helperはhot functionへ
inlineされており、transition function自体はinline costがthresholdを超えてcall boundaryが残った。memory runtimeはその後、
使用したoffset/load/store helperだけを生成するようにし、strict warning optionと`-O2`を同時に使えることをfocused testで
確認した。

product parameterのdirect entryを導入する前は、local実験でClangのinline thresholdをhot functionのcostより少し上げると
branch-heavy heapが改善した。しかしnested fieldまでdirect entryへ渡す現在の生成物では、同じ方法が通常thresholdより約5%
遅くなった。call boundaryを越すaggregate costが既に減り、code duplicationのcostが上回ったと判断し、inline hintは採用しない。
全direct tail-recursive functionへの`always_inline`もregular numeric transformを悪化させ、tail pathと別の再帰pathを併せ持つ
関数をGCCがcompileできなかった。

primitive比較を直ちに`if`条件として消費する経路は、Boolのtagged sumを作らずCの条件式へ直接loweringするようにした。
focused testではsum valueと`switch`の除去を確認したが、代表workloadの実行時間とbinary sizeに有意な変化はなかった。
さらにC backend全体で`Bool`を0/1の`uint8_t`としてspecializeし、local binding、short-circuit、closure capture、product、
extern ABIを含む経路からpayloadのないsum structを除去した。source-levelでは引き続きtransparentな`[Unit, Unit]`であり、
一般のsum表現は変更しない。同一のClang buildを交互に測定すると、branch-heavy heapは直前のgenerated Cから約13%短縮し、
regular numeric transformはdirect C比約1.01だった。最適化後のtext sizeは前者で不変、後者で約0.2%増であり、改善を
code size削減とは解釈しない。

product parameterを持つfunctionは、fieldを個別に受けるknown-call用direct entryと、aggregate parameterを受ける
function-value用closure entryへ分けた。closure entryはdirect entryへのthunkとして残るため、first-class functionの
calling conventionは変えない。同一測定内でbranch-heavy heapはdirect C比約2.07から約1.28へ改善し、regular numeric
transformは約1.04だった。hot functionはinline cost 385、threshold 225のままcall boundaryが残ったため、前者の改善は
主にaggregateをC call ABIから外した効果と判断する。

direct entryではnested productもleafまで展開すると、最外層だけを展開した版からbranch-heavy heapが同一測定内で約5%短縮し、
regular numeric transformにも退行はなかった。C targetのparameter数へ無制限に依存しないようleaf数は16個までとし、超える
productは従来のaggregate calling conventionへfallbackする。明示的なnested product値とfunction-value用entryは引き続き
元のproduct表現を使う。

## C表現の横断監査

current generated C、Clang `-O2`後のLLVM IR、extern境界と動的closureを個別に含む独立設計のsynthetic programを比較した。
source、生成物、計測dataは`.scratch/`だけに置いた。

| 対象 | C source上の表現 | `-O2`後の結果 | 判断 |
|---|---|---|---|
| ANF binding | local variableと`(void)` | dead valueとcopyは除去 | 読みやすさ上は冗長だがhot-path costではない |
| `Unit` | 1-byte struct | parameter/resultが不要なら除去され、store helper resultは`void`化 | scalar化の性能根拠なし |
| `Ptr` | addressだけを持つstruct | function parameterはLLVM `ptr`、accessorはinline | struct自体の性能根拠なし |
| known-call product | aggregate構築とdirect entry | 代表hot pathからproduct型が消える | 現在のfield direct entryで対処済み |
| 一般sum | tagとpayload union | extern resultではaggregate return、tag branch、invalid-tag pathが残る | public ABIとvariant選択に必要 |
| `Bool` case | `uint8_t`の`switch` | internalな既知値では消え、extern resultではvalidation branchが残る | host contract境界のcheckとして維持 |
| function value | code pointerとenvironment pointer | 動的選択ではpairとindirect callが残る | first-class closureの意味に必要 |
| numeric/memory helper | helper callと`memcpy` | helperはinlineされscalar load/storeになる | wrap、trap、unaligned accessの意味に必要 |
| `String`とruntime arena | descriptor、copy、allocation list | 使用経路またはpublic runtime symbolとして残る | lifetimeとhost ABIのcontractに必要 |

代表generated Cではproduct型名が243箇所、unused warning抑制が424箇所、`switch`が7箇所あったが、最適化後のIRでは
product型と`switch`は0箇所、stack allocationは`MalContext`用の1箇所だった。C sourceの大きさをそのまま実行時costと
みなせないことを確認した。

残った差で目立つのは意味論の過剰なmaterializationではなく、memory contractの違いである。generated codeのscalar accessは
alignment 1で、異なる`Ptr`がaliasしないとは仮定できない。direct Cはtyped、aligned storageとallocation由来のalias情報を
optimizerへ渡せる。malの`Ptr`はunaligned accessとaliasを許し、externが返すregion間の非alias性を規定しないため、現行仕様の
まま`restrict`や強いalignmentを付けるのは誤りである。

したがって現在のbranch-heavy workloadについて、`Unit`、`Ptr`、一般sum、closureを一律scalar化する次の変更は行わない。
次に調査する場合は、最適化後にも残る個別のproduct resultまたは間接callをsynthetic programで再現できた場合に限る。
memory側を進めるなら、まずalias/alignmentを表現する新しいlanguage/extern contractが必要かを仕様変更として判断し、C emitter
だけで事実を仮定しない。

### 間接callとproduct result

代表workloadの未最適化Cにはprogram entryの間接callと、productを返すknown callが各1箇所ある。`-O2`後はentryが
main functionへのdirect callになり、productを返すcalleeもcallerへinlineされるため、どちらの境界も残らない。独立した
synthetic programでも、top-level functionのlocal aliasと、直前に構築して呼ぶcapturing closureはcode pointer、environment、
environment allocationを含めて除去された。

runtimeの条件で複数のfunction valueから選択するsynthetic programでは、最適化後もcode/environment pairと間接callが残った。
このcallをdirect化するには、選択分岐の各armへcallを複製するか、function argumentを受けるcalleeを候補ごとにcloneする
defunctionalizationが必要になる。現在のIRは候補集合、call frequency、specialization budgetを持たず、代表workloadのhot pathにも
間接callはない。一律の分岐複製はcode sizeとinstruction cacheを悪化させ得るため採用しない。

product resultはtarget C ABIへ委ねた場合、小さい2-scalar productはcalleeをinlineしない設定でもLLVM上の2 register valueになった。
一方、inline thresholdを超える12-scalar productは96-byteのcaller-owned `sret`領域へcalleeが直接書き込んだ。後者へfield別の
out parameter entryを追加しても、全fieldを使うcallではwriteを減らせない。使用fieldだけを返すspecializationはcallee内のeffectを
維持したresult-use analysisとfunction cloningを必要とし、単なるproduct ABIの改善ではない。

将来これらを再検討する条件は、最適化後のprofileで間接callまたはlarge `sret`がhotであること、独立設計のsynthetic regressionで
構造を固定できること、closure共通entryをfallbackとして残しつつclone数を制限できることの三点とする。それまではoptimizerが
既に除去するalias追跡を中間表現へ追加せず、product result用entryも増やさない。

## 検証

通常のcompiler変更はrepository rootからpinned environmentで次を実行する。

```nu
cargo fmt --manifest-path compiler/Cargo.toml --check
cargo clippy --manifest-path compiler/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path compiler/Cargo.toml
```

local corpusの検証commandは`.scratch/`内のREADMEを正とし、tracked documentから特定のcontestや問題に依存するpathを
contractにしない。

performance comparisonでは各variantを同じinput、同じstdout検査、同じoptimization optionで準備し、shell起動costを
除いて反復する。例は次の形とする。

```nu
(hyperfine -N --warmup 3 --runs 10
    --input maximum.in
    ./solution
    ./generated-o2
    ./baseline)
```

測定結果を更新するときは、日付、toolchain、workload、warmup/run数、stdout検証の有無を一緒に記録する。

## M7で行わないこと

- performanceだけを理由にarray、collection、loop、moduleを言語へ追加しない。
- closureとして渡される関数のcalling conventionを、direct callだけの測定から削除しない。
- strict float option、integer wrap helper、trap checkをbenchmarkのために無効化しない。
- absolute timeを異なるmachine間の合否判定に使わない。
