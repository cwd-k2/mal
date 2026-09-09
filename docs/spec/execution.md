# 実行意味論

Status: Current v0.5 profile

## 評価戦略

mal は strict call-by-value である。式、引数、lambda body は source order で左から右、上から下へ評価する。

```mal
f(a(), b(), c())
```

では `f`、`a()`、`b()`、`c()`、application の順になる。通常の関数が `extern` を呼び得るため、compiler は観測可能な順序を変更してはならない。

## scope と closure

ラムダはlexical scopeを持ち、bodyから参照する外側のparameterとlocal bindingをcaptureする。

ラムダ式を評価すると関数値が生成される。関数値は概念上、ラムダのcodeと、lexically captureしたlocal valueのenvironmentからなるclosureである。environmentにはラムダ式を評価した時点の値をby-valueで保持する。

```mal
makeAdder :: Int32 -> (Int32 -> Int32) := \(x) {
    \(y) { x + y };
};

addTen := makeAdder(10);
result := addTen(5);
```

`result` は `15` となる。`x` の binding は immutable なので、後から別の値へ変化しない。

top-level binding、external operation、predefined binding、compiler primitiveは全programから直接参照でき、closureごとの
environmentに保存する必要はない。external operationは通常のfunction valueであり、そのapplicationがhost operationを
実行する。参照または受け渡しだけではhost境界を越えない。

```mal
outer :: Int32 -> (Unit -> (Unit -> Int32)) := \(x) {
    middle := \() {
        \() { x };
    };

    middle;
};
```

inner lambdaが参照する`x`は、inner closureを構築するmiddle lambdaにも自動的に転送される。

closure は定義した scope の外へ返したり、他の関数へ渡したりしてよい。function equality は存在せず、program から code と environment を分解・観察することはできない。

言語意味論はenvironmentの物理的な配置や回収方式を規定しない。必要なstorageを確保できなければtrapする。
reference compilerの現在の方式は[implementation notes](../implementation/compiler.md)に記録する。

compiler は観測可能な動作を変えない限り、capture 除去、lambda lifting、stack allocation などにより environment allocation を省略してよい。

`Symbol`をcaptureした場合も、その意味とlifetime authorityはmalに属し、environmentから到達できる間は値が保持される。
external opaque valueをcaptureしても、そのresourceに新しいownership規則は加わらない。詳細は
[EngramとExtern](engrams.md)に従う。closure自体の決定理由は[D003](../history/decisions/D003.md)、
lexical captureの決定理由は[D007](../history/decisions/D007.md)に記録する。

## 再帰

`for` と `while` はなく、反復は再帰で表す。

```mal
sum :: Int64 -> Int64 := \(n) {
    if (n == 0)
        then { 0 }
        else { n + sum(n - 1) };
};
```

単一のvalue name pattern、型annotation、直接のlambda RHSを持つbindingは、そのlambda bodyから自分自身を参照できる。これは通常のsequential bindingに対する唯一の自己参照例外である。product pattern、annotationのないbinding、lambdaを括弧などの別の式で包んだRHSには適用しない。forward referenceとmutual recursionはない。

direct tail recursion を loop へ lower してよいが、program から観測できる評価順序を変えてはならない。

## 整数

整数は固定幅二の補数である。`+`、`-`、`*` は signed/unsigned とも bit width で wrap する。signed overflow を backend の undefined behavior にしてはならない。

次のoperationにはpreconditionがある。

- integer divisionまたはremainderのdivisorは0でない
- 最小signed integerのdivisionまたはremainderではdivisorは`-1`でない
- shift countは0以上、left operandのbit width未満である

`<<`と`>>`のright operandはleft operandと同じ整数型で、結果も同じ型である。`<<`は数学的な`2^count`倍を
operandのbit widthでwrapしたbit patternを返す。unsigned `>>`はlogical shift、signed `>>`はsign bitを複製する
arithmetic shiftとする。これらのpreconditionに違反したprogramの実行結果は保証しない。backendはpreconditionを
満たすoperationの結果をCのsigned shiftの偶発的な挙動へ依存させてはならない。
現在の設計理由は[D035](../history/decisions/D035.md)に記録する。

## 浮動小数点

`Float32`と`Float64`の演算はIEEE 754-2019のbinary32/binary64に従い、rounding modeを常にround-to-nearest, ties-to-evenとする。rounding mode、exception flag、quiet/signaling NaNの区別はmalから観測・変更できない。floating-point exceptionはtrapしない。

各`+ - * /`はoperand型の精度で個別に丸める。compilerは結果を変える式の再結合、暗黙のfused multiply-add、余分な中間精度を使用してはならない。subnormalは保持し、flush-to-zeroしてはならない。

zero除算、有限値のoverflow、invalid operationはIEEE 754に従ってinfinityまたはNaNを返す。NaNを生成するprimitive演算について、NaNのsignとpayload、およびoperandからのpayload伝播は未指定である。

比較は次の規則に従う。

- `+0.0 == -0.0`はtrueで、大小比較でも両者は等しい。
- どちらかがNaNなら`==`はfalse、`!=`はtrueである。
- どちらかがNaNなら`< <= > >=`はすべてfalseである。

演算規則の設計理由は[D009](../history/decisions/D009.md)に記録する。

## trap

trap は現在の mal program の評価を即座に異常終了する。mal code から捕捉・回復する構文はない。trap までに完了した `extern` の作用は巻き戻さない。

有効なEngramの構成に必要なstorageを確保できない場合と、そのstorage sizeをtargetで表現できない場合はtrapする。
precondition違反はtrapではなく、特定の実行結果を保証しない。reference implementation固有のresource limitや
internal invariant failureはこの言語上のtrap条件に含めない。

reference compilerのC runtimeは理由をstderrへ出力して`abort()`する。portableなprocess exit codeは規定しない。
host adapterは回復不能なcontract violationをgenerated headerの`mal_trap`で同じ終了へ写像できる。

pointer accessのregion、permission、lifetime違反とpointer offsetのprecondition違反はhost contract違反であり、
特定の実行結果を保証しない。詳細は[memory](memory.md)に定める。

## core calculus

表面構文を除いた概念上の core は次である。

```text
e ::= variable | literal | lambda | application
    | product | sumInjection | case
    | primitive | hostOperation | fix
```

`Bool` は `[Unit, Unit]`、`if` と論理演算は `case` へ消去できる。`::` は型情報、`:=` はlambda application、blockの末尾式はlambdaの結果へ消去できる。これは実装を強制する定義ではなく、表面機能を追加するときの意味論上の基準である。
