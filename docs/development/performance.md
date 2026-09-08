# generated C performance評価

Status: Current measurement record

この文書はgenerated Cの性能調査方法、現在のbaseline、得られた結果と採否判断を定める。
言語の意味は[`spec/`](../spec/)、通常の検証commandは[test policy](testing.md)を正とする。
改善軸と実装順は[generated program最適化計画](generated-program-optimization.md)に置く。

wall-clock値はconformanceではなく、同じ環境内で変更前後を比較するための観測値である。時間そのものをCI testへ
固定しない。behavior、generated Cの構造、同一machineでの反復比率を分けて検証する。

## 調査の発端

localのalgorithm corpusを、indexed storageとalgorithmをmal側に置いて実装した。behavior caseと
maximum-order smokeがpublic `malc build`経路で成功した。新しいcollection primitiveは必要なく、`Ptr`、byte offset operator、
numeric scalar load/storeでgraph storage、priority queue、state transition、規則的なnumeric transformを表現できた。

この結果はmemory mechanismの不足よりgenerated codeのcostを次に調べる根拠になる。branch-heavyなheap操作と
規則的なnumeric loopはC optimizerへの反応が異なるため、両方を代表workloadにする。

## 2026-09-07全corpus baseline

interactiveな053を除くTypical90の79問について、mal版と同じalgorithmのhandwritten Cを用意した。Clang 21.1.8の`-O2`、
同じmaximum-order input、shellなし、warmup 3回、20反復を標準条件とした。C側の269 sampleと、79個のmaximum-order inputに
おけるmal/Cのstdoutを測定前に検証した。複数解を許すsampleは意味を検査した。

比率は`mal / direct C`とし、1より大きいほどCが速い。

| Population | Count | Median ratio | Geometric mean |
|---|---:|---:|---:|
| 全非interactive問題 | 79 | 1.11 | 1.19 |
| 両実行時間が1 ms以上 | 62 | 1.16 | 1.17 |
| 両実行時間が5 ms以上 | 51 | 1.20 | 1.23 |
| 両実行時間が10 ms以上 | 40 | 1.19 | 1.26 |

±5%を同等とするとmalが速いものは12、同等は19、Cが速いものは48だった。1 ms未満はprocess起動の比率が大きいため、
optimizationの順位には使わない。

最大の差は006の7.90倍、008の6.34倍、027の5.25倍、016の2.76倍だった。006、008、027ではflat `Symbol`のbyte scanに
materialization判定とmanaged aggregateのretain/releaseが残る。027では100,000 tokenをhost scratch bufferへ読み、別の
mal-controlled allocationへadmitしてから固定長recordへcopyする。016、029、032、043ではproduct parameterをflattenした
direct entryの内側またはtail edgeでaggregate stateが再構築される。

最適化後LLVM IRでも027のbyte loopに`Symbol` descriptorの`memcpy`、reference count更新、release、rope判定が残った。
したがって以前の3 workloadでは消えていたaggregateとmanaged bookkeepingを、全corpusの新しい再現例に基づいて再検討する。
実装対象とnegative caseは[最適化計画](generated-program-optimization.md)を正とする。

一方、030のsieve、045のsubset DP、065のNTTはdirect Cと同等以上または近い。規則的なnumeric/`Ptr`処理の結果は、新しい
collection primitiveを性能だけのために追加する根拠にならない。

source、input、expected output、direct C、generated C、Hyperfine JSONなどのraw artifactはlocalの`.scratch/`に置き、
tracked repositoryには含めない。

## 2026-09-08最適化後の再測定

同じClang 21.1.8、maximum-order input、warmup 3回、20反復で、79問を現在のcompilerから再生成した。
測定前にmal側の269 sample、C側の269 sample、79個のmaximum-order inputにおけるmal/Cのstdoutを再検証した。
両方を`-O2`とpublic buildのstrict float optionでbuildした。

実装方式の差をcompiler差へ混ぜないため、比較fixtureは行単位の同形ではなく、各言語で同じ意図を自然に表す実装へ揃えた。
005は3個の作業bufferを再利用し、012と028は入力を保存せず処理し、055は同じinclude/exclude再帰で列挙する。
032は同じ順序で全候補を探索しつつ、mal側はbest値を返し、C側はsearch stateを更新する各言語で自然な形にした。043は
固定容量のhole-based heapとし、005、012、016、023、028、032、055のC側storageとcounterはmal sourceの`Int64`へ合わせた。
027のhost adapterはadmission中のdataとcapacityをbyte loop外で保持し、C側もtokenごとの動的admissionと、その後のrecord処理を
分離した。

| Population | Count | Median ratio | Geometric mean |
|---|---:|---:|---:|
| 全非interactive問題 | 79 | 1.07 | 1.06 |
| 両実行時間が1 ms以上 | 60 | 1.13 | 1.07 |
| 両実行時間が5 ms以上 | 51 | 1.15 | 1.12 |
| 両実行時間が10 ms以上 | 39 | 1.14 | 1.11 |

±5%を同等とするとmalが速いものは16、同等は18、Cが速いものは45だった。1 ms未満の分類数はprocess起動の揺れを
含むため、初回測定との増減をoptimization効果として扱わない。全体の幾何平均は1.19から1.06へ、5 ms以上は1.23から
1.12へ、10 ms以上は1.26から1.11へ縮小した。

borrowed direct entryと不変tail slotからのborrowにより、006は7.90倍から1.20倍、008は6.34倍から1.07倍へ縮小した。
borrow導入前のgenerated Cにあったloop内の`Symbol` retain/releaseは消え、owned tail slot自身の終了時releaseだけが残る。borrowed parameterを
resultへ保存する経路ではcopyを維持する。

最終的な主な残差は016の2.17倍、005の1.59倍、032の1.43倍である。043は意図を揃えたheap実装で1.21倍、027は1.16倍まで
縮小した。016ではclosureとして使われないtop-level functionのgeneric representationを省略し、boundedなdirect entryだけへ
弱いinline hintを付けると改善した。一方、generic entryの省略だけでは変化せず、最終binaryには非tail再帰の内側helper callが残り、
direct Cでは対応するreductionがloopへ変換される。残差の本体はclosure表現ではなく、このrecursive reductionの最適化差である。

027は100,000 tokenに対して約400,000回の`Symbol` release境界を通っていた。releaseをtranslation unit内へinternalizeすると、
optimizerが引数形状とcalling conventionをspecializeでき、admission accessorのloop外保持と合わせて1.16倍になった。
所有権移譲後のzero descriptorを含むため最適化余地はあったが、必要なrelease semantics自体は維持している。allocationを無効化した
実験は差を支配せず、外部buffer adoptionやallocator変更の根拠にはならなかった。

032は同じ探索を各言語で自然に記述しても1.43倍で、scalar stateを渡すcall topologyが残る。005は同じbuffer再利用と`Int64`で
1.59倍であり、unalignedかつalias可能な`Ptr` access、明示的なwrap/trap、helper control flowを分離して調べる必要がある。現行
contractから`restrict`、強いalignment、narrow integerを推測して差を隠さない。

## 先行baselineから採用した改善

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

通常のcompiler検証は[test policy](testing.md)に従う。

local corpusの具体的な検証commandはraw artifactと同じ場所で管理する。

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

## 維持する制約

- closureとして渡される関数のcalling conventionを、direct callだけの測定から削除しない。
- strict float option、integer wrap helper、仕様が要求するruntime failure処理をbenchmarkのために無効化しない。
- absolute timeを異なるmachine間の合否判定に使わない。
