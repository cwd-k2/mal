# 性能調査toolと作業領域

Status: Current development policy

この文書は性能調査に使うtoolと、local生成物を置く作業領域を定める。最適化を採用する測定条件は
[generated program最適化policy](generated-program-optimization.md)、通常の完了判定は[test policy](testing.md)、
採用または棄却した結果は[性能測定履歴](../history/performance/)を正とする。

## Development shell

`nix develop`はcompiler toolchainに加えて次の調査toolを提供する。

| Tool | 用途 |
|---|---|
| `hyperfine` | warmupと反復を伴うwall-clock測定、JSON出力 |
| `valgrind` | Memcheck、Callgrind、Cachegrind、Massifによるmemory error、命令、cache、heap調査 |
| GNU `time` | peak RSS、page fault、context switchを含むprocess resource測定 |

これらは互いに異なる観測を所有する。wall-clock比較には`hyperfine`、再現しやすい命令・cache指標には
CallgrindまたはCachegrind、heapの時系列にはMassif、process全体のresident memoryにはGNU `time`を使う。
Valgrind下の実行時間をnative wall-clockとして比較しない。

Linux `perf`はhost kernelの設定、counter access、virtualizationに依存するためdevelopment shellの再現可能な
baselineには含めない。必要な調査では利用環境とcommandを結果に記録する。

## 基本command

Nushellから、対象programとinputを明示して実行する。

```nu
hyperfine --shell=none --warmup 3 --runs 20 --export-json result.json ./program
open --raw input.txt | valgrind --tool=callgrind --callgrind-out-file=callgrind.out ./program
open --raw input.txt | valgrind --tool=cachegrind --cachegrind-out-file=cachegrind.out ./program
open --raw input.txt | valgrind --tool=massif --massif-out-file=massif.out ./program
open --raw input.txt | ^time -v ./program
```

比較対象へstdinが必要な場合、Hyperfineでは`--input input.txt`を使う。比較では同じbinary生成条件、input、stdout、
warmup、run数を揃え、実行順の偏りを避ける。採否に使う結果にはcommandとtool versionも残す。

## `.scratch`の境界

`.scratch/`はgit管理外のlocal workspaceであり、第三者由来のfixture、調査中のsource、Nushell script、未採用の
raw measurementを置ける。compilerの仕様、test、現在の設計判断は置かない。

再生成可能なexecutable、object、LLVM/C artifact、Valgrind output、maximum-order inputは調査中だけ保持する。
調査を終えると削除し、再生成commandを近接するREADMEまたはscriptに残す。将来の判断に必要な結論と再現条件は
`docs/history/performance/`へ要約し、ignored fileだけを根拠にしない。Pythonを前提とする補助scriptは追加せず、
structured dataの生成・集計にはNushellを使う。
