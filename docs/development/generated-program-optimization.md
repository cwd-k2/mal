# generated program最適化policy

Status: Current policy

この文書はLLVM backendとchecked-in C runtimeの最適化を採用するgateを定める。言語semanticsは[`spec/`](../spec/)、control表現は
[application control lowering](../implementation/application-control-lowering.md)、検証commandは[test policy](testing.md)を正とする。
backend世代ごとの測定と判断は[performance history](../history/performance/backend/)に保存する。

## 優先順位

1. result、effect order、trap、owner lifetime、bounded native stackを保持する。
2. program固有のpolicyをLLVM、program非依存のmechanismをC runtimeに置く責務境界を保持する。
3. 実行時間、allocation、code size、compile timeのうちworkloadで支配的なcostを減らす。

未計測の複雑化、特定fixtureだけのspecial case、LLVM optimizerが既に安定して行う局所変換の再実装は採用しない。

## 構成

最適化は対象のauthorityを持つstage内で個別techniqueとして定義する。`execution/optimization`はcontinuationとcall selection、
`backend/llvm/optimization`はadmitted execution planを変更しないtarget固有のemission decisionを所有する。driverは各stageへ有効な
technique集合を明示的に渡す。後段はtechnique identityではなく、所有stageが検証したplanだけを読む。

`call_pattern`は例外的にprogramを書き換えるtechniqueであり、`execution/optimization`のdecisionでは表せないため独自のstageに置く。書き換え後のprogramは、binderとatomの識別子が一意である不変条件をdebug buildで検査する。
`execution/native_recursion`も例外であり、recursive region、call mode、frameのplanから決まるため、それらの入力となる`OptimizationPlan`には
置けない。これらのplanの後に独自のplanとして構成し、同じ入力から再構成した結果との一致をdebug buildで検査する。

空のtechnique集合は全admitted programを実行できるbaselineである。production集合は採用済みtechniqueの明示的な合成であり、別の
意味論や別のbackend contractを持たない。新しいtechniqueが既存plan型または無関係なstageの変更を要求する場合は、optimization追加ではなく
authority境界の変更として先に検討する。

execution ownership planではowner lifetimeのfactとresponsibilityの後継を構成し、唯一のowner successorへのhandoffを設定によらず
`Consume`へ正規化する。LLVM backendはこのplanとstorage再利用のdecisionを分ける。dead responsibilityの`Drop`は設定によらず行い、
`Symbol` operationとbyte `*`が移されたresponsibilityのstorageをruntime representationとcapacityに基づいて再利用する選択だけをoptional
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

## 言語間比較の基準

CやRustとの比較は、同じ表に置けるという理由だけで同じ意味の比較として扱わない。各比較実装を次のどちらかに分類し、
primaryな倍率はsemantic parity同士から出す。

- semantic parityは、共有identity、更新の観測、carrier lifetime、必要なinputとoutput storage、algorithmを揃える。
  source上で同じ抽象を表すだけでなく、一方だけが既知のinputを定数化してstorageを消さないよう、全実装で同じruntime input boundaryを使う。
- hand-lowered lower boundは、高階値、managed identity、tagged representationなどを人手で除いた実装である。到達可能な下限と
  残存costの大きさを示すが、言語またはcompiler間のprimaryな速度比には使わない。

representation差そのものを調べる場合は、同じ意味を保つ複数のreferenceを置く。例えばnested identityでは、headerとpayloadを
一allocationへ詰めたreferenceだけでなく、stable objectとrelocatable backingを分けたreferenceも測り、shared identityの費用と
allocation policyの費用を分離する。比較対象のoptimizerが一方のstorageだけを消した場合は、その値を削除せずlower boundとして
分類し直す。測定対象に不利な処理をreferenceへ足して倍率を整えることも、意味上必要な処理を測定対象から外すこともしない。

## 再調査の入口

現在の生成programで再調査する境界は次の三つである。測定根拠、不採択案、再現値は
[genericsとmanaged container](../history/performance/generics.md)を正とする。

1. `State`はdictionaryやgeneric dispatchではなく、関数を返すproducerから最終applicationまでのclosure representationが残る。
   result join、callback call、self-recursive edgeを一つのcontinuation demandとして証明し、`_foldFrom`、`bind<State>`、step callbackを
   新しいcreatorを残さず融合できる場合だけdeforestする。単独のresult-application workerは再試行しない。
2. nested Bufferはstable objectとrelocatable backingの二allocationが残る。任意capacityのco-allocationは再試行せず、growthしないこと、
   byte ownerへ移されないこと、独立identityのlifetimeを壊さないことを通常planが証明できる場合に限り、conditionalなrepresentationを
   比較する。まずpacked referenceとsplit referenceの両方を用意し、意味論costとallocation policyを分ける。
3. scalar control loopはgeneric、sum、closure、allocationが最終IRから消えた後も、LLVM 21のPHI順序で8または13 instructionになる。
   明示PHIを含むMal側の順序付けはLTOで保存されなかったため追加しない。LLVM更新時に同じreproducerを再測定する。

`State`のdirect C/Rust loopはhand-lowered lower boundでありsemantic parityではない。`focus`もCだけが既知input storageを消した測定を
primary比較にしない。次回のcross-language更新では、この分類をtableに明記してからwall-clock、instruction、allocation、peak memory、
最終IRを採る。

## 記録

採用または棄却に将来の実装判断へ必要な情報がある場合は、日付、commit、環境、command、workload、raw measurement、判断を
`docs/history/performance/`へ記録する。active documentへ時系列statusを残さない。
