# generated program最適化計画

Status: Implemented decisions and follow-up gates

この文書はreference C backendが生成するprogramのruntime性能について、改善軸、着手順、各段階の完了条件を管理する。
現在の測定結果は[generated C performance](performance.md)、managed valueの正しさは
[Engram ownership](../implementation/ownership.md)、言語とhostのauthorityは
[EngramとExtern](../spec/engrams.md)を正とする。

## 目標と非目標

目標は、source semanticsとauthorityを変えず、生成物に残る不要なaggregate構築、retain/release、allocation、tag分岐を
減らすことである。C sourceの短さやcompiler固有の未定義動作は目標にしない。

次を全段階の制約とする。

- operandとeffectの評価順を保持する。
- source preconditionを満たすprogramへ新しいfailureを加えない。
- `Symbol`とclosureのidentity、到達可能性、lifetime authorityをmalに残す。
- `Ptr`のregion、permission、alias、alignmentをbackendが推測しない。
- first-class function callとextern ABIに必要なgeneric representationを、direct callだけの測定から削除しない。
- 最適化後のLLVM IR、deterministic counter、wall-clockの少なくとも二つで変更理由を確認する。

## 観測したcostの分類

| 軸 | 現在残るcost | 主なcorpus |
|---|---|---|
| managed lifetime | 保存またはreturnされる値には必要なshareとreleaseが残る | managed valueを持つ一般program |
| recursive reduction | 非tail再帰の結果を結合するhelperがcallとして残り、C optimizerがloopへ変換できない | 016 |
| aggregate call topology | 複数scalarからなるstateをdirect call間で受け渡す | 032 |
| `Symbol` lifecycle boundary | admission後のdescriptor releaseをruntime helperへ渡す | 027 |
| scalar memory contract | unalignedかつalias可能な`Ptr` accessと明示的なwrap/trapを維持する | 005などの数値・table workload |
| first-class representation | 動的に選択されるclosureとlarge product resultに一般表現を使う | synthetic case |

managed lifetime、call topology、`Symbol` lifecycleは現行authorityのまま不要な境界だけを改善できる。memory contractは
sourceまたはextern contractに新しい事実を表現しない限り変更しない。narrow integer representationも、全operationで値域と
wrap semanticsを証明できる独立解析なしには導入しない。

## A. borrow-preserving lowering

primitive、known direct call、pattern projectionのためだけに作られ、保存もreturnもされないaggregateを
`ephemeral aggregate`として扱う。そのfieldは元bindingからborrowしたままconsumerへ渡し、一時aggregateにowning shareを
作らない。

reference backendは、隣接するproduct構築とbyte access、memory operation、borrowed known direct call、直後のpattern projectionを
この経路へloweringする。product bindingのuseがconsumerで終わることをownership解析で確認できない場合と、owned entryへ
transferするcallはaggregate経路へfallbackする。

field expressionはsource orderで一度ずつ評価し、新しくowned resultを作るfieldはaggregateへcopyせずleaf temporary自身が
consumer完了までshareを持つ。consumer後は未transferのleaf temporaryを逆順でdestroyする。したがってaggregateのshareを
省略しても、borrow元または一時ownerが存在しない時間を作らない。

この規則は`Symbol`専用にしない。managed product、sum payload、closure fieldへ型再帰で適用する。consumerがresult、binding、
capture、tail parameterへ保存する場合は、現在のlast-use transferまたはcopy規則へ戻る。calleeを静的に特定できないcallと
extern resultにも適用しない。

完了条件は、flat `Symbol` byte scanのloopからretain/releaseが消え、alias再使用、branch、managed fieldを保存するnegative caseで
share不足を起こさないことである。

## B. aggregate stateのleaf slot化

product parameterを持つdirect entryでは、受け取ったleafからbody冒頭にproductを再構築せず、projectionをleaf localへ直接
解決する。direct self-tail loopもproduct全体ではなくleafごとのmutable slotを持つ。tail edgeは次のleafをすべて評価してから、
managed leafのcopyまたはtransfer、現在slotのdestroy、次slotへのinstallを行う。

generic closure entry、明示的に値として使うproduct、indirect callはstruct representationを維持する。leaf数の上限を超えるentryも
現在のaggregate fallbackを使う。これはsource productの表現変更ではなくknown direct pathのcalling conventionである。

reference backendは、function冒頭でproduct parameterを個別bindingへ分解し、各self-tail edgeが直前に同じarityの
productを構築する場合にbinding slotを使う。binding自身がnested productの場合はdirect entryのleaf引数からその値だけを復元し、
最外層のparameter全体は作らない。次のbindingをsource orderでtemporaryへ評価し、last-useのmanaged valueはtransfer、
それ以外はcopyしてから旧slotをdestroyする。この局所形状を満たさないtail edgeはaggregate stateへfallbackする。

完了条件は、対象loopの最適化後IRから不要なaggregate `alloca`、`memcpy`、`memset`が消え、managed productを含むtail edgeの
通常・sanitizer testが通ることである。

