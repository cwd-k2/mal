# LLVM backend performance測定履歴

Status: Historical record

この文書はLLVM backendとchecked-in C11 runtimeを分離した後のpublic buildについて、LTO採用時の測定と判断を保存する。
現在の採用条件は[generated program最適化policy](../../development/generated-program-optimization.md)、責務境界は
[実行backend](../../design/execution-backend.md)を正とする。

## 2026-09-10 — public buildへのLTO採用

測定対象は`65310cb`を基点とするLLVM backendと、この変更で追加した`-flto`である。pinned environmentの
Clang 21.1.8、Hyperfine 1.20.0、Nushell 0.114.1を使った。local algorithm corpusのうち、interactive caseとplatform `libm`を
linker inputに要する2 caseを除く77 caseを対象とした。各Mal programはpublic `malc build`、direct C baselineは同じ
Clangの`-O2`とstrict floating-point optionでbuildした。

全263 sampleのstdoutと77 maximum-order inputのMal/direct C間stdoutが一致した後、各組を3回warmupし、実行順を交互に変えて
20回測定した。比率は`Mal / direct C`であり、1より大きければdirect Cが速い。

| Population | LTOなし median / geometric mean | LTOあり median / geometric mean |
|:---|---:|---:|
| 全77問 | 1.07x / 1.14x | 1.03x / 1.06x |
| 両方1 ms以上、61問 | 1.10x / 1.13x | 1.06x / 1.06x |
| 両方5 ms以上、50問 | 1.15x / 1.15x | 1.07x / 1.08x |

LTOありの全体分類はMalが速い8問、±5%以内35問、direct Cが速い34問だった。sub-millisecond caseはprocess起動時間の影響が
大きいため、採否の主根拠にはしていない。

LTO有無を同じMal program同士で交互に測った診断caseでは、LTOあり/なしのmedian比が016で0.94、029で0.73、080で0.71だった。
LLVM moduleとC runtimeを別translation unitのままcompileすると、generated frame pushから
`mal_control_reserve_frame`を毎回callし、capacity内に収まるfast pathもcall境界を越える。LTOはこの境界をinlineできた。
program固有のframe layoutと遷移をLLVMに、program非依存のstorage growthをCに置くsource責務は変更していない。

第二指標としてLTO後のexecutable file sizeを集計した。77問合計はMal 1,268,448 bytes、direct C 1,239,064 bytesで1.02倍、
個別比率の中央値は1.01倍だった。速度改善と引き換えの大きなartifact肥大は観測していない。

以上から、`-flto`をpublic buildの標準optionとして採用する。LTOは責務境界をまたぐhelperの最適化手段であり、正しさ、ABI、
bounded native stackの前提にはしない。なお、この時点の`runtime/c11/symbol.c`はreference-counted flat storageであり、rope実装は
含まない。Symbol連結の表現変更はこのLTO判断とは別に測定、設計する。

raw measurementはcorpusと同じignored scratch treeに保存した。

## 2026-09-10 — external library入力を含む全79問

repeatableな`--clang-arg`を追加した変更で009と018へ`-lm`を渡し、全269 sampleと79 maximum-order inputのstdoutを検証した。
同じ3 warmup、交互20回で追加測定した009はMal 84.86 ms、direct C 72.58 msで1.17倍、018はともに0.90 msで同等だった。
既存77問と合わせた中央値は1.03倍、幾何平均は1.06倍で、LTO採用判断は変わらない。両方5 ms以上の51問では中央値1.07倍、
幾何平均1.08倍だった。

## 2026-09-10 — Symbol ropeとdead ownerのconsuming concat

flat runtimeの`883ac71`、persistent ropeを復元した`97f224d`、control CFGのbackward livenessからdead ownerをmoveする
consuming concatを加えた`55633f9`を比較した。環境はpinned Clang 21.1.8、Hyperfine 1.20.0、Nushell 0.114.1である。
workloadはtail loopで1 byteの`Symbol`を末尾へ反復連結し、最後にlengthを観測する。同じstdoutとexit statusを確認してから、
次のcommandで3 warmup、20 runを測定した。

