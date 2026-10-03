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

同じcommitとtoolchainで、State actionをsourceから除き、同じBuffer inputとscalar recurrenceをMalの直接self-tail functionへ
手でlowerした値も測った。これはsemantic parityではなくoptimizationの到達可能なlower boundである。native medianは1.43 ms
（1.23--2.58 ms）、Callgrindは2,136,603 instructions、allocationはBufferの2回、1,600,088 bytes、pre-LTO IRは10,969 bytes、
3 definitionsだった。errorは0で全heap blockを解放した。従ってcontinuation specializationが回収し得る主costは約98%のdynamic
instructionと反復ごとの600,001 allocationだが、この値へ合わせるためにfunction valueを保存できる元programの意味を狭めない。

既存のC `volatile` inputとRust `black_box` inputも同じ環境で再buildし、上の二つのMal binaryと一roundごとに実行順を回転して
20 runを採った。このrunner内のmedianはState Mal 10.22 ms、direct Mal 2.80 ms、C 2.71 ms、Rust 3.04 msだった。絶対時間は同日の
単独測定より高く、別runとの直接比較には使わないが、四者間ではdirect MalがC/Rustと同じ帯域にある。Callgrindはdirect Mal
2,136,603、C 715,562、Rust 800,902 instructionsだった。Cは1 allocation、1,600,000 bytes、Rustは標準runtime込み10 allocation、
1,602,620 bytesであり、以前の測定と一致する。これら三つのdirect形は引き続きhand-lowered lower boundであって、Stateの
semantic parity比較には分類しない。

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

### HaskellとCPS表現による原理の切り分け

2026-10-03にGHC 9.10.3で、同じ20万stepのrecurrenceを`State.Strict`、`State.Lazy`、rank-2 CPS encoding、直接strict foldで
比較した。Haskell版は`replicate count 1`のlistを共通inputとするためMal、C、Rustとのsemantic parity比較ではなく、GHC内で
function representationが残るかだけを見るprobeである。`-O2`、3 warmup、20 runの結果は次のとおりだった。

| Haskell representation | Native median | Callgrind instructions | RTS allocated bytes |
|:---|---:|---:|---:|
| `State.Strict` | 3.47 ms | 17,069,748 | 9,658,296 |
| `State.Lazy` | 3.60 ms | 17,067,347 | 9,657,928 |
| rank-2 CPS | 3.58 ms | 17,065,990 | 9,657,680 |
| direct strict fold | 3.85 ms | 17,067,332 | 9,658,056 |
| `NOINLINE` bind | 5.33 ms | 51,780,262 | 41,658,224 |

Tidy Coreでは先頭三形が直接版と同じunboxed self-tail workerへ集約された。strict版と直接版のallocation差は240 bytesであり、
listを含む固定cost以外にstep比例のState actionは残らない。一方、同じ`S -> (A, S)`表現でbind bodyだけを`NOINLINE`にした形は
32,000,168 bytesと34,710,514 instructionsを余分に使った。従ってGHCでの消去はlazy evaluationやCPS representation固有の効果ではなく、
producerとconsumerのbodyをsimplifierが同時に見て、recursive call patternをspecializeできた結果である。

個別flagも`-O0`、`-O1`、`-O2`および`-fno-specialise`、`-fno-spec-constr`、`-fno-worker-wrapper`、
`-fno-strictness`、`-fno-call-arity`、`-fno-cpr-anal`、`-fno-enable-rewrite-rules`を比較した。`-O1`の時点でState版と
直接版のallocation差は240 bytesになり、`-fno-specialise`、`-fno-worker-wrapper`、`-fno-cpr-anal`を個別に外しても同じだった。
`-fno-spec-constr`やrewrite rule無効化は両方のlist costを同量増やしたが、State固有のstep allocationを戻さなかった。このprobeから
GHC内部の単一passを必要条件とは断定しない。少なくともbindのunfoldingを越えるgeneral inliningと、結果需要を再帰workerへ伝える
simplificationの組合せが本質であり、State名を認識するrewriteは不要である。

同じ日にMal sourceを`StateC<S, R, A> = ((A, S) -> R, S) -> R`というCPS encodingへ書き換え、現行production optimizerでも
測定した。これは通常Stateより悪化し、1,400,003 allocations、64,000,112 requested bytes、228,168,552 instructions、native median
8.66 msとなった。pre-LTO IRも144,422 bytes、26 definitionsへ増えた。通常Stateは同じ交互20 runで4.60 msだった。CPS化だけでは
closureをresultからcallback argumentへ移すだけであり、現行call-pattern specializationはrecursive producer-consumer chain全体を
消さない。従ってsource APIをCPSへ変更する案は不採択とし、compiler側の一般的なclosed-slice deforestationを対象とする。

通常Stateから`pure`、`bind`、`fmap`、`foldEach`をすべて人手でinlineし、`_fold(values, index, sum)`が一つのaction closureを返して、
そのbodyから次の`_fold` resultを直ちに適用する形も測った。この形にはgeneric operationもcallback parameterもないが、各stepで
再帰resultのclosure一つが残った。allocationは200,003回、9,600,128 bytes、Callgrindは30,766,288 instructions、pre-LTO IRは
15,997 bytes、5 definitionsだった。同じ交互20 runのnative medianは2.73 msで、通常Stateの4.63 msより改善するが直接版の1.67 msには
届かない。局所inliningは三つあったstep closureを一つへ減らせるが、self-recursive functionのresult calling conventionを
`(arguments, state) -> result`へ変えない限り最後のstep比例allocationを消せない。

