# generated C performance測定履歴

Status: Historical measurement record

この文書はgenerated Cの性能調査方法、測定baseline、結果と原因調査の経緯を記録する。
言語の意味は[`spec/`](../../spec/)、通常の検証commandは[test policy](../../development/testing.md)を正とする。
現在の未解決課題、改善軸、実装順は
[generated program最適化計画](../../development/generated-program-optimization.md)に置く。

wall-clock値はconformanceではなく、同じ環境内で変更前後を比較するための観測値である。時間そのものをCI testへ
固定しない。behavior、generated Cの構造、同一machineでの反復比率を分けて検証する。

## 現在のbaseline

interactiveな053を除くTypical90の79問を、Clang 21.1.8の`-O2`、同じmaximum-order input、warmup 3回、
交互20 roundで比較した。比率は`mal / direct C`とし、1より大きいほどCが速い。

| Population | Count | Median ratio | Geometric mean |
|---|---:|---:|---:|
| 全非interactive問題 | 79 | 1.03 | 1.04 |
| 両実行時間が1 ms以上 | 61 | 1.06 | 1.05 |
| 両実行時間が5 ms以上 | 51 | 1.07 | 1.07 |
| 両実行時間が10 ms以上 | 43 | 1.06 | 1.06 |

±5%を同等とするとmalが速いものは11、同等は34、Cが速いものは34だった。規則的なnumeric/`Ptr`処理は
direct Cと同等以上または近く、新しいcollection primitiveを性能だけのために追加する根拠はない。現在、profileによって
compilerの責務へ分離できたactiveなcost modelはapplication control loweringのtransition costである。

個別caseの数値はcompiler変更の採否ではなく、fixture差、source責務、backend責務を分離する証拠として読む。1 ms未満の
caseと異なるmachine間の絶対時間は順位付けに使わない。

## 測定経路の訂正

最初の集計後、binaryの`.comment`と最適化後assemblyを監査し、mal側はGCC 15.3.0、direct C側はClang 21.1.8で
buildされていたことを確認した。local build runnerが`CC`を固定せず、interactive環境の`CC=gcc`を継承したことが原因だった。

runnerがmal側にもClang 21.1.8を明示するよう訂正し、同じmaximum-order input、warmup 3回、20反復で79問を再測定した。
測定前にmal側の269 sample、C側の269 sample、79個のmaximum-order inputにおけるmal/Cのstdoutを再検証した。
両方を同じClang、`-O2`、public buildのstrict float optionでbuildした。

最初の再測定はHyperfineが一方のbinaryを20回すべて実行してから他方を実行する順序だった。043を含む長時間caseで、先行する
command groupにhost負荷の時間変動が偏ることを確認したため、現在値は1回ずつのHyperfine測定を20 round行い、roundごとに
mal/Cの先行順を反転したものを正とする。Nushellから直接測った経過時間は短時間caseへ数msのrunner costを加えたため採用せず、
process時間は各Hyperfine invocationに測らせる。

実装方式の差をcompiler差へ混ぜないため、比較fixtureは行単位の同形ではなく、各言語で同じ意図を自然に表す実装へ揃えた。
005は3個の作業bufferを再利用し、012と028は入力を保存せず処理し、055は同じinclude/exclude再帰で列挙する。
032は同じ順序で全候補を探索しつつ、mal側はbest値を返し、C側はsearch stateを更新する各言語で自然な形にした。043は
固定容量のhole-based heapとし、005、012、016、023、028、032、055のC側storageとcounterはmal sourceの`Int64`へ合わせた。
027のhost adapterはadmission中のdataとcapacityをbyte loop外で保持し、C側もtokenごとの動的admissionと、その後のrecord処理を
分離した。

mixed-toolchainで得た比率は調査候補の発見にだけ使い、現在値やcompiler改善幅には使わない。source、生成物、Hyperfine JSONなどの
raw artifactはlocalの`.scratch/`に置き、tracked repositoryには含めない。

## application control lowering実装前後

