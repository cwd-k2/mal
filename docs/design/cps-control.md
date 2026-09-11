# 明示的control functionとport

Status: Experimental design; not part of the current v0.5 profile

この文書は、通常関数とcontinuationを同時に受け取るcontrol functionを型で分け、既存のcontinuation applicationに近い
surfaceからearly exitと限定されたresumptionを記述する試験設計を定める。現行の規範は[`spec/`](../spec/)であり、
この文書の構文と意味論をv0.5 programへ適用してはならない。採否は[最小性の方針](minimality.md)に照らして判断する。

## 試験の境界

この試験では次を前提とする。

- control functionはvalue parameterと全handlerを一度のsaturated applicationで受け取る。
- control port名はlocal bindingであり、型のnominal identityにしない。
- resumable portは送出型`Y`と応答型`S`を持つ`Y => S`だけとする。
- 通常関数`Y -> S`をhandlerへ渡す場合は、control handlerへの局所的なadaptationを行う。
- control portのhandlerはdeepであり、渡されるresumptionはabortiveなone-shot delimited continuationとする。
- answer typeとresumption frameのsemantic ownershipはsaturated call siteが決める。
- linear typeと暗黙のgeneral stack captureは導入しない。
- control functionだけに新しいsource-level control contractを加え、通常関数の意味論を置き換えない。

effect row、general async、externを越えるresumption transport、およびcontrol functionのpartial applicationは範囲に含めない。

## 最小性による分割

既存の通常関数は、calleeが複数地点から異なるpayloadで呼出側へ脱出することや、呼出側がproducerの残りを明示的に再開することを
型に表せない。直和を返せばabortive exitは表せるが、中間の直和值を構築した後に呼出側で除去するため、producer内のoperationへ応答して
同じproducerを再開するcontractにはならない。この差だけをcontrol functionとportの追加理由とする。

試験対象を次の単位に分ける。

1. abortive exitだけを持つcontrol functionとsaturated application
2. `Y => S` port、明示的resumption、deep forwarding
3. `Abrupt` pathと通常pathを局所的に合流させる`when`
4. 空直和の除去とzero-port controlを同じ規則で扱う`Empty`

第1段階はcase eliminationとの融合と明示的完了だけを検証でき、第2段階はその規則を変更せずresumable portを加える。通常関数handlerの
ために別のdirect port型は導入しない。`when`はsiteごとのfallthrough continuationやUnit lambdaを要求せず、既存の`if`へ局所的に
desugarできるため第3の小さいsurfaceとする。`Empty`は空直和とzero-continuation eliminationを既存のproduct・sum規則の端点として閉じ、
zero-port controlを別のbottom機構にしないため第4の型規則とする。

各単位の採否には、parse、name resolution、型規則、評価規則、lowering、およびpositive・negative・edge caseのfocused testを要求する。
control functionの`extern`宣言とresumptionのhost transportはABI、authority、lifetimeが別途定まるまで拒否する。

## control arrow

通常関数`B -> A`に対し、control functionを`B => A`と書く。後者は概念上次の型を持つ。

```text
B => A = forall R. B -> (A -> R) -> R
```

answer type `R`はsurface syntaxへ公開しない。各saturated applicationがplain exitのhandler resultから一つの具体的な`R`を決め、
それらが同じresult型を持つことを検査する。同じcontrol functionを別のsiteから異なる`R`で呼んでよい。calleeは`R`の値、型表現、
layoutを受け取らず、call siteが構成したopaqueなhandlerまたはcontinuationへtail transferするだけである。

`=>`は非結合とし、control functionをparameterまたはresultに置く場合は括弧を要求する。

```mal
apply :: (Int32 => Int32, Int32) => Int32;
make :: Config => (Input => Ast);
```

control function value自体は参照、capture、受け渡しできる。禁止するのはvalue parameterだけを与えた中間計算の構築である。

## port signatureとalias

`=>`の右辺は単一のabortive exit型、または順序付きport signatureとする。

```text
Delta         ::= T | portSignature
portSignature ::= "[" "]"
                | "[" T ("," T)* "]"
```

これはparserが保持する構文であり、各`T`のport kindは後述のcanonical typeからcheckerが決める。singletonのadmissionはitemの
port kindに従う。

`Y -> S`はportではなく、常にfunction valueをpayloadとするplain exitである。plain型だけを置く`[A]`は一項直和として導入せず、
不正なまま予約する。一方、canonical itemがcontrol arrowである`[Y => S]`は単一のcontrol port signatureなので有効とする。

