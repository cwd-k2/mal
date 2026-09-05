# generated C performance評価

Status: Current M7 handoff

この文書はreference compilerの性能調査方法、2026-09-05時点のbaseline、M7で次に確認する順序を定める。
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

`-O2 -pg`の関数別計測では、directionごとのrelaxationが約79%、それを呼ぶshortest-path loop本体が約19%を占め、
inputとallocationはsampling粒度未満だった。Clangのoptimization reportではscalar memory helperとwrap helperはhot functionへ
inlineされており、relaxation function自体はinline costがthresholdを超えてcall boundaryが残った。memory runtimeはその後、
使用したoffset/load/store helperだけを生成するようにし、strict warning optionと`-O2`を同時に使えることをfocused testで
確認した。

local実験でClangのinline thresholdをhot functionのcostより少し上げると、branch-heavy heapはdirect C比約1.15まで
改善したため、このcall boundaryの除去には実益がある。一方、全direct tail-recursive functionへの`always_inline`はregular
numeric transformをdirect C比約1.16へ悪化させ、tail pathと別の再帰pathを併せ持つ関数をGCCがcompileできなかったため
採用しない。問題固有のinline指定も行わず、まずBool control flowなどcallee自体のgenerated-C costを減らす。

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

## 次の担当者が行う順序

### 1. local fixtureを固定する

branch-heavy heapとnumeric transformのalgorithm、input generator、direct C baseline、stdout検査を`.scratch/`内の
local fixtureとして保つ。benchmark commandは通常のtestから分離し、maximum-order測定は明示的なcommandで起動する。
tracked testへ移すのは、特定の問題に由来しない最小のsynthetic regressionだけとする。

### 2. branch-heavy heap workloadをprofileする

まず現在の`generated-o2`をsampling profilerとcompiler optimization reportで調べる。少なくとも次を分離する。

- inputとallocationに費やす時間
- heap `push` / `pop`
- directionごとのrelaxation
- `loadInt64` / `storeInt64`相当のhelper
- integer wrap helperとBool tag branch

generated functionが番号だけで追跡しにくい場合は、optimizationより先にtop-level binding名をC名またはcommentへ残す。
profiling結果を得る前にBoolやproductの大規模な表現変更を始めない。

### 3. public buildの`-O2`を検証する

`compiler/src/driver.rs`が渡すstrict float optionを削らず、`-O2`を加えた実験を行う。Cのundefined behaviorを利用して
速くなった結果は受け入れない。全numeric wrap、division、shift、float rounding、trap、ABI native testを通した後、
既定buildへ採用するか決定する。採用時は[compiler usage](compiler-usage.md)とdriver testを同じ変更で更新する。

### 4. loweringを一項目ずつ改善する

profileが支持する場合、次の順で小さく検討する。

1. Boolをcontrol flowとして消費するだけの経路で、sum valueと`switch`のmaterializationを避ける。
2. immutable top-level/self callとして既知のcallで、argument productの一時値を避ける。
3. closure共通calling conventionを保ったまま、direct entryとfunction-value用thunkを分離する。
4. small scalar memory helperが`-O2`後にも残る場合だけ、inlineしやすいemissionへ変える。

各項目は`core`、`ANF`、closure conversion、C emitterのどこがその表現を所有するかを
[responsibilities](../implementation/responsibilities.md)に従って決める。C emitterで前段の意味を再解析する形にしない。

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