```nu
hyperfine --shell=none --warmup 3 --runs 20 --export-json /tmp/mal-symbol-consuming.json /tmp/mal-unwind-inspect/concat-80000 /tmp/mal-unwind-inspect/consume-80000 /tmp/mal-unwind-inspect/consume-1000000 /tmp/mal-unwind-inspect/consume-2000000
```

| Representation | concat回数 | median | range |
|---|---:|---:|---:|
| flat copy (`883ac71`) | 80,000 | 77.23 ms | 74.96–125.98 ms |
| consuming flat/rope (`55633f9`) | 80,000 | 0.82 ms | 0.73–1.69 ms |
| consuming flat/rope (`55633f9`) | 1,000,000 | 4.12 ms | 3.84–4.97 ms |
| consuming flat/rope (`55633f9`) | 2,000,000 | 7.69 ms | 7.05–11.24 ms |

wall-clockが5 ms以上の2,000,000回caseを含め、反復数に対してほぼ線形に増えた。第二指標はlinkerの
`--wrap=malloc`、`--wrap=realloc`、`--wrap=free`で取得した。20,000回のappendと20,000回のprependを別々に構築し、
異なるtree shapeのequality、末尾access、host materializationを行う同一workloadでは、`97f224d`が554,409 allocation、
`55633f9`が24 allocationだった。normal return後のlive allocationはいずれも0であり、現行regressionは同じworkloadを
32 allocation以下かつlive allocation 0に固定する。

ropeはshared concatをpath copyで平衡化し、length、byte access、equality、`Symbol.write`ではmaterializeしない。LLVM側は
CFG livenessをauthorityとしてbinding直後のdead shareをreleaseし、dead concat operandだけをC runtimeへmoveする。runtimeは
渡されたownerが一意なflat storageなら幾何的にreserveして再利用し、LLVMからmoveされていないownerをreference countだけで消費しない。
この責務分離でbyte-wise semanticsとhost ABIを変えず、rope復元とconsuming concatを採用する。

### prepend capacity

`55633f9`のflat storageは末尾capacityだけを持ち、一意なright operandへprependするときも既存bytesを毎回`memmove`していた。
`7fceba2`でcapacity内の開始offsetをruntime private representationへ加え、前後どちらの余白も幾何的に確保した。同じ条件で
1 byteを先頭へ反復連結するworkloadを測定した。

```nu
hyperfine --shell=none --warmup 3 --runs 20 --export-json /tmp/mal-symbol-prepending.json /tmp/mal-unwind-inspect/prepend-80000 /tmp/mal-unwind-inspect/prepend-offset-80000 /tmp/mal-unwind-inspect/prepend-offset-1000000 /tmp/mal-unwind-inspect/prepend-offset-2000000
```

| Representation | prepend回数 | median | range |
|---|---:|---:|---:|
| 末尾capacityのみ (`55633f9`) | 80,000 | 41.89 ms | 40.23–85.03 ms |
| 開始offsetあり (`7fceba2`) | 80,000 | 0.90 ms | 0.76–1.09 ms |
| 開始offsetあり (`7fceba2`) | 1,000,000 | 4.49 ms | 4.22–6.70 ms |
| 開始offsetあり (`7fceba2`) | 2,000,000 | 8.21 ms | 7.80–11.41 ms |

2,000,000回caseを含めappendと同じほぼ線形の増加になった。開始offsetはruntime private storageだけのmechanismであり、
LLVM owner move plan、`Symbol`の値、C host descriptorは変更しないため採用する。

## 2026-09-17 — v0.6 API移行後の全79問再監査

typed `Region`、`Address`、`Cursor`を使うv0.6 APIへ全caseを移行した後、同じalgorithmのdirect C baselineを全79問について
再検証した。Clang 21.1.8、Hyperfine 1.20.0、malc 0.6.0-dev、Nushell 0.114.1を使い、全269 sampleと79個の
maximum-order inputでstdoutの一致を確認した。各組は3回warmup後、実行順を交互に変えて20回測定した。

