# managed Engram性能検証

Status: Current measurement policy and baseline

この文書はC backendのmanaged Engram最適化に対する測定方法、回帰条件、現在のbaselineを管理する。
ownership correctnessと実装方式は[実装規約](../implementation/ownership.md)、一般的なgenerated Cの性能記録は
[generated C performance](performance.md)、通常の検証commandは[test policy](testing.md)を正とする。

## 対象

現在のbackendは次のcostを削減する。各方式の安全条件は実装規約を参照し、この文書では重複させない。

- owned bindingとowned parameterのlast-use transfer
- 一意なflat `Symbol` bufferを使うconsuming concat
- callに閉じたlocal closureのstack environment
- shared concatのbalanced ropeと遅延materialization
- known direct callに限定したowned entry

focused regressionは`compiler/tests/c_emit/calls.rs`と`symbol.rs`に置く。managed tail recursionは100万iteration、
case bindingを含むtail edgeは10万iterationをnative Cとして実行する。closure-use testはaliasだけでなく、recursive
self closureをfunction valueとして使う経路がheapへfallbackすることも検査する。

## pressure suite

localのignored `.scratch/pressure/`は次のworkloadを持つ。

| Workload | Shape | 主に検査するcost |
|---|---|---|
| `symbol-churn` | 100万回の短寿命concat | allocationとreleaseの残留 |
| `symbol-growth` | 1 byteずつ成長する1万byteの値 | append時のallocationとbyte copy |
| `symbol-prepend` | helper越しに1 byteずつ追加する1万byteの値 | owned direct callとprepend reuse |
| `symbol-rope` | borrowed境界を越える5000回のprepend | ropeの平衡性とmaterialization |
| `closure-churn` | 20万個の短寿命capturing closure | environment allocationとcleanup |
| `aggregate-churn` | 20万回のmanaged product、sum、case | field copyとpath-local cleanup |

runnerはiteration数と`Symbol` bytesを別C translation unitへ渡し、allocator builtinを無効にしてClangによるworkloadの
除去を防ぐ。小さいpeak live-allocation上限と終了時live allocationゼロを検査し、通常buildとASan/UBSan buildを実行する。
ptrace環境ではLeakSanitizerを使えないため、leakはruntime counterで検査する。

```nu
nu .scratch/pressure/run.nu
nu .scratch/pressure/run.nu --sanitize
```

2026-09-07時点では両方が全caseを通過した。`symbol-growth`の1万byte構築は約9,999回のallocationからtest上限32回以内に
減少し、`closure-churn`の20万environment allocationは0になった。実用algorithmではTypical90の206 sampleと
maximum-order 40 checksがpublic `malc build`経路を通過した。絶対時間はCIの合否条件にせず、同一環境の変更前後だけを
比較する。

## 採用条件

- source semantics、C host ABI、trap、評価順序を変えない。
- alias再使用、branch/case、aggregate、return、direct tail edge、closure escapeのnegative caseを置く。
- retain/releaseまたはallocationの削減をgenerated Cかdeterministicなcounterで確認する。
- 通常のcompiler testに加え、pressure suiteを通常とsanitizerの両方で通す。
- wall-clock比較ではtoolchain、input、stdout、optimization option、warmup/run数を揃える。
- Clangの偶発的なDCEではなく、未最適化Cまたはcounterでも改善を説明できることを確認する。

region化、reference count方式の置換、thread-safe ownership、automatic memoizationは、profile上の必要性または言語範囲の
変更が生じるまで行わない。mal functionは`extern`を呼び得るため、immutabilityだけではmemoizationのpure条件にならない。