application control lowering実装直前の`a7fc89c`と、direct・indirect・first-class cycleおよびmanaged frame transferまで
実装した`b9cfded`を比較した。両compilerを同じRust 1.97.1でrelease buildし、同じMal sourceからClang 21.1.8 `-O2`と
public buildのstrict float optionでbinaryを生成した。wall-clockはHyperfineでwarmup 3回、交互30 roundとし、各pairの
stdout一致を確認した。比率は`after / before`である。

| Control形状 | Workload | After (ms) | Before (ms) | Ratio |
|---|---|---:|---:|---:|
| direct self non-tail | Hanoi depth 26 | 121.77 | 102.31 | 1.19 |
| direct self non-tail | linear depth 250,000を10回 | 12.04 | 8.58 | 1.40 |
| direct self tail | 1,000,000遷移を50回 | 6.18 | 6.20 | 1.00 |
| acyclic direct helper + self tail | 1,000,000遷移を50回 | 11.98 | 12.00 | 1.00 |
| indirect tail forwarder | 100,000遷移を500回 | 8.92 | 6.23 | 1.43 |
| first-class non-tail cycle | depth 10,000を400回 | 14.16 | 6.21 | 2.28 |
| captured `Symbol` first-class cycle | depth 10,000を400回 | 61.34 | 77.16 | 0.80 |

direct self tailとacyclic direct callは実装前と同等であり、application全体がdispatcherへ変わったわけではない。一方、unmanagedな
explicit frameのpush、resume、dispatchは通常のC recursionより高く、自然なHanoiは手動defunctionalize版の既存77.89 msにも
届かなかった。managed fixtureだけはframe ownerのmoveによって退行せず改善したが、この一例をunmanaged frame costの相殺根拠には
しない。indirect tail fixtureは実装前もClangがC tail callを除去してdepth 1,000,000を完了したため、対応toolchain上では新しい
portableなstack boundと引き換えにtransition costだけが増えた。

深度を増やすと、direct linear depth 1,000,000、first-class non-tail depth 300,000、captured `Symbol` first-class depth 50,000は
afterが完了し、beforeはsignal 11で終了した。したがってC stack boundの目的は達成しているが、throughputの完了条件は満たしていない。

既存79問corpusも同じcommit pair、Clang、input、stdout検査、warmup 3回、交互20 roundで再測定した。

| Population | Count | Median ratio | Geometric mean | After faster / parity / slower |
|---|---:|---:|---:|---:|
| 全非interactive問題 | 79 | 1.00 | 1.03 | 1 / 71 / 7 |
| 両実行時間が5 ms以上 | 52 | 1.00 | 1.03 | 0 / 47 / 5 |
| 両実行時間が10 ms以上 | 44 | 1.00 | 1.04 | 0 / 39 / 5 |

5 ms以上で5%を超えて退行したのは016が1.73倍、032が1.56倍、029が1.31倍、023が1.25倍、055が1.10倍だった。
080もbefore 3.52 msからafter 6.26 msへ1.78倍になった。016はnon-tail linear search、029はsegment treeのbranching recursion、
032、055、080はbranching enumeration、023はprofile pair列挙を含み、いずれもrecursive SCC内のnon-tail edgeがhot pathにある。

最適化後LLVM IRでは、代表的な032の行数が694から976、`call`が53から66、branchが64から89へ増えた。6退行caseのbinary
text sizeも約12%から19%増えた。これは単なる測定揺れではなく、typed frameとdispatcherがoptimizer後にも残ることと整合する。
次の改善ではstack boundを外さず、direct recursive SCCの複数遷移をまとめる表現を独立fixtureと全corpusの両方で評価する。

### control region refinement

`0b80b98`ではpossible call graphのrecursive SCCごとにarenaを分離し、実行中のtopとcurrent frameをC localへ移した。旧実装を
同じtoolchainとoptionで生成したbinaryを`current`、region版を`region`として、focused fixtureをwarmup 3回、交互30 roundで比較した。
比率は`region / current`である。

