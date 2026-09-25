# loop combinatorのperformance測定履歴

Status: Historical record

この文書は、`loop`のようなself-recursive combinatorと、それを重ねたsource定義のhigher-order関数を、直接のself recursionと比べた測定と、その結果から採用した規則を保存する。現在の最適化課題と採用条件は
[generated program最適化policy](../../development/generated-program-optimization.md)、規則そのものは
[application control lowering](../../implementation/application-control-lowering.md)と[managed valueのownership](../../implementation/ownership.md)を正とする。

## 2026-09-25 — 直接再帰との差と三つの規則

環境はpinned environmentのClang 21.1.8、Callgrind 3.27.1、Hyperfineである。すべて`malc build`のproductionで、各variantの終了statusとresultが
一致することを確認した後、Callgrindのinstruction数と、warmup 1回・8 runのHyperfine medianを記録した。wall-clockは同一host内でもrunごとに
±10 ms程度動いたため、instruction数を主根拠にした。workloadは同じ計算を直接再帰（`direct`）と`loop`経由（`loop`）で書いたものである。

| workload | 内容 | 変更前 direct / loop（Minstr） | 変更後 loop（Minstr） |
|:---|:---|---:|---:|
| dijkstra | `examples/csr-dijkstra`の`_loop`利用を8000 nodeへ拡大。managedなtupleをcaptureし、loopを入れ子にする | 546 / 3272 | 549 |
| matmul, separate combinators | 同じsignatureのcombinatorを別名で三段に重ねた200×200 | 44 / 1777 | 99 |
| matmul, shared combinator | 同じ`upto<UInt64>`を三段に重ねた200×200 | 44 / 1061 | 99 |
| shared combinator | 一つの`upto<UInt64>`を三種のcallbackで使う | 632 / 1500 | 504 |
| buffer state | Bufferをloop stateに載せて200万要素を走査 | 376 / 2628 | 2628 |
| buffer capture | Bufferをcaptureして走査 | 376 / 377 | 377 |

差の原因は三つに分かれた。それぞれretain/releaseの除去、cycleの除去、callbackのinlineという別の量であり、独立に測った。

- **callee集合が型だけで決まる。** callbackがすべて`(UInt64, UInt64) -> UInt64`だと、入れ子の内側のcallbackが外側のcallbackを呼び得るように見え、
  全体がrecursive regionになってframeとdispatchを毎反復で払っていた。callbackの型を段ごとに変えるとinstruction数は1061Mから99Mへ減り、
  IRから`mal_control_reserve_frame`が消えた。closure生成からcalleeへ届くfunctionをflowで求め、型互換なfunctionのうち届くものだけをtargetにした。
- **captureの読み取りがreferenceを取る。** dijkstraの差は、retain/releaseをno-opにした診断用buildで546Mへ一致した（差はすべてreference count操作）。
  captureから読んだmanaged valueは、activeなenvironmentが保持するためlenderなしのaliasとしてborrowする。frameはそのようなaliasがliveならenvironmentを運び、
  tail callはenvironmentを手放した後に走るためaliasを渡さない。
- **同じfunctionを複数のclosureで共有する。** flowはcontext insensitiveなので、`step`に三つのcallbackが届く`upto<UInt64>`は入れ子でもcycleのまま残り、
  callbackもinlineされなかった。LTO後のIRをもう一度`-O2`に通すと1500Mが527Mになったが、2周目の最適化はcompile timeを増やすだけで採用しなかった。
  代わりに、closureを受け取るtop-level functionを、call siteが渡すclosure集合ごとに複製する`call_pattern` stageを追加した（production集合のみ）。
  複製したfunctionは自分のcallbackだけを呼ぶため、cycleとregionが消え、callbackがinlineされた。example corpusのtext sizeは変わらないか小さくなった
  （`csr-dijkstra` 7692→6796 bytes、`json-query` 25218→23250 bytes）。複製数はfunction数の4倍に64を足した数までである。

Buffer stateでは、callbackがborrowedなparameterから結果へ値を渡す時の`retain`と、呼び出し側が渡した値を捨てる時の`release`が反復ごとに対になる。
除去にはcalleeが宣言するowned parameter conventionが必要で、closureのdispatchが複数のtargetを持つ限り一つのcall siteが二つのconventionを満たせない。
Bufferをcaptureして走査するcaseは直接再帰と一致するため、実装せず、conventionの設計を要する課題として残した。

## 同じ調査で見つけた不具合

同じgeneric関数を別の型で二度specializeすると、各instanceが元のbinderの`ValueId`を共有し、ownership planがborrowの事実をinstance間で取り違えた。
入れ子の`upto<Unit>`と`upto<UInt64>`でclosure environmentが解放後にretainされた。specializerがinstanceごとに全binderへ新しい識別子を与えることで
直した。

## 再現

workloadと測定scriptはignoredな`.scratch/loop-perf/`に置いた。repository rootの`nix develop`内で、
`nu .scratch/loop-perf/tools/measure.nu --label <name>`が全variantを測定し、`memcheck-examples.nu`と`memcheck-spec.nu`が全exampleと
spec corpusをValgrind Memcheckで実行する。
