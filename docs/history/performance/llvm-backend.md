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
