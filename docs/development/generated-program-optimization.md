# generated program最適化policy

Status: Current policy

この文書はLLVM backendとchecked-in C runtimeの最適化を採用するgateを定める。言語semanticsは[`spec/`](../spec/)、control表現は
[application control lowering](../implementation/application-control-lowering.md)、検証commandは[test policy](testing.md)を正とする。過去のgenerated Cに
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

execution ownership planではowner lifetimeのfactとresponsibilityの後継を構成し、唯一のowner successorへのhandoffを設定によらず
`Consume`へ正規化する。LLVM backendはこのplanとstorage再利用のdecisionを分ける。dead responsibilityの`Drop`は設定によらず行い、
`Symbol` operationが移されたresponsibilityのstorageをruntime representationとcapacityに基づいて再利用する選択だけをoptional
techniqueとする。通常のconstant propagation、instruction combination、dead-code elimination、inliningは独自実装せずpinned LLVMへ委ねる。

## technique追加contract

新しいtechniqueは次を同じ変更で満たす。

1. 対象のauthorityを持つstageの`optimization/`直下に、一つの適用規則だけを所有するmoduleを置く。
2. stageの`Technique`と`OptimizationSet::production`へ採用を明示し、空集合および単独集合を構成可能に保つ。
3. semantic inputを変更せず、集約`OptimizationPlan`へ後段が既存contractで読めるdecisionを追加する。
4. authorityからdecision集合を再構成するexact validatorを追加する。
5. 不適用、単独適用、baselineとのobservable behavior一致、productionで狙ったcostが減ることをそれぞれ適切なboundaryで検証する。

techniqueの無効化でprogram admission、型、ABI、runtime contractが変わる場合はこのcontractを満たさない。新しいsemantic factまたはbackend
capabilityが必要なら、そのownerの通常planを先に拡張し、optimization moduleへ事実の推論を代行させない。

## correctness baseline

`--optimization`の`production`は採用済みのexecution technique集合、LLVM technique集合、Clang `-O2 -flto`を合成した既定であり、
`baseline`は空のtechnique集合、Clang `-O0`、LTOなしでdebugと差分検証を行う経路である。両者のcommandとtoolchain上の扱いは
[`malc`利用contract](compiler-usage.md#build-toolchain)を正とする。result、effect trace、trap、owner終状態、bounded native stackは
`baseline`でも成立し、`production`との差分に依存しない。

性能の採否では`production`を比較対象とする。LTO有無を調査する場合は同じobservable resultを先に確認し、差分をsource責務の移動ではなく
cross-translation-unit optimizationの効果として扱う。

wall-clockは同じinput、warmup、run数で交互に測り、5 ms未満のcaseを採否の主根拠にしない。noiseを含む単発値ではなくmedianと範囲を残す。
instruction count、branch、allocation counter、peak resident memory、artifact sizeなど再現しやすい第二指標を少なくとも一つ併用する。
toolの選択、基本command、local生成物の扱いは[性能調査toolと作業領域](performance-investigation.md)を正とする。

## workload

- scalar arithmeticとbranch
- direct call、self-tail loop、deep non-tail unwind
- first-class call cycleとheterogeneous frame
- managed Bufferのread/write、`Symbol` comparison/concatenation、C host copy
- managed product、sum、closure environment、HostMappable valueのextern round trip
- checked-in example corpus

意味論fixtureとperformance fixtureを兼用してよいが、期待resultとresource invariantを先に固定する。performance差だけを理由に検証を弱めない。

## 記録

採用または棄却に将来の実装判断へ必要な情報がある場合は、日付、commit、環境、command、workload、raw measurement、判断を
`docs/history/performance/`へ記録する。active documentへ時系列statusを残さない。