## C. `Symbol` observationのfast/slow分離

runtime descriptorは、連続bytesを直接指す値をnon-null `data`、未materialize ropeをnull `data`で区別する。byte accessと
equalityのsmall wrapperは連続値を直接処理し、no-inline slow pathだけがropeをmaterializeする。source-level index
preconditionは再検査しない。

同じlive `Symbol`をloopで観測するときは、borrow-preserving loweringによりdescriptorのretainを発生させない。extern parameterと
`storeSymbol`はcallまたはcopyの前に連続表現を一度確定し、そのborrowをoperation中だけ使う。

regression gateはflat、literal、rope、materialized ropeの同値性、flat observationのmaterialization count zero、
最適化後IRでsmall wrapperが消えてdata loadとslow callが分離されることとする。

## D. `Symbol` admission

extern resultのadmissionは、外部bufferを後からcopyする方式から、runtimeが確保した未公開bufferへtrusted adapterが書き、
完成時にimmutable `Symbol`としてpublishする方式へ置き換える。旧helperとの後方互換は維持しない。具体的なABI判断は
[D036](../design/decisions/D036.md)を正とする。

builderのreserveとconcatのunique flat buffer拡張は、同じcapacity growthとallocation primitiveを使う。hostは構築中bufferの
一時的なwrite capabilityだけを持ち、finish後のpointerを保持できない。外部bufferのadoptionと任意deallocatorの登録は行わない。

完了条件は、tokenごとのhost scratch allocationとadmission copyが消え、既知長・逐次grow・空・途中drop・allocation failureを
ABI testで検査し、027を再測定することである。

## E. branch、result、callの局所specialization

AからDの後にも残るhot aggregate resultだけを対象にする。候補は、全call siteが既知で、使用field集合が小さく、callee内のeffectを
一度だけ保てるfunctionである。result destinationへの直接書込み、case-of-constructorの簡約、単一call-site helperの限定的な
body統合を個別に比較する。

一律の`always_inline`、無制限のfunction clone、一般sum ABIの削除は行わない。完了条件は最適化後にも残る`sret`、tag branch、
間接callのいずれかをfocused fixtureで固定し、code sizeを含めて改善を説明できることである。

## F. allocatorとmemory contract

不要なretainとadmission allocationを除いた後もallocationが支配的な場合だけ、`MalContext`内のsize-class recyclingを検討する。
対象は`Symbol` flat storageとclosure environmentで、終了時live allocation zeroとpeak上限を維持する。region化、thread-safe RC、
GCへの置換はこの段階へ含めない。

alias、alignment、region分離を使う最適化は、既存`Ptr` contractから導けない。必要性が残る場合は、backend attributeを先に付けず、
sourceまたはextern contractで誰がその事実を選び保証するかを仕様課題として扱う。

## 現在の採否判断

AからDは実装済みであり、direct self-tail stateはnested product bindingを含めてbinding slotへ分割する。既知direct callだけで
使われるtop-level functionはclosure global、初期化、generic entryを生成せず、leaf数16以下のproduct direct entryには弱い
`static inline` hintを付ける。first-class useが一つでもあればgeneric representationを維持する。全functionへの
`always_inline`はcode sizeと他workloadを悪化させるため採用しない。

027ではadmission bufferのdataとcapacityをbyte loop外で保持するようhost fixtureを揃えたうえで、translation unit内だけで使う
`mal_symbol_release`を`static inline`にした。これによりC optimizerがinternal calling conventionと引数形状をspecializeできる。
allocationを無効化した実験の効果は差を支配しなかったため、Fのallocator recyclingは導入しない。

016ではdirect-only表現と限定的なinline hintにより差は縮んだが、非tail再帰の内側reduction callが残る。generic closure entryの
併設自体を除くだけでは変化しなかったため、closure表現を原因とはしない。次の候補は、effectと結合則を証明してreductionを
accumulator loopへ変換する独立した最適化、またはbudget付きの限定的なbody統合である。いずれも016専用の形をbackendで
推測せず、適用条件とcode-size gateを先に定める。

032に残るscalar stateのcall topologyは、caller/calleeを跨ぐSROAまたは限定的なbody統合の候補とする。005のmemory contractは
`Ptr`のalias、alignment、regionをsourceから導けず、wrap/trap semanticsも必要なため、attribute付与やnarrow integer化を行わない。
まず最適化後IRとcounterでどの制約が支配的かを分離し、新しい事実が必要ならlanguageまたはextern contractの仕様課題として扱う。
79問の測定値と比較fixtureの基準は[generated C performance](performance.md)を正とする。

今後新しいprofileが採用gateを満たす場合も、一つのcost modelごとにfocused generated-C test、native execution、通常のcompiler
testを通す。managed lifetimeへ触れる変更は[managed Engram性能](ownership-performance.md)の通常・sanitizer pressure suiteも通す。
ABI置換はgenerated header、host helper、全repository adapterを分けずに更新する。
