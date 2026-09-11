# CPSに基づく通常関数の明示的return binder

Status: Experimental design; not part of the current v0.5 profile

この文書は、通常関数が既に持つreturn edgeへlocal nameを与え、early returnとsum variantの選択を既存の関数型と
continuation applicationの上で記述する試験設計を定める。現行の規範は[`spec/`](../spec/)であり、この文書の構文と意味論を
v0.5 programへ適用してはならない。採否は[最小性の方針](minimality.md)に照らして判断する。

## CPSとの関係

通常関数`A -> B`は、compiler内部では呼出後の計算を明示した次のCPSへ変換できる。

```text
CPS(A -> B) = forall R. A -> (B -> R) -> R
```

sourceでは通常関数`A -> B`のまま、呼出側が持つreturn continuationへlocal nameを与える。`(a)[return] { body }`の`return`は
CPS変換後の`B -> R`に対応し、その一回のfunction invocationを完了するlexically scopedなcontrol binderである。

resultがsumなら、CPS上ではsum valueを一つのcontinuationへ渡した後のcase eliminationを、variantごとのcontinuationへ融合できる。
複数return binderはこの対応をsourceへ公開する。観測可能な評価規則は通常のsum applicationと同じで、呼出側ではactive variantの
continuationだけを評価する。CPSはsource規則を導き、loweringは既存のcalling conventionとcontrol IRへ収束する。
compiler内部のcontinuation表現との関係は[関連調査](../research/prior-art.md#controlとcontinuation)に置く。

## 試験の境界

この試験のsurfaceは次からなる。

- 通常lambdaのresultを現在のfunction invocationから返すlocal return binder
- sum resultのvariantを選んで返す複数return binder
- `Abrupt` pathとfallthroughを合流させる`when`
- 空直和`Empty`とzero-continuation elimination

producerへ値を返して再開するoperationは通常のcallbackで表す。callbackを必須にする場合は他のargumentと同じproduct parameterへ
含める。

```mal
produce :: (Int32, Int32 -> Int32) -> Int32 :=
    (item, yield) {
        reply := yield(item);
        reply + 1
    };
```

`yield(item)`は通常のfunction applicationとして`Int32`を返し、producerは`reply + 1`から評価を続ける。

## 最小性

通常関数には既にparameter、result型、return edgeがある。local return binderはそのedgeへ名前を付ける。sum resultの複数binderは
variant injectionと通常returnへlowerし、呼出側は既存のexhaustive continuation applicationでactive variantだけを処理する。

試験対象を次の単位に分ける。

1. 単一return binderによるearly return
2. sum variantごとの複数return binder
3. `Abrupt` pathと通常pathを局所的に合流させる`when`
4. 空直和の除去を既存のproduct・sum規則の端点として閉じる`Empty`

各単位の採否には、parse、name resolution、型規則、評価規則、lowering、およびpositive・negative・edge caseのfocused testを要求する。
return binderと`Abrupt`はcore loweringの入力境界で消去する。coreはlambda bodyを通常のvalue expression、branch、sum injection、
zero-arm caseへ変換し、後段のcontrol loweringが通常のlambda resultを既存のreturn terminatorへ接続する。source-levelのbinderと
completion judgmentはANF以降へ渡さない。

## 単一return binder

通常lambdaはparameter groupとblockの間にreturn binder groupを置ける。期待result型に対する形は次のとおりとする。

```text
lambda            ::= "(" lambdaParameter? ")" returnBinderGroup? block
returnBinderGroup ::= "[" "]"
                    | "[" VALUE_IDENT ("," VALUE_IDENT)* "]"
```

binderはparameterと同じlambda-local scopeでbody全体から参照でき、外側のvalue nameは通常の規則でshadowできる。同じlambdaの
parameterまたは別のreturn binderと重複する名前はcompile-time errorになる。

| Binder group | Expected result | Meaning |
|---|---|---|
| なし | `B` | 従来どおりbody末尾の`B`をreturn |
| `[return]` | `B`（`Empty`以外） | `B`全体を受ける明示的return |
| `[onA, onB, ...]` | 同じ項数のsum | 各binderが対応variantのpayloadを受ける明示的return |
| `[]` | `Empty` | binderを持たない明示的完了 |

return binder groupを置いたbodyのcompletionは[`Abrupt`の規則](#abrupt-completion-judgment)に従う。

```mal
absolute :: Int32 -> Int32 :=
    (x)[return] {
        when (x >= 0) {
            return(x)
        };

        return(-x)
    };
```

期待関数型を`A -> B`とすると、parameterは従来どおり`A`に対して検査し、return binderは`B`を受け取るlocal control variableになる。
`return(value)`は`value : B`を検査して現在のfunction invocationを完了し、呼出後の同じpathへ戻らない。

```mal
increment :: Int32 -> Int32 :=
    (x)[return] {
        return(x + 1)
    };
```

return binder applicationはそのfunctionの既存return terminatorへlowerする。`f(a)`は従来どおり`B`を返し、`f(a)[k]`は既存規則どおり
`k(f(a))`を意味する。

## curried function

return binderは、それを宣言した一つのlambdaのresult型に対応する。

```text
A -> B -> C = A -> (B -> C)
```

したがって外側のbinderは`B -> C`、内側のbinderは`C`を受け取る。

```mal
chooseAdder :: Bool -> Int32 -> Int32 :=
    (addOne)[returnFunction] {
        when (addOne) {
            returnFunction((x) { x + 1 })
        };

        returnFunction(
            (x)[returnValue] {
                when (x == 0) {
                    returnValue(10)
                };

                returnValue(x + 2)
            }
        )
    };
```

内側のlambdaは外側のreturn binderをcaptureできない。外側のinvocationが完了した後も内側のclosureは生存できるため、escape analysisの
結果にかかわらず一律に拒否する。複数valueを一つのreturn boundaryで扱う場合は、従来どおりproduct parameterを使う。

## sum return binder

期待result型が`[A, B, ...]`なら、lambdaは項数と同じ複数return binderを宣言できる。

```mal
Result :: [Int32, Symbol];

compute :: Bool -> Result :=
    (enabled)[ok, err] {
        when (enabled) {
            ok(42)
        };

        err("disabled")
    };
```

位置`i`のbinderは第`i`項の型を受け、そのvariantを構築して通常returnする。概念上、次へlowerする。

```text
ok(value) = return(0[Result](value))
err(error) = return(1[Result](error))
```

呼出側の`compute(flag)[onOk, onErr]`は現行のsum eliminationであり、scrutineeを一度評価してactive variantのcontinuationだけを
評価する。compilerは中間sum valueが観測されない場合にinjectionとeliminationを融合してよいが、評価順を変えてはならない。

`ok`と`err`はこのlambda内だけの名前であり、`Result`の型identityやglobal constructorを作らない。関数外でResult値だけを構築する
場合は、従来どおり`0[Result](value)`または`1[Result](error)`を使う。

sum resultにも一つだけreturn binderを宣言できる。その場合はvariantではなくsum value全体を受け取り、通常の単一return binderと同じ
規則になる。二つ以上ならalias展開後の項数との完全一致を要求し、省略、追加、部分指定を認めない。

## `Abrupt` completion judgment

`Abrupt`はsource typeやsubtypeではなく、checkerが式の通常完了を分類するjudgmentである。式の検査結果を
`Value(T)`または`Abrupt`とする。

```text
join(Value(T), Value(T)) = Value(T)
join(Value(T), Abrupt)   = Value(T)
join(Abrupt, Value(T))   = Value(T)
join(Abrupt, Abrupt)     = Abrupt
```

checked ASTは`Value` expressionと`Abrupt` control expressionを異なるvariantとして保持する。`Abrupt`へ期待型や仮のvalue typeを
付けず、optional type fieldやflagで両者を兼用しない。core loweringはこの区別を受け取る唯一の後段であり、lexicalな後続を
組み替えて通常のcore expressionへ変換する。

異なる二つの`Value`型は従来どおりtype errorになる。return binder applicationとempty eliminationは`Abrupt`になる。
return binder groupを持つlambda bodyは`Abrupt`だけを受理し、binderのない従来lambda bodyは期待result型`B`に対する`Value(B)`を
要求する。したがってbinderを宣言したlambdaは、すべてのreachable pathをreturn binder applicationまたはempty eliminationで
完了する。両形式を一つのlambda内で混在させない。

binding RHSは`Value(T)`でなければならず、`Abrupt`から値をbindingできない。blockの途中で`Abrupt`になった後にsource itemが続く場合は
unreachable codeとしてcompile-time errorにする。`if`とsum eliminationは上のjoinを用いるため、一方のbranchがreturnし、もう一方が
通常値を返す形を一般の`Never`型なしに検査できる。

strictに値を要求するproduct、operand、argument、callee、continuationの評価中に`Abrupt`が現れた場合は、囲む式も`Abrupt`になり、
その評価順で後にあるsubexpressionはunreachableとして拒否する。return binderはcallee位置でのapplicationだけを認め、通常のvalue
argument、return、aggregate、binding RHS、またはclosure capture位置では拒否する。

## early returnとwhen

`when`は予約語であり、functionやvalue identifierではなく、Boolに対するUnit control expressionを開始する。

```text
whenExpr ::= "when" "(" expression ")" block
```

```mal
classify :: Int32 -> Symbol :=
    (x)[return] {
        when (x == 0) {
            return("zero")
        };

        when (x == 1) {
            return("one")
        };

        return("many")
    };
```

`when (condition) { body }`は`if (condition) then { body } else { () }`へdesugarする。conditionは一度だけ評価し、bodyのcompletionは
`Value(Unit)`または`Abrupt`でなければならない。前節のjoinにより`when`全体は常に`Value(Unit)`になる。

この規則によりsiteごとに異なる後続を閉じ込めた`noop`や、returnをUnit lambdaへeta-expandするhelperは不要になる。bare `[]`を
暗黙のfallthroughには使わない。

## Empty

現行仕様で予約されている`[]`を値型の位置では空直和`Empty`とする。値とinjection constructorは存在せず、`value[]`は
zero-continuation eliminationとして`Abrupt`になる。bare `[]`自体を式やfallthrough continuationにはしない。

`Empty`は値のない型、`Abrupt`は現在のpathが通常完了しないというjudgmentであり、同一ではない。`Empty`をparameter、aggregate field、
通常関数のresultとして使うことは他のvalue typeと同じく認めるが、その値を構築する閉じた通常計算は存在しない。

`A -> Empty`のlambdaはzero return binderだけを宣言できる。値を受ける単一binderとbinderなしのfallthroughは認めない。

```mal
never :: Unit -> [] :=
    ()[] {
        never()[]
    };
```

bodyは`Abrupt`でなければならない。`never()`は`Value(Empty)`だが、直後の`[]`によるeliminationを含む`never()[]`は`Abrupt`になる。
zero binderは暗黙のfallthroughやbottom valueを導入せず、function resultとreturn binder数の対応を空の場合まで閉じる。

## compiler責務と採択条件

採択する場合は次の責務を既存ownerへ追加する。

| Stage | Proposed responsibility |
|---|---|
| `lexer` | `when`をkeywordとしてtoken化する |
| `parser` / `ast` | return binder group、`when`、empty typeとzero-continuation applicationのsource構造をadmitする |
| `resolve` | return binderのidentityとscopeを構成し、lexical captureを拒否する |
| `check` | result型に対するbinder数とparameter型、使用位置、`when`、`Empty`を検査し、`Value`と`Abrupt`を異なるchecked variantへadmitする |
| `formatter` | checkerの型分類に依存せず、ASTが保持するreturn binder group、`when`、empty syntaxを出力する |
| `core` | lexicalな後続を組み替え、single returnをlambda result、multiple returnをsum injectionとlambda result、`when`をbranch、empty eliminationをzero-arm caseへlowerして評価順を固定する |
| `anf` / `closure` / `control` | source-levelのbinderや`Abrupt`を再解釈せず、通常のcore expressionとzero-arm caseを既存のcontrol表現へlowerする |

`execution`以降は通常function call、sum value、sum eliminationだけを受け取り、return binderやcompletion judgmentを受け取らない。

単一return、sum return、curried境界、`when`、`Empty`の各単位でpositive、negative、evaluation orderをfocused testにする。
binderを宣言したbodyのfallthrough、return後のsource item、binderの保存・返却・argument化・closure capture、binder数不一致、
`Abrupt`からのbinding、empty valueの構築をnegative caseへ加える。
