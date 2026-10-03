# continuation specialization

Status: Current implementation design

この文書は、specialized programで関数を返すproducerと、その関数を一度だけ適用するconsumerを融合し、途中のclosure
representationを消去するoptimizationの境界を定める。言語上のfunction valueと評価順は
[実行意味論](../spec/execution.md)、closure-converted programの現行責務は
[compilerの責務境界](responsibilities.md)、採択条件は
[generated program最適化policy](../development/generated-program-optimization.md)を正とする。

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

一つのvalueがsum payload、Buffer、host boundary、slice外のcaptureまたはfunction argument、あるいは二つ以上のapplicationへ流れる場合は
escapeとし、そのsliceを採択しない。source function valueを保存または返せる通常のpathは元programへ残す。局所productへのpackは、bindingを
持ち、そこから伝播する全field useが同じsliceのparameter、capture、call、resultへ閉じ、rewriteでproduct自体も消去する場合だけ内部transport
として扱う。aggregateへ入ったことだけを根拠にruntime保存とみなさず、未追跡field、wildcard storage、sum、Bufferへ流れる場合はescapeとする。

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

実装上は、sliceを二種類の内部edgeへ分けてから一度にrewriteする。

- resultへ到達し得るclosure targetが一つでcapture shapeも一つなら、function valueをそのcapture productへ置き換え、producer resultと
  対応するhost parameterを同じ型へ変える。applicationはtargetへのdirect callとし、capture productと元argumentをparameterにする。
- result joinやself recursionで複数targetが合流する場合は共通のruntime representationを作らず、上のdemand workerへ分岐ごとの
  capture productとargumentを渡して各targetを直接呼ぶ。

前者はslice内部のtransport表現であり、それだけを独立したoptimizationとして採択しない。`State`では`step`、`fmap<State>`、内側の
`bind<State>`が一targetのedge、`_foldFrom`の終了と再帰pathが複数targetの合流になる。両方を同じplanへ含めることで、`action`
parameterへ作成済みclosureを渡さず、最終workerだけを追加して内側のcreatorを残す失敗を避ける。

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

## 一般性と非責務

この変換は`State`、`bind`、`fmap`などのsource identityを認識しない。対象は、関数型resultに唯一のapplication demandがあり、
その間のproducer、join、callback、self recursionを閉じたsliceとして証明できる任意のprogramであり、planとrewriteの語彙へmonad
operationを持ち込まない。

一般的なinliningはsliceを露出させる前処理になり得るが、recursive resultのcalling conventionを単独では変えない。sourceをCPSへ
限定したり、特定のoperation familyをrewriteしたりしない。closure arena、stack allocation、runtime ABI、関数型全体の
representation変更も本stageの責務ではなく、closed sliceを証明できないprogramは元のclosure semanticsを使う。採択時の代替案、
検証結果、測定値は
[genericsとmanaged container](../history/performance/generics.md#continuation-specializationの採択)を正とする。
