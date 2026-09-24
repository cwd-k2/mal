# result boundaryとcompletion

Status: Accepted v0.6

この文書はlexical result boundary、式のcompletion、`when`、空直和`[]`の規則を定める。
通常のlambda、application、sum eliminationの規則は[式とbinding](expressions.md)を正とする。

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
できない。nested lambdaから外側のbinderを参照することは、closureの実際のescapeにかかわらずcompile-time errorである。

通常のapplicationと同じく、`result(value)`と`value[result]`は同じbinder applicationである。Unit payloadでは
`result()`、`()[result]`、`[result]`が同じ意味になる。複数binderの位置`i`を適用するとindex `i`のsum valueを構築する。
直和値を構築するsource-levelの方法はdirect result blockのbinderだけであり、binderはglobal constructor、first-class injection
function、nominal identityを作らない。

result binderを適用するとargumentを評価した後、blockのresult edgeへ移り、result block全体が選択した値で正常完了する。
bodyはすべてのreachable pathで自身または外側のresult binder application、またはempty eliminationにより
`Abrupt`でなければならず、block自身のresult binderを少なくとも一つのreachable pathで適用しなければならない。同じlambda
invocationに属する外側のresult binderは内側のresult blockを越えて外側blockを完了し、内側blockのresult edgeには合流しない。

この構文はfunction valueを作らない。`[k]`だけならUnitを`k`へ渡すapplicationである。binderを持たない
`[] => body`は認めない。

## completion judgment

checkerは式の通常完了をsource typeとは別に`Value(T)`または`Abrupt`へ分類する。`Abrupt`は型でもsubtypeでもなく、期待型や
仮のvalue typeを持たない。

```text
join(Value(T), Value(T)) = Value(T)
join(Value(T), Abrupt)   = Value(T)
join(Abrupt, Value(T))   = Value(T)
join(Abrupt, Abrupt)     = Abrupt
```

異なる二つの`Value`型は通常どおりtype errorになる。result binder applicationとempty eliminationは`Abrupt`である。
期待result型を`B`とするlambda bodyは`Value(B)`または`Abrupt`でよい。後者のinvocationは正常にはreturnしない。
early resultを使うlambdaではbody全体をdirect result blockにし、そのblockが作った`Value(B)`でlambdaを正常完了する。

binding initializerは`Value(T)`でなければならない。block途中の式が`Abrupt`なら、それより後のsource itemはunreachable codeとして
compile-time errorになる。product要素、operator operand、argument、callee、continuationなどstrictに値を要求する位置で
`Abrupt`になれば囲む式も`Abrupt`になり、評価順で先にある式の作用は保持し、後にあるsubexpressionはunreachableとして拒否する。
短絡演算とsum eliminationでは選択されないbranchをunreachableとはしない。

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

## 評価とlowering

result binder applicationはargumentを通常の順序で一度評価した後、宣言したblockのresult edgeへ接続する。sum binderはsum injectionを
行って同じresult edgeへ接続する。lambda bodyの`Value` pathはlambda resultへ接続する。`when`はBool branch、empty eliminationは
zero-arm sum eliminationへ変換する。core loweringは
`Abrupt` pathでは後続を捨て、`Value` pathにだけlexicalな後続を接続する。この変換は既に完了した`extern`作用を省略・重複・並べ替えない。

result binderとcompletion judgmentはcore境界で消去する。複数の`Value` pathが同じlexical continuationへ進む場合、core、ANF、
closure IRは後続を複製せず非first-classなjoin identityへのtransferとして保持し、control loweringがinput付きstateへのjumpへ変換する。
`Abrupt` pathは対応するresult boundaryへ進むか、empty eliminationにより正常完了しない。