| Control形状 | Region (ms) | Current (ms) | Ratio |
|---|---:|---:|---:|
| direct self non-tail Hanoi | 118.19 | 133.07 | 0.89 |
| direct self non-tail linear unwind | 9.24 | 12.62 | 0.73 |
| direct self tail | 6.78 | 6.75 | 1.00 |
| acyclic direct helper + self tail | 13.00 | 12.83 | 1.01 |
| indirect tail forwarder | 9.84 | 9.76 | 1.01 |
| first-class non-tail cycle | 14.66 | 14.76 | 0.99 |
| captured `Symbol` first-class cycle | 68.01 | 67.15 | 1.01 |

既存79問もwarmup 3回、交互20 roundでregion版と旧実装を直接比較した。

| Population | Count | Median ratio | Geometric mean | Region faster / parity / slower |
|---|---:|---:|---:|---:|
| 全非interactive問題 | 79 | 1.00 | 0.99 | 10 / 65 / 4 |
| 両実行時間が5 ms以上 | 53 | 1.00 | 0.99 | 6 / 46 / 1 |
| 両実行時間が10 ms以上 | 47 | 0.99 | 0.99 | 6 / 40 / 1 |

元の退行6 caseの比率は016が1.04、023が0.94、029が0.84、032が0.91、055が0.93、080が0.77で、幾何平均は
0.90だった。5 ms以上で唯一1.07となった001はwarmup 5回、交互50 roundの再測定で1.00となった。direct tail、acyclic direct、
managed first-class cycleは同等帯を維持し、通常・sanitizerのmanaged pressure fixtureも完了した。したがってregion storageは採用する。
一方、016と通常のC recursionに対するfocused caseの残差はarenaの局所化では消えない。次段は個別frame fieldの削減ではなく、
explicit transitionとboundedなC call batchingを同じcontinuation対応の下で比較する。

depth 20のHanoiをCallgrind 3.27.1で比較すると、region版は通常C再帰に対してinstruction referenceが0.84、data referenceが
0.80だったが、conditional branchは2.48、branch mispredictionは2.04だった。同じ生成binaryの手動defunctionalize経路に対しては、
region版のinstruction referenceが1.32、data referenceが1.75、conditional branchが1.50、branch mispredictionが3.05である。
nativeのwarmup 3回、交互30 roundではregion版と通常C再帰はHanoiで1.04、linear unwindで1.06だった。

first-class non-tail cycleをdepth 10,000で1回実行したCallgrind比較では、region版は変換前の通常C call経路に対してinstruction
referenceが2.26、data referenceが3.78、conditional branchが5.26だった。Hanoiだけからmemory traffic削減を結論できず、共通machine
ではcode identity選択、entry dispatch、resumeが独立した主要costである。したがって次の実験はframeを省略せず、fuelが残る間だけ
callee workerをC callして既知resumeへ直行し、fuel 0では同じactivationのdispatcherへ戻す。

local direct-self prototypeではcall前に従来frameを作り、pure dispatcherとC-call workerを別functionにした。Hanoiのwarmup 3回、
交互30 roundでは`B = 1`が119.60 ms対119.09 msで1.00、`B = 2`が137.58 ms対118.99 msで1.16だった。dispatcherとworkerを
分けない初期prototypeは`B = 1`でも4.57倍となり、runtimeでfuel 0でも生成functionに再帰edgeがあるだけでoptimizerのloop形成を
妨げた。分離後もcontinuationをarenaとC activationへ常時二重化する利益はなく、このrefinementは棄却した。

常時二重化を避け、fuel切れ時だけC continuationをbounded scratchへspillして逆順にarena化するstandalone C modelも作成した。
scalar引数、同一binary、depth 26、warmup 3回、各30 runではdirect Cが158.23 ms、`B = 8`が187.08 ms、`B = 16`が
183.16 ms、`B = 32`が185.09 msで、比率は順に1.18、1.16、1.17だった。batch幅を増やしてもspill protocolのcostを回収せず、
production emitterとmanaged owner遷移は実装しない。