| Population | Count | Median ratio | Geometric mean |
|:---|---:|---:|---:|
| 全非interactive問題 | 79 | 1.08x | 1.14x |
| 両実装1 ms以上 | 59 | 1.12x | 1.16x |
| 両実装5 ms以上 | 52 | 1.12x | 1.16x |
| 両実装10 ms以上 | 39 | 1.12x | 1.17x |

±5%を同等とするとMalが速い10問、同等24問、direct Cが速い45問だった。各問題のmedian分布はMalが
min 0.392 ms、mean 53.891 ms、median 11.087 ms、p95 285.524 ms、max 1052.137 ms、direct Cが
min 0.392 ms、mean 45.188 ms、median 9.383 ms、p95 271.508 ms、max 813.832 msだった。

最大の相対差は032の3.12倍、最大の絶対時間caseは023のMal 1052.137 msに対してdirect C 813.832 msだった。一方、
056は0.61倍、045は0.67倍でMalが速く、047は1.00倍だった。この分布は特定のcollection primitive追加を正当化せず、
現行のLTO採用条件も変更しない。個別のmin、mean、median、p95、maxと全raw sampleはignored scratch corpusに保存した。

## 2026-09-17 — identity continuation正規化

`b6b05f7`で、call結果をaliasとjoinだけでfunction resultへ転送するidentity continuationを`control` stageでtail callへ
正規化した。この変換はexample固有の探索形や再帰深度を使わず、effectを持たず結果をそのまま転送するcontrol graphだけから導出する。
032では不要な16-byte frameが消え、残るframe constructorが一種類になったためtagとfooterも不要となり、frame sizeは96 bytesから
80 bytesになった。

同じClang 21.1.8、Hyperfine 1.20.0、malc 0.6.0-dev、Nushell 0.114.1で全269 sampleと79 maximum-order inputを
再検証し、3 warmup、交互20回で全79問を再測定した。

| Population | Count | 正規化前 median / geometric mean | 正規化後 median / geometric mean |
|:---|---:|---:|---:|
| 全非interactive問題 | 79 | 1.08x / 1.14x | 1.03x / 1.04x |
| 両実装1 ms以上 | 59 | 1.12x / 1.16x | 1.06x / 1.06x |
| 両実装10 ms以上 | 39 | 1.12x / 1.17x | 1.06x / 1.07x |

032はMal 117.668 ms / direct C 37.705 msの3.12倍から、61.252 ms / 37.904 msの1.62倍になった。005は
2.26倍から0.99倍、006は2.03倍から1.07倍、056は0.61倍から0.31倍になった。全体分類はMalが速い10問、±5%以内33問、
direct Cが速い36問である。

隣接するframe pop/pushをLLVM emission前に同じarena slotへ融合する案も、managed ownerを含むcaseで意味を保持した上で測定した。
22段の二分再帰を50回測定したmedian差はnoise範囲内で、最終executableは両者ともtext 3,742 bytes、disassembly 529行だった。
差は独立loadの順序だけで、Clangが既に同じstorage遷移へ縮約していた。通常の局所変換をcompilerへ重複実装しないpolicyに従い、
このtechniqueは採用しなかった。

## 2026-09-18 — 退役continuation frame容量の再利用

後続変更後の055と080では、最初のrecursive resultをresumeしたpathだけが第2のnon-tail callへ到達し、第2 frameは退役した第1 frameより
小さい。一方、032のchild callはfunction入口からも到達する。`execution/frame`で通常入口とresumeを区別するmust-dataflowを構成し、
全到達pathが同じ退役frameを持つsiteだけについて、target layout上で収まるframeを同じtop位置へ書く正規形にした。managed fieldを
含むframeでもownerはresume時にlocal slotへ移管済みであり、退役bytesにはresponsibilityが残らない。途中のnative callによるarenaの
再配置を許すため、書込み前にはstorage pointerを再取得する。

同一のClang 21.1.8、production profile、maximum-order inputで3回warmup後に20回交互測定した。055の変更前後比較ではmedianが
463.280 msから453.414 msへ2.1%短縮した。080は5 ms付近で採否の根拠にせず、適用されない032はIRとtext sizeが不変だった。

