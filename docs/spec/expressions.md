# 式と binding

Status: Accepted v0.6

literalとoperatorは[literalとoperator](operators.md)に定める。

## binding

`:=` は immutable binding を作る。型 annotation は `::` で書く。

```mal
main :: Unit -> Int32 := () -> {
    x := 10i32;
    y :: Int32 := x + 20;
    y;
};
```

再代入はない。内側の scope では同じ名前を shadow できる。binding の scope は、その宣言の直後から現在の block の末尾までである。

直積は pattern で分解できる。

```mal
(name, length) := value;
(_, y) := point;
```

pattern は identifier、`_`、product pattern からなる。pattern 内で同じ identifier を二度 binding してはならない。

## ラムダ

```mal
add :: (Int32, Int32) -> Int32 :=
    (a, b) -> a + b;
```

lambdaは周辺から与えられる期待関数型に対して検査し、自身から関数型を推論しない。期待関数型がなければ
compile-time errorである。bindingのRHSに直接lambdaを書く場合は、bindingの型annotationが期待型になる。

期待関数型のparameter型が`Unit`ならlambdaは空pattern、そうでなければ括弧内のpatternをparameter型に対して検査する。
`(a, b)`はproduct parameterを分解し、`(value)`はparameter型全体を一つの名前へbindingする。`_`とnested product
patternもbindingと同じ意味を持つ。bodyは期待関数型のresult型に対して検査する。resultがさらに関数型でbodyの
result expressionがlambdaなら、この規則を再帰的に適用する。parameterごとの型annotationとlambda自身の戻り型構文はない。

ラムダbodyから参照する外側のlocal bindingはby-valueでlexically captureされる。

```mal
makeAdder :: Int32 -> (Int32 -> Int32) :=
    (x) -> (y) -> x + y;
```

top-level binding、predefined binding、compiler primitiveはenvironmentへcaptureせず直接参照する。nested lambdaだけが
さらに外側のlocal valueを参照する場合も、compilerが各lambda境界を通して値を転送する。