prototype撤去後に残したregion arenaへのlocal pointer表現も、`0b80b98`生成物との交互30 roundでHanoiが1.01、linear unwindが
1.00であり同等帯だった。

## 個別調査

| Case | 分離した境界 | 現在の判断 |
|---:|---|---|
| 005 | modulo回数 | source fixtureを訂正し、backend変更なし |
| 011 | loop不変値とallocation alias | sourceで不変値を所有し、一律LTOと`noalias`を棄却 |
| 016、032 | known-call product | 最適化後IRでcallが消えるためactive課題ではない |
| 027 | `Symbol` release | 必要なlifetime semanticsを維持し、allocation変更を棄却 |
| 029 | streamingとinteger幅 | source fixtureを訂正し、同幅Cとparity |
| 043 | heap invariantとdirection mapping | source fixtureを訂正し、backend固有rewriteを棄却 |
| 063 | scan条件とarea reduction | source fixtureを訂正し、同形Cとparity |

### 043 — heap invariantとdirection mapping

この時点で絶対差が大きい残差は043の約1.15倍だった。popped distanceをdirection loopへ明示的に渡すsource variantは、
元のmal版と交互測定で同等だった。最適化後IRでも対応するloadはloop invariantになっているため、同じ値のsource-level
引き回しをbackend変更へ一般化しない。

043のheap bubble-upは`index == 0`で停止し、else側で`(index - 1) / 2`を計算していた。実際のheap indexは非負だが、
この条件だけではoptimizerがelse側の負数を除外できず、最適化後IRに4箇所のsigned divisionが残った。停止条件をheapの
不変条件どおり`index <= 0`にすると、sourceのdivisionをshiftへ書き換えなくても4箇所とも`lshr`になった。maximum-order inputの
交互20回比較では元のmal版から約5%短縮し、全corpus roundのdirect C比は約1.15だった。全79問の中央値1.04、幾何平均1.05と
10/32/37の分類は維持された。この改善はsourceが所有するheap invariantの訂正であり、負数を含み得る一般のsigned divisionを
shiftへ置き換えるbackend ruleにはしない。

043の残差について、heapの2-field entry移動を16-byte `memcpy`へ置き換えるとassemblyは4個のscalar load/storeから
2個のvector moveになったが、交互測定は同等だった。row/column stepを二つの算術式へ変えたvariantは約2%退行し、一つの
product resultへまとめたvariantも約1%の範囲だった。見える命令数だけからmemory representationやdirection固有のloweringを
追加せず、次の仮説にはhot instructionを直接示すprofileを要求した。

WSL2ではhardware counterを取得できなかったため、043の両binaryを`cpu-clock`で各10回samplingした。generated側の約99%は
entry body、direct C側の約99%は`main`にあり、距離更新の比較がそれぞれ約47%、壁判定が約11%、heap pop後のstale判定が
約11--13%を占めた。特定helperや防御処理が突出せず、同じ探索責務へcostが分布していた。一方、assemblyではdirect Cだけが
directionからrow/column差分をtable lookupし、mal版は二つの分岐関数を比較列へ展開していた。

言語仕様がarrayの通常表現とする`Ptr`とscalar load/storeを使い、8個のdirection差分を一つのtable ownerへまとめると、
最適化後のhot loopは二つのindexed loadになった。変更前との交互30回比較は中央値で約8.5%短縮し、direct Cとの別の
交互30回比較は約1.04倍、全corpusの交互20 roundでは約1.06倍だった。全corpus中央値は1.03、幾何平均は1.05である。
これはdirection固有のcompiler rewriteではなく、mappingを所有するsource fixtureが意図をdataとして表した訂正とする。
残差は測定揺れを含むparity境界付近にあり、samplingから独立したbackend costを特定できないため043を現在の最適化課題から外す。

### 029 — streamingとinteger幅

