# LLVM backend performance測定履歴

Status: Historical record

この文書はLLVM backendとchecked-in C11 runtimeを分離した後のpublic buildについて、LTO採用時の測定と判断を保存する。
現在の採用条件は[generated program最適化policy](../../development/generated-program-optimization.md)、責務境界は
[実行backend](../../design/execution-backend.md)を正とする。

## 2026-09-10 — public buildへのLTO採用

測定対象は`65310cb`を基点とするLLVM backendと、この変更で追加した`-flto`である。pinned environmentの
Clang 21.1.8、Hyperfine 1.20.0、Nushell 0.114.1を使った。Typical 90のうち、interactiveな053、platform `libm`を
linker inputに要する009と018を除く77問を対象とした。各Mal programはpublic `malc build`、direct C baselineは同じ
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

raw measurementはignored scratch treeの`.scratch/typical90/performance/*/llvm-results.json`と
`.scratch/typical90/performance/llvm-results.md`に保存した。

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
