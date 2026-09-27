# LLVM backend performance測定履歴

Status: Historical record

この文書はLLVM backendとchecked-in C11 runtimeを分離した後のpublic buildについて、LTO採用時の測定と判断を保存する。
現在の採用条件は[generated program最適化policy](../../development/generated-program-optimization.md)、責務境界は
[実行backend](../../implementation/execution-backend.md)を正とする。

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
032は4,254 bytesのままである。sample、maximum-order input、managed frameのruntime fixtureで結果とowner lifetimeを確認した。

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
誤最適化を検出したため、このtechniqueは棄却した。正しい後続案にはscopedな別mechanismが必要である。

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

続いて同じregion invocation内のcontrol storage pointerとcapacityをlocal viewへcacheし、growthとcontrol frameを持つnative Mal call後だけ
runtimeから再取得した。3 warmup、回転30 roundで自然な032は60.23 msから54.62 msへさらに9.3%、029は151.12 msから142.75 msへ
5.5%短縮した。Cachegrindのdata referenceは331,318,817から
254,149,393へ23.3%、writeは115,742,866から63,514,611へ45.1%減った。一方instructionは1,390,886,499から1,429,903,582へ
2.8%増えたため、この改善は命令削減ではなくcontext経由のmemory dependencyを切った結果である。

## 2026-09-20 — direct self-tail parameterのleaf handoff

`453be2e`から現行`make` APIで032を再生成すると、candidateを進めるdirect self-tail edgeに、変化しないparameter fieldのidentity copyが
残っていた。execution planで転送後argumentとparameter patternのbinding対応、control use、ownershipを統合し、安全なleaf patternと
entry prefixをadmitした。LLVM backendはこのplanを物理leaf slotとprefix後へのback edgeへ変換した。これによりbackendがsemanticな
適用条件を再推論せず、特定の問題、source名、型の組合せにも依存せず、LLVMのmem2regがloop-carried leafをphiへ変換できる。

maximum inputの出力を変更前、変更後、値返却C、mutable-best Cで照合した。Callgrind 3.27.1のinstruction referenceは
1,429,901,337から1,256,528,743へ12.1%減り、executable textは7,152 bytesから6,736 bytesへ縮小した。同じbinaryをwarmup 3回、
回転30 roundで測ったmedianは変更前82.204 ms、変更後80.754 ms、値返却C 57.472 ms、mutable-best C 55.799 msだった。
wall-clockの1.8%差はhost noiseに対して小さいが、hot edgeのmachine-level identity copy消失とdeterministicなinstruction削減が一致するため
採択した。変更後の全269 sampleと79 maximum-order inputもdirect Cと一致した。raw sampleはignored scratchの
`032/artifacts/measurements/self-tail-leaves-comparison.json`と
`self-tail-leaves.callgrind`へ保存した。

## 2026-09-20 — activation-local temporaryのentry配置

genericな`loop<A, B>`を100万回実行すると、recursive control region自体はLLVM function内のback edgeになっていたにもかかわらず
native stack overflowした。sum構築とpayload抽出の一時`alloca`をcase block内で実行しており、同じfunction invocationが戻るまで
周回ごとのstack領域が退役しなかったことが原因だった。backend内を再調査すると、Packed builder finish、canonical memoryからの
sum load、Packed連結のresult storageにも同じ配置があった。これらのstatic-size temporaryをfunction entryへ一度だけ配置し、state
emission中のblockには`alloca`を残さない形へ統一した。

`5df3f05`を変更前としてrelease compilerを別worktreeで作り、現行pressure suiteを同じsourceとproduction profileで生成した。
全6 workloadは変更前後ともstatus 0だった。3 warmup、20 runのHyperfineによるoutlier scanでは、5 ms未満の5 workloadに判断可能な
回帰はなく、`aggregate-churn`のmedianは7.59 msから2.20 msへ短縮した。Callgrind 3.27.1のinstruction referenceは
33,377,800から22,177,786へ33.6%、executable textは2,695 bytesから2,349 bytesへ減った。短いworkloadのwall-clock差は
採択根拠にせず、bounded native stackの回復を正しさの理由、hotなsum pathのinstructionとtext削減を副次的な改善とする。