029のmal fixtureは全brickのrangeを二つの`Ptr` arrayへ保存してから処理していたが、direct Cは一件ずつ読み、その場で
queryとassignを完了していた。後から参照しない入力履歴を除き、`placeBricks`が一件の入力から更新までを所有するstreaming形へ
直すと、変更前との交互30回比較は約0.99倍で性能上は同等だった。不要な`Input` tail stateとallocationを除くfixture訂正として
採用するが、改善とは数えない。

通常のdirect Cはtree storage、index、counterに32-bit `int`を使うため、streaming mal版との交互30回比較は約1.12倍だった。
同じalgorithmとstreamingを保ったままC側も`int64_t`へ揃えると約1.02倍になった。029の通常corpus比率は狭い型を含む
fixture差として残し、narrow integer specializationやtail aggregateのbackend cost modelへ一般化しない。

### 比較条件の整合 — 011と063

比率では011が1.48倍、063が1.44倍だった。011のindexとcounterをすべて`Int64`相当へ揃えたvariantは約1.57倍で、狭い型は
差の主因ではなかった。063でcounterとstorageを`Int64`へ揃え、`__builtin_popcount`を同じshift-and-count loopへ置き換えると
約1.27倍まで縮んだ。063の元の比率全体をbackend costとは扱わず、残差だけをcontrol flowとstorage表現の調査対象にする。

### 027 — `Symbol` release

027は100,000 tokenに対して約400,000回の`Symbol` release境界を通っていた。releaseをtranslation unit内へinternalizeすると、
optimizerが引数形状とcalling conventionをspecializeできることを最適化後IRで確認した。訂正後のwall-clockは1.23倍である。
所有権移譲後のzero descriptorを含むため最適化余地はあったが、必要なrelease semantics自体は維持している。allocationを無効化した
実験は差を支配せず、外部buffer adoptionやallocator変更の根拠にはならなかった。

### 005 — modulo回数

005ではmal fixtureがcellごとに積と加算結果を別々にmoduloし、direct Cの1回に対し2回のdivisionを実行していた。
両operandがmodulo済みで積と加算が`Int64`範囲内にあることをsourceで保ったまま、合計に対する1回だけへ揃えると
0.99倍になった。typed/aligned accessの診断variantは約1%、host実装を見せるLTO variantは測定上の改善がなかった。
したがってこの差は`Ptr` contractを広げたりbackendがwrap semanticsから演算を除いたりする根拠にはならない。

### 横断仮説 — alignment、LTO、wrap semantics

043でもtyped/aligned accessとLTOの診断variantは改善せず、direct Cへ`-fwrapv`を付けたvariantも通常buildと1.00倍だった。
したがってalignment、host allocationのtranslation unit境界、signed wrap semanticsは現在の主原因候補から外す。

011ではgenerated Cとhost CをLTOしたvariantが通常buildの約0.71倍まで短縮したため、LTOをbuild policyの軸として79問すべてで
通常buildと直接比較した。両方が5 ms以上の52問では中央値、算術平均とも約0.99倍で、043は1.00倍だった。一方059は
50回の再測定でも約1.15倍へ退行した。特定fixtureのtranslation-unit境界を隠すために一律LTOを有効化せず、build policy候補から
外す。extern allocatorがfreshか、異なる呼び出し結果がaliasしないかは現行host contractにないため、011の結果だけから
`malloc`/`noalias`相当の属性も付けない。

011のLTO有無を最終IRとassemblyで比較すると、heap sortのrecord moveとbest値のreductionは同形であり、LTOによる新しい
loop vectorizationもなかった。通常buildのDP loopでは`dp`への条件付きstore後にjob durationを再loadするが、LTO buildでは
host側`calloc`のnoaliasなresultが見えるためdurationとrewardがloop外へhoistされていた。両値をjobごとに一度読み、scalarとして
DP loopへ渡すmal variantは、LTOなしでも同じ再loadを除去した。maximum-order inputの交互20回測定ではdirect C比約0.93となり、
元の約1.48の差は消えた。この結果はrecord表現のbackend specializationではなく、fixtureが実際に知る不変性をsourceで表す
根拠とする。一般のextern resultにfreshnessやnon-aliasを仮定する根拠にはしない。

