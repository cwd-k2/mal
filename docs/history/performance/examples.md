# example corpus性能測定履歴

Status: Historical record

この文書はrepository内の全exampleを同じ条件で横断測定した結果を保存する。現在の最適化採用条件は
[generated program最適化policy](../../development/generated-program-optimization.md)、通常の検証は
[test policy](../../development/testing.md)を正とする。

## 2026-09-27 — 全20 entry pointのbaseline / production比較

`d173750b`を基点とし、`fallible-tree`と`recoverable-file`にresult binderを直接渡すsource整理を加えた状態で、
`examples/`の37個の`.mal` sourceから到達する20個の`program.mal`を測定した。toolchainはmalc 0.6.0-dev、
Clang 21.1.8、Valgrind 3.27.1、Nushell 0.115.1である。`baseline`は空のtechnique集合、Clang `-O0`、LTOなし、
`production`は採用済みtechnique、Clang `-O2 -flto`を使った。

入力を取るexampleには次の固定fixtureを与えた。`json-query depth`には64段のnested array、`mini-database`には空のdatabaseへの
存在しないkeyの`get`を10回と`quit`、`brainfuck-llvm`にはchecked-in `hello.bf`、`recoverable-file`には9,000 bytesのfileを使った。
他は引数と標準入力を必要としない。baselineとproductionでstdoutとexit statusが一致した。`buffer-handles`のexit status 38は
READMEに記載された期待値である。

Callgrindはprocess初期化を除くため`main`でcollectionを開始し、cache simulationとbranch simulationを有効にした。text sizeは
GNU `size`のtext列、machine instruction数は最終executableのdisassemblyから数えた。allocationはMemcheck、同時live memoryはnative stackを
含むMassifから得た。次表の変化率はbaseline比で、負値は減少を表す。peak totalはMassifのheap、allocator overhead、native stackを
同じsnapshotで合計した値である。

| Example | production命令 | 命令変化 | production text (B) | text変化 | allocation回数 | peak total (B) |
|:---|---:|---:|---:|---:|---:|---:|
| `relation-views` | 4,695 | -72.9% | 4,704 | -85.0% | 20 -> 20 | 7,744 -> 7,744 |
| `csr-dijkstra` | 9,907 | -83.6% | 8,912 | -82.1% | 34 -> 34 | 7,744 -> 7,744 |
| `spreadsheet` | 2,446 | -79.0% | 5,105 | -81.3% | 6 -> 4 | 7,744 -> 7,744 |
| `buffer-tree` | 1,940 | -64.2% | 4,378 | -79.7% | 6 -> 4 | 7,744 -> 7,744 |
| `fallible-tree` | 12,650 | -58.7% | 8,282 | -76.9% | 62 -> 60 | 7,744 -> 7,744 |
| `json-query` | 20,671 | -96.9% | 23,348 | -91.8% | 35 -> 33 | 18,368 -> 10,400 |
| `mini-database` | 47,301 | -89.3% | 14,071 | -87.7% | 106 -> 106 | 32,984 -> 27,808 |
| `brainfuck-llvm` | 33,648 | -76.7% | 34,050 | -78.0% | 163 -> 31 | 15,816 -> 13,656 |
| `typed-memory` | 1,503 | -53.8% | 2,882 | -86.2% | 4 -> 4 | 7,744 -> 7,744 |
| `symbol-round-trip` | 3,290 | -38.8% | 4,745 | -77.6% | 7 -> 6 | 7,744 -> 7,744 |
| `socket-packet` | 7,786 | -63.8% | 7,835 | -78.5% | 38 -> 37 | 7,744 -> 7,744 |
| `recoverable-file` | 8,212 | -41.4% | 6,118 | -82.3% | 13 -> 13 | 15,648 -> 14,112 |
| `resizable-buffer` | 4,739 | -39.8% | 4,551 | -78.0% | 11 -> 11 | 7,744 -> 7,744 |
| `buffer-handles` | 2,754 | -88.8% | 8,909 | -89.0% | 4 -> 2 | 7,744 -> 7,744 |
| `generic-loop` | 3,753,566 | -99.1% | 5,477 | -89.2% | 12 -> 12 | 7,744 -> 7,744 |
| `tail-recursion` | 5 | -100.0% | 1,455 | -70.6% | 0 -> 0 | 7,744 -> 7,744 |
| `print-and-closure` | 3,380 | -12.0% | 2,633 | -52.7% | 2 -> 2 | 7,744 -> 7,744 |
| `opaque-aggregate` | 1,419 | -40.5% | 1,938 | -80.4% | 1 -> 1 | 7,744 -> 7,744 |
| `numeric-conversion` | 2,947 | -10.2% | 1,954 | -62.1% | 1 -> 1 | 7,744 -> 7,744 |
| `strict-float` | 5 | -98.2% | 1,455 | -73.7% | 0 -> 0 | 7,744 -> 7,744 |

