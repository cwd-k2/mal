# 直和除去continuationからのresult binder参照の導入計画

Status: Exploratory

この文書は、二つ以上のcontinuationによる直和除去で、lambda literalのcontinuationを`if`のbranchと同じlexical branchとして扱い、
result binderの参照と適用を許す案と、判断前に解決する論点を管理する。現在の言語規則は[`spec/`](../spec/)を正とする。
この提案は採択前のため、現在のprogramがこの規則へ依存してはならない。受理範囲を導く理論は
[その場でapplicationされるlambda literalとresult binderの理論的根拠](redex-and-result-binder.md)に置く。

## 目的と範囲

`if`と`when`のbranchは同じlambda invocationに属するexpression blockで、外側のresult binderを適用して早期脱出できる。
payloadを受ける直和除去のcontinuationはlambdaとして書くため外側のbinderを参照できず、成功側の続きをcontinuationの中へ
入れ子にするしかない。[`fallible-tree`](../../examples/fallible-tree/program.mal)の`buildTree`が例である。

原則は「その場でapplicationされるlambda literalはbeta redexであり、中断を持たない」とし、最初の段階の範囲を二つ以上の
continuationを持つ`value[f0, f1, ...]`に限る。単一continuationとcallee位置への拡張は[導入順](#導入順)に示す。

## 現状

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

二つ以上のcontinuationを持つ直和除去で、各continuationを次のどちらかとして扱う。

- **lambda literal**: `(pattern) -> body`をそのままの位置に置いた場合、bodyは直和除去を含むlambdaと同じinvocationに属する
  branchである。patternは対応する項のpayloadを束縛し、そのscopeはbodyに限る。bodyは外側のresult binderを参照して適用できる。
  bodyの中に書いたlambdaは従来どおりnested lambdaであり、外側のbinderをcaptureできない。
- **binder名**: continuationの位置に書いたresult binder名`k`は、payloadを受けて`k`を適用するcontinuationと同じ意味を持つ。
  binderのparameter型はその位置の項の型と一致しなければならず、binder groupの中の位置とは無関係である。

括弧は意味を持たないため、括弧で包んだlambda literalも同じ扱いにする。resolverは`Parenthesized`を保持するので、判定の前に
括弧を剥がす。binderをこの位置以外で値として使うこと、変数へ束縛したfunction、`f(x)`のような式のcontinuationは従来どおり
通常のfunction valueであり、binderを参照できない。

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

`s : [A0, ..., An]`（`n ≥ 1`）のcontinuation `ci`は、lambda literal `(pi) -> bi`なら`pi : Ai`の下で`bi`が`Value(T)`または
`Abrupt`であり、binder名なら`Abrupt`である。[completionのjoin](../spec/control.md#completion-judgment)は`if`と同じ規則を
適用する。`Abrupt`のcontinuationは結果型を制約せず、少なくとも一つが`Value(T)`なら全体は`Value(T)`、すべてが`Abrupt`なら
全体が`Abrupt`である。異なる`Value`型はtype errorである。`Bool = [Unit, Unit]`なので、`if`は同じ規則の特別な場合である。

評価は`s`を一度評価してactive variant `i`を得た後、`bi`を`pi`へpayloadを束縛して評価する。他の`bj`は評価せず、そのbodyが
unreachableかどうかも判定しない。`Abrupt`より前に完了した作用の順序は変わらない。

## 既存規則との関係

- [control.md](../spec/control.md#direct-result-block)の「nested lambdaから外側のbinderを参照することはcompile-time error」に、
  lambda literalとbinder名のcontinuationを例外として明記する。
- [直和の除去](../spec/expressions.md#直和の除去)のcontinuationの「function」という記述を、上の判定に合わせて改める。
  非選択continuationの遅延と、continuation数による除去とapplicationの区別は変えない。
- [実行意味論](../spec/execution.md#core-calculus)のcore境界への消去にsum eliminationを加える。
- [D049](../history/decisions/D049.md)の前提を改めるdecisionを加える。
- syntaxを変えないので[grammar](../spec/grammar.md)、formatter、Tree-sitterは変わらない。

## lowering、生成コードへの影響

core、ANF、closure、controlの各段は、`Case`のarmがpattern束縛とbodyを持つことと、`Value` pathのlexical joinを既に扱っている。
`if`とdirect result blockが同じ形を作るためである。変えるのは、直和除去のcontinuationをlambda値の`Call`として作る部分だけで、
closure、control、executionの各段は新しい形を受け取らない。具体的には次を変更または確認する。

- resolverは、該当するlambda literalに新しいlambda frameを作らず、parameterを囲むlambdaのlocal scopeで束縛する。
- checkerは、continuationごとにcompletionを求めてjoinし、すべてが`Abrupt`の直和除去を専用の`Abrupt`種別として返す。
- checked ASTのcontinuation表現は、lambda値とbranchを区別できる形にする。editor indexは`Abrupt`種別を網羅的に走査するため、
  新しい種別と表現を走査しなければcontinuation内のhover、definition、referencesが欠ける。
- `contains_control`のような、controlを含む部分木の判定がbranchのbodyを見るようにする。
- payloadをpatternへ直接束縛する経路の[managed value ownership](../implementation/ownership.md)を確認する。従来はpayloadを
  一度temporaryへ束縛してlambdaのparameterへ渡していた。

現行ではcontinuationが外側の変数をcaptureするとclosure environmentがheapへ確保される。最小の例（`r[(n) -> n + base, () -> base]`）で、
`baseline`と`production`のどちらも実行時に1回のallocationが残り、同じ選択を`if`で書くと0回だった。continuationごとに別functionと
呼出しも作られる。branchとしてinline化すればこれらは構造上生じない。continuation内の自己再帰が`DirectSelfTail`の対象になる
可能性もあるが未測定であり、採択の根拠にしない。性能は[generated program最適化policy](../development/generated-program-optimization.md)に
従い、`production`を対象にallocation counterを併用して別途測る。

`Value`か`Abrupt`かをeditorが利用者へ示す機能は現行にない。`Abrupt`のbranchのhoverや、後続がunreachableである理由の表示は、
[editor tooling](../development/editor-tooling.md)の別の設計判断として分けて扱う。

## 導入順

1. 仕様案で原則、lambda literalとbinder名の判定、completionを固定し、D049の前提を改めるdecisionを書く。
2. **A**: 二つ以上のcontinuationの直和除去を、resolver、checker、core loweringの順に実装する。
3. 適合testを加え、`fallible-tree`をこの形へ書き換えてextern traceが変わらないことを確認する。
4. **Cへの拡張**: 単一continuationの`x[(n) -> body]`と`((n) -> body)(x)`を、resolverで既存のblockとbindingへ置き換えて扱う。
   これは`:=`が既に持つ意味そのものなので、checkerとcore loweringに新しい形は要らない。要確認は、payloadを受けない
   `() -> body`がUnit型を要求すること、diagnosticとeditor indexのspanを保つこと、引数、callee、application順が変わらないことである。

## 検証境界

- resolve: 直接置いたlambda literalの外側binder参照が受理され、bodyの中のnested lambda、変数へ束縛したlambdaからの参照は従来の
  診断で拒否される。Cの段階では単一continuationとcallee位置も同じ扱いになる。
- check: `Value`と`Abrupt`のjoin、すべてが`Abrupt`の場合の後続unreachable、型の異なる`Value`の拒否、binder名のparameter型不一致、
  payloadを受けない項の`() -> body`とpayloadを捨てる`_`。
- lowering: 非選択continuationの作用が起きず、`Abrupt`より前の作用の順序が保たれる。continuationが外側の変数をcaptureしても
  closure environmentのallocationが生じない。
- ownership: managed payload（`Symbol`、`Buffer`）を束縛したcontinuationで、正常完了と早期脱出のどちらでもowner終状態が入れ子形と
  一致する。
- cross-boundary: cleanupを伴う早期脱出のextern traceが、入れ子形と同じ順序でbaselineとproductionの両方で一致する。
- 言語規則の適合caseを`crates/mal-compiler/tests/spec/`へ加え、[conformance matrix](../development/conformance.md)に行を追加する。

## 代替案

- **A（本文の案）**: 直和除去のlambda literalを、その場でapplicationされるredexとしてbranchとみなす。構文は変わらない。
  二重読みは、redexが中断を持たないという理論で説明できる。単一continuationと`((n) -> body)(x)`には同じ扱いが及ばず、現行の
  不整合の一部が残る。
- **B**: branch専用のcontinuation形を新設する。たとえば`(pattern) => body`と書けば、lambdaとbranchが構文で分かれる。構文、
  formatter、Tree-sitter、editor metadataを加える。Aの理論が成り立つなら区別は見た目のためだけに要り、`:=`が同じredexを別構文なしで
  扱っていることと整合しない。
- **C**: Aの原則を一般化し、その場でapplicationされるlambda literalすべてをredexとして扱う。不整合が残らない一方、影響は
  resolverが「その場でapplicationされるlambda」を判定する範囲まで広がり、`f(a)`と`a[f]`の同一性、closure captureの解析、
  editor indexのscopeを同時に確認する必要がある。Aを部分集合として先に導入し、単一continuationとcallee位置を後から加える。
  これが現在の方針である。
- **D049の候補**: bare blockをcontinuationとして置く。payloadを受けられず、non-Unitの項では別の規則が要るため採らない。
- **現状維持**: 入れ子と結果の再包装を各call siteで書き続ける。

## 採択前の未決事項

方針はCの原則を採り、Aから始めることに決めた。次が残る。

- D049の前提を改めるdecisionと、`control.md`のbinder制約の書換えを、Aの導入と同時に行う。
- Cへの拡張で、`() -> body`のUnit型要求とdiagnostic・editor indexのspanをどう保つか。
- 診断の文言。binder名のcontinuationの型不一致と、値として使えない位置の区別をどう示すか。
