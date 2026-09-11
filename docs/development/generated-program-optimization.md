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

## 構成

最適化は対象のauthorityを持つstage内で個別techniqueとして定義する。`execution/optimization`はcontinuationとcall selection、
`backend/llvm/optimization`はadmitted execution planを変更しないtarget固有のemission decisionを所有する。driverは各stageへ有効な
technique集合を明示的に渡す。後段はtechnique identityではなく、所有stageが検証したplanだけを読む。

空のtechnique集合は全admitted programを実行できるbaselineである。production集合は採用済みtechniqueの明示的な合成であり、別の
意味論や別のbackend contractを持たない。新しいtechniqueが既存plan型または無関係なstageの変更を要求する場合は、optimization追加ではなく
authority境界の変更として先に検討する。

LLVM backendではowner lifetimeのfactとstorage再利用のdecisionを分ける。dead ownerのreleaseは設定によらず行い、`Symbol` concatへ
dead operandをmoveする選択だけをoptional techniqueとする。通常のconstant propagation、instruction combination、dead-code elimination、
inliningは独自実装せずpinned LLVMへ委ねる。

## correctness baseline

public `build`は既定で空のexecution technique集合、空のLLVM technique集合、Clang `-O0`、LTOなしの`baseline` profileを使う。
`--optimization production`だけが採用済みtechniqueとClang `-O2 -flto`を有効にする。両profileは同じpinned Clang、target、strict
floating-point optionを使い、ambient `CC`を継承しない。result、effect trace、trap、owner終状態、bounded native stackはbaselineで
成立し、productionとの差分に依存しない。

性能の採否ではproduction profileを比較対象とする。LTO有無を調査する場合は同じobservable resultを先に確認し、差分をsource責務の移動ではなく
cross-translation-unit optimizationの効果として扱う。

wall-clockは同じinput、warmup、run数で交互に測り、5 ms未満のcaseを採否の主根拠にしない。noiseを含む単発値ではなくmedianと範囲を残す。
instruction count、branch、allocation counter、peak resident memory、artifact sizeなど再現しやすい第二指標を少なくとも一つ併用する。

## workload

- scalar arithmeticとbranch
- direct call、self-tail loop、deep non-tail unwind
- first-class call cycleとheterogeneous frame
- flat/rope `Symbol`のread、comparison、concatenation、host materialization
- managed product、sum、closure environment、extern round trip
- checked-in example corpus

意味論fixtureとperformance fixtureを兼用してよいが、期待resultとresource invariantを先に固定する。performance差だけを理由に検証を弱めない。

## 記録

採用または棄却に将来の実装判断へ必要な情報がある場合は、日付、commit、環境、command、workload、raw measurement、判断を
`docs/history/performance/`へ記録する。active documentへ時系列statusを残さない。
