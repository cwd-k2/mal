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
| managed borrow | nested stateのprojectionからowned known callまでにshareを作る | 006、008 |
| aggregate state | 明示的に値として使うproductとdirect形状外のstateを構築する | 032、043 |
| `Symbol` admission | tokenごとにmal-controlled storageを確保する | 027 |
| branchとresult | first-class function valueに一般result表現が残る | 043 |
| call boundary | generic closure entryを併設する016で内側helperへのdirect callが残る | 016 |
| allocator | 短命なEngram allocationを汎用allocatorへ戻すが支配的ではない | 027 |
| memory contract | unalignedかつalias可能な`Ptr` accessがvectorizationを制約する | 数値・table workload |

managed borrow、aggregate state、`Symbol` admissionは現行authorityのまま改善できる。memory contractだけはsourceまたはextern contractに新しい事実を表現しない限り
変更しない。narrow integer representationも、全operationで値域とwrap semanticsを証明できる独立解析なしには導入しない。

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

AからDは実装済みであり、direct self-tail stateはnested product bindingを含めてbinding slotへ分割する。79問の再測定結果と
残差の根拠は[generated C performance](performance.md)を正とする。

現在のprofileでは、016の既知direct pathに内側helperへのcall boundaryが残る。限定的なbody統合はEの候補だが、
code sizeと適用条件を固定するfocused fixtureがまだないため、この変更には導入しない。027の`free`無効化によるwall-clock短縮も
約10%で差を支配しないため、Fのallocator recyclingは導入しない。
memory contractとnarrow integer representationは、source contractまたは独立した証明解析なしに変更しない。

今後新しいprofileが採用gateを満たす場合も、一つのcost modelごとにfocused generated-C test、native execution、通常のcompiler
testを通す。managed lifetimeへ触れる変更は[managed Engram性能](ownership-performance.md)の通常・sanitizer pressure suiteも通す。
ABI置換はgenerated header、host helper、全repository adapterを分けずに更新する。