### signed right shift — backend lowering

signed `>>`のportable C展開は、以前はunsigned logical shiftへsign maskを合成していた。063の最適化後IRでは、定数1のshiftにも
`lshr`、sign-bit抽出、`or`が残っていた。型幅内の補数をlogical shiftして再反転する等価式へ変更すると、Cのsigned shiftへ
依存せず、Clangは単一の`ashr i64`へ縮約した。dynamic shiftを1億回行う独立fixtureでは、交互20回測定の中央値が
158.3 msから146.5 msへ約7%短縮し、assemblyもloop内の分岐とmask合成から` sar`へ変わった。

一方、063全体は変更前後とも約1.44--1.45倍であり、このoperationはworkloadを支配していない。全79問の分布にもmaterialな
変化はなかった。この変更は063向けの局所最適化ではなく、仕様済みarithmetic shiftをoptimizerへ直接見せるinteger loweringの
責務として採用する。

### 063 — scanとarea reduction

063のsame-widthかつportable popcountなdirect Cを100回再測定すると、mal/direct Cの中央値は約1.27だった。direct側も
malと同じ不一致時early exitへ揃えるとdirect C自体がさらに短縮したため、残差をcontrol-flowの不一致だけには帰属できない。
columnごとに一度決まる基準値を`countColumns`で読み、比較だけを担う`sameRows`へscalarで渡すsource variantは、最適化後IRから
row loop内の基準値loadを除いたがwall-clockは同等だった。early exitをやめて一致状態を運ぶfull-scan variantは約27%退行した。
また、subsetごとのclearを直接`memset`へ置き換えたvariantは同等、Int64 accessへalignmentとtyped aliasを仮定したvariantは
約4%改善、さらにLTOを併用しても追加改善はなかった。いずれも単独で残差を説明せず、現行`Ptr` contractを広げる根拠にはしない。

`cpu-clock` samplingでは、063のgenerated entry bodyにあるfrequency reductionが支配的で、最適化後IRは4要素ずつの
`<2 x i64>` load、`llvm.smax`、`llvm.vector.reduce.smax`を生成していた。sourceはsubset内の最大頻度を求めた後に
selected row数を掛けていたが、問題が必要とする値は全subsetを通した最大面積である。reductionの責務を`bestArea`へ戻し、
scan中に`rows * count`と既存bestを比較すると、IRはscalar `smax`のunrolled loopになった。

変更前との交互50回比較は中央値で約41%短縮した。型幅とportable popcountを揃えたfull-scan Cに対して約0.71倍、
不一致時early exitも揃えたCに対して約0.99倍だった。通常の全corpus比較では063はdirect Cの約0.85倍となり、全79問の
中央値は1.03、幾何平均は1.04になった。これはvectorizationを抑制するbackend ruleではなく、中間値ではなく最終判断を
所有するsource fixtureの訂正とする。063にも独立したbackend costが残らないため、現在の最適化課題から外す。

## 採用済みのcompiler改善

2026-09-05の3 workload比較では、未最適化public buildに対してC compilerの`-O2`がbranch-heavy heapを875.2 msから
390.1 msへ、numeric transformを465.9 msから395.1 msへ短縮した。この根拠とstrict float testによりpublic buildへ`-O2`を
採用した。その後Boolの0/1 specializationとproduct parameterのdirect entryを採用し、現在の全corpus baselineへ引き継いだ。

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
short state transitionの`-O2`前後が同程度であることだけから、特定のoptimizationが無効だとは判断しない。

`-O2 -pg`の関数別計測では、hot transitionが約72%、それを呼ぶloop本体が約22%を占め、
inputとallocationはsampling粒度未満だった。Clangのoptimization reportではscalar memory helperとwrap helperはhot functionへ
inlineされており、transition function自体はinline costがthresholdを超えてcall boundaryが残った。memory runtimeはその後、
使用したoffset/load/store helperだけを生成するようにし、strict warning optionと`-O2`を同時に使えることをfocused testで
確認した。