| 問題 | Mal min / mean / median / p95 / max (ms) | direct C min / mean / median / p95 / max (ms) | Mal / C median |
|:---|:---|:---|---:|
| 032 | 86.045 / 93.133 / 91.048 / 105.615 / 106.543 | 51.652 / 58.855 / 57.135 / 73.097 / 84.848 | 1.59x |
| 055 | 447.459 / 480.817 / 454.778 / 613.212 / 625.213 | 463.215 / 503.520 / 468.126 / 642.592 / 671.597 | 0.97x |
| 080 | 4.453 / 4.976 / 4.811 / 6.007 / 7.270 | 2.869 / 3.414 / 3.075 / 4.929 / 6.198 | 1.56x |

055のtext sizeは3,950 bytesから3,806 bytes、disassemblyは522行から481行へ減った。080は3,838 bytesから3,758 bytes、
032は4,254 bytesのままである。各20個の値は
`.scratch/typical90/performance/{032,055,080}/frame-replacement-vs-c.json`、変更前後の値は同directoryの
`frame-replacement-before-after.json`に保存した。sample、maximum-order input、managed frameのruntime fixtureで結果とowner lifetimeを確認した。

## 2026-09-18 — control storage fast pathの確定inline

program固有のframe layoutとtop offsetを持つLLVM側に対し、`mal_control_storage`とcapacity内の
`mal_control_reserve_frame`は単なるarena field accessである。一方、capacity growthはprogram非依存のruntime責務である。この境界に従い、
field accessとcapacity判定を`always_inline`、growth loopをoptimizer判断のinternal helperとした。これにより大きなcontrol functionでLTOの
cost modelがruntime callを残してもfast pathは失われず、既にLTOが全体をinlineできるprogramの生成物は変わらない。

同じcurrent compilerで再構築した029では、Packed版のmedianが218.845 msから181.925 msへ16.9%短縮した。Region版は
146.314 msから145.825 msで同等、text sizeも4,663 bytesで不変だった。既存のframe-heavyな032、055、080は変更前後でtext sizeが
それぞれ4,254、3,806、3,758 bytesのまま一致し、median差も±1%内だった。029 Packedのtext sizeは9,225 bytesから9,169 bytesへ減った。
raw sampleはignored scratchの029にある`control-auto-before-after.json`と
`region-control-auto-before-after.json`、および032、055、080にある
`control-inline-before-after.json`へ保存した。

## 2026-09-18 — Packed unique appendとcompact capability environment

`pack`のbuilderは一意なので、capacity内appendはprogram固有のelement strideとvalueを使う通常経路であり、allocationとgrowthだけが
program非依存のruntime mechanismである。unique appendへstrideを明示して通常経路を`always_inline`とし、growthを独立した
`noinline` helperへ分けた。capacityの二倍化loopは`__builtin_clzll`による次の二冪の計算へ置き換え、LTOによる全target幅分のloop展開を除いた。
また、closed programで唯一のscoped inhabitantを持つfunction型はlifetime dispatchを必要としないため、compact capability representationから
environment tagと利用時の`ptrmask`を除いた。ordinary functionと型を共有してcompactにできないcapabilityは従来のtagを保持する。

029 maximum inputの交互30回測定では、Packedのmedianは267.011 msから247.574 msへ7.3%短縮した。幅500,000、query 0の
初期化単独では交互20回のmedianが22.075 msから15.457 msへ30.0%短縮した。変更後のPackedとcurrent Regionの交互30回比較は
190.183 ms対163.917 msで1.16xだった。Packed executableのtext sizeは9,169 bytesから6,789 bytesへ縮小した。
growthをoptimizer判断でinlineした診断版は6,885 bytesで、実行時間に有意な改善がなかったため、責務境界と小さい生成物が一致する
分離版を採択した。raw sampleはignored scratchの029にある`packed-fast-append-before-after.json`、
`packed-fast-append-initialization.json`、`packed-fast-append-vs-region.json`へ保存した。

## 2026-09-18 — byte view authorityとvacant carrier初期化

