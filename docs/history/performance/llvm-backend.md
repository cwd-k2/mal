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
