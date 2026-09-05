# 実行意味論

Status: Current v0.5 profile

## 評価戦略

mal は strict call-by-value である。式、引数、lambda body は source order で左から右、上から下へ評価する。

```mal
f(a(), b(), c())
```

では `f`、`a()`、`b()`、`c()`、application の順になる。通常の関数が `extern` を呼び得るため、compiler は観測可能な順序を変更してはならない。

## scope と closure

ラムダは lexical scope を持ち、capture listに明示した外側のparameterとlocal bindingだけをcaptureできる。capture listを省略したラムダはcapture-freeである。

ラムダ式を評価すると関数値が生成される。関数値は概念上、ラムダのcodeと、capture listに列挙したlocal valueのenvironmentからなるclosureである。environmentにはラムダ式を評価した時点の値をby-valueで保持する。

```mal
makeAdder :: Int32 -> (Int32 -> Int32) := \(x :: Int32) {
    return \<x>(y :: Int32) {
        return x + y;
    };
};

addTen := makeAdder(10);
result := addTen(5);
```

`result` は `15` となる。`x` の binding は immutable なので、後から別の値へ変化しない。

top-level binding、predefined binding、compiler primitiveは全programから直接参照でき、closureごとのenvironmentに保存する必要はない。external symbolは通常のidentifierとして値にせず、`extern symbol(...)`の形でだけ呼び出す。

unlistedの外側local valueはlexical scope内に見えていても参照できない。nested lambdaが複数のlambda境界を越えて値を使う場合、各境界で明示的に受け渡す。

```mal
outer := \(x :: Int32) {
    middle := \<x>() {
        return \<x>() {
            return x;
        };
    };

    return middle;
};
```

inner lambdaのcapture listに現れる`x`はmiddle lambda内での参照でもあるため、middle自身も`x`をcaptureしなければならない。compilerはtransitive captureを暗黙に追加しない。

closure は定義した scope の外へ返したり、他の関数へ渡したりしてよい。function equality は存在せず、program から code と environment を分解・観察することはできない。

言語意味論は environment の物理的な配置や回収方式を規定しない。reference compiler は capture を持つ closure environment を program-lifetime arena に配置し、v0.5 では個別に回収しない。environment allocation に失敗した場合は trap する。

compiler は観測可能な動作を変えない限り、capture 除去、lambda lifting、stack allocation などにより environment allocation を省略してよい。

`String`をcaptureした場合はcopyableなdescriptorをenvironmentへ保持し、そのbytesはprogram終了まで有効である。external opaque valueをcaptureしても、そのresourceに新しいownership規則は加わらない。詳細は[`extern` contract](extern.md)に従う。closure自体の決定理由は[D003](../design/decisions.md#d003-v04-は-lexical-closure-を持つ)、明示capture syntaxは[D007](../design/decisions.md#d007-capture-listを明示する)に記録する。

## 再帰

`for` と `while` はなく、反復は再帰で表す。

```mal
sum :: Int64 -> Int64 := \(n :: Int64) {
    return if (n == 0) then {
        0
    } else {
        n + sum(n - 1)
    };
};
```

単一のvalue name pattern、型annotation、直接のlambda RHSを持つbindingは、そのlambda bodyから自分自身を参照できる。これは通常のsequential bindingに対する唯一の自己参照例外である。product pattern、annotationのないbinding、lambdaを括弧などの別の式で包んだRHSには適用しない。forward referenceとmutual recursionはない。

direct tail recursion を loop へ lower してよいが、program から観測できる評価順序を変えてはならない。

## 整数

整数は固定幅二の補数である。`+`、`-`、`*` は signed/unsigned とも bit width で wrap する。signed overflow を backend の undefined behavior にしてはならない。

次は trap する。

- integer division または remainder の divisor が 0
- 最小 signed integer を `-1` で割る、または remainder を求める
- shift count が負、またはleft operandのbit width以上
- `byteAt` の index が範囲外

`<<`と`>>`のright operandはleft operandと同じ整数型で、結果も同じ型である。`<<`は数学的な`2^count`倍を
operandのbit widthでwrapしたbit patternを返す。unsigned `>>`はlogical shift、signed `>>`はsign bitを複製する
arithmetic shiftとする。backendはCの範囲外shiftやsigned shiftの偶発的な挙動へ依存してはならない。
設計理由は[D014](../design/decisions.md#d014-shift-countはleft-operandと同じ型とする)に記録する。

## 浮動小数点

`Float32`と`Float64`の演算はIEEE 754-2019のbinary32/binary64に従い、rounding modeを常にround-to-nearest, ties-to-evenとする。rounding mode、exception flag、quiet/signaling NaNの区別はmalから観測・変更できない。floating-point exceptionはtrapしない。

各`+ - * /`はoperand型の精度で個別に丸める。compilerは結果を変える式の再結合、暗黙のfused multiply-add、余分な中間精度を使用してはならない。subnormalは保持し、flush-to-zeroしてはならない。

zero除算、有限値のoverflow、invalid operationはIEEE 754に従ってinfinityまたはNaNを返す。NaNを生成するprimitive演算について、NaNのsignとpayload、およびoperandからのpayload伝播は未指定である。

比較は次の規則に従う。

- `+0.0 == -0.0`はtrueで、大小比較でも両者は等しい。
- どちらかがNaNなら`==`はfalse、`!=`はtrueである。
- どちらかがNaNなら`< <= > >=`はすべてfalseである。

演算規則の設計理由は[D009](../design/decisions.md#d009-floatは-ieee-754-2019-の固定profileとする)に記録する。

## trap

trap は現在の mal program の評価を即座に異常終了する。mal code から捕捉・回復する構文はない。trap までに完了した `extern` の作用は巻き戻さない。

reference compilerのC runtimeは理由をstderrへ出力して`abort()`する。portableなprocess exit codeは規定しない。
host adapterは回復不能なcontract violationをgenerated headerの`mal_trap`で同じ終了へ写像できる。

pointer accessのregion、permission、lifetime違反はtrapではなくhost contract違反であり、特定の実行結果を
保証しない。pointer offsetがtargetで表現できない場合だけはtrapする。詳細は[memory](memory.md)に定める。

## core calculus

表面構文を除いた概念上の core は次である。

```text
e ::= variable | literal | lambda | application
    | product | sumInjection | case
    | primitive | externCall | fix
```

`Bool` は `[Unit, Unit]`、`if` と論理演算は `case` へ消去できる。`::` は型情報、`:=` は lambda application、terminal `return` は lambda の結果へ消去できる。これは実装を強制する定義ではなく、表面機能を追加するときの意味論上の基準である。