product parameterのdirect entryを導入する前は、local実験でClangのinline thresholdをhot functionのcostより少し上げると
branch-heavy heapが改善した。しかしnested fieldまでdirect entryへ渡す現在の生成物では、同じ方法が通常thresholdより約5%
遅くなった。全direct functionへの強いinline hintはcode duplicationのcostが上回るため採用しない。全direct tail-recursive
functionへの`always_inline`もregular numeric transformを悪化させ、tail pathと別の再帰pathを併せ持つ関数をGCCがcompileできなかった。
現在はclosureとして使われず、leaf数16以下のproduct direct entryだけに弱い`static inline` hintを付けている。

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
| known-call product | aggregate構築とdirect entry | 先行3 workloadでは消えたが全corpusのtail stateには残る | leaf slot化を再検討 |
| 一般sum | tagとpayload union | extern resultではaggregate return、tag branch、invalid-tag pathが残る | public ABIとvariant選択に必要 |
| `Bool` case | `uint8_t`の`switch` | internalな既知値では消え、extern resultではvalidation branchが残る | host contract境界のcheckとして維持 |
| function value | code pointerとenvironment pointer | 動的選択ではpairとindirect callが残る | first-class closureの意味に必要 |
| numeric/memory helper | helper callと`memcpy` | helperはinlineされscalar load/storeになる | wrap、trap、unaligned accessの意味に必要 |
| managed Engram | descriptorのretain/release | 保存されるaggregateやtail stateにはretain/releaseが残る | borrow-preserving loweringを継続 |

先行3 workloadでは最適化後IRからproduct型と`switch`が消えたが、全corpusでは同じ結論を一般化できなかった。C sourceの
aggregate数ではなく、最適化後にも残る個別のretain、aggregate slot、tag、callを判断材料にする。

memory contractの差は別軸として残る。generated scalar accessはalignment 1で、異なる`Ptr`がaliasしないとは仮定できない。
現行仕様のまま`restrict`や強いalignmentを付けるのは誤りであり、managed borrowやaggregate stateの改善と混ぜない。

`Symbol` byte accessとequalityはnon-null `data`を持つ連続値をsmall wrapperで直接処理し、未materialize ropeだけを
no-inline slow pathへ送る。flat scanのdeterministic materialization countはbyte数と同じ26回から0回になり、Clang `-O2`後の
IRではwrapper callが消え、data loadとslow callの分岐に分かれる。byte accessだけが消費する一時productもborrowしたleafを
直接渡すため、同じscanのretain/releaseは26/27回から0/1回になった。保存されるaggregateとtail stateは別のcostとして残る。

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

全corpus再測定後もfirst-class valueとして必要な`large sret`は残るが、既知direct callのhot pathを支配する例はなかった。
新しいprofileで支配的な例を得た場合に限り、独立したfixtureで構造を固定し、closure共通entryをfallbackとして残し、clone数を
制限できることを採用条件として再検討する。

## 測定の再現条件

通常のcompiler検証は[test policy](../../development/testing.md)に従う。

local corpusの具体的な検証commandはraw artifactと同じ場所で管理する。

performance comparisonでは各variantを同じinput、同じstdout検査、同じoptimization optionで準備し、shell起動costを
除いて反復する。pair比較はroundごとに先行順を反転し、各process時間をHyperfineで測る。local corpusでは次の形とする。

```nu
nu .scratch/typical90/performance/benchmark-pair.nu \
    maximum.in current-results.json ./solution ./baseline \
    --warmup 3 --runs 20
```

測定結果を更新するときは、日付、toolchain、workload、warmup/run数、stdout検証の有無を一緒に記録する。

## 維持する制約

- closureとして渡される関数のcalling conventionを、direct callだけの測定から削除しない。
- strict float option、integer wrap helper、仕様が要求するruntime failure処理をbenchmarkのために無効化しない。
- absolute timeを異なるmachine間の合否判定に使わない。
