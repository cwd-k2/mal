# 明示的returnとcompletion

Status: Current v0.5 profile

この文書は通常関数のreturn edgeへlocal nameを与えるreturn binder、式のcompletion、`when`、空直和`[]`の規則を定める。
通常のlambda、application、sum eliminationの規則は[式とbinding](expressions.md)を正とする。

## return binder

lambdaはparameter groupとbodyの間にreturn binder groupを置ける。

```mal
absolute :: Int32 -> Int32 :=
    (x)[return] -> {
        when (x >= 0) return(x);
        return(-x)
    };
```

期待関数型を`A -> B`とすると、parameterは`A`に対して検査し、一つのbinderは`B`をparameterとして受ける。
`return(value)`は`value : B`を検査し、現在のfunction invocationを`value`で完了する。そのcall siteの後続へは戻らない。
関数型と通常のcall siteは変わらず、`absolute(value)`は`Int32`を返す。

通常のapplicationと同じく、`return(value)`と`value[return]`は同じbinder applicationである。Unit payloadでは
`return()`、`()[return]`、`[return]`が同じ意味になる。

binderはparameterと同じlambda-local scopeでbody全体から参照できる。同じlambdaのparameterまたはreturn binderとの名前重複は
compile-time errorである。外側のvalue nameは通常どおりshadowできる。binderはcallee位置でのapplicationにだけ使え、値として
binding、argument、aggregate、return、captureできない。nested lambdaから外側のbinderを参照することは、closureの実際のescapeに
かかわらずcompile-time errorである。

return binderは宣言した一つのlambdaのresult型だけに対応する。`A -> B -> C`の外側のlambdaでは`B -> C`、内側では`C`が
result型であり、両者のreturn boundaryは異なる。

## sum return binder

result型が`[A, B, ...]`なら、項数と同じ二つ以上のbinderを宣言できる。

```mal
Result :: [Int32, Symbol];

compute :: Bool -> Result :=
    (enabled)[ok, err] -> {
        when (enabled) ok(42);
        err("disabled")
    };
```

位置`i`のbinderは第`i`項のpayloadを受け、index `i`のsum valueを返す。binder数はalias展開後の項数と完全に一致しなければならず、
省略、追加、部分指定は認めない。sum resultに一つだけbinderを置いた場合はvariant binderではなく、sum value全体を受ける通常の
単一return binderである。直和値を構築するsource-levelの方法はlambdaまたはdirect result blockのsum binderだけであり、
binderはglobal constructor、first-class injection function、nominal identityを作らない。

## direct result block

lambda body内では、parameter groupを伴わないresult binder group、`->`、body expressionを一つのexpressionとして書ける。

```mal
answer :: Unit -> Int32 := () -> {
    [done] -> done(42)
};

choose :: Bool -> [Int32, Symbol] := (condition) -> {
    [integer, symbol] -> {
        when (condition) integer(42);
        symbol("disabled")
    }
};
```

`[k] -> body`は期待result型`T`を必要とし、`k : T`をbodyへ導入してbodyを直ちに実行する。複数のbody itemが
必要ならblock expressionを置く。二つ以上のbinderでは
期待result型が同じ項数の直和`[T0, T1, ...]`でなければならず、位置`i`のbinderは`Ti`を受けて第`i`項を選ぶ。
binderのscope、callee位置への制限、重複、nested lambdaからのcapture拒否はlambda return binderと同じである。

result binderを適用するとargumentを評価した後、blockのresult edgeへ移り、result block全体が選択した値で正常完了する。
bodyはすべてのreachable pathでresult binder application、外側lambdaのreturn binder application、またはempty eliminationにより
`Abrupt`でなければならず、block自身のresult binderを少なくとも一つのreachable pathで適用しなければならない。外側lambdaの
return binderはresult blockを越えてlambdaを完了し、result blockのresult edgeには合流しない。

この構文はfunction valueを作らず、`[()[k] -> body]`への省略でもない。`[k]`だけなら従来どおりUnitを`k`へ渡す
applicationである。binderを持たない`[] -> body`は認めず、empty resultにはlambdaのempty return binder groupを使う。

## completion judgment

checkerは式の通常完了をsource typeとは別に`Value(T)`または`Abrupt`へ分類する。`Abrupt`は型でもsubtypeでもなく、期待型や
仮のvalue typeを持たない。

```text
join(Value(T), Value(T)) = Value(T)
join(Value(T), Abrupt)   = Value(T)
join(Abrupt, Value(T))   = Value(T)
join(Abrupt, Abrupt)     = Abrupt
```

異なる二つの`Value`型は通常どおりtype errorになる。return binder applicationとempty eliminationは`Abrupt`である。
return binder groupのないlambda bodyは期待result型`B`に対する`Value(B)`、groupのあるbodyは`Abrupt`でなければならない。
したがって明示的returnを使うlambdaでは、すべてのreachable pathがreturn binder applicationまたはempty eliminationで完了する。

binding initializerは`Value(T)`でなければならない。block途中の式が`Abrupt`なら、それより後のsource itemはunreachable codeとして
compile-time errorになる。product要素、operator operand、argument、callee、continuationなどstrictに値を要求する位置で
`Abrupt`になれば囲む式も`Abrupt`になり、評価順で先にある式の作用は保持し、後にあるsubexpressionはunreachableとして拒否する。
短絡演算とsum eliminationでは選択されないbranchをunreachableとはしない。

## when

`when`は`Bool`に対する`Unit` control expressionである。

```mal
classify :: Int32 -> Symbol :=
    (x)[return] -> {
        when (x == 0) return("zero");
        return("other")
    };
```

`when (condition) body`は`if (condition) then body else ()`と同じである。bodyは一つのexpressionであり、複数のbody itemが
必要ならblock expressionを置く。conditionは一度だけ評価し、bodyのcompletionは
`Value(Unit)`または`Abrupt`でなければならない。`when`全体は`Value(Unit)`になり、false pathだけが後続へfall throughする。

## Empty

`[]`は空直和`Empty`である。値は存在せず、payloadを受けるsum return binderも作れない。`value[]`は`value : []`を要求する
zero-continuation eliminationであり、`Abrupt`になる。bare `[]`は式ではない。

`Empty`は値のない型、`Abrupt`は現在のpathが通常完了しないcompletionであり、同一ではない。`Empty`はparameter、aggregate field、
通常関数やexternal operationのresultなど、他のvalue typeと同じ型位置に置ける。閉じた通常計算から値を構築する方法はない。

`A -> []`のlambdaはempty return binder groupだけを宣言でき、bodyは`Abrupt`でなければならない。

```mal
never :: Unit -> [] :=
    ()[] -> never()[];
```

## 評価とlowering

return binder applicationはargumentを通常の順序で一度評価した後、現在のlambda resultへ接続する。sum binderはsum injectionを行って
同じresult edgeへ接続する。`when`はBool branch、empty eliminationはzero-arm sum eliminationへ変換する。core loweringは
`Abrupt` pathでは後続を捨て、`Value` pathにだけlexicalな後続を接続する。この変換は既に完了した`extern`作用を省略・重複・並べ替えない。

return binderとcompletion judgmentはcore境界で消去する。複数の`Value` pathが同じlexical continuationへ進む場合、core、ANF、
closure IRは後続を複製せず非first-classなjoin identityへのtransferとして保持し、control loweringがinput付きstateへのjumpへ変換する。
`Abrupt` pathはjoinを経由せずlambda resultへ進む。
