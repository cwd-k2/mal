# 明示的control functionとport

Status: Experimental design; not part of the current v0.5 profile

この文書は、通常関数とcontinuationを同時に受け取るcontrol functionを型で分け、既存のcontinuation applicationに近い
surfaceからearly exitと限定されたresumptionを記述する試験設計を定める。現行の規範は[`spec/`](../spec/)であり、
この文書の構文と意味論をv0.5 programへ適用してはならない。採否は[最小性の方針](minimality.md)に照らして判断する。

## 試験の境界

この試験では次を前提とする。

- control functionはvalue parameterと全handlerを一度のsaturated applicationで受け取る。
- control port名はlocal bindingであり、型のnominal identityにしない。
- portは位置と送出型・応答型、およびresumptionをhandlerへ公開するかで構造的に区別する。
- control portのhandlerはdeepであり、渡されるresumptionはabortiveなone-shot delimited continuationとする。
- answer typeとresumption frameのruntime ownershipはsaturated call siteが決める。
- linear typeと暗黙のgeneral stack captureは導入しない。
- control functionだけを明示的なcontrol representationへlowerし、通常関数を置き換えない。

effect row、general async、externを越えるresumption transport、およびcontrol functionのpartial applicationは最初の範囲に含めない。

## control arrow

通常関数`B -> A`に対し、control functionを`B => A`と書く。後者は概念上次の型を持つ。

```text
B => A = forall R. B -> (A -> R) -> R
```

answer type `R`はsurface syntaxへ公開しない。各saturated applicationがplain exitのhandler resultから一つの具体的な`R`を決め、
それらが同じresult型を持つことを検査する。arrow portのhandler型は`R`から独立する。同じcontrol functionを別のsiteから異なる`R`で
呼んでよい。calleeは`R`の値、型表現、layoutを受け取らず、call siteが構成したopaqueなhandlerまたはcontinuationへtail transferする
だけである。以前の`B -> *A`はmetatheory上の説明にだけ使い、`*A`をsource typeやruntime valueにしない。

`=>`は非結合とし、control functionをparameterまたはresultに置く場合は括弧を要求する。`=>`の直後にあるbare `[...]`はport
signature、右辺全体を囲む`=> (T)`は単一のvalue type `T`としてparseする。port item内の冗長な括弧は除去してからport kindを決め、
個別itemの括弧だけでportをvalue payloadへ切り替えない。

```mal
apply :: (Int32 => Int32, Int32) => Int32;
make :: Config => (Input => Ast);
```

control function value自体は参照、capture、受け渡しできる。禁止するのはvalue parameterだけを与えた中間計算の構築である。

## port signature

`=>`の右辺は単一のabortive exit型、または順序付きport signatureとする。

```text
P ::= A        abortive exit
    | Y -> S   direct port
    | Y => S   control port

Delta ::= A | [P, P, ...]
```

どちらのarrow portもoperationが`Y`をhandlerへ送り、`S`を得てproducerを再開する。`Y -> S`はhandlerを通常関数として受け、handlerが
正常に返したときproducerの残りをそのresultで一度だけ再開する。`Y => S`はproducerの残りをabortiveなdelimited continuationとして
handlerへ明示する。handlerが正常に制御を渡すpathではresumptionを末尾で一度呼び、呼出後はhandlerへ戻らない。shot数はhandler全体の
application回数ではなく、一回のoperation occurrenceが捕捉した残りの使用回数を指す。port名は定義lambdaだけが与える。