captureの時点とlifetimeは[実行意味論のclosure規則](execution.md#scope-と-closure)に従う。

lambdaの`->`の右辺は一つのexpressionである。複数のbindingやexpression statementが必要ならblock expressionを置く。
blockは0個以上のbody itemと最後のresult expressionからなり、最後の`;`はoptionalである。改行は構文に影響せず、
result expressionのないblockとreturn statementはない。

lambda bodyは通常の値で完了するほか、すべてのpathが`Abrupt`でもよい。完全な規則は
[result boundaryとcompletion](control.md#completion-judgment)に定める。lambdaのresult edgeへ名前を付ける専用構文はなく、
early resultや直和の構築にはbody全体をdirect result blockにできる。

```mal
log :: Symbol -> Unit := (message) -> print(message);
```

## direct block

blockは単独のexpressionとして置ける。body itemをsource orderで評価し、最後のexpressionがblockの値になる。block内の
bindingはblockの外へ出ず、外側のlocal bindingとresult binderは通常どおり参照できる。

```mal
answer :: Unit -> Int32 := () -> {
    value :: Int32 := {
        base :: Int32 := 40;
        base + 2
    };
    value;
};
```

`{ body }`は`() -> { body }`の省略ではない。前者はその場で評価するblockであり、後者は`Unit` parameterを持つfunction
valueを作る。したがってbare blockは期待関数型から暗黙にlambdaへ変換されない。top-level initializerには置けない。

result continuationを導入するblockは[direct result block](control.md#direct-result-block)に定める。

## application

applicationはcontinuationを先に書く`f(a)`とvalueを先に書く`a[f]`のどちらでも表せる。
calleeがvalue nameであるapplicationは、receiver-first application（UFCS）として第一引数を先に書く
`a.f(...)`でも表せる。

```mal
f()
f(x)
f(x, y)
makeFunction()(x)
x[f]
x[f][g]
x.f()
x.f(y)
x.f(y).g()
```

`f(a)`と`a[f]`、`f()`と`()[f]`はそれぞれ同じapplicationである。`[f]`は`()[f]`のUnit valueを省略した形、
`f(a, b)`は`f((a, b))`である。`g(f(a))`、`g(a[f])`、`f(a)[g]`、`a[f][g]`も同じapplicationの列を表す。
表記に依存しない評価順は[評価戦略](execution.md#評価戦略)に定める。

receiver-first applicationの`a.f()`は`f(a)`、`a.f(b, c)`は`f(a, b, c)`と同じapplicationである。
generic calleeでも`a.f<T>(b)`は`f<T>(a, b)`と同じapplicationである。
`f`はreceiverの型から探索せず、source位置で通常のvalue nameとして解決してから既存のfunction application型規則を
適用する。receiver、残りの引数、calleeの順に評価する。

`.`、value name、parenthesized argument listは全体で一つのapplication suffixである。`a.f`はexpressionではなく、
field access、property、method value、bound functionを導入しない。

[AddressとBuffer](memory.md)のoperationは通常のexpressionとして評価する。`Symbol`と`Buffer`のlengthは`#`で表す。

external declarationが導入する名前も通常のfirst-class function valueである。参照や受け渡しではhost operationを
実行せず、applicationしたときだけ[`extern`境界](extern.md)を越える。

## Memory expression

`Address`の先はhost contractの領域であり、malは直接dereferenceやoffset計算を行わない。C hostとの固定長copyでは、
要素型がcanonical representationを決める。

```mal
readUInt64 :: Address -> UInt64 :=
    (address) -> from<UInt64>(address, 0usize, 1usize).get(0usize);
```

型、評価、preconditionは[AddressとBuffer](memory.md)に定める。

## if

`if`は`Bool`に対するcontinuation applicationのsurface syntaxであり、core termではない。conditionの括弧、
`then`、`else`はすべて必須であり、各branchには一つのexpressionを置く。複数のbody itemやbranch-local bindingが
必要ならblock expressionを使う。標準の表記ではconditionの後、`then`、`else`をそれぞれ別の行に置く。

```mal
absolute :: Int32 -> Int32 := (x) ->
    if (x < 0)
    then -x
    else x;
```

condition は `Bool`、すなわち構造的に `[Unit, Unit]` と等しい型でなければならない。両 branch の結果型は同一でなければならない。branchに置いたblock内のbindingはそのblockにだけscopeを持つ。

上の形は次へdesugarする。conditionは一度だけ、選択したbranchより先に評価する。

```mal
(x < 0)[
    () -> x,
    () -> -x
]
```

`then` は `Bool` の index 1、`else` は index 0 に対応する。`else if` 専用構文はなく、必要なら `else` のexpressionに別の `if` を置く。

## 直和の除去

```mal
getOrZero :: MaybeInt32 -> Int32 :=
    (value) -> value[
        () -> 0,
        (x) -> x
    ];
```

二つ以上のcontinuationを持つ`value[f, g, ...]`は直和を除去する。
continuation数は直和の項数と一致し、位置`i`のcontinuationは第`i`項をparameterとするfunctionでなければならない。
すべてのresult型は同一である。scrutineeとcontinuationの評価は[評価戦略](execution.md#評価戦略)に定める。

作用と評価順を捨象した値の対応だけを見れば、`f : A -> X`と`g : B -> X`に対するcontinuation列は
copairing `[f, g] : [A, B] -> X`とみなせ、`value[f, g]`はそのcopairingの`value`へのapplicationと読める。
ここでの`[f, g]`は型の対応を説明するmetanotationであり、source expression、function value、またはfunctionのproductを
構築するものではない。sourceのcontinuation expressionを先に一つの値へまとめる書換えは、
[評価戦略](execution.md#評価戦略)が定める非選択continuationの遅延を保存しないため、一般には認めない。

一つのcontinuationを持つ`value[f]`はvalueの型にかかわらず通常のapplicationである。直和を一つのfunctionへ渡す場合も
この規則を使い、直和除去との違いはcontinuation数から一意に決まる。

continuationを持たない`value[]`は空直和のeliminationであり、[completion規則](control.md#empty)に従う。

## expression statement

lambda body では式の結果を捨てられる。

```mal
log("done");
```

これは概念上 `_ := log("done");` と同じである。