追加したgeneric loop exampleはbaselineとproductionの両方で100万transitionを実行し、production binaryは128 KiBのstack上限でも
status 0、通常実行のmaximum RSSは1,440 KiBだった。focused LLVM artifact testはsumだけでなく上記のPacked経路もrecursive fixtureで
生成し、全temporary `alloca`が最初のback edgeより前のentry blockにあることを検査する。

## 2026-09-23 — Buffer logical operandとallocation alias scope

RegionとPackedを廃止してmanaged `Buffer`へ移行した直後、011のmedianは48.437 ms、056は20.740 msとなり、direct Cの
6.840 ms、8.960 msに対して7.08倍、2.31倍だった。Callgrind 3.27.1のinstruction referenceは011が
2,038.73 million、056が670.10 millionで、Cの234.89 million、120.60 millionを大きく上回った。no-LTOの011では
`mal_runtime_environment_retain`と`release`をそれぞれ約150.12 million回実行していた。

core loweringがBufferのreceiver、index、valueをmanaged productへ詰め、Buffer operationはそのproductをborrowしていた。この
productはsourceの値ではなく、各accessでBufferをretain/releaseする翻訳上の一時ownerだった。Buffer operationのlogical operandを
core、ANF、closure、control、execution ownershipまで個別に保持し、それぞれをborrowする形へ修正した。これはoptionalなbackend
optimizationではなく、sourceにないownershipをstage間で導入しない表現上の修正である。ANF testでreceiver、index、valueの評価順と
非product表現を、LLVM artifact testで一時retainがないことを、native testでgrowth前後のalias観測を検査する。

同じmaximum-order inputで011は11.541 ms、362.18 million instructions、056は11.834 ms、326.19 million instructionsとなった。
037のC比は3.82倍から1.31倍、063は2.30倍から0.85倍へ下がった。011は不変なjob rowをBuffer更新前に一度観測するsourceへ直すと
6.174 ms、237.20 million instructionsとなり、Cの6.77 ms、234.89 millionに揃った。027は一文字ごとの`from<UInt8>`が毎回新しい
Bufferを割り当てていた。host storageの既知範囲を一度だけadmitする形へ直すと、C比は3.19倍から1.11倍になった。いずれもdataの
意味をcompilerへ固定せず、sourceが持つ観測境界を明示した変更である。

残る056のelement storeによるactive-data slot loadの再読込には、Buffer object allocationとelement storage allocationが別である
runtime invariantだけをLLVM alias scopeで表した。slot loadへ`alias.scope`、element accessへ対応する`noalias`を付ける一方、LTOされる
C runtimeのslot更新は無注釈のままなのでgrowthを跨ぐclobberを保つ。slotへ独自TBAA typeを付ける診断案は、C側がemitterのTBAA treeを
共有しないためD064と同じ未証明の仮定となり、棄却した。

安全なalias scope版を3 warmup、交互20 roundで測ると、056は10.651 ms、Cは8.750 msで1.22倍、011は6.079 ms、Cは
6.747 msで0.90倍だった。056のinstruction referenceは267.72 millionまで減ったがCの120.60 millionに対して2.22倍であり、残差は
別のloop形状として扱う。raw sampleと調査記録はignored scratchの各measurement JSONと
`performance/buffer-migration.md`に保存した。

## 2026-09-26 — 現行corpusのdirect Cとの差の再分類

native再帰のpersistent borrowとparameter scalarizationを入れた現行compilerで、Typical90のmaximum-order corpusを再生成した。
066を除く78問はすべてdirect Cとstdoutが一致した。2 warmup、交互3回の診断走査では全78問のMal / C比のmedianが1.039倍、
双方5 ms以上の52問が1.048倍だった。短い測定で比率が上位だった5問は、同じbinaryを2 warmup、交互10回で再測定した。

| 問題 | Mal | direct C | Mal / C |
|:---|---:|---:|---:|
| 021 | 19.907 ms | 15.031 ms | 1.324x |
| 039 | 10.141 ms | 8.303 ms | 1.221x |
| 003 | 10.629 ms | 8.518 ms | 1.248x |
| 043 | 272.561 ms | 221.479 ms | 1.231x |
| 068 | 17.434 ms | 13.793 ms | 1.264x |