```mal
produce :: Input => [Item => Reply, Result, Error] :=
    (input)[yield, return, throw] {
        reply := yield(item);
        // Continue with reply, then finish through return, throw, or a CPS tail-call.
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

同じportをdirectにするとhandler parameterだけが変わる。

```text
Input => [Item -> Reply, Result, Error]
= forall R. Input -> (Item -> Reply) -> (Result -> R) -> (Error -> R) -> R
```

direct portはproducerへ戻る入口であってterminal answerではない。したがって`A => [B -> C]`のようにdirect portしかないsignatureは、
operationから戻るたびに計算を続けるが、有限の正常完了先を持たない。通常はplain exitを併記し、direct-only signatureはloop、diverge、
trap、または別のcontrol tail-callで動き続ける計算にだけ使う。

plain port binderはabortive control variable、direct port binderは通常関数、control port binderは該当operationのcallee位置でだけ使える
control variableとする。control variableは最初の実装では通常値として保存、返却、captureしない。handler位置で同じcontrol portを
指定するforwardingは専用規則で認める。

## case eliminationとの関係

abortive portだけを持つcontrol functionは、結果の直和を構築するproducerと、その直和に対するcase eliminationを融合したものと
みなせる。通常関数とcontrol functionは次の対応を持つ。

```text
f : X -> [A, B]    f(x)[onA, onB]
f : X => [A, B]    f(x)[onA, onB]
```

前者は`f(x)`が直和値を生成した後でactive variantのcontinuationを選ぶ。後者は概念上
`forall R. X -> (A -> R) -> (B -> R) -> R`というChurch encodingを持ち、直和値を中間結果にせずcalleeがportを直接選ぶ。
pure、total、answer-parametricでabortiveな範囲では両者は同型だが、control functionではproducer bodyの任意の地点からexitを選び、
すべてのpathがいずれかのportまたはcontrol tail-callで明示的に完了する。

bare signatureと括弧で囲んだsum valueにも同じcase融合の差がある。

```text
X => [A, B]   = forall R. X -> (A -> R) -> (B -> R) -> R
X => ([A, B]) = forall R. X -> ([A, B] -> R) -> R
```

前者はproducerが二本のcontinuationから直接一つを選び、後者は一本のcontinuationへsum valueを渡してから利用側がcase eliminationする。
括弧は通常の型位置にあるsumの意味を変えず、`=>`の右辺で中間のsum valueをmaterializeするか、eliminationをproducerへ融合するかだけを
区切る。

direct port `Y -> S`はcase branchが`S`を普通に返した後でproducerを一度再開する。control port `Y => S`はcase branchへ`Y`を渡すだけでなく、
branchへproducerの残りをabortiveな`S -> R`として渡す。branchは通常returnの代わりにこの継続へ`S`を渡すため、early exitを含むCPSで
handlerを記述できる。この意味でcontrol applicationは、producerの継続を明示したcase eliminationである。残りを複製する一般の
algebraic handlerまでは含めない。

ただし両者の評価規則は同一ではない。通常の直和除去はscrutineeを得てからactive continuationだけを評価するが、control
applicationはcallee bodyを開始する前に全handlerを評価してcontrol environmentを設置する。この差を保ったまま、同じ`[...]`を
「結果をどのcontinuationへ渡すか」という共通のsurfaceとして使う。

## 定義と明示的完了

単一exitを持つ定義は次の形になる。

```mal
increment :: Int32 => Int32 :=
    (x)[return] {
        return(x + 1)
    };
```

producer bodyの期待型は抽象answer typeであり、通常の`Int32`を暗黙にliftしないため、bodyを`x + 1`だけで終える定義は不正になる。
各control pathはabortive exitのapplication、または同じanswer typeを持つcontrol tail-callで完了しなければならない。

```mal
forward :: Int32 => Int32 :=
    (x)[return] {
        increment(x)[return]
    };
```

abortive exit applicationはchecker内部で`Abrupt`と分類し、通常値を生成して同じpathへ戻るとは扱わない。一般の`Never`型や
subtypingは導入しない。

## saturated application

control functionにはvalue parameterと全handlerを同時に渡す。

```mal
increment(41)[print]

produce(input)[
    (item)[resume] {
        reply := handleItem(item);
        resume(reply)
    },
    (result) { finish(result) },
    (error) { report(error) }
]
```

`f : B => A`なら`f(x)[g]`全体が一つのapplicationであり、`f(x)`単独は式にならない。したがって`g(f(x))`も不正とする。
`f : B -> A`については、従来どおり`f(x)[g]`と`g(f(x))`を同じapplication chainとして扱う。

handlerはsignatureと同数を位置順に指定し、省略、追加、部分適用を認めない。評価はvalue argument、callee、handlerのsource順で行い、
すべてがvalueになってからcallee bodyを開始する。通常の直和除去でactive continuationだけを評価する規則は、calleeの実行前に
control environment全体を設置するこのapplicationには適用しない。

型検査では、`f : B => Delta`と`x : B`に対してsite固有の`R`を一つ導入する。plain exitのhandlerは`A -> R`、direct portのhandlerは
宣言どおり`Y -> S`、control portのhandlerはanswer-polymorphicな`Y => S`で検査する。周囲に期待型があればそれを`R`に使い、なければ
plain handler bodyまたは既知のplain handler functionのresultから決める。解けない場合やplain handler間で一致しない場合は
compile-time errorとし、default answer typeは設けない。全handlerの検査後、application全体の型を`R`とする。plain exitを持たない
signatureのapplicationはzero-port applicationと同様に`Abrupt`になる。

parserとcheckerは、calleeのarrowを確定する前に`f(x)`を独立した通常applicationとして確定してはならない。formatterとdiagnosticも
`=>` callの二つのargument groupを一つの構文単位として扱う。

`f(a)[k]`の`k`はcontrol function全体のanswer continuationであり、そのapplicationはcalleeから見てabortiveである。`f(a)`単独を
値にできない規則は未処理のcontinuationを暗黙に残さない。この試験はdynamic context全体をcaptureする`call/cc`を導入しない。
control portがhandlerへ渡すcontinuationもsaturated call siteをdelimiterとし、呼出元handlerへ戻らない。どちらのcontrol variableも
通常値としてcaptureまたは複製できないため、answer binderとresume binderは一回のdynamic pathで高々一度だけ使われる。

## handlerとresumption

portごとのhandler shapeは次になる。

```text
port       handler shape                 contextual type

