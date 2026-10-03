# genericsとmanaged containerの生成物

Status: Historical measurement record

この文書は、2026-10-03のLLVM backendでgenerics、higher-kinded operation family、nested Bufferが
どこまで消去され、どこにruntime costが残るかをCおよびRustの対応programと比較した記録である。
言語規則は[generics](../../spec/generics.md)、[operation family](../../spec/operation-families.md)、
[AddressとBuffer](../../spec/memory.md)を正とし、Buffer単体の以前の測定は
[Buffer生成物とownership cost](buffer.md)を参照する。

## 測定条件

対象は`c5ee965e`、malc 0.6.0-dev、Clang 21.1.8、Rust 1.97.1、Valgrind 3.27.1、
Intel Core Ultra 7 258Vである。MalとCは`-O2 -flto`、Rustは`-C opt-level=3 -C lto=fat
-C codegen-units=1 -C panic=abort`でbuildした。全buildと測定processをsystemd cgroupの2 GiBに制限した。

同じ最終checksumを得る五つのworkloadを用意した。

| Case | 反復と意味 | 比較の分類 |
|:---|:---|:---|
| `control` | genericなsum protocolによる1,000万回のLCG fold | semantic parity。C loop、Rust iterator foldと同じscalar recurrence |
| `nested-buffer` | 65,536個の8要素identityを外側containerへ保持し、半数をaliasへ置換して変更 | semantic parityだがrepresentation contrast。CとRustはheaderとpayloadを一allocationに詰める |
| `map` | generic operation familyを使う131,072 entryのopen-addressing map | semantic parityだがrepresentation contrast。同じcapacity、hash、slot algorithmを使う |
| `state` | `State<USize>`と`foldEach<State<USize>>`で20万要素を畳む | hand-lowered lower bound。CとRustは関数値representationを持たない直接loop |
| `focus` | `extend<Focus>`で50万要素の3点移動平均を作る | mixed lower bound。Cだけが既知inputを定数化して一方の配列を消した |

`state`のC入力は`volatile` storage、Rust入力は`black_box`を通し、closed expression全体の定数畳み込みを防いだ。
最初のC版が5 instructionまで定数化されたためである。Rustは`std::process::exit`ではなく`main -> ExitCode`を使い、
局所containerを通常どおりdropしてから終了する。前者を使った初回測定では`focus`の2 MB allocationが解放されず、
言語間比較ではなくharnessの欠陥になった。

wall-clockは3 warmup後、Mal、C、Rustの順をroundごとに回転して20回測った。次表はmedianである。
Callgrindは`main`からcollectionを始めた。Rustの`main`には標準runtime初期化が約10万instruction含まれるが、
反復量の大きいcaseの結論には影響しない。

表の倍率は測定結果の記録であって、五件を一つの言語rankingへまとめるものではない。`state`と`focus`は上表のlower boundであり、
Malに対するprimaryなcross-language比ではない。`nested-buffer`も次回はpacked referenceだけでなくstable objectとbackingを分けた
referenceを併記し、shared identityとallocation policyのcostを分離する。

| Case | Mal | C | Rust | Mal / C | Mal / Rust |
|:---|---:|---:|---:|---:|---:|
| `control` | 3.86 ms | 1.95 ms | 3.89 ms | 1.98x | 0.99x |
| `nested-buffer` | 7.85 ms | 3.60 ms | 4.05 ms | 2.18x | 1.94x |
| `map` | 6.16 ms | 5.47 ms | 4.73 ms | 1.13x | 1.30x |
| `state` | 4.89 ms | 1.57 ms | 1.68 ms | 3.11x | 2.92x |
| `focus` | 3.93 ms | 2.64 ms | 3.72 ms | 1.49x | 1.06x |

絶対時間はprocess起動とCPU frequencyの影響を受ける。特に`state`はnative allocatorが小さいallocationを速く処理するため、
Callgrind上のinstruction比ほどwall-clock差は大きくない。採択判断には時間だけでなく、次の命令、allocation、IRを使う。

## 動的instruction