動的命令は全20件で減少し、中央値は68.6%減だった。最終text sizeも全件で減少し、中央値は80.1%減、最終machine instruction数の
中央値は86.2%減だった。`generic-loop`の100万要素foldは最終assemblyで4要素単位のSIMD loopになり、Callgrindでは
429,520,934命令から3,753,566命令へ減った。このcaseだけはwall-clockもprocess起動時間より十分長いbaselineが得られ、3 warmup後の
20回で30.0 msから0.705 msになった。production側は1 ms未満なので、採否の主根拠には動的命令数を使う。`tail-recursion`と
`strict-float`は結果判定まで定数化され、`main`が5命令になった。

LLVM emission直後でClang処理前のIR instruction数は、specializationを含むproduction planにより8件で0.6–15.7%増えた。しかしその8件を
含む全件で最終text、machine instruction、動的命令が減った。したがってpre-LTO IR sizeだけでは生成物のcostを判断できず、最終artifactと
実行時指標を併用する必要がある。

Massifの`pages-as-heap`測定はloaderを含む約4.5 MBが支配的で、productionでの増加は`tail-recursion`の1,240 bytesと
`print-and-closure`の232 bytesだけだった。通常のheap測定では`json-query`の最大useful heapが4,770 bytesから8,639 bytesへ増えたが、
productionでstdinとstdoutの4 KiB stdio bufferが同時にliveになったためである。native stackの縮小により同じcaseのpeak totalは
18,368 bytesから10,400 bytesへ減った。全40実行のMemcheckでinvalid accessはなく、exit時のlive allocationは0だった。

result binderへのsource整理を単独比較した場合、`fallible-tree`のprogram内動的命令差はbaselineで+1、productionで-10、
`recoverable-file`は両profileとも0だった。allocationとpeak memoryは変わらない。これは性能最適化としてではなく、sum branchが既存の
result constructorへそのまま脱出する意図をsourceに直接表すため採用する。source定義の汎用`id`へ置き換える案はbaselineで通常callを増やし、
productionでは概ねLTOにより消えるため、exampleの正規形にはしない。

raw CSV、Callgrind、Massif、Memcheck output、build artifact、集計用Nushell script、Hyperfine JSONはignored
`.scratch/example-audit/`に保存した。result binderとidentityの分離比較は`.scratch/identity-experiment/`に保存した。この監査では
直ちに新しいcompiler optimizationを採択できるproduction回帰は見つからなかった。

### 最終生成物に残る不要候補

`generic-loop`のhot loop自体にはclosure dispatchもdata load/storeも残らず、4要素ごとのSIMD loopになった。一方、その外側には
capturing environmentの40-byte `malloc`が7回ある。Callgrindでは最終ELFに残った6個の`mal_function_*`は一度もcallされず、callback本体は
すべて`main`へinlineされている。それでもcode pointerとcaptureをenvironmentへ格納し、reference countを初期化し、最後に7個の
environment destructorと`free`を実行する列が残る。100万要素caseではloop外の全costを合わせても約3,100命令で0.1%未満だが、短い
higher-order callでは固定costとcode sizeの比率が上がる。call-pattern specializationがcallee targetを単一化した後もclosure parameterと
構築operation自体は残すため、LLVMはallocationのfailure pathとcaptureのdestructionを消せない。