A          (value) { body }              A -> R
Y -> S     (value) { body }              Y -> S
Y => S     (value)[resume] { body }       forall Q. Y -> (S -> Q) -> Q
```

direct handlerはresumptionを受け取らない。producerがportを呼ぶたびに通常applicationとして評価し、正常に得た`S`をproducerの
一意な残りへ渡す。handlerのdivergeまたはtrapを除き、この残りを破棄または複製する方法はない。この限定された意味で`Y -> S`を
one-resume portと呼ぶが、handler value自体のone-shot性や一般のlinear function typeは意味しない。

Unit payloadのcontrol handlerは既存lambdaと同じく`()[resume] { ... }`と書く。`resume`のapplicationは`Abrupt`であり、resultを
bindingしたり、その後へfall throughしたりできない。

```mal
()[resume] {
    when chooseDefault() {
        resume(false)
    };

    resume(true)
}
```

port位置と値型の`Y => S`は同じanswer-polymorphicなcontrol function型である。outer producerがoperationを行うと、handlerをそのsiteの
answer type `R`へinstantiateし、producerの残りをhandlerの`resume`へ渡す。`resume`はhandler bodyにとってopaqueかつabortiveであり、
`R`の値を返さない。

direct portとcontrol portは、pure、total、parametricな意味では同型になる。

```text
Direct(Y, S)  = Y -> S
Control(Y, S) = forall Q. Y -> (S -> Q) -> Q

lift : Direct(Y, S) -> Control(Y, S)
lift(h) = forall Q. (y, k) { k(h(y)) }

lower : Control(Y, S) -> Direct(Y, S)
lower(g)(y) = g(y)[identity_S]
```

`lift(h)`は`k`を末尾で一度だけ使い、`lower`はanswer typeに`S`、continuationにidentityを選ぶ。parametricityによりcontrol handlerは
未知の`Q`を自力で構成または観測できず、正常完了するには渡されたcontinuationへ制御を移す。この制約がない固定answer typeの一般的な
algebraic handlerとは異なり、resumption結果の結合とmulti-shotは表現しない。理論上の比較は
[関連調査](../research/prior-art.md#algebraic-effectとresumption)に置く。

明示的な`Y => S` portへ通常関数を渡す場合は`lift`を専用adaptationとして利用できる。外側のcontrol port `outer`を内側の同型portの
handler位置へ書くforwardingは、概念上次へeta-expandする。plain exitとdirect portのforwardingはbinderをそのまま渡す。

```text
(y)[resume] {
    s := outer(y);
    resume(s)
}
```

port signature内では、個別itemを囲む括弧は意味を変えない。

```mal
B => [A => C, D]   // Same control-port signature.
B => [(A => C), D]
B => [A -> C, D]   // Same direct-port signature.
B => [(A -> C), D]
```

functionを含む直和全体を一つのvalueとしてexitする場合も、`=>`の右辺全体を囲む。

```mal
Config => ([Request -> Response, Error])
Grammar => ([Input => Ast, BuildError])
```

前者はcallbackまたはerror、後者はcontrol functionまたはbuild errorを一つのsum valueとしてhandlerへ渡す。対応する
`Config => [Request -> Response, Error]`と`Grammar => [Input => Ast, BuildError]`は二つのportを持ち、producer自身がarrow portを
operationとして呼ぶ。単一のfunction valueをexitする場合は`Config => (Request -> Response)`または`Grammar => (Input => Ast)`と書く。
functionも他のvalue typeと同じpayloadであり、これはcontrol固有の能力ではなくclosureによる明示的な遅延である。返されたcontrol
functionも後のsiteでvalue parameterと新しいcontinuationを同時に受け取り、未飽和な`f(x)`を値として作る規則にはならない。

control portのhandlerはdeepとする。評価文脈を`E`とした概念的なoperation規則は次になる。

```text
handle E[yield(y)] with h
  = h(y)[(s) { handle E[s] with h }]
