# completionとlexical edgeの合成

Status: Accepted v0.7

この文書は、式のcompletionをsequence、branch、lexical result boundaryで合成する規則を定める。result binder、`when`、
空直和`[]`の個別規則は[result boundary、`when`、`Empty`](control.md)を正とする。

## Completion summary

式から到達できるlocalな正常完了とlexical result boundaryへのtransferを、概念上はedgeからvalue payloadへの有限対応として表せる。

```text
F(normal) = value A
F(κ)      = value B
```

`normal`は現在のevaluation contextが受けるedge、`κ`は特定のlexical result boundaryのidentityである。edgeがsummaryに
なければ、その式からその行先へ到達するpathはない。これは規則を記述するmetanotationであり、checkerの内部表現を有限mapへ限定しない。

checkerのcompletion judgmentは、normal edgeの有無だけを見るprojectionである。

```text
F(normal) = value T  => Value(T)
F(normal)がない      => Abrupt
```

`Abrupt`は型でもsubtypeでもなく、期待型や仮のvalue typeを持たない。projection後も、result transferを表すchecked termは
destination identityとpayloadを保持し、boundaryへの接続が終わる前に異なるtransferを同一のcontrol moveへ変えない。
empty eliminationはnormal edgeもdestination edgeも持たない。

## Sequenceとstrict context

`e1; e2`と`x := e1; e2`は、`e1`のnormal edgeだけを`e2`へ接続する。`e1`からlexical result boundaryへ向かうedgeは
bindingと`e2`を迂回して外側へ伝播する。normal edgeがなければ`e2`はunreachableである。

binding initializerは`Value(T)`でなければならない。block途中の式が`Abrupt`なら、それより後のsource itemはunreachable codeとして
compile-time errorになる。product要素、operator operand、argument、callee、continuationなどstrictに値を要求する位置で
`Abrupt`になれば囲む式も`Abrupt`になり、評価順で先にある式の作用は保持し、後にあるsubexpressionはunreachableとして拒否する。
短絡演算とsum eliminationでは選択されないbranchをunreachableとはしない。

## Branch

`if`の各branchから到達できるedgeは、destination identityごとにjoinする。片側だけにあるedgeは残り、同じedgeが両側にある場合は
payload typeが一致しなければならない。normal edgeにも同じ規則を使うため、completionのprojectionは次になる。

```text
join(Value(T), Value(T)) = Value(T)
join(Value(T), Abrupt)   = Value(T)
join(Abrupt, Value(T))   = Value(T)
join(Abrupt, Abrupt)     = Abrupt
```

異なる二つの`Value`型はtype errorになる。二つの`Abrupt` branchが異なるresult boundaryへ移る場合も、checked termはそれぞれの
destinationを保持する。sum eliminationのin-place branchも同じ規則でjoinする。

`when (condition) body`のfalse pathは`Unit`を持つnormal edgeである。bodyがresult boundaryへだけ移る場合、後続へ到達するのは
false pathだけになる。

## Lexical result boundary

`[k] => body`はfreshなboundary identity `κ`を作り、`k`をそのdestinationへ対応させてbodyを検査する。binder groupでは、各binderが
構築したsum valueを同じ`κ`へ送る。result binder applicationはargumentを通常の順序で一度評価してから`κ`へ移り、そのpathは
bodyのlocal contextへ戻らない。

boundaryはbodyのsummaryから自分宛ての`κ` edgeだけを受け、そのpayloadをblockのnormal edgeへ接続する。外側のboundaryへ向かうedgeは
変更せずに伝播する。bodyはnormal fallthroughを持たず、自身の`κ`へ到達するreachable pathを少なくとも一つ持たなければならない。

```mal
nested :: Bool -> Int64 :=
    (condition) -> [outer] => {
        value :: Int64 := [inner] => {
            when (condition) outer(1i64);
            inner(2i64)
        };
        outer(value)
    };
```

この例でinner boundaryは`inner`へのtransferだけを`value :=`が受けるnormal completionへ変換する。`outer(1i64)`のpathはbindingと
最後の`outer(value)`を迂回し、外側のboundaryへ直接到達する。

## Lambda resultとlowering

期待result型を`B`とするlambda bodyは`Value(B)`または`Abrupt`でよい。前者のnormal edgeはinvocation resultへ接続し、後者の
invocationは正常にはreturnしない。ordinary lambdaは外側のresult binderをcaptureせず、applicationごとにfreshなinvocation resultを持つ。

result binderとcompletion judgmentはcore境界で消去する。複数のnormal pathが同じlexical continuationへ進む場合、core、ANF、
closure IRは後続を複製せず、non-first-classなjoin identityへのtransferとして保持する。result transferは対応するjoinへのjump、
empty eliminationはzero-arm sum eliminationになる。この変換は、transferより前に完了した`extern`作用を省略、複製、並べ替えしない。