parserはbare `[...]`をport signatureとして保持するが、itemをportへ分類しない。checkerが各itemのtype aliasをcanonical typeへ展開し、
最上位がcontrol arrowならcontrol port、それ以外ならplain exitと分類する。括弧は通常どおり型のgroupingにだけ使い、分類を変更しない。
これにより次の二つは同じsignatureになる。

```mal
Responder :: Item => Reply;

Input => [Item => Reply, Result]
Input => [Responder, Result]
```

signature内でcontrol function valueそのものをplain exitにすることはできない。必要ならproductまたはsumへ明示的に包み、最上位型を
control arrow以外にする。単一exitなら右辺全体を括弧でgroupingできる。

```mal
Config => (Request -> Response)
Grammar => (Input => Ast)
Grammar => ([Input => Ast, BuildError])
Grammar => [(Input => Ast, Unit), BuildError]
```

最初の型は通常関数、次はcontrol function、三つ目はcontrol functionまたはerrorのsum valueを単一exitへ渡す。最後はproductに包んだ
control function valueとerrorを別々のplain exitにする。aliasはこれらのcanonical typeを変えない。

control portはoperationが`Y`をhandlerへ送り、`S`を得てproducerを再開する。producerの残りをabortiveなdelimited continuationとして
handlerへ明示する。handlerが正常に制御を渡すpathではresumptionを末尾で一度呼び、呼出後はhandlerへ戻らない。shot数はhandler全体の
application回数ではなく、一回のoperation occurrenceが捕捉した残りの使用回数を指す。port名は定義lambdaだけが与える。

```mal
produce :: Input => [Item => Reply, Result, Error] :=
    (input)[yield, return, throw] {
        reply := yield(item);
        return(finish(reply))
    };
```

answer typeを`R`とすると、この型は概念上次へ展開する。

```text
forall R.
    Input
 -> (forall Q. Item -> (Reply -> Q) -> Q)
 -> (Result -> R)
 -> (Error -> R)
 -> R
```

plain exit binderはabortive control variable、control port binderは該当operationのcallee位置でだけ使えるcontrol variableとする。
control variableは通常値として保存、返却、またはlexical closureへcaptureしない。同じportをhandler位置へ指定するforwardingは専用規則で
認める。

## case eliminationとの関係

abortive portだけを持つcontrol functionは、結果の直和を構築するproducerと、その直和に対するcase eliminationを融合したものと
みなせる。

```text
f : X -> [A, B]    f(x)[onA, onB]
f : X => [A, B]    f(x)[onA, onB]
```

前者は`f(x)`が直和値を生成した後でactive variantのcontinuationを選ぶ。後者は概念上
`forall R. X -> (A -> R) -> (B -> R) -> R`というChurch encodingを持ち、calleeがportを直接選ぶ。

bare signatureと括弧で囲んだsum valueには次の差がある。

```text
X => [A, B]   = forall R. X -> (A -> R) -> (B -> R) -> R
X => ([A, B]) = forall R. X -> ([A, B] -> R) -> R
```

通常の直和除去はscrutineeを得てからactive continuationだけを評価する。control applicationはcallee bodyを開始する前に全handlerを
評価してcontrol environmentを設置する。この評価規則の差を保ったまま、同じ`[...]`を結果の配送先を並べるsurfaceとして使う。

## 定義、application、評価順

単一exitを持つ定義は次の形になる。

```mal
increment :: Int32 => Int32 :=
    (x)[return] {
        return(x + 1)
    };
```

producer bodyは明示的に完了し、通常値をfall throughさせない。各reachable pathはabortive exitのapplication、同じanswer
continuationをforwardするcontrol tail-call、またはplain exitを持たないcontrol callで静的に終端しなければならない。評価途中の
operationがtrapまたはdivergeする可能性は通常完了の型付けを変えない。

```mal
forward :: Int32 => Int32 :=
    (x)[return] {
        increment(x)[return]
    };
```

control functionにはvalue parameterと全handlerを同時に渡す。

```mal
increment(41)[(value) { showInt32(value) }]

produce(input)[
    (item)[resume] {
        resume(handleItem(item))
    },
    (result) { finish(result) },
    (error) { report(error) }
]
```

`f : B => Delta`なら`f(x)[handlers]`全体が一つのapplicationであり、`f(x)`単独は式にならない。handlerはsignatureと同数を
位置順に指定し、省略、追加、部分適用を認めない。