この5問をCallgrind 3.27.1で調べると、043以外は計算本体より入力形式の差が大きかった。次表の入力差は双方の
`__isoc99_scanf` inclusive instruction差であり、最後の列はprogram全体のinstruction差に占める割合である。この時点ではMalが
`Int64`を一つずつ読み、direct Cが問題ごとに`int`または`int64_t`を一つの入力record単位で読むという、整数幅とcall粒度の二つの差が
混在していた。

| 問題 | Mal instructions | C instructions | `scanf`差 | 全差に占める割合 |
|:---|---:|---:|---:|---:|
| 021 | 358.02 M | 311.10 M | 38.80 M | 82.7% |
| 039 | 177.23 M | 155.42 M | 19.40 M | 89.0% |
| 003 | 184.03 M | 156.22 M | 20.70 M | 74.4% |
| 068 | 383.28 M | 315.48 M | 57.90 M | 85.4% |

次点の062、026、017、010、028でも、program全体のinstruction差に占める同じ`scanf`差はそれぞれ88%、77%、59%、54%、94%だった。
当初はこれを主に整数幅の差と解釈したが、入力をCと同じ`Int32`へ変更した021のinstructionは358.02 Mから357.62 Mへしか減らず、
`scanf` inclusiveも328.88 Mのままだった。支配的だったのは幅ではなく、Malが辺の2整数やqueryの4整数を4回のscalar host callに分け、
Cが一回の`scanf`で一つの入力recordを読むcall粒度の差である。幅の整合はdomainとstorageを揃えるために必要だが、このinstruction差の
説明にはならない。

そこで共通hostに`Int32Pair`、`Int64Pair`、weighted edge、weighted queryなど、source上の一入力recordを返すoperationを追加した。
特定問題のalgorithmをhostへ移さず、Cと同じformat parse一回でproductを返す。010の一出力行もproductを一回で出力し、012の
Yes/Noも一回のhost operationに揃えた。これによりI/Oを含む比較でhost boundaryの分割数が結果を支配しなくなった。

ただしarityごとのoperationは診断用の過渡形であり、corpusの恒久的なhost interfaceにはしない。自然な収束先は、`fread`でbyte blockを
補充するscannerと、byte bufferへ整数をformatして一括flushするwriterを一つずつ持ち、Malとdirect Cの両方が同じ実装を使う形である。
Mal sourceは再びscalarな`readInt32`、`readInt64`、`writeInt64`を使い、record arityをhost ABIへ列挙しない。scannerは符号、範囲、EOF、
不正tokenを明示的に検査し、writerは正常終了時と明示flush時のerrorを伝える。比較用Cだけが`scanf`/`printf`のformat parseを使う状態も
残さず、I/O layerを共有した上でcompiler生成部分の比率を測る。

整数storage幅の差はworking setにも現れた。Mal sourceはindex、parent、tagなども一つの`Buffer<Int64>`へ置き、C sourceは多くを
`int`またはbyte arrayへ分ける。5回測定したmaximum RSSとminor faultのmedianは次の通りだった。

| 問題 | Mal RSS | C RSS | Mal / C | Mal / C minor faults |
|:---|---:|---:|---:|---:|
| 021 | 14,276 KiB | 6,768 KiB | 2.11x | 3,407 / 1,481 |
| 039 | 8,772 KiB | 5,700 KiB | 1.54x | 2,042 / 1,067 |
| 003 | 6,256 KiB | 4,672 KiB | 1.34x | 1,647 / 869 |
| 043 | 81,736 KiB | 80,836 KiB | 1.01x | 20,080 / 19,834 |
| 068 | 4,164 KiB | 2,628 KiB | 1.58x | 770 / 378 |

021、039、003の最終binaryには別の`mal_function_*`が残らず、sourceのBuffer view helperとtail recursionはLTO後の`main`へ統合されていた。
したがってこれらの残差を関数呼出し一般には帰属させない。型に応じた32-bit storage、入力record単位のhost operation、同じdata layoutでの
比較を先に整える必要がある。

