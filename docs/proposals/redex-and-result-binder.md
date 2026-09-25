# その場でapplicationされるlambda literalとresult binderの理論的根拠

Status: Exploratory

この文書は、[直和除去continuationからのresult binder参照](sum-continuation-result-binder.md)の受理範囲を導く理論を管理する。
contractと実装計画は提案側を正とする。採択後は、この文書の原則を[値、解釈、control](../design/value-interpretation-and-control.md)へ
移し、この文書は削除する。

## lambdaは中断、redexは中断を残さない

lambdaは評価の中断（suspension）を導入する。bodyはapplicationまで実行されず、複数回applicationされ得て、定義したblockの
完了後にも残り得る。result binderが「nested lambdaから参照できない」のは、この中断がbinderの生存範囲
（block invocationの実行中）を越え得るからである。禁止の根拠はlambdaの構文ではなく、値としてのlambdaが持つ中断にある。

lambda literalをその場で一度だけapplicationする形は中断を残さない。strictなmalではこれはbeta redexである。

```text
x := e; rest        =   (x -> rest)(e)   =   e[(x) -> rest]          （let）
s[(p0) -> b0, ...]  =   case s of inl p0 => b0 | ...               （case）
```

[core calculus](../spec/execution.md#core-calculus)は`:=`をlambda applicationへ消去できると定める。beta簡約はpatternの束縛変数へ
実引数を置換するだけで、束縛名の選択は意味に影響しない（α同値）。したがってredexのbodyは囲むlambdaと同じinvocationに属する。
この読みは既に`:=`とblockで使われている。次は意味が同じ三つの書き方だが、現行は最初のものだけがbinderを参照できる。

```mal
n := x; k(n)              // 受理
x[(n) -> k(n)]            // 拒否: result binder cannot be captured
((n) -> k(n))(x)          // 拒否: 同上
```

現行の規則は、beta redexであるlambda literalをbindingとそれ以外で区別している。これは提案と無関係に存在する不整合である。

## CPSとreturn binder

result binderはCPSに基づく明示的return binderとして導入された（旧`docs/design/cps-return.md`、commit `14a41a58`。
現在の規範は[control.md](../spec/control.md)）。通常関数`A -> B`は概念上`∀R. A -> (B -> R) -> R`のCPSに対応し、binderは
そのinvocationのreturn continuation `k`にlambda-localな名前を与える。この見方で境界は次のように定まる。

- **lambda値は新しい`k`を導入する。** 各invocationが自分のreturn edgeを持ち、bodyが実行される時点は外側の`k`の生存範囲を
  越え得て、複数回でもあり得る。
- **`let`、`case`、`if`は`k`を継承する。** CPS変換ではこれらは新しいreturn continuationを作らないadministrative redexで、
  bodyや分岐は囲むinvocationの`k`のまま評価される。binderは`k`を継承する文脈すべてで見える。`:=`、`if`、`when`、
  direct result blockは既にこの範囲にあり、提案は直和除去のlambda literalを加える。

既存の禁止を緩めるのではなく、「`k`を継承する文脈」の判定を、同じ性質を持つ直和除去のcontinuationへ広げる。
lambda値には新しい`k`が要るという根拠は変わらない。binderは囲むblockのjoin pointへのjumpとして働くsecond-classな
continuation targetで、[control.md](../spec/control.md#評価とlowering)がcore境界で「非first-classなjoin identityへのtransfer」として
保持することと一致する。jumpは、そのinvocation内のtail contextなら`let`のbodyや`case`の分岐からも行える。
行えないのは、中断されたlambda値のbodyからだけである。

この規則はescape analysisではない。[prior-art](../research/prior-art.md#closure)が採らないとした「escapeしなければcapture可」は
closure値の実際のescapeで受理を変える。ここではredexにclosure値が作られないという分類で判定するため、実装や最適化の結果に
依存しない。

## 入れ子は手書きのCPS、平坦な形は直接形とabort

現行のcontinuationの入れ子は、成功側の残りの計算`rest`をcontinuationの本体へ入れる、手書きのCPS変換にあたる。提案の平坦な形は
その`rest`を外へ出す。失敗側が`Abrupt`（現在のevaluation contextを捨てる）であることから、二つは次の等式で結ばれる。

```text
E[ s[(n) -> n, () -> k(u)] ]   =   s[(n) -> E[n], () -> k(u)]

s[(n) -> { rest(n) }, () -> failed()]         // 入れ子（CPS形）
v := s[(n) -> n, () -> failed()]; rest(v)     // 平坦（直接形）
```

`E`は評価contextで、`Abrupt`側は`E`を捨てるため`E[k(u)] = k(u)`が成り立つ。これは`case`のcommuting conversionと、jumpが
評価contextを捨てるというjoin pointの標準的な等式である。この等式を左から右へ機械的に適用すると`E`が各armへ複製されるが、
[D047](../history/decisions/D047.md)のlexical joinは複製せず共有するので、`rest`はどちらの形でも一度だけ生成される。D047が棄却した
synthetic lambdaとcallによる後続の共有は、現行のcontinuation lambdaが実質的に行っているものである。

これはcps-returnがbinderでnoop continuationとeta expansionを不要にしたことの、直和の除去側への延長である。理論が与えるのは
受理範囲と等式であり、表記の選択（提案の代替案AとB）は設計判断として残る。

## 操作的意味とaffine control

[実行意味論](../spec/execution.md#評価戦略)は、直和除去が選択されたcontinuationだけを評価してpayloadへ一度適用すると定める。
lambda literalの評価は作用を持たず、そのclosureはbinding、引数、結果、captureへ渡らないため、programから観測できない
（function equalityとenvironmentの観察手段はない）。したがって`s[(p0) -> b0, (p1) -> b1]`は、payloadをpatternへ束縛してbodyを
評価するcase分岐と観測上同じである。既存programの動的意味は変わらず、受理集合だけが広がる。

binder applicationの意味は`if`のbranchと同じで、現在位置へ戻らずlexical targetへ移る。選択は一度だけなのでdynamic
continuationは複製されず、[affine control](../design/value-interpretation-and-control.md#選択とaffine-control)を保つ。

## D049との関係

[D049](../history/decisions/D049.md)は、sum eliminationのcontinuation位置へbare blockを置く案を、外側binderへの到達、non-Unit
payload、単一continuationとの関係、expression continuationとの混在を一つの規則で説明できるまで保留した。提案はbare blockではなく
lambda literalを使うのでpayloadはpatternとして束縛され、外側binderへの到達は上の原則、混在は要素ごとの独立な判定で説明できる。

D049はlambdaの即時applicationへの展開がstatic semanticsを変えることを理由にdirect blockを別構文とした。これは現行の規則が
その場でapplicationされるlambda literalを、中断を持つlambda値と同じ境界として扱う前提に立つ。上の原則では境界の根拠は値としての
lambdaの中断であり、redexには当てはまらない。`(pattern) -> body`は常にlambdaと読まれ、その場で一度applicationされるならredexとして
簡約される、という一つの読みで済み、位置による二重読みにはならない。採択時にD049の前提を改めるdecisionが要る。

## 適用範囲

原則はcontinuation数によらず「その場でapplicationされるlambda literalはbeta redexであり、中断を持たない」である。最初の段階を
二つ以上のcontinuationに限るのは理論の境界ではなく、変更を小さくするためである。`f(a)`と`a[f]`は同じapplicationなので、
単一continuationの`x[(n) -> body]`だけをbranch扱いにすると、同じapplicationの`((n) -> body)(x)`とbinder参照の可否が食い違う。
二つ以上のcontinuationにはcall構文による同値な形がなく、[直和の除去](../spec/expressions.md#直和の除去)がcontinuation数から
一意に決める別の操作なので、この食い違いを増やさずに済む。
