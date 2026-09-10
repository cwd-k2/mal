# generated program最適化policy

Status: Current policy

この文書はLLVM backendとchecked-in C runtimeの最適化を採用するgateを定める。言語semanticsは[`spec/`](../spec/)、control表現は
[application control lowering](application-control-lowering.md)、検証commandは[test policy](testing.md)を正とする。過去のgenerated Cに
対する測定と判断は[performance history](../history/performance/generated-c.md)に保存する。

## 優先順位

1. result、effect order、trap、owner lifetime、bounded native stackを保持する。
2. program固有のpolicyをLLVM、program非依存のmechanismをC runtimeに置く責務境界を保持する。
3. 実行時間、allocation、code size、compile timeのうちworkloadで支配的なcostを減らす。

未計測の複雑化、特定fixtureだけのspecial case、LLVM optimizerが既に安定して行う局所変換の再実装は採用しない。

## baseline

比較するbinaryは同じpinned Clang、target、`-O2`、strict floating-point option、LTO設定でbuildする。ambient `CC`を継承しない。
correctness baselineはLTOなしとし、LTOありでも同じobservable resultになることを別に確認する。

wall-clockは同じinput、warmup、run数で交互に測り、5 ms未満のcaseを採否の主根拠にしない。noiseを含む単発値ではなくmedianと範囲を残す。
instruction count、branch、allocation counter、peak resident memory、artifact sizeなど再現しやすい第二指標を少なくとも一つ併用する。

## workload

- scalar arithmeticとbranch
- direct call、self-tail loop、deep non-tail unwind
- first-class call cycleとheterogeneous frame
- flat `Symbol`のread、comparison、concatenation
- managed product、sum、closure environment、extern round trip
- checked-in example corpus

意味論fixtureとperformance fixtureを兼用してよいが、期待resultとresource invariantを先に固定する。performance差だけを理由に検証を弱めない。

## 記録

採用または棄却に将来の実装判断へ必要な情報がある場合は、日付、commit、環境、command、workload、raw measurement、判断を
`docs/history/performance/`へ記録する。active documentへ時系列statusを残さない。
