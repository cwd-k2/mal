# CPSに基づく通常関数の明示的return binder

Status: Adopted in the current v0.5 profile

この文書は明示的returnを採択した設計理由と実装境界を記録する。利用者が従う構文、型、scope、completion、評価規則は
[明示的returnとcompletion](../spec/control.md)を正とする。

## 採択した構造

通常関数`A -> B`は、compiler内部では呼出後の計算を明示した次のCPSへ変換できる。

```text
CPS(A -> B) = forall R. A -> (B -> R) -> R
```

return binderは新しい関数型やfirst-class continuationを加えず、通常関数が既に持つ一回のinvocationのreturn edgeへ
lambda-local nameを与える。sourceの関数型とcall siteは`A -> B`のままであり、binder applicationは既存のlambda resultへ落ちる。

sum resultの複数binderはvariant injectionと同じ選択を名前付きで行う。呼出側は既存のexhaustive sum eliminationを使うため、
producerとconsumerを結ぶ新しいprotocol、nominal constructor、runtime representationを追加しない。producerへ値を返して再開する
operationはabortive returnではなく、従来どおり通常のcallbackで表す。

## 最小性の判断

採択単位は単一return binder、sum binder、`when`、空直和`Empty`である。これらは独立primitiveの寄せ集めではなく、次の一つの
境界を閉じる。

- binderは既存のlambda return edgeにだけauthorityを持つ。
- `Abrupt`はsource typeを増やさず、値を要求する位置と全path returnをcheckerで判定する。
- `when`は`Abrupt` branchとfallthroughをBool eliminationとして局所的に合流させる。
- `Empty`はzero-variant sumとzero-continuation eliminationとしてproduct/sumの端点を閉じる。

この構成はsiteごとの`noop` continuation、Unit lambdaへのreturnのeta expansion、一般の`Never` subtype、exception runtime、
first-class continuation lifetimeを不要にする。追加したsurfaceは既存coreへの局所的で完全な変換を持ち、hidden allocationや
host contractを要求しないため、[最小性の判定基準](minimality.md#機能を加える判定基準)を満たす。

## authorityと責務

return binderが完了できるauthorityは、それを宣言したlambda invocationだけが持つ。binderをnested closureへcaptureすると
invocation終了後のlifetimeと複数回callの意味を新たに定義する必要があるため、resolverが一律に拒否する。escape analysisの結果で
surface semanticsを変えない。

checkerはchecked ASTで`Value(Expression)`と`Abrupt(ControlExpression)`を別variantにし、`Abrupt`へ仮の型を与えない。
core loweringはlexicalな後続を`Value` pathだけへ接続し、return pathでは捨てる。single returnはlambda result、sum returnは
sum injectionとlambda result、`when`はbranch、empty eliminationはzero-arm caseになる。

この境界でreturn binderとcompletion judgmentを消去する。ANF、closure、control、execution、backendは通常のexpression、lambda result、
branch、sumだけを受け取り、source-level controlを再解釈しない。
