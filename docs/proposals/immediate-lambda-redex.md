# その場でapplicationされるlambda literalの拡張

Status: Exploratory

この文書は、[lambdaの中断とredex](../design/value-interpretation-and-control.md#lambdaの中断とredex)の原則を、まだ適用していない
単一continuationとcallee位置のlambda literalへ広げる案と、判断前に解決する論点を管理する。現在の言語規則は[`spec/`](../spec/)を正とする。
この提案は採択前のため、現在のprogramがこの規則へ依存してはならない。

## 目的と範囲

`:=`、`if`、`when`、[直和除去のcontinuation](../spec/expressions.md#直和の除去)は、その場でapplicationされるlambda literalを
中断を持たないredexとして扱い、外側のresult binderを参照できる。同じredexである次の二つは扱われず、意味が同じ三つの書き方のうち
binding形だけがbinderを参照できる。

```mal
n := x; k(n)              // 受理
x[(n) -> k(n)]            // 拒否: result binder cannot be captured
((n) -> k(n))(x)          // 拒否: 同上
```

`f(a)`と`a[f]`は同じapplicationなので、両者は同じ規則で扱う。この提案は、その場でapplicationされるlambda literalすべてを
redexとし、原則から不整合を残さないことを目的とする。

## 提案するcontract

単一continuationの`x[(n) -> body]`と、callee位置の`((n) -> body)(x)`（括弧は剥がして判定する）を、`n := x; body`と同じblockと
bindingとして扱う。`:=`が既にこの意味を持つので、checkerとcore loweringに新しい形は要らない。resolverが「その場でapplicationされる
lambda literal」を判定し、既存のblockとbindingへ置き換える。

## 要確認

- payloadを受けない`() -> body`は引数がUnit型であることを要求する。bindingへ置き換えると`_`はどの型も受けるため、Unit型の検査を
  落とさない置き換えにする。
- diagnosticとeditor indexのspanを保つ。置き換えで生じるblockとbindingの位置を、元のlambdaとapplicationのspanへ対応させる。
- 引数、callee、applicationの評価順が変わらない。calleeがlambda literalなら評価に作用はなく、順序は引数の評価だけで決まる。
- lambda parameterに`self_binding`を持つ再帰lambdaや、変数へ束縛したlambdaは対象にならない。値として渡るlambdaはredexではない。
- `x[k]`のようなbinder名の単一continuationは既に通常のapplicationであり、変わらない。

## 検証境界

- resolve: 単一continuationとcallee位置のlambda literalが外側binderを参照でき、変数へ束縛したlambdaと、その中のnested lambdaは
  従来の診断で拒否される。
- check: `() -> body`のUnit型要求、`_`と`(a, b)`のpattern、期待型の伝播。
- lowering: `n := x; body`と同じcore、closure environmentのallocationがない。
- editor: hover、definition、referencesが置き換えの前後で同じspanを返す。
- 言語規則の適合caseを`crates/mal-compiler/tests/spec/`へ加える。

## 採択前の未決事項

- resolverでの置き換えとcheckerでの分岐のどちらを実装の単位にするか。resolverで置き換えるとeditor indexとdiagnosticのspanの
  対応が課題になり、checkerで分岐するとchecked ASTに新しい形が要る。