043は別で、入力が6整数だけなので2,474.73 M対1,422.98 M instructionsの差はheap本体にある。最終assemblyではheap pushが
`mal_function_14`として残り、4,992,769回呼ばれた。Bufferのlogical countが過去最大heap sizeへ達した2,992,773回は、2個の`Int64`を
`fill`してcountを伸ばす。`mal_runtime_buffer_count`は17.96 M、`mal_runtime_buffer_fill`は287.31 M instructionsを使い、push worker全体は
1,251.98 M instructionsだった。direct Cは最大capacityを一度`malloc`し、同じheap loopから未初期化slotへ直接書く。Malで最大領域を
`new(0)`で逐次構築した旧variantは不要な反復を増やして遅かったが、これは現行の一括`fill`とは異なる。

現行`make<Int64>(capacity)`はcalloc済みcapacityとlogical count 0を作り、直後のzero `fill`はruntimeの`zeroed_until`まで実byteを
書かずにcountだけ延ばせる。最大16 transition / cellと4個のseedを上限としてheap entry capacityを`cellCount * 16 + 4`、slot数を
その2倍にし、一度のzero `fill`でlogical countを延ばす形へcanonical sourceを変更した。push時のgrowthを外した変更前後は、
3 warmup、交互20回のmedianが270.72から249.93 msへ7.7%短縮した。
Callgrind instructionは2,474.73 Mから1,872.54 Mへ24.3%減り、pushは`main`へinlineされた。maximum RSSは81,736対81,732 KiB、
minor faultはともに20,080で増えなかった。変更後canonicalとdirect Cの交互20回は249.17対217.76 ms、1.14倍だった。したがって
この部分は新しいBuffer APIの不足ではなく、既存のcapacityと連続一括初期化をsourceが使っていなかった差である。

変更後のconditional branchはMal 226.83 M、C 225.74 M、mispredictは11.49 M、11.32 Mで、heap loopの反復・分岐形状はほぼ揃った。
一方、Cachegrindのdata referenceはMal 530.05 M、C 333.13 Mで1.59倍だった。LL data missは9.30 M、9.07 Mに留まるため、残差は
working setやmain memory trafficではなくhot dataの余分なload/storeである。最終assemblyでCは`Entry`のdistanceとstateを`movups`で
128-bit pairとして移すが、Malの`Buffer<Int64>` heapは二つのscalar load/storeを別々に出す。Malのdirection loopはview offset、
Buffer owner、data pointer、heap size、gridとdistanceのbaseを同時にliveにし、各directionで複数のstack reloadも生じる。次の比較対象は
heapを`Buffer<(Int64, Int64)>`としてentry単位で運ぶ表現と、Buffer viewのbase/offsetが作るregister pressureである。

canonical sourceから手製のheaderと`(Buffer<Int64>, offset)` viewを除き、distancesを`Buffer<Int64>`、direction stepsとheapをそれぞれ
`Buffer<(Int64, Int64)>`へ分けた。sample 3件とmaximum inputはdirect Cと一致した。旧一括Buffer版と3 warmup、交互30回で比較すると
medianは258.98から248.40 msへ4.1%短縮した。別の3 warmup、交互20回では自然な版254.67 ms、direct C 233.88 msで1.09倍だった。
Callgrind instructionは1,872.54 Mから1,561.09 Mへ16.6%、data referenceは530.05 Mから410.23 Mへ22.6%減った。branch数と
LL data missは変わらず、maximum RSSとminor faultも81,732 KiB、20,080で同じだった。product化により末尾heap nodeの移動は
128-bit pairになり、offset arithmeticとstack reloadが減ったが、
child nodeの比較後の移動にはscalar 2本が残る。読みやすさと性能が同じ方向へ改善したため、この自然な表現をcanonicalとした。

同じ手製view、`appendZeroedSlots`、workspace headerのいずれかを持つcanonical sourceは、043を直した後も78問中34問あった。
これは個々のalgorithmが要求する表現ではなく、Packed/Regionからmanaged Bufferへ移行した時期の共通慣習である。すべてを機械的に分割せず、
まずC比上位の021、039、003、068、062、026、017、028、013を、sourceの論理collectionとBufferが一対一になる自然な版で再測定する。

この方針で021、039、003、068、062、026、017、013に加え、上位へ移った077、054、035、012も整理した。index、offset、parent、queueは
`Buffer<Int32>`、flagは`Buffer<UInt8>`または`Buffer<Int8>`、距離と重みだけを`Buffer<Int64>`にした。辺、座標、heap itemのように
一つの値として移動するrowはproduct Bufferとし、query resultを同じ巨大Bufferの末尾へ置くheader/view表現を廃止した。013のheap popは
nodeとdistanceを別配列と副作用cellで返す形から`(Int32, Int64)` resultへ、068と012は出力を蓄積せず入力順に出す形へ戻した。

