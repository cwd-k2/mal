# continuation specialization

Status: Draft implementation boundary

この文書は、specialized programで関数を返すproducerと、その関数を一度だけ適用するconsumerを融合し、途中のclosure
representationを消去するoptimizationの境界を定める。言語上のfunction valueと評価順は
[実行意味論](../spec/execution.md)、closure-converted programの現行責務は
[compilerの責務境界](../implementation/responsibilities.md)、採択条件は
[generated program最適化policy](../development/generated-program-optimization.md)を正とする。

## 対象

最初の対象は、concrete specialization後の`State` chainである。`foldEach<State<USize>>`が返すactionは最終consumerで一度だけ
適用されるが、現行programには次の三種類のclosure creatorが反復ごとに残る。

- `step`と`fmap<State<USize>>`が返すaction。
- `bind<State<USize>>`がactionと次のproducerを保持するaction。
- `_foldFrom<State<USize>>`の終了pathで`pure<State<USize>>`が返すaction。

単独のfunction resultだけをworkerへ変えても、callback callとself-recursive resultを通るcreatorが残る。この形は既に測定上
不採択であり、[genericsとmanaged container](../history/performance/generics.md#result-application-workerの不採択)に記録している。

## 所有stage

`continuation_specialization`をclosure conversionと`control` loweringの間に置く。このstageは、関数型resultへ到達する唯一の
application demandを解析し、閉じたproducer-consumer sliceを別のcalling conventionへ複製する。既存`call_pattern`はinputの
closure集合に基づく複製とcaptureのparameter liftを引き続き所有し、その完了後のprogramを本stageへ渡す。

この分離により、`call_pattern`の固定点とgrowth budgetへresult demandを混ぜない。`flow`の`ClosureFlow`は到達し得るfunction codeの
集合を提供してよいが、creator instance、唯一のapplication、effect intervalを証明するauthorityにはしない。

本stageもoptionalなprogram rewriteである。`Technique::ContinuationSpecialization`が無効なら入力programをそのまま返し、
`baseline`のadmission、意味、実行可能性を変えない。

## Continuation demand

continuation demandは、関数型valueに対する一つのapplicationを、そのargument型とresult型、およびapplication siteのidentityで表す。
解析はconsumerからproducerへ次のedgeだけを逆向きにたどる。

1. binding間の`Atom` aliasとblock result。
2. `Goto`と、そのtargetであるresult joinのparameter。
3. function callのresultとcallee bodyのresult。
4. self-recursive callのresultと同じspecialized functionのresult。
5. `MakeClosure`と、作られるfunctionのbody。
6. demanded worker内で直接適用されるfunction parameterと、そのworkerを呼ぶ全siteの対応argument。

一つのvalueがaggregate、sum payload、Buffer、capture、host boundary、別のfunction argument、または二つ以上のapplicationへ流れる場合は
escapeとし、そのsliceを採択しない。source function valueを保存または返せる通常のpathは元programへ残す。

関数parameterからcall site argumentへ進むedgeは、対象hostの全call siteが同じcall-pattern copyへ解決され、各argumentのproducerが
閉じたsliceへ含まれる場合だけ作る。一部のcall siteだけをworker signatureへ合わせず、runtime tagやnullable closureで欠けたcaseを
表現しない。

## Effect interval

producer callと最終applicationの間では、`Atom` alias、pattern分解、result join transfer以外のoperationを越えない。applicationの
argumentはproducer callより前から利用可能なatom、またはliteralでなければならない。これにより、producerとargumentの評価、extern、
trap、Buffer operation、managed valueの取得と解放を前後へ移動しない。

callback body内で次のproducerを得て直ちに適用する場合も同じ規則を再帰的に使う。途中に観測可能なoperationがある場合は、そのoperationを
worker body内の元の位置に保てることをsliceが示さなければ不採択とする。operationをpureと推測したり、LLVMの最適化結果から
effect不在を逆算したりしない。

## Rewrite

`f : A -> (P -> R)`に閉じたdemand `p : P`がある場合、元の`f`を残したまま
`f$demand : (A, P) -> R`をfresh identityで作る。worker内では次の規則を同時に適用する。

- resultが`MakeClosure(g, captures)`へ到達するpathは、closureを作らず`g`のbodyを`captures`と`p`で実行するworker callへ変える。
- resultが`h(a)`へ到達するpathは、`h$demand(a, p)`へ変える。
- result joinは各predecessorから同じdemandを受け、function valueではなく適用後の`R`を合流する。
- demanded worker内のfunction parameter applicationは、call-patternで確定した各argument producerのworkerへ接続する。
- self-recursive result edgeは同じworker identityへ戻し、各反復でclosure creatorを作らない。

workerは元のparameter、capture、join、atom、function identityを再利用しない。複製には既存のfresh identity allocatorとcopy budgetを
共有し、budget内でslice全体を作れない場合は部分rewriteを行わない。元wrapperはslice外の通常callのために残してよい。

rewrite後のhot sliceには、置き換え対象だったfunction valueを作る`MakeClosure`を一つも残さない。creatorをcapture tuple、別のclosure、
またはruntime tagへ置き換えるだけの変換は完了とみなさない。

## Validation

planはrewrite前programから再構成可能な次のfactを保持する。

- consumer applicationと、その唯一のproducer path。
- 通過するresult、join、callback parameter、self-recursive edge。
- sliceに含まれるcreatorとcall siteの完全な集合。
- application argumentが利用可能になる位置と、越えてよいeffect interval。
- 元functionからworkerへの対応とcopy budget。

debug buildのexact validatorは元programから同じplanを再構成し、rewrite後programについてfresh identity、型、全call siteのsignature、
slice内creator不在を検査する。validatorがrewriteの正しさを新たに推論するのではなく、admissionが使ったauthorityと出力の一致だけを
確認する。

## 最初の採択gate

実装は次を同じ変更で満たした場合だけproduction集合へ加える。

1. 非再帰のproducer、result join、callback、self recursionを個別のfocused testで検査し、escape、複数consumer、effect interval、
   不完全なcall-site集合をnegative caseにする。
2. `State` fixtureで`baseline`と同じresult、trap、effect trace、終了時owner状態を得る。
3. `State`の反復hot pathから対象creatorが消え、allocationが反復数に比例しないことをartifactとallocation counterで検査する。
4. productionで既存example corpusを実行し、Memcheck errorと終了時live allocationを0に保つ。
5. pre-LTO definition、最終IR、動的instruction、wall-clockを再測定し、変換前およびhand-lowered lower boundと区別して記録する。

最初の実装では任意のcontinuation calculus、closure arena、stack allocation、runtime ABI、関数型全体のrepresentation変更へ広げない。
上記のclosed sliceを証明できないprogramは元のclosure semanticsを使う。