```

右辺のcontinuationは`E`のうちsaturated callまでを捕捉し、同じhandlerを再設置する。handlerの`resume` binderからだけ参照でき、通常値への
保存、返却、closure captureは認めない。`resume(s)`はframeを消費して`E[s]`へ移り、handlerへ戻らない。これによりdeepかつone-shotな
abortive delimited continuationになり、multi-shotのためのframe複製やpersistent ownerは不要になる。

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

`when (condition) { body }`は`if (condition) then { body } else { () }`へdesugarする。conditionは一度だけ評価し、bodyは
`Unit`または`Abrupt`でなければならない。`when`は通常関数でも使え、`unless`は追加しない。

この規則によりsiteごとに異なる後続を閉じ込めた`noop`や、exitをUnit lambdaへeta-expandするhelperは不要になる。bare `[]`を
暗黙のfallthroughには使わない。

## 空のsignature

この試験では、現行仕様で予約されている`[]`を値型の位置では空直和`Empty`、control signatureの位置ではzero-port signatureとする。
`value[]`はzero-continuation elimination、`f(value)[]`はzero-handlerのsaturated applicationである。control signature `B => []`は
terminal portを持たず、正常完了する有限計算を構成できない。

plain型だけを置く`[A]`は一項直和として導入せず、不正なまま予約する。一つのcontinuationが通常applicationを意味する既存規則と
衝突し、型として`A`と同一視しても表現力を増やさないためである。単一のabortive portには`B => A`を使う。一方、
`B => [Y -> S]`と`B => [Y => S]`はarrowを持つ単一port signatureなので有効とする。function valueを単一exitにする場合は
`B => (Y -> S)`または`B => (Y => S)`と書く。`B => [(Y -> S)]`と`B => [(Y => S)]`はitemの括弧を除去してarrow portとして
解釈し、function value exitにはしない。

`[]`は値を持たない型、`Abrupt`は現在のpathが通常完了しないというcheckerの判定であり、同一ではない。`empty[]`による
Empty eliminationと、zero-handlerのsaturated application `f(value)[]`はともに`Abrupt`を生む。zero-port signatureの展開は次になる。

```text
B => [] = forall R. B -> R
```

したがって`(value)[] { ... }`で定義するcontrol functionはdiverge、trap、または別のzero-exit callへtail-forwardする以外に
正常完了できない。bare `[]`自体を式やfallthrough continuationにはしない。

## compiler境界と実装段階

現在のcontrol IRは`Call`に静的な`resume: StateId`を持ち、recursive region内のnon-tail callだけにtyped frameを構成する。
frameは一つのarenaへpushされ、return時にpopする。region外callはnative stackを使えるため、この表現から任意のdynamic stack
suffixを直接captureしてはならない。

各saturated call siteは具体的な`R`に対するhandler bridge、resume target、frame ownershipを構成する。callee側のcontrol ABIはそれらを
opaqueなtail-transfer targetとして扱い、answer valueをcallee共通の表現へeraseしない。first-class control functionの呼出先が
静的に一つへ決まらない場合も、signatureは同一なのでtarget dispatchとanswer-specific bridgeを分離できる。control functionを
`extern`として宣言することとresumptionをhostへtransportすることは、別のABIが定まるまで拒否する。

最初の段階は単一または複数のabortive exit、direct port、`Abrupt` join、`when`、saturated `=>` callを実装する。次の段階でcontrol
port、`(value)[resume]`、deep forwardingを実装する。control portのresumptionは常にnon-escapingかつone-shotなので、call siteが所有する
消費型frameをbaselineとし、persistent closureやframe cloneを導入しない。

実装時にはport forwardingのconcrete lowering、resumption application後のunreachable判定、frameを一度だけ消費する規則を
representation invariantとして固定する。各段階はpositive、negative、evaluation order、managed capture、深い再帰をfocused testで
固定し、control-port段階ではresumption後のfallthrough、複数application、保存、返却、closure captureをnegative caseへ追加する。