stack allocationも根本解にはしない。escapeしない短命environmentの`malloc` / `free`は減らせるが、反復ごとのenvironment構築と
applicationは残り、unbounded iterationをnative stackへ積むとbounded native stackのcontractも失う。有限tagへの
defunctionalizationも、payloadの生成と再帰的なcontinuation transportを残すだけなら同じである。tag、payload、applicationをまとめて
loop parameterへ変える時点でcontinuation specializationと同じclosed-slice証明が必要になる。

### continuation specializationの採択

2026-10-03に、`call_pattern`後のclosure programへcontinuation specializationを実装した。最終consumerから唯一のfunction-result
demandを逆伝播し、既知call、creator、局所product、result joinをsymbolicに展開する。再帰contextごとにcapture内のclosure codeを
静的identityとして保持し、scalar、Bufferなどの値の葉だけをworker parameterへ平坦化する。元wrapperはescape可能な通常のfunction
value用に残し、閉じたsliceだけをcapture-free workerへ差し替える。

同じ20万stepの`state`を変換前binary、変換後production、hand-lowered Mal、C、Rustの順序をroundごとに回転し、3 warmup後20 run
測定した。C/Rustとhand-lowered Malは引き続きlower boundであり、Stateとのsemantic parity実装ではない。

| 実装 | Native median | Callgrind instructions | Allocations / requested bytes |
|:---|---:|---:|---:|
| 変換前State Mal | 10.23 ms | 106,588,002 | 600,003 / 38,400,112 B |
| specialized State Mal | 3.18 ms | 2,218,649 | 9 / 1,602,160 B |
| hand-lowered Mal | 2.89 ms | 2,136,603 | 2 / 1,600,088 B |
| C direct loop | 2.68 ms | 715,562 | 1 / 1,600,000 B |
| Rust direct loop | 2.97 ms | 800,902 | 10 / 1,602,620 B |

specialized Stateは変換前からinstructionを97.9%減らし、hand-lowered Malの3.8%上まで到達した。native medianは変換前の31.1%、
hand-lowered Malの1.10倍、Cの1.19倍、Rustの1.07倍である。9 allocationのうちBuffer以外は反復数に依存しないtop-level closureなどの
固定costであり、step比例の600,001 allocationは消えた。Memcheckはerror 0、9 allocs / 9 freesで、終了時に全blockを解放した。

元wrapperを意味保持のため残すのでpre-LTO IRは50,534 bytes / 19 definitionsから70,028 bytes / 21 definitionsへ増えた。一方、LTO後の
ELF textは4,795 bytesから4,243 bytesへ減った。pre-LTOの3 allocation siteも元wrapper内に残るが、変換後hot pathからは到達せず、
動的allocation counterがその区別を確認している。

baselineとproductionのend-to-end testでは、再帰State chainのresult、effect順、最終Buffer内容、trapを両modeで照合した。現行example
10件も両modeでMemcheckを再実行し、通常実行する7件はclean、入力を省いたためexit 2となる3件もmemory error 0だった。これにより
閉じたsliceを証明できないprogramのfallbackと既存corpusを含めて採択gateを満たした。

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
specialization、call-pattern rewrite、LTOで型と高階dispatchを消せる。`State`も閉じたproducer-consumer sliceでは
continuation specializationが関数を値として返すrepresentationを消去する。これはPoolのoccupancy、authority、coordinate modelとも、
Buffer elementの`Storable`判定とも別のoptimizer責務である。

`Storable`は「placeがcarrier lifecycleを完結できるか」というsource admission、`Lifecycle = Trivial | Owned`はその実装計画、
`Representable`はcanonical memory copy、`HostMappable`はpublic C boundaryという現在の分離を保つ。
`duplicate<Focus>`がnested opaque carrierとして動いたことは、`Managed`を`Storable`の代わりのsource predicateへ持ち上げずとも、
representationとlifecycleを再帰できることを確認している。

この測定から残った実装境界は次の二つである。再調査の順序とcross-language比較の現在のgateは
[generated program最適化policy](../../development/generated-program-optimization.md#再調査の入口)を正とする。

1. nested Bufferはplain identityから不要な`stride`を除いた後も残るobjectとbackingの二重allocationを対象にする。既に退けた
   任意capacity co-allocationを繰り返さない。byte ownerへ変換されないこととaccess patternを区別できるprogram factを得てから別案を測る。
   shared identity、独立lifetime、growth後のdata pointer再取得を保つ。
2. `control`の13-instruction recurrenceはPHI順序だけでCの8-instruction形へ変わるが、明示的なsource field順のPHIもLTOが
   並べ替えることを確認した。Mal IRへheuristicな順序付けを追加せず、LLVM optimizerの比較reproducerとして監視する。

IxPoolの非公開runtime kernelはこれらを解くための汎用allocatorやclosure arenaへ拡張しない。Pool source semanticsとLLVM loweringを
導入する時点では、今回のBuffer lifecycle、allocation分布、generic erasureを比較基準に使うが、無関係なoptimizer責務をIxPoolへ
集めないことがminimalityである。

raw CSV、Callgrind、Massif、Memcheck、pre-codegen bitcode、比較source、Nushell runnerはignored
`.scratch/performance/generics-audit/`と`.scratch/performance/continuation-specialization/`に保存した。
