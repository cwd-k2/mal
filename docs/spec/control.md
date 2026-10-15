# result boundary、`when`、`Empty`

Status: Accepted v0.7

この文書はlexical result boundary、`when`、空直和`[]`の個別規則を定める。sequence、branch、boundaryを跨ぐcompletionの規則は
[completionとlexical edgeの合成](control-composition.md)、通常のlambda、application、sum eliminationの規則は
[式とbinding](expressions.md)を正とする。

## direct result block

lambda body内では、result binder group、`=>`、body expressionを一つのexpressionとして書ける。`->`はfunction typeとlambda、
`=>`はlexical result boundaryの導入だけに使う。

```mal
answer :: Unit -> Int32 := () -> [done] => done(42);

choose :: Bool -> [Int32, Symbol] :=
    (condition) -> [integer, symbol] => {
        when (condition) integer(42);
        symbol("disabled")
    };
```

`[k] => body`は期待result型`T`を必要とし、`k : T`をbodyへ導入してbodyを直ちに実行する。複数のbody itemが
必要ならblock expressionを置く。二つ以上のbinderでは
期待result型が同じ項数の直和`[T0, T1, ...]`でなければならず、位置`i`のbinderは`Ti`を受けて第`i`項を選ぶ。
binder数はalias展開後の項数と完全に一致しなければならず、省略、追加、部分指定は認めない。一つのbinderは期待result型全体を
parameterとして受ける。binderはblock-local scopeを持ち、group内の名前重複はcompile-time errorである。外側のvalue nameは
通常どおりshadowできる。binderはcallee位置でのapplicationにだけ使え、値としてbinding、argument、aggregate、result、capture
できない。ただし[直和の除去](expressions.md#直和の除去)のcontinuationとして直接置いたbinder名は、payloadを適用する
applicationとして扱う。値としてのlambdaのbodyから外側のbinderを参照することは、closureの実際のescapeにかかわらず
compile-time errorである。直和除去のcontinuationとして直接置いたlambda literalは値を作らず、囲むinvocationに属する
branchなので、この対象ではない。

通常のapplicationと同じく、`result(value)`と`value[result]`は同じbinder applicationである。Unit payloadでは
`result()`、`()[result]`、`[result]`が同じ意味になる。複数binderの位置`i`を適用するとindex `i`のsum valueを構築する。
直和値を構築するsource-levelの方法はdirect result blockのbinderだけであり、binderはglobal constructor、first-class injection
function、nominal identityを作らない。

result binderを適用するとargumentを評価した後、blockのresult edgeへ移り、result block全体が選択した値で正常完了する。
bodyはすべてのreachable pathで自身または外側のresult binder application、またはempty eliminationにより
`Abrupt`でなければならず、block自身のresult binderを少なくとも一つのreachable pathで適用しなければならない。同じlambda
invocationに属する外側のresult binderは内側のresult blockを越えて外側blockを完了し、内側blockのresult edgeには合流しない。
この接続規則は[lexical result boundary](control-composition.md#lexical-result-boundary)に定める。

この構文はfunction valueを作らない。`[k]`だけならUnitを`k`へ渡すapplicationである。binderを持たない
`[] => body`は認めない。

## when

`when`は`Bool`に対する`Unit` control expressionである。

```mal
classify :: Int32 -> Symbol :=
    (x) -> [return] => {
        when (x == 0) return("zero");
        return("other")
    };
```

`when (condition) body`は`if (condition) then body else ()`と同じである。bodyは一つのexpressionであり、複数のbody itemが
必要ならblock expressionを置く。conditionは一度だけ評価し、bodyのcompletionは
`Value(Unit)`または`Abrupt`でなければならない。`when`全体は`Value(Unit)`になり、false pathだけが後続へfall throughする。
branchとの合成は[Branch](control-composition.md#branch)に定める。

## Empty

`[]`は空直和`Empty`である。値は存在せず、payloadを受けるsum result binderも作れない。`value[]`は`value : []`を要求する
zero-continuation eliminationであり、`Abrupt`になる。bare `[]`は式ではない。

`Empty`は値のない型、`Abrupt`は現在のpathが通常完了しないcompletionであり、同一ではない。`Empty`はparameter、aggregate field、
通常関数やexternal operationのresultなど、他のvalue typeと同じ型位置に置ける。閉じた通常計算から値を構築する方法はない。

`A -> []`のlambda bodyは`Abrupt`でなければならない。

```mal
never :: Unit -> [] :=
    () -> never()[];
```
