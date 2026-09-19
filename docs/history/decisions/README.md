# 設計決定履歴

Status: Historical records

この文書は同じdirectoryにある個別の設計決定記録への入口だけを持つ。規則は[`spec/`](../../spec/)、判断の本文と
statusは各decisionを正とし、ここでは現行判断とhistorical recordを分けて示す。

## 現行判断

| Area | Decisions |
|---|---|
| compiler | [D002](D002.md)、[D041](D041.md)、[D045](D045.md)、[D046](D046.md)、[D047](D047.md)、[D065](D065.md)、[D066](D066.md)、[D067](D067.md) |
| editor tooling | [D044](D044.md) |
| closure | [D003](D003.md)、[D007](D007.md)、[D038](D038.md) |
| application、sum、Bool、`if` | [D004](D004.md)、[D005](D005.md)、[D043](D043.md)、[D049](D049.md)、[D051](D051.md) |
| minimalism | [D008](D008.md)、[D033](D033.md)、[D055](D055.md)、[D057](D057.md)、[D061](D061.md) |
| managed ownership | [D033](D033.md)、[D035](D035.md)、[D041](D041.md)、[D055](D055.md)、[D057](D057.md) |
| Float | [D009](D009.md)、[D019](D019.md) |
| literalとscalar operation | [D011](D011.md)、[D013](D013.md)、[D020](D020.md)、[D021](D021.md)、[D025](D025.md)、[D035](D035.md) |
| externとopaque value | [D012](D012.md)、[D015](D015.md)、[D016](D016.md)、[D039](D039.md)、[D040](D040.md)、[D053](D053.md)、[D054](D054.md) |
| top-level initialization | [D018](D018.md) |
| source file requirement | [D032](D032.md) |
| genericsとexternal memory | [D052](D052.md)、[D056](D056.md) |
| EngramとExternのauthority | [D031](D031.md)、[D033](D033.md)、[D035](D035.md)、[D040](D040.md)、[D052](D052.md)、[D053](D053.md)、[D054](D054.md) |
| Packed execution | [D058](D058.md)、[D061](D061.md)、[D062](D062.md)、[D063](D063.md)、[D064](D064.md) |

D012はD016とD032、D022はD024、D031はD033、D033はD034とD055でrefineされているが、元の判断を撤回していない。
D008のmemory management節はD033が、D009、D022、D028、D029、D033のtrapに関する一部はD035が置き換える。
D022、D024、D027、D031のsource spellingはD037が置き換える。
D007のlambda parameter source spellingはD038が置き換える。
D001とD018のcall siteに`extern`を置くsource spellingはD039が置き換える。
D041はD002のRust compilerを維持し、最初のC execution backendをLLVM backendへ置き換える。
D042はD004のsum injection spelling、D005の`case`による説明、D023のsource syntaxを置き換える。
D048はD042のsource-level sum injection constructorを置き換える。
D049はD048のlambdaだけに置いたsum construction boundaryをdirect result blockにも拡張する。
D050はD038のlambda parameterとD049のdirect result blockのsource spellingを置き換え、`if`と`when`を含むbodyをexpressionに統一する。
D051はD050のarrow共有とlambda return binderを置き換え、lambdaとdirect result blockを独立した構文へ整理する。
D052はD022、D024、D027、D030、D037のmemory source modelを置き換え、D013、D031、D035の意味論を新しいsyntax、layout、
generic transferへ適用する。
D053はD040のSymbol mappingを置き換え、HostMappableだけをpublic C ABIへ出す。D040のtyped value、call capability、terminal
returnに関する判断は維持する。
D054はD053のAddress-only boundaryを維持しつつ、canonical memoryとpublic C carrierの変換をentry sourceのnamed alias helperへ
限定して追加する。
D055はD033のborrowed parameter、owned result、managed leafの再帰規則を維持し、唯一のowner successorへのlast-use handoffを
optional optimizationからexecution ownership planの正規形へ置き換える。runtime operationによるstorage再利用は置き換えない。
D056はD052のcanonical layoutとRegion/Packed transferを維持し、Cursor loadのresultとRegion indexを置き換える。
D057はD055のresponsibility保存則を維持し、aggregate ownerがlifetimeを包含するlocal aliasをowned bindingからborrowへ置き換える。
D058はD055とD057のresponsibility保存則をcall boundaryへ適用し、caller authorityが全pathを包含するparameterとargumentをborrowへ
置き換える。
D061は三つのintrinsic capabilityとtarget解析を一つの`Buffer<A>` authorityとdirect operationで置き換える。
D062は仕様が許す二つのprepare時点からcallback前の一度を選び、D060のlazy preparationを置き換える。
D063はD059のcapability単位のstable accessを置き換え、non-growing Buffer helperのinternal ABIをactive dataへ変換する。
D064は同じABIへfunction invocation中だけ有効なactive dataの`noalias` contractを加える。
D065は自己再帰の全edgeが保持するparameter fieldを、semantic live-inに残したまま物理frameから除く。
D066はrecursive region invocation内のcontrol topをlocal SSAへ置き、native Mal call境界で共有topへ同期する。
D067は同じinvocationでcontrol storageとcapacityをcacheし、growthとnative Mal call後に更新する。

## 後継があるhistorical record

| Record | Current successor |
|---|---|
| [D001: local capture禁止](D001.md) | [D003](D003.md) |
| [D006: `b'…'` byte literal](D006.md) | [D025](D025.md) |
| [D010: byte型としてのEngram](D010.md) | [D031](D031.md) |
| [D017: immutable byte型名](D017.md) | [D031](D031.md) |
| [D026: Engram descriptor memory operation](D026.md) | [D031](D031.md) |
| [D027: storage-size query](D027.md) | [D052](D052.md) |
| [D028: Engram operator](D028.md) | [D031](D031.md)で`Symbol` operationへrefine |
| [D029: Engram concatenation](D029.md) | [D031](D031.md)で`Symbol` operationへrefine |
| [D030: process argument descriptor](D030.md) | [D052](D052.md) |
| [D034: C adapter ownership helper](D034.md) | [D040](D040.md) |
| [D036: Symbol admission builder](D036.md) | [D040](D040.md) |
| [D014: shift countを一律にtrap](D014.md) | [D035](D035.md) |
| [D023: index付き`case` arm](D023.md) | [D042](D042.md) |
| [D042: applicationをvalueとcontinuationの双方向表記に統一する](D042.md) | [D048](D048.md) |
| [D048: 直和の構築をsum return binderに限定する](D048.md) | [D049](D049.md) |
| [D050: bodyをexpressionに統一しbinder境界へ`->`を置く](D050.md) | [D051](D051.md) |
| [D059: Packed builder dataのstable direct access](D059.md) | [D061](D061.md)、[D063](D063.md) |
| [D060: editのlazy preparation](D060.md) | [D062](D062.md) |
| [D022: pointer primitive](D022.md)、[D024: pointer memory operation](D024.md)、[D037: 型修飾memory primitive](D037.md) | [D052](D052.md) |

historical record内の旧構文や旧名称は当時の判断を保存するために残す。現在のsyntaxやbehaviorとして引用せず、
必ずsuccessorと[`spec/`](../../spec/)を確認する。
