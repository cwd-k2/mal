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

retainの桁あふれtrapを外し、count >= 1を`assume`で伝えると、Buffer stateは2628Mから1027Mになり、要素ごとのRC操作が内側のループから消えた。残る差はvectorizeされないことで、trapとreleaseの分岐が最適化の途中までループ内に残るためである。LTO後のIRをもう一度最適化すると377Mになり、直接再帰と一致した。typical90の79問は出力が全て一致し、新旧の時間比のmedianは0.996だった。

calleeが保持する引数をcallerがownedで渡す規則（`ownership/convention`）を入れると、Buffer stateは1027Mから377Mになり、直接再帰の376Mと一致した。

以下は規則を入れる前の分析である。Buffer stateでは、callbackがborrowedなparameterから結果へ値を渡す時の`retain`と、呼び出し側が渡した値を捨てる時の`release`が反復ごとに対になる。
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

## 非tail再帰のハイブリッド実行

C版の`fib(40)`は175 ms、frame方式のmalは591 msだった。C prototypeで深さの判定を比べると、stack pointerの比較は負荷が測れず、メモリ上のcounterは2.65倍、深さを引数で運ぶ方法は13%遅かった。native版を入口のstack pointer比較付きで出し、予算を使い切ったらframe版へ渡す形にすると、`fib(40)`は245 msになった。call siteごとに分岐する形（366 ms）は、native activationがcontrol arenaの記録を毎回更新し、二つの再帰呼び出しの合流が最適化器の再帰除去を妨げたため採らなかった。50M段の再帰は、OSのstackを128 KiBに絞っても392 MBで完走し、frame方式のメモリ効率が保たれる。typical90の79問は出力が全て一致し、時間比のmedianは1.002だった。

## 2026-09-26 — native再帰のstack観測とpersistent parameter

Typical90のmaximum-order corpusを再調査すると、非tail self recursionの差はframe版へ実際に切り替わるcostではなく、浅いnative
activationが毎回払うstack guardとmanaged parameter ownershipに分かれた。029、032、068、077、080はいずれもmaximum inputでは
64 KiBのnative予算内に収まり、frame版への動的切替は発生しなかった。native recursionを無効にした比較では、029は150.14から
146.39 msへわずかに短縮した一方、032は41.79から52.32 msへ悪化したため、frame方式への一律な復帰は採らない。

stack guardがruntime helper内の`__builtin_frame_address(0)`を読む形では、inlining後もnative functionへframe pointerを要求した。
LLVM IR側で`llvm.stacksave`のlogical stack pointerを読み、runtimeへ値として渡す形へ変えると、029は10回の交互測定で
150.29から144.31 ms、Callgrind instructionは3,218 millionから3,139 millionへ減った。guardを外す診断版の080は
3.80から2.48 ms、conditional branchは4.23 millionから2.13 millionへ減ったが、bounded native stackを失うため採れない。
guardをself call siteへ移す版も3.9 msに対して4.0 msで改善せず、coldなframe fallbackが存在する限りcallee-saved registerの
退避と通常ABIのcontext引数がleaf activationにも残った。固定depth引数、call-site切替、専用worker contextは、いずれも安全性を
保つ代わりにhot pathの別のcostへ置き換えるため採らなかった。

029の主要差は、全self edgeで同じ二つのBufferを転送するにもかかわらず、各activationのparameter分解が二つをretainし、終了時に
releaseすることだった。`execution/native_recursion`で全self edgeのparameter対応を取り、使用するmanaged leafがすべて保持される
functionだけをborrowed parameterとしてownership planへ渡した。同期callerはnative版からframe版へ切り替わった後もcall完了まで
authorityを保持するため、nested aliasの通常livenessからlenderが消えるpathでもpersistent lenderとして使える。managed leafを
変更するedgeは従来のownershipに残す。

この変更後の029は144.8から119.7 msへ17%、Callgrind instructionは3,139 millionから2,391 millionへ24%、conditional branchは
388.59 millionから229.28 millionへ41%減った。maximum inputの25万行は変更前と一致した。78問の短い再走査（warmup 1、交互3回）も
すべてstdoutがCと一致し、029のmedian比は1.44倍から1.09倍へ下がった。小さいOS stackとValgrind Memcheckを組み合わせたmanaged
recursion fixtureも全件通過した。080に残るguard costは、bounded stackを維持したまま再帰不変fieldをnative内部ABIから分離する
一般的なparameter scalarizationなしには除かない。