`5a496d0`でLLVM内の`Symbol`と`Packed<A>`をowner、byte offset、countではなく、owner、active data address、countのviewへ変更した。
ownerはlifetime、dataは現在の観測範囲を支配し、index、slice、変換、比較はowner representationを再解釈しない。concatと`edit`の
storage再利用境界だけがdataからowner-relative offsetを導出する。同時にownership planのpattern destinationを`Store`から
`Initialize`へ改めた。control bindingのcarrierは初回にvacantであり、self-tailで再利用する前にも旧responsibilityは`Consume`または
edge `Drop`で終了するため、backendが格納時に旧ownerを推測してreleaseする必要はない。

maximum-order inputを3 warmup、交互20回で測定した。active data addressへの変更は001を20%短縮し、その後のvacant carrier初期化は
さらに35%短縮した。最終のPacked/Region medianは001が7.171/7.023 ms、007が44.810/42.435 ms、010が
17.182/17.220 ms、050が2.178/2.052 ms、078が14.364/13.373 msだった。029は193.088/166.949 msの1.16倍で
変わらず、immutable indexingの残差とscoped mutationの残差を分離できた。raw sampleはignored scratchの各問題にある
`active-data-before-after.json`、`vacant-carrier-before-after.json`、`vacant-carrier-vs-region.json`へ保存した。

tree corpusとして003のimmutable raw edges、offsets、neighborsをPacked、BFS queueとdistanceだけをRegionにした。helperへ不要な
Graph全体を渡す版はPacked/Region 1.66倍だったが、観測するneighborsだけを渡すと16.911/13.590 msの1.24倍になった。LTO後にも
callerがownerを保持するnon-tail helper callごとにそのownerをretain/releaseしている。残る境界はcaller-boundedなinternal parameterの
borrow proofであり、backend-localなretain/release相殺ではない。

032のimmutable time tableとban tableもPacked化した。要素数は最大でも110でgrowthは探索時間に対して無視できるが、aggregate
`search`を保持してから二つのPacked fieldへ分解する初版はPacked 389.450 ms、Region 69.060 msの5.64倍だった。直接parameter
patternにすると110.950/92.380 msの1.20倍になり、同じmanaged leafをaggregateとfield bindingの双方で所有していたことを分離した。
`D057`に従いaggregate ownerがfield lifetimeを包含する場合にfieldをborrowすると、元のsourceのまま110.210/92.630 msの
1.19倍になった。optimized IRではcandidateのself-tail反復にあったfield retain/releaseが消え、non-tail child activationとcaller
frameの双方がownerを必要とする境界だけに残った。

固定100,000要素を一度構築し、同じbuilder capability内でdisplay cycleを探索する058はPacked 1.660 ms、Region 1.580 msの
1.05倍だった。小さいimmutable storageで深いnon-tail探索を行う032の差と合わせ、残差を償却growthだけには帰着できない。

変更後に全269 sampleと79 maximum-order inputをdirect Cと再検証し、3 warmup、交互20回で全問を再測定した。全79問のmedian比は
1.03倍、幾何平均は1.05倍、両方5 ms以上の51問では1.05倍と1.07倍だった。±5%を同等とするとMalが速い9問、同等34問、
direct Cが速い36問であり、Packed変更によるsuite全体の回帰は認められない。個別結果とraw sampleはignored scratchの
`llvm-results.md`と各`llvm-results.json`を正とする。

## 2026-09-19 — growth後のPacked direct access

固定長のmutable workspaceを使う011と063では、builderのgrowth完了後もunique `get`と`put`がruntime callを経由し、element storeが
builder metadata loadを無効化していた。admitted application graphとcontrol continuationから後続の`New`、`NewUnique`、edit `Put`が
ないsiteを保守的に選び、checked-in runtimeのdata slot contractを通してLLVMから直接load/storeするtechniqueを採択した。この選択は
runtime表現に依存するためexecution factにはせず、空集合で通常のcapability callへ戻るLLVM optimization planに置いた。

