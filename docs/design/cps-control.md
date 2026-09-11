# 明示的control functionとport

Status: Experimental design; not part of the current v0.5 profile

この文書は、通常関数とcontinuationを同時に受け取るcontrol functionを型で分け、既存のcontinuation applicationに近い
surfaceからearly exitと限定されたresumptionを記述する試験設計を定める。現行の規範は[`spec/`](../spec/)であり、
この文書の構文と意味論をv0.5 programへ適用してはならない。採否は[最小性の方針](minimality.md)に照らして判断する。

## 試験の境界

この試験では次を前提とする。

- control functionはvalue parameterと全handlerを一度のsaturated applicationで受け取る。
- control port名はlocal bindingであり、型のnominal identityにしない。
- resumable portは位置と送出型・再開型だけで構造的に区別する。
- handlerはdeepであり、resumptionは通常のfunctionと同じく複数回applicationできる。
- answer typeとresumptionのruntime ownershipはsaturated call siteが決める。
- linear typeと暗黙のgeneral stack captureは導入しない。
- control functionだけを明示的なcontrol representationへlowerし、通常関数を置き換えない。

effect row、general async、externを越えるresumption transport、およびcontrol functionのpartial applicationは最初の範囲に含めない。

## control arrow

通常関数`B -> A`に対し、control functionを`B => A`と書く。後者は概念上次の型を持つ。

```text
B => A = forall R. B -> (A -> R) -> R
```

answer type `R`はsurface syntaxへ公開しない。各saturated applicationがhandler result型から一つの具体的な`R`を決め、全handlerが
同じresult型を持つことを検査する。同じcontrol functionを別のsiteから異なる`R`で呼んでよい。calleeは`R`の値、型表現、layoutを
受け取らず、call siteが構成したopaqueなhandlerまたはresumptionへtail transferするだけである。以前の`B -> *A`はmetatheory上の
説明にだけ使い、`*A`をsource typeやruntime valueにしない。

`=>`は非結合とし、control functionをparameterまたはresultに置く場合は括弧を要求する。

```mal
apply :: (Int32 => Int32, Int32) => Int32;
make :: Config => (Input => Ast);
```

control function value自体は参照、capture、受け渡しできる。禁止するのはvalue parameterだけを与えた中間計算の構築である。

## port signature

`=>`の右辺は単一のabortive exit型、または順序付きport signatureとする。

```text
P ::= A        abortive exit
    | Y <- S   resumable port

Delta ::= A | [P, P, ...]
```

`Y <- S`は、operationが`Y`をhandlerへ送り、handlerが`S`を渡して計算を再開することを表す。`<-`は通常のfunction arrowではなく、
function型をpayloadとするabortive exitからresumable portを構文上区別する。port名は定義lambdaだけが与える。

```mal
produce :: Input => [Item <- Reply, Result, Error] :=
    (input)[yield, return, throw] {
        reply := yield(item);
        // Continue with reply, then finish through return, throw, or a CPS tail-call.
    };
```

answer typeを`R`とすると、この型は概念上次へ展開する。

```text
forall R.
    Input
 -> (Item -> (Reply -> R) -> R)
 -> (Result -> R)
 -> (Error -> R)
 -> R
```

plain port binderはabortive control variable、resumable port binderは該当operationのcallee位置でだけ使えるcontrol variableとする。
最初の実装では通常値として保存、返却、captureしない。handler位置で同じportを指定するforwardingは専用規則で認める。

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

型検査では、`f : B => Delta`と`x : B`に対してsite固有の`R`を一つ導入し、各portを`R`について展開したhandler型で検査する。
周囲に期待型があればそれを`R`に使い、なければhandler bodyまたは既知のhandler functionのresultから決める。解けない場合や
handler間で一致しない場合はcompile-time errorとし、default answer typeは設けない。全handlerの検査後、application全体の型を`R`とする。

parserとcheckerは、calleeのarrowを確定する前に`f(x)`を独立した通常applicationとして確定してはならない。formatterとdiagnosticも
`=>` callの二つのargument groupを一つの構文単位として扱う。