| Case | Mal | C | Rust | Mal / C | Mal / Rust |
|:---|---:|---:|---:|---:|---:|
| `control` | 16,250,027 | 10,000,013 | 16,350,163 | 1.63x | 0.99x |
| `nested-buffer` | 51,918,265 | 21,881,457 | 22,390,932 | 2.37x | 2.32x |
| `map` | 7,603,640 | 3,671,943 | 4,382,386 | 2.07x | 1.74x |
| `state` | 106,588,002 | 717,022 | 800,806 | 148.65x | 133.10x |
| `focus` | 46,425,192 | 24,145,664 | 35,507,544 | 1.92x | 1.31x |

`control`は最終LTO moduleが`main`一つ、callとallocationが0になり、generic callback、sum branch、closure carrierは残らない。
CとMalのpre-codegen IRはいずれも同じrecurrenceを8 step展開するが、最終machine loopはCが8 instruction、Malが13 instructionである。
後続調査では、二つの最終IRに残る漸化式そのものではなく、loop headerに並ぶ二つの`phi i64`の順序まで原因を狭めた。Cはindex、state、
Malはstate、indexの順である。Malの最終IRでこの二行だけを入れ替えて同じLLVM 21へ渡すと、演算やmetadataを変えずにCと同じ
8 instruction形になった。exit blockの配置、`llvm.loop.mustprogress`、`min-legal-vector-width`、callbackの`alwaysinline`は結果を
変えなかった。したがって差はruntime generic dispatch、sum、closure、loop stateの意味、あるいはMalの漸化式の消去失敗ではなく、
同値なSSAのPHI worklist順序に対するLLVM MachineCombinerの感度である。Rustの命令数はMalとほぼ同じだった。

現行backendは自己末尾parameterをentry allocaへ分解して更新し、LLVMのSROAとmem2regが後からPHIを作る。この経路ではsource productの
field順を最終PHI順として保証できない。後続試作では、自己末尾parameterのunmanaged product leafをbackendでsource field順の明示PHIへ
変え、managed leafだけを従来slotへ残した。pre-LTO IRはindex、state、保存値の順になったが、LTO後にLLVMが再びstate、indexへ
並べ替え、machine loopは13 instructionのままだった。この試作は複雑さだけを増やすため撤回した。store順の反転、無意味な先行load、
明示PHI順のいずれも最終machine combineを制御するcontractではない。

従ってこの差をMalのself-tail lowering defectとして独自に矯正しない。同じLLVM versionで最終SSAの二行を交換すると8 instructionになる
ことはoptimizer regression用のreproducerとして残せるが、source semanticsやbackend IRへPHI順序の疑似contractを追加する根拠には
ならない。実行時間と命令差は引き続き比較表へ記録し、LLVM側のcanonicalizationが変わったときに再測定する。

`map`も最終moduleは`main`、trap、Buffer destructorの3 definitionだけで、operation dictionaryやtype inspectionはない。
初期化、put、getはdirect loopへinlineされている。Malのinstruction差は48-byteのtagged slot、Buffer countとprobe終了条件、
trap可能なstorage pathにあり、generic operation familyのdispatch costではない。native medianがCの1.13倍に留まることも、
specialization後のhot pathが直接実行されていることと整合する。

`focus`はpre-LTOの11 definitionが3 definitionになり、closure environment allocationは残らない。`extend<Focus>`とrule callbackは
二つのBuffer loopへinlineされた。Cは全要素が定数3であることから入力配列のallocationを消し、MalとRustは入力と出力を保持するため、
Cを意味論上必要なmemory costとはみなさない。Rustと比べた1.31倍のinstruction、1.06倍の時間が、同じ二配列を保持した場合の
より近い比較である。

## allocationとmemory

Memcheckのrequested bytesとallocation回数、Massifで同時にliveだったheap、allocator overhead、native stackの合計を示す。

| Case | 実装 | Allocations | Requested bytes | Peak total | Native peak RSS |
|:---|:---|---:|---:|---:|---:|
| `control` | Mal / C / Rust | 0 / 0 / 9 | 0 / 0 / 2,620 | 7,744 / 7,744 / 7,744 B | 1,440 / 1,436 / 2,296 KiB |
| `nested-buffer` | Mal / C / Rust | 131,074 / 65,537 / 65,546 | 10,485,872 / 5,242,880 / 6,294,076 | 12,059,216 / 6,235,472 / 7,334,320 B | 13,084 / 6,936 / 8,640 KiB |
| `map` | Mal / C / Rust | 2 / 1 / 10 | 6,291,544 / 6,291,456 / 6,294,076 | 6,295,968 / 6,295,896 / 6,296,712 B | 7,452 / 7,448 / 8,188 KiB |
| `state` | Mal / C / Rust | 600,003 / 1 / 10 | 38,400,112 / 1,600,000 / 1,602,620 | 1,601,264 / 1,600,408 / 1,601,256 B | 2,972 / 2,968 / 3,572 KiB |
| `focus` | Mal / C / Rust | 4 / 1 / 11 | 4,000,176 / 2,000,000 / 4,002,620 | 4,000,768 / 2,000,424 / 4,001,344 B | 5,276 / 3,352 / 5,872 KiB |