評価は既存applicationと同じくvalue argument、calleeの順に行い、その後handlerをsource順に評価する。すべてがvalueになってから
callee bodyを開始する。通常の直和除去にある「active continuationだけを評価する」規則はcontrol applicationには適用しない。

型検査ではsite固有の`R`を一つ導入する。plain exit handlerは`A -> R`、control port handlerはanswer-polymorphicな`Y => S`で
検査する。周囲に期待型があればそれを`R`に使い、なければplain handler bodyまたは既知のplain handler functionのresultから決める。
解けない場合やplain handler間で一致しない場合はcompile-time errorとし、default answer typeは設けない。plain exitを持たない
signatureのapplicationは`Abrupt`、それ以外のapplicationは`Value(R)`になる。

parserとcheckerはcalleeのarrowを確定する前に`f(x)`を独立した通常applicationとして確定してはならない。formatterとdiagnosticも
`=>` callの二つのargument groupを一つの構文単位として扱う。

## `Abrupt` completion judgment

`Abrupt`はsource typeやsubtypeではなく、checkerが式の通常完了を分類するjudgmentである。式の検査結果を
`Value(T)`または`Abrupt`とし、control function bodyは`Abrupt`を要求する。

```text
join(Value(T), Value(T)) = Value(T)
join(Value(T), Abrupt)   = Value(T)
join(Abrupt, Value(T))   = Value(T)
join(Abrupt, Abrupt)     = Abrupt
```

異なる二つの`Value`型は従来どおりtype errorになる。abortive exit application、`resume` application、すべてのplain exitを外側へ
forwardするcontrol tail-call、plain exitを持たないcontrol application、およびempty eliminationは`Abrupt`になる。

binding RHSは`Value(T)`でなければならず、`Abrupt`から値をbindingできない。blockの途中で`Abrupt`になった後にsource itemが続く場合は
unreachable codeとしてcompile-time errorにする。`if`とsum eliminationは上のjoinを用いるため、一方のbranchがexitし、もう一方が
通常値を返す形を一般の`Never`型なしに検査できる。

strictに値を要求するproduct、operand、argument、callee、handlerの評価中に`Abrupt`が現れた場合は、囲む式も`Abrupt`になり、
その評価順で後にあるsubexpressionはunreachableとして拒否する。これによりabortive applicationを値として利用せず、各control transferを
到達可能な式の末尾に置く。

control variableは通常値でないため、通常のvalue argument、return、aggregate、binding RHS、またはclosure capture位置では拒否する。
plain exit binderのapplicationは`Abrupt`、control port binderのoperationは`Value(S)`、handler内の`resume` applicationは`Abrupt`になる。
対応するhandler位置でのforwardingも専用規則で認める。`resume`後には同じdynamic pathの後続へ到達できないため、branchごとに
`resume`が現れる場合も一回のoperation occurrenceが作ったresumptionは高々一度だけ消費される。

## handlerとresumption

handler shapeは次になる。

```text
port       handler shape                 contextual type

A          (value) { body }              A -> R
Y => S     (value)[resume] { body }       forall Q. Y -> (S -> Q) -> Q
```

通常関数`h : Y -> S`をcontrol portへ渡す場合は、checkerが認める専用adaptationとして概念上次へ展開する。

```text
lift(h) = (y)[resume] { resume(h(y)) }

lower(g) = (y) { g(y)[identity_S] }
```

`h`が正常returnすればそのresultでproducerを一度再開する。`h`がtrap、diverge、またはexternal operationを行う場合も、`lift`はそれらを
追加、省略、並べ替えしない。逆に`g : Y => S`はanswer typeを`S`、handlerをidentityにして通常関数へ戻せる。この意味で単一exitの
control functionと通常関数は同型であり、その同型な二形式をport kindとして重複させない。backendは`lift`の形を専用のreturn bridgeへ
最適化してよい。