## handlerとresumption

abortive exit `A`には通常lambda、resumable port `Y <- S`にはcontinuation parameterを持つhandler lambdaを対応させる。

```text
port       handler shape                 contextual type

A          (value) { body }              A -> R
Y <- S     (value)[resume] { body }       Y -> (S -> R) -> R
```

Unit payloadのhandlerは既存lambdaと同じく`()[resume] { ... }`と書く。handler bodyはapplication文脈から決まった具体的な`R`を
返すため、resumption結果を結合できる。

```mal
()[resume] {
    left := resume(false);
    right := resume(true);
    merge(left, right)
}
```

`Y <- S`が要求するhandlerをanswer type固定の`Y =>_R S`と書けば、first-classな`Y => S`は`forall R. Y =>_R S`である。
したがってportは独立したcontrol function typeではなく、外側のanswer typeにspecializeされた負の位置のslotである。

operationを呼ぶ側では`yield(y)`が`S`で再開するため、`Y <- S`と通常関数`Y -> S`の入出力は同じに見える。通常関数は
任意のanswer typeについて次のhandlerへliftできる。

```text
lift_R(f)(y, resume) = resume(f(y))
```

このliftは通常関数をresumable handler slotへ置くときの専用adaptationとして利用できる。また、外側のresumable port `outer`を
内側の同型portのhandler位置へ書くforwardingは、概念上次へeta-expandする。plain exitのforwardingはbinderをそのまま渡す。

```text
(y)[resume] {
    s := outer(y);
    resume(s)
}
```

逆変換は固定された`R`について一般には存在しない。handlerはresumeを0回または複数回呼び、具体的な`R`を結合できるためである。
純粋、全域、answer-parametricでresumeをちょうど一度使う`forall R. Y =>_R S`だけに限定すれば、`R = S`とidentity continuationを
選ぶことで`Y -> S`へ戻せる。この限定された同型は、通常関数とresumable portを同一の型constructorにする理由にはしない。

次の二型は異なる。曖昧さを避けるため、control functionをexit payloadにするときは括弧を必須とする。

```mal
B => [(A => C), D] // Exit with a first-class control function value.
B => [A <- C, D]   // Send A to a handler and resume with C.
```

handlerはdeepとする。評価文脈を`E`とした概念的なoperation規則は次になる。

```text
handle E[yield(y)] with h
  = h(y, (s) { handle E[s] with h })
```

resumptionはcall siteが決めた具体的な`S -> R`を持つfirst-class functionであり、handlerから返すclosureへcaptureしてもよい。
0回、1回、複数回applicationできる。multi-shot invocationは捕捉地点以降のexternal operationも呼び出しごとに再実行し、通常の
strict evaluation orderに従う。answer scopeからescapeしたresumptionとcaptureは、その値が到達可能な間call siteのmanaged ownerを
保持する。resumptionがescapeせず各pathで高々一度だけtail applicationされると証明できる場合だけ、compilerは観測可能な意味を
変えずlinearなcontrol frameへ最適化してよい。

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

`[A]`は一項直和として導入せず、不正なまま予約する。一つのcontinuationが通常applicationを意味する既存規則と衝突し、型として
`A`と同一視しても表現力を増やさないためである。単一portのcanonical spellingには`B => A`を使う。

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

最初の段階は単一または複数のabortive exit、`Abrupt` join、`when`、saturated `=>` callを実装する。次の段階で`Y <- S`、
`(value)[resume]`、deep forwarding、multi-shotをpersistent closureまたは同等のbaseline表現で実装する。最後にone-shotと証明できる
siteのarena frame化をoptional optimizationとして評価する。

実装時にはport forwardingのconcrete loweringと、persistent ownerを各resumption invocationがshareする規則をrepresentation invariant
として固定する。各段階はpositive、negative、evaluation order、managed capture、深い再帰をfocused testで固定し、resumable段階では
同じresumptionを0回、1回、複数回使うcase、handlerからescapeするcase、各回のextern traceを追加する。