改善候補は、specialized copyについてclosure argumentがcode pointerとして不要になったことと、captureをcalleeへ直接渡せることを証明する
変換である。実行意味論は物理的なenvironment配置を規定せず、不要になったstorageのallocation failureも観測対象にしないため、heap allocationの
省略やstack配置自体は許される。ただしclosureがcreatorのtail callを越えて使われる場合、単純なcreator activation内のstack配置ではlifetimeを
満たさない。まずcode identityとcapture storageを別々に扱い、後者はlambda liftingまたはspecialized call chainへのcapture引数化として
owner終状態まで証明する必要がある。

このほか、最終assemblyには定数falseを作った直後に`test`して分岐する列が`generic-loop`に1箇所、`buffer-handles`に2箇所、
`recoverable-file`に1箇所残った。いずれも各実行で高々一度通る数命令であり、現corpusでは最適化追加の根拠にならない。
allocation / runtimeの大きい箇所では、`fallible-tree`の`free`が動的命令の20.2%、`brainfuck-llvm`のconsuming Symbol連結が37.5%を占めた。
前者はfailureとpartial cleanupを反復するexampleの主目的、後者は生成LLVM textの構築そのものである。`mini-database`は10個のcommandで
106 allocationと最多だがbaseline / productionで回数は同じで、全allocationが解放されている。これらは今回の入力に対する意図した仕事と
不要なcompiler bookkeepingを区別し、直ちにsourceを書き換える対象にはしない。

## 2026-09-27 — direct call後のclosure code pointer省略

上記監査で見つかったdead function bodyに対し、possible application graph上であるfunctionへ届き得る全siteがそのfunctionへの
`DirectCall`に確定した場合だけ、そのfunctionのclosure carrierへcode addressを格納しないdecisionを`execution/optimization/direct_call`へ
追加した。function値にはequalityもrepresentation観測もなく、internal functionをhostへ渡せないため、indirect dispatchが残らないcode addressは
観測不能である。capture environmentのallocation、reference count、destructionは変更しない。複数targetが同じindirect siteへ届くnegative caseも
decision testで固定した。

`1f749399`のproduction artifactと同じ20 exampleを比較した。stdoutとexit statusは全件一致した。Callgrindの動的命令は
`generic-loop`で149、`relation-views`で8、`csr-dijkstra`で5減り、他17件は同数だった。最終ELFに残る`mal_function_*` symbolは
合計40個から28個、machine instructionは合計146、GNU `size`のtextは合計491 bytes減った。`generic-loop`では6個のdead function bodyが
すべて消え、textは5,477 bytesから5,042 bytesへ減った。

`json-query`だけはtextが200 bytes、machine instructionが25増えた。code addressをzeroにしたことでLLVMがUnicode escape検査の4回loopを
unrollしたためであり、function symbol数と測定fixtureの動的命令は変わらなかった。全体のcode sizeと3件の動的costを減らし、backend固有の
再推論を増やさず、pointer storeを省く原理的なdecisionなので採用した。この変更だけでは`generic-loop`の7個の40-byte environment allocationは
残る。capture storageの除去はcode identityとは別の、owner lifetimeを含む変換として扱う。

## 2026-09-27 — 局所direct closureのlambda lifting

`call_pattern`へ、局所生成されたcapturing closureの全利用がaliasまたはdirect applicationで、target functionがself closureを持たない場合に、
captureを通常parameterへ移す変換を追加した。closure creatorはcapture-freeになり、application argumentがcaptureと元argumentのproductへ
変わる。managed captureを含めて既存のparameter ownershipがowner transferを決めるため、stack environment専用のlifetime規則は追加しない。
closureを別functionへ渡すnegative caseでは従来environmentを保持する。