control handlerの`resume`はhandler bodyにとってopaqueかつabortiveであり、resultをbindingしたり、その後へfall throughしたりできない。
answer polymorphismにより未知の`Q`を自力で構成または観測できず、正常にproducerへ戻るpathは`resume`へ末尾で制御を移す。
producer全体のearly exitはcontrol handlerが未知の`Q`を作る機能ではなく、producerに宣言されたplain exitが担う。
一般のalgebraic handlerとの差は[関連調査](../research/prior-art.md#algebraic-effectとresumption)に置く。

外側のcontrol portを内側の同型portのhandler位置へ書くforwardingは、概念上次へeta-expandする。

```text
(y)[resume] {
    s := outer(y);
    resume(s)
}
```

control portのhandlerはdeepとする。評価文脈を`E`とした概念的なoperation規則は次になる。

```text
handle E[yield(y)] with h
  = h(y)[(s) { handle E[s] with h }]
```

右辺のcontinuationは`E`のうちsaturated callまでを捕捉し、同じhandlerを再設置する。handlerの`resume` binderからだけ参照でき、
通常値への保存、返却、lexical closure captureは認めない。`resume(s)`はframeを消費して`E[s]`へ移り、handlerへ戻らない。

## early exitとwhen

`when`はfunctionではなく、Boolに対するUnit control expressionとする。

```mal
classify :: Int32 => Symbol :=
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
`Value(Unit)`または`Abrupt`でなければならない。前節のjoinにより`when`全体は常に`Value(Unit)`になる。`when`は通常関数でも使え、
`unless`は追加しない。

この規則によりsiteごとに異なる後続を閉じ込めた`noop`や、exitをUnit lambdaへeta-expandするhelperは不要になる。bare `[]`を
暗黙のfallthroughには使わない。

## Emptyと空のsignature

現行仕様で予約されている`[]`を値型の位置では空直和`Empty`、control signatureの位置ではzero-port signatureとする。
`value[]`はzero-continuation elimination、`f(value)[]`はzero-handlerのsaturated applicationである。どちらも`Abrupt`になり、
bare `[]`自体を式やfallthrough continuationにはしない。

`Empty`は値を持たず、injection constructorも存在しない。`[]`は値のない型、`Abrupt`は現在のpathが通常完了しないというjudgmentであり、
同一ではない。`Empty`をparameter、aggregate field、通常関数のresultとして使うことは他のvalue typeと同じく認めるが、その値を構築する
閉じた通常計算は存在しない。

zero-port signatureの展開は次になる。

```text
B => [] = forall R. B -> R
```

`(value)[] { ... }`で定義するcontrol functionは、別のplain exitを持たないcontrol callへtail-forwardする再帰によってのみ
well-typedな通常実行を続けられる。途中でtrapまたはdivergeする可能性はあるが、通常値をfall throughさせられない。control portだけを
持つsignatureもoperationから戻った後の有限な完了先を持たないため、applicationは`Abrupt`になる。

## compiler authorityと採択条件

現在のcontrol IRとexecution planのauthorityは
[application control lowering](../development/application-control-lowering.md)および
[compilerの責務境界](../implementation/responsibilities.md)にある。この試験を採択する場合は、現在の文書を暗黙に拡張せず、次の責務を
各ownerへ追加する。

| Stage | Proposed responsibility |
|---|---|
| `parser` / `ast` | `=>` type、control lambda、二群のsaturated application、port signature、`when`、empty syntaxのsource構造をadmitする。port kindは決めない |
| `resolve` | port binderとresume binderのidentity、scope、lexical capture禁止を検査する |
| `check` | alias展開後のport分類、control variableの使用位置、site固有answer type、handler adaptation、`Abrupt` judgment、`Empty`規則を所有する |
| `formatter` | checkerのport分類に依存せず、ASTが保持する二群application、control lambda、`when`、empty syntaxを出力する |
| `core` / `anf` / `closure` / `control` | `when`のdesugaringと評価順を固定し、delimiter、operation、answer transfer、resume edgeを明示したIRを構成する |
| `execution` | delimiterまでのone-shot semantic frame、live value、owner transfer、消費済み以後の到達不能関係を構成する |
| `backend/llvm` | admission済みframeのlayout、handler bridge、tail transfer、consume operationを具体化する |
| `runtime/c11/control.c` | frameの型やportを解釈せず、必要なbyte storageのcapacity、growth、releaseだけを扱う |

現在のexecution frameはrecursive region内のnon-tail callだけを対象とする。control resumptionはregion外callや同一function内のoperationも
delimiterまでsuspendし得るため、既存frame集合へ条件なしに混在させない。採択時には、通常call frameとcontrol resumption frameを
一つのsemantic frame planへ一般化するか、異なるadmitted representationとして分けるかを
`application-control-lowering.md`と`responsibilities.md`で確定する。

abortive control、control port、`when`、`Empty`の各単位でpositive、negative、evaluation order、managed capture、深い再帰をfocused
testにする。control portではresumption後のfallthrough、複数application、保存、返却、closure captureをnegative caseへ加え、
通常関数handler adaptationがresult、effect order、trap、divergeを保存することも検査する。
