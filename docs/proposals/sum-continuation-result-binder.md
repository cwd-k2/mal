# 直和除去continuationからのresult binder参照の導入計画

Status: Exploratory

この文書は、二つ以上のcontinuationによる直和除去で、lambda literalのcontinuationを`if`のbranchと同じlexical branchとして扱い、
result binderの参照と適用を許す案と、判断前に解決する論点を管理する。現在の言語規則は[`spec/`](../spec/)を正とする。
この提案は採択前のため、現在のprogramがこの規則へ依存してはならない。

## 目的と範囲

`if`と`when`のbranchは同じlambda invocationに属するexpression blockであり、外側のresult binderを適用して早期脱出できる。
一方、payloadを受ける直和除去のcontinuationはlambdaとして書くため外側のbinderを参照できず、成功側の続きをcontinuationの中へ
入れ子にするしかない。[`fallible-tree`](../../examples/fallible-tree/program.mal)の`buildTree`が例である。

原則は「その場でapplicationされるlambda literalはbeta redexであり、中断を持たない」とし、最初の段階の範囲を二つ以上の
continuationを持つ`value[f0, f1, ...]`に限る。単一continuationのapplication、callee位置のlambda literalへの拡張は
[導入順](#導入順)に示す。binderを値として使う位置は変えない。

## 現状

現行の`malc`は次を受理または拒否する。

```mal
v := if (c) then 1i32 else fail();   // 受理: elseはAbruptで、joinはValue(Int32)
x[k]                                 // 受理: k(x)と同じbinder application
v := r[(n) -> n, () -> fail()];      // 拒否: result binder cannot be captured
v := r[ok, fail];                    // 拒否: result binder is not a value
```

`if`のbranchはASTが`ExpressionBlock`を持ち、lambdaではない。直和除去のcontinuationは`Lambda`であり、resolverはlambdaごとに
invocationを分けるため、内側から外側のbinderを参照すると拒否される。core loweringは各continuationをlambda値の`Call`として
`Case` armへ置く。

## 提案するcontract

二つ以上のcontinuationを持つ直和除去で、各continuationを次のどちらかとして扱えるようにする。

- **lambda literal**: `(pattern) -> body`をそのままの位置に置いた場合、bodyは直和除去を含むlambdaと同じinvocationに属する
  branchである。patternは対応する項のpayloadを束縛し、そのscopeはbodyに限る。bodyは外側のresult binderを参照して適用できる。
  bodyの中に書いたlambdaは従来どおりnested lambdaであり、外側のbinderをcaptureできない。
- **binder名**: continuationの位置に書いたresult binder名`k`は、payloadを受けて`k`を適用するcontinuationと同じ意味を持つ。
  binderのparameter型はその位置の項の型と一致しなければならない。binder groupの中の位置とは無関係である。

binderをこの位置以外で値として使うこと、括弧で包んだlambda、変数へ束縛したfunction、`f(x)`のような式のcontinuationは
従来どおり通常のfunction valueであり、binderを参照できない。

```mal
buildLeftSubtree :: Allocator -> NodeBuildResult := (allocator) -> [created, failed] => {
    left  := allocator.createLeaf(3)[(n) -> n, () -> failed()];
    right := allocator.createLeaf(4)[
        (n) -> n,
        () -> { left.releaseSubtree(allocator); failed() }
    ];
    allocator.createOwnedBranch(2, left, right)[created, failed]
};
```

### completionと型

各continuationのbodyは`Value(T)`または`Abrupt`であり、[completionのjoin](../spec/control.md#completion-judgment)を`if`と
同じ規則で適用する。`Abrupt`のcontinuationは結果型を制約しない。少なくとも一つが`Value(T)`ならば、直和除去全体は`Value(T)`、
すべてが`Abrupt`ならば直和除去全体が`Abrupt`である。複数の`Value`型が異なる場合はtype errorである。

選択されないcontinuationは評価されず、そのbodyがunreachableかどうかも判定しない。scrutineeの評価と、`Abrupt`より前に
完了した作用の順序は変わらない。

## 既存規則との整合

### 型付けと評価の規則

[completion judgment](../spec/control.md#completion-judgment)を`Γ ⊢ e ⇒ Value(T)`または`Γ ⊢ e ⇒ Abrupt`と書く。
`s : [A0, ..., An]`、`n ≥ 1`（continuationは二つ以上）、各continuation `ci`のpayload型を`Ai`とする。lambda literal `(pi) -> bi`では
`Γ, pi : Ai ⊢ bi ⇒ Ci`とし、binder名`k`では`k`のparameter型が`Ai`であるときCiを`Abrupt`とする。

```text
Ci ∈ { Value(T), Abrupt }   少なくとも一つは Value(T)
--------------------------------------------------
Γ ⊢ s[c0, ..., cn] ⇒ Value(T)

Ci = Abrupt  (すべてのi)
--------------------------------------------------
Γ ⊢ s[c0, ..., cn] ⇒ Abrupt
```

これは`join`の畳込みであり、`if`の規則と同形である。`Bool = [Unit, Unit]`なので、`if`は`n = 1`かつpayloadが
Unitの特別な場合として同じ規則に含まれる。

評価は`s`を一度評価してactive variant `i`を得た後、`bi`を`pi`へpayloadを束縛して評価する。他の`bj`は評価しない。
`bi`が`Abrupt`のときは、それより前に完了した評価だけが残り、対応するresult boundaryへ移る。この二つの規則は
[実行意味論](../spec/execution.md#評価戦略)の直和除去と、[control.md](../spec/control.md)の`Abrupt`の定義から導かれ、
新しい概念を持ち込まない。

### lambdaの意味からの位置づけ

lambdaは評価の中断（suspension）を導入する。bodyはapplicationまで実行されず、複数回applicationされ得て、定義したblockの
完了後にも残り得る。result binderが「nested lambdaから参照できない」のは、この中断がbinderの生存範囲
（block invocationの実行中）を越え得るからである。禁止の根拠はlambdaの構文ではなく、値としてのlambdaが持つ中断にある。

一方、lambda literalをその場で一度だけapplicationする形は中断を残さない。strictなmalではこれはbeta redexである。

```text
x := e; rest        =   (x -> rest)(e)   =   e[(x) -> rest]          （let）
s[(p0) -> b0, ...]  =   case s of inl p0 => b0 | ...               （case）
```

[core calculus](../spec/execution.md#core-calculus)は`:=`をlambda applicationへ消去できると定めている。beta簡約は
patternの束縛変数へ実引数を置換するだけで、束縛名の選択は意味に影響しない（α同値）。したがってredexのbodyは
囲むlambdaと同じinvocationに属する。この読みは既に`:=`とblockで使われている。次は意味が同じ三つの書き方だが、
現行は最初のものだけがbinderを参照できる。

```mal
n := x; k(n)              // 受理
x[(n) -> k(n)]            // 拒否: result binder cannot be captured
((n) -> k(n))(x)          // 拒否: 同上
```

現行の規則は、beta redexであるlambda literalをbindingとそれ以外で区別している。これは本提案と無関係に存在する
不整合であり、本提案の直和除去のlambda literalはその一部を解消する。選択された一つのcontinuationだけがpayloadへ
一度applicationされるため、`case`の分岐と読める。

result binderは、囲むblockのjoin pointへのjumpとして働くsecond-classなcontinuation targetである。
[control.md](../spec/control.md#評価とlowering)がbinderをcore境界で「非first-classなjoin identityへのtransfer」として
保持することとも一致する。join pointへのjumpは、そのinvocation内のtail contextであれば`let`のbodyや`case`の分岐からも
行える。行えないのは、中断されたlambda値のbodyからだけである。

### CPSとreturn binderからの位置づけ

result binderはCPSに基づく明示的return binderとして導入された（旧`docs/design/cps-return.md`、commit `14a41a58`。
現在の規範は[control.md](../spec/control.md)）。通常関数`A -> B`は概念上`∀R. A -> (B -> R) -> R`のCPSに対応し、
binderはそのinvocationのreturn continuation `k`にlambda-localな名前を与えるものである。この見方で、本提案の境界は
次のように定まる。

- **lambda値は新しい`k`を導入する。** `(x) -> body`の各invocationは自分のreturn edgeを持ち、bodyが実行される時点は
  外側の`k`の生存範囲を越え得て、複数回でもあり得る。nested lambdaから外側のbinderを参照できないのは、この理由による。
- **`let`、`case`、`if`は`k`を継承する。** CPS変換ではこれらは新しいreturn continuationを作らないadministrative redexで、
  bodyや分岐は囲むinvocationの`k`のまま評価される。したがってbinderは、`k`を継承する文脈すべてで見える。
  `:=`、`if`、`when`、direct result blockは既にこの範囲にあり、本提案は直和除去のlambda literalを加える。

つまり、既存の禁止を緩めるのではなく、「`k`を継承する文脈」の判定を、既に継承している構文から、同じ性質を持つ
直和除去のcontinuationへ広げる。lambda値には新しい`k`が要るという根拠は変わらない。

この規則はescape analysisではない。[prior-art](../research/prior-art.md#closure)が採らないとした「escapeしなければcapture可」は、
closure値の実際のescapeで受理を変える。本提案は、redexにclosure値が作られないという分類で判定するため、
実装やcompiler最適化の結果に依存しない。

**入れ子形は手書きのCPS、平坦な形は直接形とabortである。** 現行のcontinuationの入れ子は、成功側の残りの計算`rest`を
continuationの本体へ入れる、手書きのCPS変換にあたる。本提案の平坦な形は、その`rest`を外へ出す。二つは、失敗側が
`Abrupt`（現在のevaluation contextを捨てる）であることから、次の等式で結ばれる。

```text
E[ s[(n) -> n, () -> k(u)] ]   =   s[(n) -> E[n], () -> k(u)]

s[(n) -> { rest(n) }, () -> failed()]         // 入れ子（CPS形）
v := s[(n) -> n, () -> failed()]; rest(v)     // 平坦（直接形）
```

`E`は評価contextで、右辺の`Abrupt`側は`E`を捨てるため`E[k(u)] = k(u)`が成り立つ。これは`case`のcommuting conversionと、
jumpが評価contextを捨てるというjoin pointの標準的な等式にあたる。二つの形が同じlexical join
（[D047](../history/decisions/D047.md)）を作れば、`rest`はどちらでも一度だけ生成される。D047が棄却した
synthetic lambdaとcallによる後続の共有は、現行のcontinuation lambdaが実質的に行っているものである。

この整理で、利用者は成功側だけを上から下へ書け、CPS変換を意識しなくてよい。これはcps-returnがbinderで
noop continuationとeta expansionを不要にしたことの、直和の除去側への延長である。

理論が与えるのは受理範囲と等式であり、AとBのどちらの表記を採るかは設計判断として残る。

### 操作的意味は変わらない

[実行意味論](../spec/execution.md#評価戦略)は、直和除去が選択されたcontinuationだけを評価してpayloadへ一度適用すると定める。
lambda literalの評価は作用を持たず、そのclosureはbinding、引数、結果、captureへ渡らないため、program上で観測できない
（function equalityとenvironmentの観察手段はない）。したがって`s[(p0) -> b0, (p1) -> b1]`は、payloadを
patternへ束縛してbodyを評価するcase分岐と観測上同じである。この提案は既存programの動的意味を変えず、受理集合だけを広げる。
現在のcore loweringが作る`Case` armはすでにpayloadのpattern束縛を持つため、この読みは実装の形にも一致する。

### binderはescapeしない

[binderの制約](../spec/control.md#direct-result-block)はnested lambdaからの参照を「closureの実際のescapeにかかわらず」
禁じる。continuationとして直接置いたlambda literalは値にならず、直和除去の一度の適用でだけ消費されるため、escapeの
可能性が構文位置から静的に決まる。binder applicationの意味は`if`のbranchと同じで、現在位置へ戻らずlexical targetへ移る。
選択は一度だけなのでdynamic continuationは複製されず、[affine control](../design/value-interpretation-and-control.md#選択とaffine-control)を保つ。

### 適用範囲

本文のcontractは二つ以上のcontinuationに限る。これは理論から導かれる境界ではなく、変更を小さくするための段階である。
`f(a)`と`a[f]`は同じapplicationなので、単一continuationの`x[(n) -> body]`だけをbranch扱いにすると、同じapplicationの
`((n) -> body)(x)`とbinder参照の可否が食い違う。二つ以上のcontinuationにはcall構文による同値な形がなく、
[直和の除去](../spec/expressions.md#直和の除去)がcontinuation数から一意に決める別の操作なので、この食い違いを増やさずに
済む。単一continuationの`x[k]`は現行どおり通常のapplicationである。

上の位置づけに従えば、原則はcontinuation数によらず「その場でapplicationされるlambda literalはbeta redexであり、中断を
持たない」である。この原則を一般化した場合の範囲と影響は[代替案](#代替案)のCに示す。

### D049の条件への回答

[D049](../history/decisions/D049.md)は、sum eliminationのcontinuation位置へbare blockを置く案を、外側binderへの到達、
non-Unit payload、単一continuationとの関係、expression continuationとの混在を一つの規則で説明できるまで保留した。
この提案はbare blockではなくlambda literalを使うのでpayloadはpatternとして束縛される。外側binderへの到達は本規則、
単一continuationは上の境界、混在は要素ごとの独立な判定（lambda literal、binder名、その他）で説明できる。

D049はlambdaの即時applicationへの展開がstatic semanticsを変えることを理由にdirect blockを別構文とした。
現行の規則は、その場でapplicationされるlambda literalを中断を持つlambda値と同じ境界として扱うため、この理由が成り立つ。
上の位置づけでは、境界の根拠は値としてのlambdaの中断であり、beta redexには当てはまらない。この見方を採ると、
`(pattern) -> body`は常にlambdaと読まれ、その場で一度applicationされるならredexとして簡約される、という一つの読みで
済み、位置による二重読みにはならない。D049の判断は、redexを境界とみなすという前提の上に立つため、採択時に
その前提を改めるdecisionを加える必要がある。

### 採択基準

[最小性](../design/minimality.md#機能を加える判定基準)のうち、次を確認する。

- 既存の組合せでは、payload付きの選択からの早期脱出が入れ子でしか書けず、`if`とのasymmetryがある。
- 型規則はcompletionのjoinの再利用で、評価規則は上の同値で説明できる。新しいcore termは要らない。
- hidden allocationは増えず、continuationのclosure構築を省ける。
- authorityは変わらない。
- 独立contractを一つ加える代わりに、「continuationの入れ子と結果の再包装」を各call siteで書く再判断が減る。
  二重読みのcostは代替案Bと比較して判断する。

## 既存規則との関係

- [result boundaryとcompletion](../spec/control.md#direct-result-block)の「nested lambdaから外側のbinderを参照することは
  compile-time error」に、本規則のlambda literalとbinder名を例外として明記する。
- [直和の除去](../spec/expressions.md#直和の除去)のcontinuationは「function」という記述を、上の三種の判定に合わせて改める。
  非選択continuationの遅延と、continuation数による除去とapplicationの区別は変えない。
- [実行意味論](../spec/execution.md#core-calculus)のcore境界への消去にsum eliminationを加える。
- syntaxを変えないので[grammar](../spec/grammar.md)、formatter、Tree-sitterは変わらない。

## Editor上の扱い

すべてのcontinuationが`Abrupt`の直和除去は、`if`と同様にcheckerが専用の`Abrupt`種別として返す。editor indexは
[checked ASTの`Abrupt`種別](../../crates/mal-frontend/src/editor/index/checked_ast.rs)を網羅的に走査しているため、
この種別を走査しなければcontinuation内のhover、definition、referencesが欠ける。これは本提案の実装範囲に含める。

一方、現行のeditorは式のcompletion（`Value`か`Abrupt`か）を利用者へ示さない。早期脱出がcontinuationの位置に
現れると、どの分岐が現在のpathを打ち切るかがsourceからは読みにくくなる。`Abrupt`のbranchのhoverや、後続が
unreachableである理由の表示は、[editor tooling](../development/editor-tooling.md)の別の設計判断として、この提案とは分けて扱う。

## Baseline実行形

resolverは位置で判定したlambda literalのcontinuationに新しいlambda frameを作らず、parameterを囲むlambdaのlocal scopeで
束縛する。captureは発生しない。checkerはcontinuationごとにcompletionを求めてjoinし、すべてが`Abrupt`の場合は直和除去を
`Abrupt`として返す。core loweringはlambda literalを`Case` armのpattern付きbodyとして`if`のbranchと同じjoinへ接続し、
binder名のcontinuationをresult transferへ変える。ANF以降は変更しない。

## lowering、生成コードへの影響

core、ANF、closure、controlの各段は、`Case`のarmがpattern束縛とbodyを持つことと、`Value` pathのlexical joinを既に
扱っている。`if`（[Bool elimination](../../crates/mal-backend/src/core/README.md)）とdirect result blockが同じ形を作るためである。
本提案でcore loweringが変えるのは、直和除去のcontinuationをlambda値の`Call`として作る部分だけで、closure、control、
executionの各段は新しい形を受け取らない。一方、次は変更または確認が必要である。

- checked ASTのcontinuation表現（lambda値とbranchを区別できる形）と、editor indexの走査。
- `contains_control`のような、controlを含む部分木の判定がbranchのbodyを見ること。
- payloadをpatternへ直接束縛する経路の[managed value ownership](../implementation/ownership.md)。従来はpayloadを
  一度temporaryへ束縛してlambdaのparameterへ渡していた。

生成コードには利点が見込める。現行ではcontinuationが外側の変数をcaptureするとclosure environmentがheapへ確保される。
最小の例（`r[(n) -> n + base, () -> base]`）で、`baseline`と`production`のどちらも実行時に1回のallocationが残り、
同じ選択を`if`で書くと0回だった。continuationごとに別functionと呼出しも作られる。branchとしてinline化すれば、
これらは構造上生じない。continuation内の自己再帰が、別functionを経由するrecursive regionではなく
`DirectSelfTail`の対象になる可能性もあるが、未測定であり、採択の根拠にしない。性能は
[generated program最適化policy](../development/generated-program-optimization.md)に従い、`production`を対象に
allocation counterを併用して別途測る。

## 導入順

決定した方針は、「その場でapplicationされるlambda literalはredexで中断を持たない」を原則とし、Aから始めることである。

1. 仕様案で原則、lambda literalとbinder名の判定、completionを固定し、D049の前提を改めるdecisionを書く。
2. **A**: 二つ以上のcontinuationの直和除去を、resolver、checker、core loweringの順に実装する。editor indexは
   continuationのparameterを通常のlocalとして扱う。
3. 適合testを加え、`fallible-tree`をこの形へ書き換えてextern traceが変わらないことを確認する。
4. **Cへの拡張**: 単一continuationの`x[(n) -> body]`と`((n) -> body)(x)`を、resolverで既存のblockとbindingへ
   置き換えて扱う。これは`:=`が既に持つ意味そのものなので、checkerとcore loweringに新しい形は要らない。
   要確認は、payloadを受けない`() -> body`がUnit型を要求すること、diagnosticとeditor indexのspanを保つこと、
   引数、callee、application順が変わらないことである。

## 検証境界

- resolve: 直接置いたlambda literalの外側binder参照が受理され、bodyの中のnested lambda、括弧で包んだlambda、
  変数へ束縛したlambdaからの参照は従来の診断で拒否される。Cの段階では単一continuationとcallee位置も同じ扱いになる。
- check: `Value`と`Abrupt`のjoin、すべてが`Abrupt`の場合の後続unreachable、型の異なる`Value`の拒否、
  binder名のparameter型不一致、payloadを受けない項の`() -> body`とpayloadを捨てる`_`。
- lowering: 非選択continuationの作用が起きず、`Abrupt`より前の作用の順序が保たれる。continuationが外側の変数を
  captureしても、closure environmentのallocationが生じない。
- ownership: managed payload（`Symbol`、`Buffer`）を束縛したcontinuationで、正常完了と早期脱出のどちらでも
  owner終状態が入れ子形と一致する。
- cross-boundary: cleanupを伴う早期脱出のextern traceが、入れ子形と同じ順序でbaselineとproductionの両方で一致する。
- 言語規則の適合caseを`crates/mal-compiler/tests/spec/`へ加え、[conformance matrix](../development/conformance.md)に
  行を追加する。

## 代替案

- **A（本文の案）**: 直和除去のlambda literalを、その場でapplicationされるredexとしてbranchとみなす。構文は変わらない。
  二重読みは、redexが中断を持たないという理論で説明できる。ただし、単一continuationと`((n) -> body)(x)`には
  同じ扱いが及ばず、現行の不整合の一部が残る。
- **B**: branch専用のcontinuation形を新設する。たとえば`(pattern) => body`と書けば、lambdaとbranchが構文で分かれる。
  構文、formatter、Tree-sitter、editor metadataを加える。Aの理論が成り立つなら、区別は意味ではなく見た目のためだけに
  要り、`:=`が同じredexを別構文なしで扱っていることと整合しない。
- **C**: Aの原則を一般化し、その場でapplicationされるlambda literal（単一continuationの`x[(n) -> body]`と
  `((n) -> body)(x)`を含む）すべてをredexとして扱う。`:=`、`if`、直和除去、callee位置のlambdaが同じ規則になり、
  不整合が残らない。一方、影響はresolverが「その場でapplicationされるlambda」を判定する範囲まで広がり、
  `f(a)`と`a[f]`の同一性、closure captureの解析、editor indexのscopeを同時に確認する必要がある。
  Aを部分集合として先に導入し、単一continuationとcallee位置を後から加える。これが現在の方針である。
- **D049の候補**: bare blockをcontinuationとして置く。payloadを受けられず、non-Unitの項では別の規則が要るため採らない。
- **現状維持**: 入れ子と結果の再包装を各call siteで書き続ける。

## 採択前の未決事項

方針はCの原則を採り、Aから始めることに決めた。次が残る。

- 括弧で包んだlambda literalをredexとみなすか。原則を意味で定義するなら括弧は関係しないが、Aの段階の判定を
  構文位置に限るか、括弧を剥がして判定するかを決める。
- D049の前提（redexを境界とみなす）を改めるdecisionと、`control.md`のbinder制約の書換えを、Aの導入と同時に行う。
- Cへの拡張で、`() -> body`のUnit型要求とdiagnostic・editor indexのspanをどう保つか。
- 診断の文言。binder名のcontinuationの型不一致と、値として使えない位置の区別をどう示すか。