MalとCは全allocationを終了時までに解放し、Memcheck errorは0だった。Rustは全caseで標準runtimeのthread情報544 bytesを
still reachableとして残すが、definitely、indirectly、possibly lostはいずれも0だった。表のRust allocationにはこの固定costを含む。

`nested-buffer`のMalは各inner Bufferについてstable objectと64-byte payloadのflat ownerを別々に確保するため、
innerごとに2 allocationとなる。外側Bufferもobjectとbackingの2 allocationで、合計は`2 * 65,536 + 2`である。
Cはreference countと8要素を一つのobjectに置き、Rustの`Rc`も一つのallocationへ置くため約半数になる。CallgrindではMalの
65,537回ずつの`malloc`と`calloc`が全instructionの約60%、終了時のmanaged destructionとalias置換時のreleaseが約27%を占めた。

これはshared identityや`Storable(Buffer<A>)`の下限ではなくrepresentation costである。ただし
[small-buffer storage](buffer.md#small-buffer-storage)で記録したとおり、任意の初期capacityをstable object末尾へ置く案は、
allocationを半減してもcache missを増やし、対象workloadの時間を改善しなかった。nested identityをflat valueへ変えることもできない。
compact header、size class、arenaなど別のallocation policyは候補だが、現測定だけからruntime全体へ導入しない。

`map`ではMalのobjectとbackingを分ける96 bytes以外、Cとrequested bytesおよびpeakがほぼ同じである。
`focus`のMalとRustも約4 MBで一致する。したがってBufferが常に大きいmemory倍率を課すのではなく、多数の小さい独立identityを作る
`nested-buffer`でstable objectの固定costが顕在化する。

## State monadで消えないもの

`state`ではgeneric constructor、`pure`、`bind`、`fmap`のinstance選択はcompile時に完了し、runtime dictionaryはない。
それでもStateのrepresentation自体が`S -> (S, A)`であり、各stepはcaptureを持つState actionを返す。
specializationとdirect-call selectionだけでは、返された関数値のenvironment lifetimeを消せない。

continuation specialization着手時の`108b6c4f`でも同じfixtureをrelease buildし直し、20 run、3 warmupでnative median
5.04 ms（4.53--6.86 ms）、Callgrind 106,588,002 instructions、Memcheck 600,003 allocations、38,400,112 requested bytesを
再確認した。errorは0で全heap blockを解放し、pre-LTO IRは50,534 bytes、19 definitionsのままである。wall-clockの差は測定揺れの
範囲であり、以下のallocationとIRを変換前baselineとする。

pre-LTO IRには19 function definition、3箇所のenvironment allocation siteがあり、LTO後にも11 definitionと24-byteまたは
80-byteの`malloc` pathが残る。20万stepで3個ずつ、Buffer本体を含め600,003 allocationとなった。Callgrindではhot worker
`mal_function_26`が全instructionの98.5%を占め、そこから20万回のowner allocation、40万回の`free`、20万回の次action生成を呼ぶ。
さらにcallee内のallocationとreleaseを合わせ、requested bytesは38.4 MBになる。同時liveなのは短命environmentと1.6 MBの入力Bufferなので、
peakは約1.6 MBに留まる。

closure ASTをspecializationと現行`call_pattern`の後で監査すると、`bind<State>`は`action`、`next`などをcaptureしたactionを返し、
`_foldFrom<State>`はその関数値を再帰resultとして運び、最外の`main`だけが完成したactionを適用していた。creatorとdirect callが同じ
local alias graphにあるclosureを対象にする現行lambda liftや、known input parameterを辿るparameter liftの証明範囲には入らない。
最後のcreatorだけをtupleへ変えても、20万回の遷移で繰り返すenvironment生成は別表現で残る。

この差を解く変換は、operation family specializationの追加ではない。関数を返す`bind` / `fmap`、再帰resultを運ぶ`foldEach`、最終的に
関数を適用するconsumerを一体にしたclosure deforestation、またはcaptureとapplicationを通常parameterへ変えるwhole-program rewriteが
必要である。
arbitraryなState actionを保存または返せる意味は保ち、最終consumer以外へescapeしないspecialized chainだけを対象にしなければならない。
environmentを一律stackへ置くことや、reference countを一律省くことはlifetimeを証明しないため不正である。

既存`ClosureFlow`のauthorityは、call siteへ到達し得るfunction codeを`FunctionId`の集合として求めるところまでである。同じfunctionを
異なるcaptureで生成したclosure instanceは意図的に同一視するため、callee selectionには十分でもenvironment消去の証明にはならない。
continuation specializationには別の解析として、creator identity、result / join / parameterを通る伝播、唯一のapplication demand、
escape不在を保持する必要がある。この解析を一つのoperation variant一覧に基づかせるため、closure ASTへdirect atom operand traversalを
集約し、call-pattern内の重複実装を削除した。この責務整理だけでは最適化を有効にせず、`state`は引き続きpre-LTO 50,534 bytes、
19 definitions、600,003 allocations、38,400,112 requested bytesである。

### result application workerの不採択

同日に、`f(argument)`が返す関数値をuse-count 1のaliasだけを経て適用する形について、元wrapperを残したまま
`(argument, application)`を受け取るworkerを複製する`call_pattern`変換を試した。非再帰の`makeAdder` fixtureではLTO後の
allocationを1回、20 byteから0へ減らし、ELF textも2,127 byteから1,455 byteへ減らした。しかしこの局所形はStateの支配costを
表していなかった。

`_foldFrom<State>`のcompletionはbody末尾ではなく`[return]`由来のjoinを通る。sink joinまでapplicationを伝播した初版はobservableと
leak検査には通ったが、Stateを600,003回、38,400,112 byteから800,003回、41,600,112 byteへ悪化させ、動的instructionも
106,588,003から132,588,425へ増やした。worker複製後に通常のcall-pattern specializationを再び固定点まで実行すると追加20万回は消えたが、
allocationは600,003回のままで、pre-LTO definitionは19から25、ELF textは4,811 byteから5,595 byteへ増えた。joinを持つproducerを
除外した版も600,003回のままであり、Stateには効果がなかった。この変換は実装から撤回した。

失敗の原因は単独のfunction resultをworker化しても、`_foldFrom`、`bind<State>`、次stepを作るcallbackを一つのcontinuationとして
融合していないことにある。必要なのはcreator単位のlambda liftではなく、result application demandをresult join、callback call、
self-recursive edgeの全体へ伝えるcontinuation specializationである。新しいclosure creatorを途中に残さず、変換後の全経路について
producerとconsumerのeffect order、completion、escape不在を同時に検証できる形でなければ採択しない。

## LLVM IRとbinary

Mal compilerが出力したpre-LTO IRと、runtime Cを含めたLLDのpre-codegen bitcodeを再びLLVM textへ出した結果である。

| Case | Pre-LTO bytes / definitions | LTO bytes / definitions / calls | Mal text | C text | Rust text |
|:---|---:|---:|---:|---:|---:|
| `control` | 19,717 / 6 | 4,080 / 1 / 0 | 1,633 B | 1,262 B | 295,232 B |
| `nested-buffer` | 19,248 / 6 | 48,851 / 6 / 77 | 5,087 B | 1,756 B | 295,540 B |
| `map` | 65,486 / 14 | 23,988 / 3 / 15 | 3,117 B | 1,631 B | 295,600 B |
| `state` | 50,534 / 19 | 48,518 / 11 / 83 | 4,795 B | 1,549 B | 295,536 B |
| `focus` | 39,417 / 11 | 41,778 / 3 / 60 | 4,969 B | 1,946 B | 296,672 B |

LTO IR bytesはruntime helperのinlineとlibc declarationを含むため、pre-LTOから増える場合があり、性能指標にはならない。
definitionの減少と、最終IRに残るallocation、direct/indirect callを対応づけるdiagnosticとして使う。Rust textは静的にlinkされた
標準runtimeを含み、Mal/Cの小さいC entryと同条件のcode-size比較ではない。Rust間の固定cost確認のため記録した。

## 現行example corpus

今回のsynthetic workloadだけでなく、generic機能を主に使う現行example 5件を同じ2 GiB制限でbuildし、実行した。
`monads-and-comonads`には`duplicate<Focus>`を追加し、opaque `Focus<A>` carrierをnested Bufferに保持して、元のinner Buffer identityを
共有する経路まで通した。全exampleはexit 0、Memcheck error 0、終了時live allocation 0だった。

| Example | Instructions | Allocations / requested | Peak total | Pre-LTO IR bytes / definitions | ELF text |
|:---|---:|---:|---:|---:|---:|
| `language-tour` | 6,082 | 5 / 4,260 B | 7,744 B | 32,747 / 13 | 3,427 B |
| `control-and-iteration` | 4,001,265 | 2 / 112 B | 7,744 B | 48,597 / 11 | 3,708 B |
| `generic-map` | 7,535 | 9 / 890 B | 7,744 B | 218,704 / 33 | 11,925 B |
| `monads-and-comonads` | 110,918 | 42 / 4,136 B | 7,744 B | 502,629 / 112 | 28,112 B |
| `indexed-graph` | 7,389 | 15 / 1,016 B | 7,744 B | 122,428 / 15 | 20,356 B |

小さいexampleのwall-clockはprocess起動が支配するので記録しない。pre-LTO IRはspecialized instanceを明示するため、特にoperation familyを
多数組み合わせる二件で大きい。一方、実行時memoryは全件でnative stackを含む7,744 bytes以下のsnapshotが最大であり、
source abstractionの数に比例するdictionaryやtype descriptorは存在しない。

## モデルと実装課題の分離

今回の結果から、genericsやmonadのために新しいruntime根本モデルを加える理由はない。`control`、`map`、`focus`は既存の
specialization、call-pattern rewrite、LTOで型と高階dispatchを消せる。`State`は型の消去ではなく、関数を値として返すrepresentationの
消去が未実装である。これはPoolのoccupancy、authority、coordinate modelとも、Buffer elementの`Storable`判定とも別の問題である。

`Storable`は「placeがcarrier lifecycleを完結できるか」というsource admission、`Lifecycle = Trivial | Owned`はその実装計画、
`Representable`はcanonical memory copy、`HostMappable`はpublic C boundaryという現在の分離を保つ。
`duplicate<Focus>`がnested opaque carrierとして動いたことは、`Managed`を`Storable`の代わりのsource predicateへ持ち上げずとも、
representationとlifecycleを再帰できることを確認している。

この測定から残った実装境界は次の三つである。再調査の順序とcross-language比較の現在のgateは
[generated program最適化policy](../../development/generated-program-optimization.md#再調査の入口)を正とする。

1. `State`のspecialized producer-consumer chainについて、単独のresult-application worker案は不採択とする。関数型resultがresult join、
   callback call、self-recursive edgeをどう通るかを一つのcontinuation demandとして証明し、`_foldFrom`、`bind<State>`、step callbackを
   同じworkerへ融合できる場合だけdeforestする。copy budget、identity一意性、exact validatorに加え、変換後に新しいcreatorがhot pathへ
   残らないことをallocation fixtureで採択条件にする。
2. nested Bufferはplain identityから不要な`stride`を除いた後も残るobjectとbackingの二重allocationを対象にする。既に退けた
   任意capacity co-allocationを繰り返さない。byte ownerへ変換されないこととaccess patternを区別できるprogram factを得てから別案を測る。
   shared identity、独立lifetime、growth後のdata pointer再取得を保つ。
3. `control`の13-instruction recurrenceはPHI順序だけでCの8-instruction形へ変わるが、明示的なsource field順のPHIもLTOが
   並べ替えることを確認した。Mal IRへheuristicな順序付けを追加せず、LLVM optimizerの比較reproducerとして監視する。

IxPoolの非公開runtime kernelはこれらを解くための汎用allocatorやclosure arenaへ拡張しない。Pool source semanticsとLLVM loweringを
導入する時点では、今回のBuffer lifecycle、allocation分布、generic erasureを比較基準に使うが、無関係なoptimizer責務をIxPoolへ
集めないことがminimalityである。

raw CSV、Callgrind、Massif、Memcheck、pre-codegen bitcode、比較source、Nushell runnerはignored
`.scratch/performance/generics-audit/`に保存した。