自然なlayoutだけを入れた20 roundの代表値では、003が1.27倍から1.10倍、017が1.21倍から1.04倍、026が1.22倍から1.05倍、
062が1.25倍から1.08倍、013が1.22倍から1.13倍、054が1.23倍から1.02倍、077が1.31倍から1.15倍になった。その後、入力record単位の
host operationを揃えると、003、013、021、026、039、062、068は0.96--1.03倍、017は0.97倍に収まった。028はscalar入力時の
1.23倍から1.08倍、010は1.22倍から1.03倍になった。035はlayoutも分けて1.01倍、012はflag分離と逐次出力を含めて1.05倍になった。

全変更後に269 sampleと79 maximum-order comparisonを通し、2 warmup、交互10 roundで78問を再測定した。双方5 ms以上の52問では
Mal / C比のmedianが1.016倍、meanが1.007倍、最大が049の1.14倍だった。全体の比率上位は5 ms未満の080が1.24倍で、これは下記の
再帰guard差と一致する。慣習差を除いた後の上位は049、044、078で、032は1.11倍だった。入力幅や一枚Bufferをcorpus全体のcompiler overheadと
誤認する状態は解消した。

再帰固有の残差は別記録に分離する。080は現行Mal 90.38 M対C 77.79 M instructionsで、conditional branch差約2.10 Mがactivationごとの
stack guardに一致する。一方068のnative hybridとframe-onlyは16.21 ms対16.43 msで、同問題のC差の主因ではなかった。032には
Malがbestを再帰resultで返しCがmutable cellへ保存するsource差がある。詳細と採らなかったnative/frame選択policyは
[loop combinatorのperformance測定履歴](loop-combinators.md)を参照する。

以上から、現行corpusのC差は一つのbackend overheadではなく、入力・整数storage幅、Bufferの構築方法、再帰guard、
source algorithm/state表現に分かれる。優先順位は、多数の上位caseへ共通する入力とdata widthを公平化し、043では既存APIによる一括構築を
canonical sourceへ反映した上で、同形の計算だけが残るcaseでLLVM loop形状と再帰guardを再測定する順とする。

## 2026-09-27 — 共有buffered scannerへの置換

過渡的な入力record別externを削除し、Malとdirect Cが同じ64 KiB `fread` scannerからscalarな`Int32`、`Int64`、`UInt64`、byte tokenを
読む形へ変更した。scannerはASCII whitespace、符号、型の範囲、token終端、EOF、`fread` error、token destination capacityを検査する。
pair、triple、weighted edgeなどはhost operationではなくMalの通常functionでscalar primitiveから構成する。

direct Cを従来どおり`-O2`だけでbuildすると、別translation unitになったscanner facadeがC側だけcallとして残り、9問のmedian比が
0.91倍になる非対称が生じた。Mal production buildは`-O2 -flto`で同じ境界をinlineしているため、direct Cも`-O2 -flto`へ揃え、
stdin facadeを両artifactでinlineした。reference Cのalgorithmは一つのtranslation unitに留まるため、このLTOは新たに分離した共有I/O境界を
揃えるために必要である。

過去にscalar `scanf`差が大きかった9問をmaximum-order input、2 warmup、交互10 roundで測定した。stdoutは各roundのwarmup時に一致を
確認した。raw sampleは各problemのignored `artifacts/measurements/scanner-vs-c.json`に保存した。

| 問題 | Mal median | direct C median | Mal / C |
|:---|---:|---:|---:|
| 021 | 13.779 ms | 11.776 ms | 1.170x |
| 039 | 7.802 ms | 7.631 ms | 1.022x |
| 003 | 7.366 ms | 7.951 ms | 0.926x |
| 068 | 11.611 ms | 10.927 ms | 1.063x |
| 062 | 11.306 ms | 11.326 ms | 0.998x |
| 026 | 8.706 ms | 9.275 ms | 0.939x |
| 017 | 41.957 ms | 41.395 ms | 1.014x |
| 010 | 9.355 ms | 8.975 ms | 1.042x |
| 028 | 15.724 ms | 13.647 ms | 1.152x |

