# managed Engram性能評価

Status: Current investigation roadmap

この文書は、correctnessが成立したC backendのownership処理について、次に調査する最適化の順序、測定方法、
採用条件を定める。ownership contractは[実装規約](../implementation/ownership.md)、一般的なgenerated Cの性能記録は
[generated C performance](performance.md)、通常の検証は[test policy](testing.md)を正とする。

## 出発点

現在のbackendは`Symbol`、capturing closure、それらを含むproductとsumについてcopy、transfer、逆順cleanupを生成する。
direct self tail callもmanaged parameterとpath-local bindingを回収しながらCのloopへlowerされる。したがって最適化は
受理可能なprogramやborrow/result contractを変えず、不要なretain、release、allocation、byte copyだけを減らす。

focused regressionは`compiler/tests/c_emit/calls.rs`と`symbol.rs`にあり、managed tail recursionでは100万iteration、
case bindingを含むtail edgeでは10万iterationをnative Cとして実行する。localのignored `.scratch/pressure/`には次の
追加workloadがある。

| Workload | Shape | 主に検査するcost |
|---|---|---|
| `symbol-churn` | 100万回の短寿命concat | allocationとreleaseの残留 |
| `symbol-growth` | 1 byteずつ成長する1万byteの値 | immutable concatの累積byte copy |
| `closure-churn` | 20万個の短寿命capturing closure | environment allocationとcleanup |
| `aggregate-churn` | 20万回のmanaged product、sum、case | field copyとpath-local cleanup |

runnerはiteration数とSymbol bytesを別C translation unitへ渡し、allocator builtinを無効にして、Clangがworkloadを
消去しないようにする。小さいpeak live-allocation上限と終了時live allocationゼロを有効にし、通常buildと
ASan/UBSan buildを実行する。ptrace環境ではLeakSanitizerを使えないため、leakはruntime assertionで検査する。

```nu
nu .scratch/pressure/run.nu
nu .scratch/pressure/run.nu --sanitize
```

2026-09-07時点では両方が全caseを通過した。実用algorithm側ではTypical90の206 sampleとmaximum-order 40 checksが
public `malc build`経路で通過した。これらの絶対時間はCIの合否条件にせず、同一環境の変更前後だけを比較する。

## 着手順

### 1. last-use transfer（実装済み）

`c_emit/body/ownership`はowned bindingの最後の使用へdescriptorをtransferし、保守的なretainと対応する実効的なreleaseを
除去する。解析はlexerやparserではなく、型とlexical blockが確定したclosure-converted IRに置く。生成元がownedであり、
同じbindingへの後続使用がなく、transfer先がbinding、aggregate field、function result、またはdirect tail callの次parameter
である場合に限定する。

parameter、environment field、case payloadはborrowから開始するため、単にlast useであるだけではmoveしない。
enclosing owner全体を同時にconsumeできる場合だけ各fieldへownershipを分配する。transferしたbindingはzero状態にして既存の
lexical cleanupをno-opにし、branchごとにlast useが異なる場合は各pathで独立に判断する。評価順序と、resultを確保してから
sourceを無効化する順序は変えない。direct tail loopが所有するparameter slotはowner全体をconsumeできる場合に含める。

materializationは`c_emit/body/statement/result.rs`、branch、case、tail edgeは`statement/control.rs`、aggregate fieldへの
ownership分配は`body/pattern.rs`が担う。`ResultOwnership`がoperation resultのowned/borrowedを表し、last-use解析はbinding側の
shareをconsumeできるかだけを追加する。focused testはlive aliasではretainを残し、branchの各pathとaggregateを経由する
direct tail edgeではtransferすることをgenerated Cとnative実行の両方で検査する。

### 2. consuming Symbol concat（実装済み）

左operandがowned bindingのlast useである場合だけ、compilerはconsuming concatを生成する。runtime ownership shareが一つなら
既存bufferへ追記し、capacity不足時は上限を検査しながら幾何的に拡張する。通常の`a + b`、同じbindingを左右に使うconcat、
後で再使用する`a`はborrowed operationを維持する。staticまたは共有中のleftは新しいbufferへcopyするfallbackへ進み、
consuming operationへ渡されたleft shareはresult構築後にreleaseする。

capacityはallocation headerにあり、C host ABIのopaque ownership field内に閉じる。空文字、allocationとreallocation failure、
overflow、共有aliasのimmutability、hostへのcontiguous byte borrowをfocused testで検査する。1万byteの`symbol-growth`は
約9,999回のallocationからtest上限32回以内になった。同一環境の通常build 7回の中央値はlast-use transfer後の約1.87 msから
約0.87 ms、ASan/UBSan buildの確認値は約70 msから約3.45 msになった。`symbol-churn`、`closure-churn`、`aggregate-churn`にも
同じ実行で退行は観測されなかった。絶対時間は環境に依存するため合否条件にはせず、allocation上限をdeterministicな回帰条件とする。

### 3. non-escaping closure environment（実装済み）

`c_emit/body/closure_use`はMakeClosureから単純alias chainを追跡し、すべてのreferenceがcallのcallee位置に
限られるlocal closureを直接callする。capturing closureのenvironmentはC stack上に置き、captureは外側の
lexical lifetime内でborrowする。return、aggregate格納、capture、別関数の引数に現れるaliasが一つでもあれば、
従来のreference count付きheap environmentへfallbackする。

function bodyはheap closureと同じenvironment pointer引数を受けるためcloneせず、product parameterの既存direct entryも
共有する。focused native testはscalarとproduct parameterの両方でtotal allocation上限0を満たし、別関数へ渡す
negative caseがheap allocation failureを維持することを確認する。`closure-churn`の20万environment allocationは0になり、
同一のpressure suiteで他caseの退行は観測されなかった。

### 4. 表現変更（現時点では不要）

rope、slice、flatten cacheは、反復concatのbyte copyが上記の局所最適化後も支配的な場合に限って検討する。host ABIは
`Symbol`のcontiguous bytesをcall中borrowできるため、ropeを採るならflatten時点、cache lifetime、byte accessの計算量を
別途設計する。論理的immutabilityを保っても内部cacheには同期と回収のpolicyが必要になる。

consuming concatによって`symbol-growth`のallocation operationは幾何的増加の上限内となり、同一環境の測定でも累積copyが
支配する兆候は解消した。この条件が維持される限り、表現変更には進まない。

automatic memoizationは初期候補にしない。mal functionは`extern`を呼び得るためimmutabilityだけではpureにならず、無制限cacheは
live setを増やす。descriptor addressは再利用され、source-level identityでもない。pure operationに限定したcacheがprofileで
必要になった場合も、keyはbytesや構造的な値とし、明示的な容量とevictionを持たせる。

## 各変更の採用条件

- source semantics、C host ABIのborrow/owned/moved規約、trapと評価順序を変えない。
- aliasを後で再使用するnegative case、branch/case、aggregate、return、direct tail edgeのfocused testを置く。
- generated Cのretain/releaseまたはallocationが対象caseで実際に減ったことを構造testかtest用counterで確認する。
- `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`を通す。
- local pressure suiteを通常とsanitizerの両方で通し、終了時live allocationゼロとpeak上限を維持する。
- wall-clockを比較する場合は同一toolchain、input、stdout、optimization option、warmup/run数を揃える。
- 改善がClangの偶発的なDCEだけでなく、未最適化Cまたはcounterでも説明できることを確認する。

region化、reference count方式の置換、thread-safe ownershipは、profile上の必要性または言語範囲の変更が生じるまで行わない。
correctnessの完成とperformance experimentを混同せず、一つの変更では一つのcostだけを検証する。
