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

### 1. last-use transfer

最初に、owned bindingの最後の使用へdescriptorをtransferし、現在の保守的なretainと対応するreleaseを除去する。
解析はlexerやparserではなく、型とlexical blockが確定したclosure-converted IR以降に置く。最初の範囲は、生成元が
ownedであり、同じbindingへの後続使用がなく、transfer先がbinding、aggregate field、function result、またはdirect
tail callの次parameterである場合に限定する。

parameter、environment field、case payloadはborrowから開始するため、単にlast useであるだけではmoveしない。
enclosing owner全体を同時にconsumeできることを証明するまではcopyを維持する。transferしたbindingはlexical cleanupから
除外し、branchごとにlast useが異なる場合は各pathで独立に判断する。評価順序と、resultを確保してからsourceを破棄する
順序は変えない。

実装候補の境界は`c_emit/body/statement/result.rs`のmaterialization、`statement/control.rs`のbranch、case、tail edge、
`body/pattern.rs`のaggregate field bindingである。`ResultOwnership`はoperation resultがownedかborrowedかを既に表すため、
新しい解析はbinding側のshareをconsumeできるかだけを追加し、型ごとのcopy/destroy規則を複製しない。

### 2. consuming Symbol concat

last-use transferが利用可能になった後、左operandをconsumeでき、runtime ownership shareが一つで、capacityが足りる場合に限り、
Symbol concatのbufferを再利用する余地を測定する。通常の`a + b`は`a`をborrowするため、`a`が後で観測可能なままbufferを
変更してはならない。compilerからconsuming operationであることを明示できない段階ではin-place変更を行わない。

capacityをallocation headerへ追加する変更はC host ABIのopaque ownership field内に閉じられるが、空文字最適化、allocation
failure、overflow、hostへのcontiguous byte borrowを維持する必要がある。`symbol-growth`で総copy量または同一環境の増加率が
改善し、短いconcatとread-heavy workloadが退行しない場合だけ採用する。

### 3. non-escaping closure environment

capturing closureがlocal callから外へ保存、return、aggregate格納されないことを証明できる場合に、environmentのstack化または
captureの直接引数化を調べる。capture-free closureは既にallocationしない。function-value用の共通calling conventionは
fallbackとして残し、候補ごとのfunction cloneを無制限に生成しない。

### 4. 表現変更

rope、slice、flatten cacheは、反復concatのbyte copyが上記の局所最適化後も支配的な場合に限って検討する。host ABIは
`Symbol`のcontiguous bytesをcall中borrowできるため、ropeを採るならflatten時点、cache lifetime、byte accessの計算量を
別途設計する。論理的immutabilityを保っても内部cacheには同期と回収のpolicyが必要になる。

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