9問の比率はmedian 1.022倍、幾何平均1.033倍だった。旧scalar `scanf`測定では同じ9問がともに約1.22倍だったため、format parseと
call粒度が作っていた集合全体の差は消えた。一方、021と028ではscalar Mal extern boundaryの回数に応じた差が再び観測できる。
record別externを使った直前の測定では両問がそれぞれ1.02倍、1.06倍だったため、021についてinstructionと生成形状を追加比較した。

021の同じscannerに対し、現行scalar extern、二整数を一度に返す診断用pair extern、生成後のscalar bridgeだけへ強制inline属性を加えた
artifactを作った。3 warmup、三者を巡回する30 roundの結果とCallgrind instructionは次のとおりだった。入力とstdoutは三者で同一である。

| 形 | median | instructions | 入力bridgeの生成形状 |
|:---|---:|---:|:---|
| scalar extern | 12.502 ms | 86.884 M | 400,002回のnative callが残る |
| scalar bridgeを強制inline | 11.414 ms | 77.818 M | callなし |
| pair extern | 11.405 ms | 76.540 M | edge用bridgeがinlineされる |

現行scalarから強制inlineへの差は一入力あたり約22.7 instructionsで、強制inline後の実時間はpair externと同じだった。したがって残差は
extern ABIに不可避なcostではなく、大きなscanner本体を取り込んだscalar bridgeをLLVMがinlineしなかった結果である。強制inlineはtextを
9,010から11,266 bytesへ25%増やしたため、全extern bridgeへの一律指定は採らない。arity別operationをinterfaceへ戻すのでもなく、
scanner parserと薄いbridgeの分離、またはcallsiteとcode sizeを考慮したinline方針を別途比較する。

この結果を受け、stdin facadeはinlineのまま、型別の`corpus_scanner_read_*` parserだけをout-of-lineに固定した。Mal bridgeとdirect Cの
callsiteはいずれもscanner状態の取得と一つのparser callだけになり、Mal bridge自体はmainへinlineされた。021のCallgrindでは両artifactの
parserが同じ400,002回、同じ71.476 M instructionsになった。全体はMal 94.151 M、C 92.200 Mで、scanner以外の差だけが残った。
parserもinlineする強制inline版より絶対instructionは増えるが、Mal bridgeだけが残る形や全bridgeへの一律inlineによるcode size増加を避け、
algorithm比較から共有I/O実装の最適化判断を分離できるため、この境界をcorpusの標準とした。

同じ9問を再buildし、2 warmup、交互10 roundで測定した結果は次のとおりだった。

| 問題 | Mal median | direct C median | Mal / C |
|:---|---:|---:|---:|
| 021 | 12.143 ms | 11.661 ms | 1.041x |
| 039 | 6.929 ms | 6.855 ms | 1.011x |
| 003 | 7.141 ms | 7.450 ms | 0.958x |
| 068 | 11.980 ms | 11.886 ms | 1.008x |
| 062 | 7.691 ms | 8.461 ms | 0.909x |
| 026 | 6.431 ms | 7.371 ms | 0.872x |
| 017 | 27.349 ms | 27.483 ms | 0.995x |
| 010 | 7.113 ms | 7.398 ms | 0.961x |
| 028 | 12.126 ms | 10.679 ms | 1.136x |

比率のmedianは0.995倍、幾何平均は0.985倍だった。021のscalar bridge由来の差は1.17倍から1.04倍へ縮小した。028の1.14倍は同じ
parser境界を揃えても残るため、scannerやextern一般のcostではなくalgorithm本体の生成形状として扱う。

共有parser境界の採用後に78問を2 warmup、交互10 roundで再測定した。双方5 ms以上の46問ではMal / C比のmedianが0.988倍、
meanが0.993倍だった。比率上位の073、027、035、032、028を3 warmup、交互30 roundで再確認すると、それぞれ1.163、1.230、
1.141、1.161、1.035倍だった。028の大差は再現せず、残る4問についてsource表現と生成形状を分離した。