同一のClang 21.1.8、production profile、maximum-order inputで3回warmup後に20回交互測定した。変更前のPacked / Region median比は
011が1.91倍、063が1.72倍、変更後はそれぞれ1.44倍と1.54倍だった。変更後の値は011が9.892 / 6.856 ms、063が
18.171 / 11.809 msで、最大入力の出力は一致した。executableのtext sizeは011が5,706 / 4,043 bytes、063が
5,415 / 3,576 bytesだった。raw sampleはignored scratchの各問題にある`stable-backend-vs-region.json`へ保存した。

011の最終assemblyではhot loop内の大半のbuilder metadata reloadが消えた。一方、同じbuilder由来のget/put carrierを診断的に一つへ
置換して再LTOした版は現行版に対して30回のmedian比が1.00倍で、Region比も1.45倍のままだった。最終mainのinstruction数は現行Packed
412、carrier置換版463、Region 311である。builder provenanceの一般化は適用範囲を広げ得るが、この残差の主因としては採択しない。

## 2026-09-19 — lazy edit preparationと自然なpack-edit-walk

004に、入力をimmutable `Packed`へ構築し、別の`Packed`を`edit`して行・列和を作り、freeze後の両方を走査して出力する版を加えた。
比較用にone-shot Packed、既存Region、同じmulti-pass走査形のRegion、direct Cを用意した。変更前の3 warmup・交互20回では、自然な
pack-edit-walkが205.49 ms、同じ走査形のRegionが186.34 msで1.10倍だった。IRではrecursive helper内の各要素について`get`と
`put`のindirect capability callが残り、edit `Put` helperがcopy-on-write判定を反復していた。

callback開始時のeager preparationは198.63 msまで短縮したが、`put`しないeditにもcopyとallocation failureを追加し得るため棄却した。
実際の`Put` applicationでだけeditable化し、inline flag判定からcopy-on-writeの`noinline` slow pathを呼ぶ形を採択した。最終assemblyでは
hot loopのindirect capability callが消え、copy本体はcold helperへ分離された。同じmaximum-order inputを3 warmup・回転30回で測定すると、
自然なPackedは201.44 ms、同じ走査形のRegionは194.18 ms、direct Cは193.93 msで、それぞれ1.04倍だった。stdoutはすべて一致した。
raw sampleはignored scratchの004にある`packed-edit-five-way.json`、`packed-edit-prepared-five-way.json`、
`packed-edit-lazy-slowpath.json`へ保存した。

## 2026-09-19 — Packed append fast pathとstable data epochの棄却

043の自然なpack-edit-walk版は、最大入力で必要になるheapの過去最大が約299万entryであるのに、Region版の固定上限に合わせて
3200万entryを`new(0)`で構築していた。heapをheader、distances、stepsの後へ置き、edit中にhigh-water markを越えた時だけ末尾へ
appendする形へ直した。これにより固定上限版のmedian 402msは269msへ短縮し、Region 226msに対する比は1.78倍から1.19倍になった。
出力はmaximum inputで一致した。

capacity内appendはruntime layoutを所有するC側の通常経路、allocationとgrowthは同じruntimeのslow pathである。この境界を保ったまま
`mal_runtime_packed_builder_new`をLTOで確定inlineした。旧固定heap版の3 warmup・回転10回では402msから360msへ10.5%短縮し、
high-water版では281msから269msへ4.5%短縮した。growth helperは引き続き`noinline`である。

`new`を含まない023 helper群についてactive data slot loadへ診断的に`invariant.load`を付けたところ、2 warmup・回転10回のmedianは
1172msから1148msへ2.0%短縮した。Regionは1001ms、direct Cは792msだった。しかし`invariant.load`はfunction内のepochではなく、
同じmemory locationが恒久的に不変であることを要求する。同じbuilder slotはcallbackの前後や別のcontrol stateで変化し得るため、
application graphとrecursive control regionを閉じてもこの契約を満たさない。zero-stride Packedとtree editのruntime fixtureが実際に
誤最適化を検出したため、このtechniqueは棄却した。正しい後続案にはscopedな別mechanismが必要である。raw sampleはignored scratchの
`.scratch/typical90/performance/023/stable-epoch-before-after.json`と
`.scratch/typical90/performance/043/packed-edit-variants.json`に記録した。