20 example corpusにはこの局所形がなく、最終artifactは全件同一だった。そこで100,001 activationがそれぞれcapturing closureを生成して
一度直接適用する独立fixtureを`c58f6bfd`のproductionと比較した。allocationは100,001回、2,400,024 bytesから0、Callgrind命令は
10,400,869から5、GNU `size`のtextは2,166 bytesから1,455 bytesへ減った。両方のMemcheckはerrorなしだった。allocation failure pathが
なくなった後は残る純粋計算をLLVMが定数化した。higher-order parameterへ渡る`generic-loop`のenvironmentを除くには、次にcapture provenanceを
call-pattern copyのparameterとnested closureへ伝播する必要がある。

## 2026-09-27 — copy-only higher-order parameterのlambda lifting

specialization後に一つのclosure targetだけが届くfunction parameterについて、parameterのproduct path、全call siteのargument construction、
self-recursive forwarding、parameter内のdirect applicationを`call_pattern`で対応付ける変換を追加した。captureをparameter pathへ置き換え、
target closureのcaptureを通常parameterへlambda-liftする。creatorまたはparameter leafがほかへescapeするcase、targetがself closureを持つcase、
複数target、runtime ownerを含むcaptureはadmitしない。

managed captureも同じ形へ変換する試作では、`generic-loop`のallocationが12回608 bytesから6回368 bytes、textが5,042 bytesから
4,466 bytesへ減った一方、Callgrind命令は3,753,417から4,002,590へ6.6%増えた。closure environmentのborrowが通常parameterの
owner transferへ変わり、`SelfTailParameterPlan`のpersistent lenderから外れたためである。parameter patternをcapture fieldへ展開しても
このownership差は解消しなかったため、managed caseは採用しなかった。copy-only captureだけをadmitした最終版では`generic-loop`を含む
20 exampleのartifactは変化せず、この退行もない。managed caseはcall-pattern側の形だけで消さず、ownership authorityがinvocation中の
不変capture lenderを表現できるようにしてから再検討する。

copy-only caseはruntimeのargument数から100,000 iterationの上限を作り、indexを`UInt64`へ加算する独立fixtureで確認した。allocationは
4回232 bytesから3回208 bytes、Callgrind命令は351,892から351,620、textは3,794 bytesから3,698 bytes、machine instructionは
430から409へ減り、Memcheckは両方errorなしだった。結果がloop上限だけで決まるfixtureでは旧形をLLVMがloopごと消した一方、変換後は
3命令のscalar loopを残したため、採否のworkloadには実際の反復計算を含むfixtureを使った。最終20 exampleは直前commitのartifactと
byte単位で一致した。

## 2026-09-27 — self-tail lenderを保つmanaged captureのlambda lifting

`SelfTailParameterPlan`が全back edgeで保持されるfieldを証明し、bodyがfresh managed ownerまたはmanaged resultを作らず、全entryが
frameなしのdirect callであるfunctionでは、外側のcallerをinvocation全体のpersistent lenderとして使うownership規則を追加した。
これにより、前節で退行したmanaged captureもenvironment borrowからparameter borrowへauthorityを保ったまま移せる。

`generic-loop`を`73553787`のproduction artifactと比較すると、Callgrind命令は3,753,417から3,752,585、textは5,042 bytesから
4,306 bytes、machine instructionは760から555、allocationは12回608 bytesから6回368 bytesへ減った。最終IRにはself-tail workerが
残った。元の7個のcapturing environmentのうち6個が消え、nested closureを別closureがcaptureする1個は残る。この残りはdirect
parameter pathだけではなくclosure captureを経るprovenance合成を要するため、同じ変換へ形だけで含めない。

20 exampleの最終artifactを直前版と比較すると、変更されたのは`relation-views`、`csr-dijkstra`、`generic-loop`の3件だった。
textは順に4,608から4,272 bytes、8,752から7,552 bytes、5,042から4,306 bytes、machine instructionは758から622、
1,744から1,387、760から555へ減った。Callgrind命令は4,687から3,260、9,902から6,967、3,753,417から3,752,585、
allocationは20回880 bytesから12回560 bytes、34回2,112 bytesから18回1,112 bytes、12回608 bytesから6回368 bytesへ減った。
他17件はbyte単位で同一だった。全20件で旧版とstdout、stderr、exit statusが一致し、Memcheckはerrorなし、終了時のlive allocationは
0だった。