073はgraph indexまで`Int64`の一枚Bufferに置き、負値の正規化を二回の剰余で書いていた。index collectionを独立した
`Buffer<Int32>`、DP値を独立した`Buffer<Int64>`とし、剰余を一回と負値への加算に直した。これは論理collectionと必要な値域に対応する
通常の表現であり、Mal / C比は30 roundで1.003倍になった。したがって073はcompiler差ではなくsource storageと演算回数の差だった。

027もlengthとhash tableを`Buffer<Int32>`へ分け、first occurrenceを回答Bufferへ蓄積せず逐次`printInt32`する形へ直した。
Callgrind instructionはMal 112.382 M、C 111.123 Mの1.011倍まで揃った。10万行のstdioを含む100 roundは15.579対14.143 msの
1.102倍だったが、algorithm部分のinstruction差を伴わないためcompiler本体の課題には分類しない。固定長Address領域、byte単位のhashと
比較、open-address tableをsourceで組む形自体は、名前のhash setを直接表せないことによるperformance目的の実装である。残差を追う場合は
backendではなく、token/Symbol admissionとhash collectionの表現力として扱う。

035は回答Bufferを除いて逐次出力してもCallgrind instructionが231.95 Mから231.45 Mへ0.5 Mしか減らず、Cの208.42 Mとの差が残った。
Cachegrindではconditional branchが32.608 M対26.174 M、data readが43.463 M対37.461 M、data writeが23.800 M対22.967 Mで、
branch mispredictは2.083 M対2.155 Mだった。生成LLVMではhotな`distance`の入口が、borrowedな`TreeDistanceIndex`を分解するときに
内包する5個のBufferをすべてretainし、未使用4個を直ちにreleaseし、使用する`depths`もreturn前にreleaseする。この処理が約50万回の
distance計算ごとに残る。treeをmanaged fieldごとの引数へ展開するsource回避は自然なproduct表現を損なうため採らず、
`execution/ownership`がborrowed parameterから得たnested product bindingのauthorityを後続のdestructureへ伝播できないcompiler課題とする。

032は従来の分類どおり、bounded native stackのためのexplicit continuation push/popとstate復元が残差である。現行Callgrindも
Mal 1,365.63 M、C 923.38 M instructionsで、値返却Cとmutable-best Cが同等だった過去の診断と一致する。したがってsource整理後の
この時点で確認できたcompiler課題は、035のnested managed product borrowと032のnon-tail recursive continuation costの二つだった。

## 2026-09-27 — nested borrow authorityの正規化

035の形を縮小すると、borrowed parameterから取り出した中間productをnative calleeへ渡し、そのproductから先に取り出したmanaged leafを
resume後にも使う場合に再現した。ownership collectorはleafから中間product、中間productからparameterへの依存辺を個別には構成していたが、
state境界の包含判定前に最終lenderへ閉じていなかった。このため中間carrierがresume stateでdeadになると、caller authorityが呼び出し全体を
包含していてもleafをownerへ昇格していた。

収集後のauthority graphを推移的に解き、中間aliasをstorageを実際に所有するlocal lenderへ置換した。borrowed ABIやactive environmentの
ようにactivation外のauthorityが包含する値は空のlocal lender集合になり、通常のlocal ownerは終端として残る。最終authorityへ到達できない
cycleはborrowの証明として採用しない。これによりsource順やproductの入れ子ではなく、最終authorityのlifetimeだけでstate境界を検証する。

修正後の035 LLVMでは、hotな`distance`入口にあった5個のBufferのretain、未使用4個の即時release、使用する`depths`のreturn前releaseが
すべて消えた。maximum-order inputのCallgrindは231,445,380から221,145,452 instructionsへ10,299,928、4.45%減った。Cは
208,420,192 instructionsで、Mal / Cは1.110倍から1.061倍へ縮小し、instruction差の44.7%を除去した。

修正前、修正後、Cを5 warmup、rotating 100 roundで同時比較したmedianは31.799、30.873、26.609 msだった。修正後は修正前より
2.9%短く、Mal / Cは1.195倍から1.160倍へ縮小した。実時間sampleの分散が大きいため、局所変更の効果量はCallgrindを主な根拠とする。
269 Mal sample、269 C sample、79 maximum-order比較、72 diagnostic variantはすべて通過した。このnested borrow defectは解消し、035に
残る12.73 M instructionsは別の生成形状として再診断する。