## 2026-09-19 — non-growing Buffer helperのinternal ABI

恒久的なmemory metadataでstable epochを表す案に代え、呼出し時点のactive data addressを値として渡すLLVM内部ABIを採択した。
対象functionはproductだけを通る`Buffer` parameterを持ち、到達するbuilder operationが`get`と`put`だけであるものに限る。
application targetとrecursive control regionを閉じ、`Buffer` result、closure capture、Bufferをcaptureするnested closure、未対応aggregate、
direct/non-direct表現が混在するindirect siteを除外する。growth可能なcallerから対象helperへ入る各境界でdata slotを一度読み、product内の
Buffer leafだけをactive data addressへ置換する。このためgrowth前後に同じhelperを呼んでも、各呼出しはその時点のaddressを受け取る。
runtime ABI、source-level `Buffer` contract、element alignmentは変更しない。baseline profileは従来のbuilder pointer ABIだけを使う。

023の自然なpack-edit-walk版を2 warmup・回転10回で変更前後比較すると、medianは1169.863 msから1138.779 msへ2.7%短縮した。
同じ測定のRegionは1013.410 ms、direct Cは794.112 msだった。slot TBAA修正後に再構築し、2 warmup・回転10回で測った四者はPacked
1134.360 ms、edit 1141.130 ms、Region 1014.750 ms、direct C 798.560 msである。9個のedit corpusはmaximum-order inputで
Packed、edit、Region、direct Cのstdoutがすべて一致した。baseline/productionのnative regressionはnested product、growth前後の
同一helper、Bufferをcaptureするnested closureを含む。誤った`invariant.load`は使わない。

同じ準備境界から、`new`時点のbuilderは常にeditableであることも導ける。`pack`のstartはeditableな空builderを作り、`edit`はcallbackへ
入る前に一度だけprepareする。したがってappendごとのeditable判定を除き、element strideをbuilder metadataから再読込せずLLVMから
runtime internal ABIへ定数で渡す。011、023、039、043の変更前後を3 warmup・交互20回で測定したpaired median比はそれぞれ
0.99、1.00、0.93、1.00だった。039のtext sizeは12,353 bytesから8,993 bytes、023は16,228 bytesから12,900 bytes、
043は11,979 bytesから9,611 bytesへ減った。このABIはsource contractではなく、coreが所有するcallback前prepare順序をruntimeへ伝える
内部境界である。

この簡約でgrowthを含む050の最終IRがdata slot loadを`new`の前からhoistし、最大入力で旧storageを参照する誤りも顕在化した。
slot loadとelement accessへ別々の独自TBAA tagを付けていたが、LTOされるC runtimeの`builder->data` storeはLLVM emitterのmetadata treeを
共有しない。したがってslot tagはC側の更新とaliasしないという、実装境界を越えた未証明の主張だった。data slot loadを保守的な
untagged accessへ戻し、mal-owned element storageのtagだけを維持した。`get`の直後に`new`してgrowthを繰り返すbaseline/production
native regressionと、通常Packed corpusのmaximum-order比較でこの境界を検査する。

## 2026-09-19 — Buffer helper invocationのnoalias active data

011の自然なpack-edit-walk版を最終assemblyまで比較すると、DPを更新するBuffer storeのために、別のimmutable Packedから読むjobの
durationとrewardがinner loopで再loadされていた。最大RSSはPacked、edit、Region、direct Cのすべてで約1.98 MiB、major faultは0、
editとRegionのmemory-management syscallはともに11回であり、working setやallocation量ではこの差を説明できなかった。

callback中のimmutable Packedとmutable Bufferを恒久的な別TBAA typeにする診断版は同じ再loadを除去したが、freeze後にはPacked resultが
Bufferと同じstorageを読むため棄却した。代わりにD063のnon-growing helper ABIへactive data pointerを一つ追加し、そのfunction引数に
だけ`noalias`を付けた。edit callback前のprepareは、同じownerを観測できるimmutable viewが残る場合にcopyするため、この契約は
function invocation中だけ成立する。複数Buffer leafは互いにaliasし得るので対象外にした。

