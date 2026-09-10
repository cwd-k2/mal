# application control境界監査計画

Status: Active audit plan

この文書はapplication control planからLLVM IRへ値を渡す境界について、型、到達可能性、managed ownerの整合性を監査する
順序と完了条件を定める。control loweringの規則は[application control lowering](application-control-lowering.md)、stageのauthorityは
[compilerの責務境界](../implementation/responsibilities.md)、owner operationは[managed value ownership](../implementation/ownership.md)、
検証方法は[test policy](testing.md)を正とする。

## 出発点

`8f7f5c3`では、異なるfunction signatureを含む共通regionについて、型の異なるreturn siteとframe tagの組合せを到達不能として生成し、
wildcard parameterへ渡したmanaged argumentをregion遷移でreleaseするよう修正した。これによりCPSと`callCc`の代表例は動作する。

この修正が示した問題はsource semanticsではなくstage境界にある。possible application graphによる型互換targetの過剰近似はsoundであり、
target集合の精度をcorrectnessの前提にしてはならない。一方、LLVM emitterが`binding == None`、型の不一致、frameの存在から前段の意味を
再推論すると、正当なexecution planを拒否するかownerを誤って扱い得る。

現時点で最初に確認する箇所は、共通regionを使わないdirect-self frame遷移である。ここには非`Unit` parameterの`binding`がない場合に
生成を拒否する分岐が残る。wildcardは値を受け取って捨てるpatternなので、まず再現testで到達可能性を確定し、managed argumentなら
一度だけreleaseするという既存規則を同じ経路へ適用できるか検証する。未確認の段階では欠陥と断定せず、このfocused caseから監査を始める。

## 保持する境界

| Stage | この監査で所有する事実 |
|---|---|
| `check` | source application、lambda parameter、resultの型が一致すること |
| `control` | parameter patternのbinding有無、state input、resume liveness |
| `execution/application` | siteごとのsoundなpossible target集合とfunction signature |
| `execution/call` | `Direct`、`DirectRegion`、`DirectSelfTail`、`Dispatch`の選択 |
| `execution/frame` | suspension site、resume state、frame payload、environment owner |
| `execution/ownership` | 型がmanaged ownerを含むかというbackend非依存な分類 |
| `backend/llvm` | 現在はhandoff先とowner operationを構成し、admission済みplanをretain、store、release、branch、`unreachable`へ変換すること |

sourceで新しい制約を加えたり、型aliasやfunctionの役割をnominalに分けたりしてbackendの不足を回避しない。flow-sensitive target解析による
集合の縮小は独立した最適化であり、監査の完了条件にはしない。LLVM emitterだけが必要とするlayoutやregisterはexecution planへ逆流させない。

## 監査するinvariant

1. 各application siteのargument型は、そのsiteで選択可能な全targetのparameter型と一致する。
2. bindingを持つparameterへのhandoffはownerを対応するslotへ移し、bindingを持たないparameterへのhandoffはmanaged ownerだけを一度releaseする。
3. handoff先を準備する前にcaller localまたはactive environmentを解放しない。
4. non-tail region callだけがframeを積み、frame payloadはresume live-inと一致する。
5. frameを積んだcallのresult型はresume input型と一致する。共通machineが列挙するそれ以外のreturn/frame組合せだけを`unreachable`にする。
6. terminal return、tail transition、frame resumeの各経路でlocal、argument、result、environment、frame fieldのownerを重複して解放せず、残さない。
7. direct native call graphはacyclicであり、region内recursionはMalの深さに比例するnative stackを使わない。

前五項のうちbackend非依存に表現できる関係はexecution planのvalidatorで確認する。利用者入力の誤りは所有するfrontend stageが診断し、
execution planの矛盾を一般的なsource errorへ変換しない。

## 実施順序

### 1. 残存する非対称経路を再現する

- 非`Unit` wildcard parameterを持つlocal direct-selfのnon-tail callをbuild、実行するfocused testを追加する。
- unmanaged argumentとcapturing closureなどのmanaged argumentを分ける。
- managed caseはallocation counterでnormal return後のlive allocationがzeroであることを確認する。
- 同じparameterをtail callするcaseも追加し、`DirectSelfTail`がbindingを必須としている箇所を監査する。

再現testが既に通る場合も削除せず、その経路のcontractを固定する。失敗した場合は共通regionの修正を条件分岐ごと複製せず、parameter handoffの
共通operationを抽出できる最小範囲を先に決める。

### 2. call modeを横断する

次の各行について、bound/wildcardとunmanaged/managedの組合せを確認する。構造上到達不能な組合せは、理由をexecution planのtestで示す。

| Call mode | non-tail | tail | 主に確認する境界 |
|---|---:|---:|---|
| `Direct` | yes | yes | native ABIへのargument/result owner handoff |
| `DirectRegion` | yes | yes | frame push、parameter handoff、environment交換 |
| `DirectSelfTail` | no | yes | frameなしのslot更新またはwildcard discard |
| `Dispatch` outside region | yes | yes | code identity、indirect native call、result owner |
| `Dispatch` inside common region | yes | yes | target別parameter、異種signature、到達不能branch |

全組合せをend-to-end fixtureへ展開しない。execution planのfocused test、LLVM artifact test、Clangでcompile/link/executeする代表caseの三層に置く。

### 3. execution planの契約を強める

testで確認した重複判断だけを前段へ移す。候補はparameter handoffを`Bind(slot)`または`Discard`として表すdispositionと、return siteから
型互換frame tagへの関係である。導入前に次を満たすことを確認する。

- `binding: Option<_>`からbackendが同じ意味を再推論する箇所が実際に複数ある。
- 新しい型がbackend固有のstorageやreference counting mechanismではなく、execution上の値の行先を表す。
- constructorが完全なplanだけを返し、`is_valid`がauthorityから期待集合を再構成できる。
- emitter内の`Option` failureが減り、不可能なplanと到達不能なruntime branchを区別できる。

return/frame関係を明示しても、共通machineの実装が小さい間は型の異なるbranchを`unreachable`として出力してよい。Cartesian productの削減は
correctnessを固定した後にcode sizeとして測定する。

### 4. failureを分類する

backendの`Option` chainで新しい不変条件違反を一般的な「admitted programを拒否した」結果へ潰さない方法を検討する。structured internal errorを
導入する場合も、公開diagnosticの恒常的な複雑化ではなく、stage、site、function、frame、破れたinvariantをtestとdebugで特定できる最小表現にする。

## 完了条件

- 上記call mode表の到達可能なhandoff経路にfocused testがあり、代表的なmanaged caseがnative executionでleak、double release、trapを起こさない。
- heterogeneous signature、同じ構造型を持つ異なる役割のfunction、nested frame、capturing escape continuationが共存して動作する。
- execution plan validatorがapplication target、call mode、frame、resume inputのbackend非依存な関係を検査する。
- LLVM emitterがparameter patternまたはpossible target集合の意味を独自に再構成せず、型の合わないruntime branchだけを明示的に`unreachable`にする。
- `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`が通る。
- `.scratch/cps/run.nu`を実行し、CPS例がすべてcompile、link、executeできることを補助確認する。scratchのbinaryや結果をtracked fixtureにはしない。

完了後は、確定した規則をowner文書とtestに残し、この一時的な順序文書を削除する。将来判断に必要な棄却案が生じた場合だけ
`docs/history/decisions/`へ移す。