変更前、変更後、Region、direct Cを3 warmup・回転20回で測った011のmedianは11.293、6.524、7.391、7.310 msだった。
shared sourceをcallback内で読みながらeditするnative regressionを通し、通常Packed 67問とedit 9問もmaximum inputでRegionおよび
direct Cとstdoutが一致した。

## 2026-09-19 — Packed appendのoverflow authority

`mal_runtime_packed_builder_new`はcount overflowに加え、現在のcountと次のcountを別々にbyte sizeへ変換していたため、同じstride境界を
一要素ごとに重複検査していた。non-zero strideでは`count < SIZE_MAX / stride`が次要素のcountとbyte範囲を同時に保証する。zero stride
だけをcount単独の上限として分け、lengthとrequiredを一度ずつ構成する形へ簡約した。

023の変更前後はCallgrindで9,619,081,931から9,404,871,079 instructionsへ2.2%減った。maximum inputの回転10回はmachine noiseが
大きく、個別medianでは1,310.9から1,155.9 ms、roundごとのpaired median比では0.99だった。039と043もpaired comparisonで
1.01倍のnoise内だった一方、039のmain textは4,124から3,791 bytesへ縮小した。したがって大幅な時間改善とは扱わず、runtimeが
所有する一つのoverflow factへ重複判断を戻すminimalityと、instruction・code size削減を採択理由とする。

## 2026-09-19 — 疎なbulkと再帰frameの分離

023の旧`pack`版は最大入力でRSS 442,572 KiB、minor fault 110,292だったのに対し、Regionとdirect Cは約187,720 KiB、
46,800だった。allocation syscallではstorageが260 KiBから512 MiBまで11回拡張されていた。2^24要素の疎なzero tableを
逐次`new(0)`したため、RegionとCでは`calloc`後に未変更のまま残るpageまで物理化したことが差の原因だった。

profile数とcompatibility pair数は入力幅から漸化式で厳密に求められる。canonical sourceでその合計を`bulk` capacityへ渡すと、
maximum outputを保ったままRSS 187,724 KiB、minor fault 46,797となった。2 warmup、回転10 roundのmedianはexact bulk
1,120.34 ms、旧pack 1,152.59 ms、Region 1,023.86 ms、direct C 803.74 msだった。これはruntime growth policyの変更ではなく、
既知の正確なsizeをsourceのconstruction authorityへ戻した改善である。

032のdirect Cはbestをmutable cellへ保存する一方、自然なmal版はbestをnon-tail再帰のresultとして全frameから返す。比較variantとして
Packedの初期値を`bulk`で作り、`edit`中の探索でbestを更新し、freeze後に読む形も測定した。3 warmup、回転20 roundのmedianは
mutable Packed 58.84 ms、同形Region 58.82 ms、自然なPacked 64.96 ms、direct C 38.00 msだった。canonical sourceは自然な再帰を
維持し、Packed mutability版をvariantとする。最終assemblyでは自然なmal版がnon-tail childごとに不変な探索contextを含む96 byteを
control frameへ保存し、Cは`Search *`一語を渡していた。semantic live-in自体は正しいため、backendが独自にfieldを削る根拠にはしない。

全自己再帰edgeで保持されるparameter fieldをexecution authorityから選び、ownership上borrowのfieldだけを物理frameから除いた後、
自然な032のframeは96 bytesから40 bytesへ縮小した。さらにregion invocation内のcontrol topをlocal slotへ置き、native Mal call境界で
共有topへ同期した。LLVMはlocal slotをSSA化し、frame push/popごとの共有top load/storeを除去した。3 warmup、回転30 roundでは029が
160.30 msから150.40 ms、5 warmup、回転50 roundでは自然な032が65.09 msから59.64 msへ短縮した。032のCallgrind instructionは
1,510,740,155から1,390,884,286、main function textは1,852 bytesから1,800 bytesへ減った。growable control storageとbounded native
stackは維持し、native recursionへは戻していない。032のresource計測5回では変更前後ともmaximum RSSのmedianは1,736 KiBで、
memory消費の増加もなかった。
