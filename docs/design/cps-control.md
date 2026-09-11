# CPS計算と明示的control port

Status: Experimental design; not part of the current v0.5 profile

この文書は、通常の値とCPS計算を型で分け、既存のcontinuation applicationを計算の実行にも使う試験設計を定める。
現行の規範は[`spec/`](../spec/)であり、この文書の構文と意味論をv0.5 programへ適用してはならない。
採否は[最小性の方針](minimality.md)に照らし、段階ごとの実装とtestから判断する。

## 目的

一つの機構から、通常の結果、複数のabortive exit、早期return、および限定されたresumptionを説明できるか検証する。
利用者がanswer type、runtime stack、handler implementationを通常の関数ごとに記述することは求めない。

この試験では次を前提とする。

- CPS計算と通常値をsource typeで区別する。
- control port名はlocal bindingであり、型のnominal identityにしない。
- resumable portは位置と送出型・再開型だけで構造的に区別する。
- resumptionは意味論上複数回呼べる。one-shot制約やlinear typeは導入しない。
- 暗黙のstack captureは行わず、CPS計算だけを明示的なcontrol representationへlowerする。

exception、async、effect row、general stack capture、およびexternを越えるresumption transportは、この試験の最初の範囲に含めない。

## 計算型

answer typeを`R`としたとき、`A`を受け取るcontinuationと`A`を生成するCPS計算を概念上次のように書く。

```text
K_R A = A -> R
*A    = forall R. (A -> R) -> R
```

`forall R`はsurface syntaxへ公開しない。`*A`はprimitiveな計算型として検査し、handler applicationごとに全handlerの
result型が同じであることからanswer typeを決める。固定された未知の`R`を`*A`内部へ保存するとは解釈しない。

`**A`は`A`を生成する計算を生成する計算であり、`*A`と定義上同一視しない。必要なら次のflattenを通常関数として表す。

```text
join : **A -> *A
join(outer, return) = outer((inner) { inner(return) })
```

`!A`は計算型に使わない。`!`はNeverや複製可能性との既存の連想が強く、将来shotnessを表す必要が生じた場合にも
選択肢として残す。malはtyped pointerを持たないため、この試験ではprefix `*`を計算型に割り当てる。

## abortive exit

単一のexitを持つ定義は次の形を候補とする。

```mal
increment :: Int32 -> *Int32 :=
    (x)[return] {
        return(x + 1)
    };
```

概念的なCPS型は`(Int32, Int32 -> R) -> R`である。CPS blockは通常値を暗黙に`*A`へliftしないため、次は不正とする。

```mal
increment :: Int32 -> *Int32 :=
    (x)[return] {
        x + 1
    };
```

全control pathは、列挙されたabortive exitのapplication、または同じanswer typeを持つCPS tail-callで完了しなければならない。
この規則は末尾tokenを常に`return`へ限定せず、`other(x)[return]`によるtail forwardingを認める。

複数exitの型には既存の直和を使う。

```mal
parse :: Input -> *[Ast, Error] :=
    (input)[return, throw] {
        // Every path ends in return(value), throw(error), or a CPS tail-call.
    };
```

これは概念上`forall R. (Ast -> R) -> (Error -> R) -> R`である。exit名は型に含まれず、位置だけが直和の項と対応する。

## 計算の実行

通常のapplicationが値を返すのに対し、CPS functionのapplicationは計算値を返す。その計算へ既存の`[]` applicationで
continuationを渡す。

```mal
increment(41)[print]

parse(input)[
    (ast) { use(ast) },
    (error) { report(error) }
]
```

`f(x)[k]`を専用call syntaxにはせず、`f(x)`で得た`*A`への通常のcontinuation applicationとする。compilerは中間の
計算closureを観測できない場合にproducerとhandlerをfusionしてよい。

early returnはabortive exitを外側のCPS blockからcaptureして表す。exit application後に同じpathの式を評価してはならない。
block中の後続処理はcompilerがcontinuationとして構成するため、条件付きearly returnの非選択branchはその後続へ進む。

## resumable port

`Y <- S`は、operationが`Y`をhandlerへ送り、handlerが`S`を渡して計算を再開する構造的なport signatureとする。
`<-`は通常のfunction arrowではなく、abortive payloadとしてのfunction型とresumable portを構文上区別する。

```mal
produce :: Input -> *[Item <- Reply, Result, Error] :=
    (input)[yield, return, throw] {
        reply := yield(item);
        // Continue with reply, then finish through return or throw.
    };
```

対応するhandlerは概念上`(Item, Reply -> R) -> R`を受け取る。したがってoperation site以降は`Reply -> R`のresumptionとして
handlerへ渡る。`get`と`put`も同じ機構で表せる。

```text
get : Unit <- State
put : State <- Unit
```

通常の`Unit -> State`と`State -> Unit` callbackで十分なprogramにはresumable portを使う必要がない。handlerが残りの計算を
受け取り、純粋なstate threading、rollback、探索などとしてoperationを再解釈する場合だけ追加の能力がある。

## resumptionのshotness

resumptionは通常のclosureと同じく複数回applicationできる。handlerが一度だけ呼べばone-shot、複数回呼べばmulti-shotになる。

```mal
((), resume) {
    left := resume(false);
    right := resume(true);
    merge(left, right)
}
```

この規則はresumption専用のlinear typeとuse checkerを不要にする。reference implementationのbaselineはresumptionを
first-class closureまたは同等のpersistent representationとして保持する。resumptionがescapeせず各pathで高々一度だけ
tail applicationされると証明できる場合、compilerは観測可能な意味を変えずlinearなcontrol frameへlowerしてよい。

multi-shot resumeは捕捉地点以降のexternal operationも呼び出しごとに再実行する。実行順は通常のstrict evaluation orderに従う。

## 空直和

現行仕様で予約されている`[]`を空直和`Empty`として使い、`value[]`をzero-continuation eliminationとする案を別途検証する。

```text
() : 空直積Unit
[] : 空直和Empty
K_R Empty = Empty -> R ~= Unit
```

この割り当てはarray syntaxには使えないという[D004](../history/decisions/D004.md)の既存判断と両立するが、現行仕様を変更する
決定ではない。`[A]`を一項直和にするか、別用途に残すかも未決とする。

## 表現できない一引数when

純粋な`*Unit = forall R. (Unit -> R) -> R`だけから、continuationを条件付きで呼ぶ次の`when`は定義できない。

```mal
when :: Bool -> *Unit :=
    (condition)[then] {
        // The false path cannot construct an arbitrary R without calling then.
    };
```

二つのbranch continuationを要求する`Bool -> *[Unit, Unit]`、通常のcallback関数、または省略branchを補う限定的なsurface sugarなら
表現できる。normal resultとoperation handlerを分離して一引数の`when(condition)[then]`を一般化する設計は、`*A`だけでなく
`A ! Effects`に相当するeffect systemを導入するため、この試験へ暗黙に混ぜない。

## compiler境界

現在のcontrol IRは`Call`に静的な`resume: StateId`を持ち、recursive region内のnon-tail callだけにtyped frameを構成する。
frameは一つのcontrol arenaへpushされ、return時にpopしてlive valueとenvironmentをresume stateへ戻す。region外callはnative stackを
使えるため、この表現から任意のdynamic stack suffixを直接captureしてはならない。

最初の実装は`*` functionをclosure conversionより前にexplicit CPSへlowerし、その中のcall後処理だけをclosureとしてreifyする。
通常functionはresumable operationを実行せず、`*` functionから通常helperを同期的にcallできる。`[]`でhandlerを適用する地点を
delimiterとし、CPS controlをnative stackへ隠さない。

この変更では少なくとも型、checked AST、CPS lowering、closure capture、control IR、managed ownerの保持、LLVM loweringを検証する。
現在のarena frameをone-shot最適化として再利用する場合も、baselineのmulti-shot result、effect order、trap、owner lifetimeと一致させる。

## 実装段階と未決事項

最初の段階は`*A`とabortive exitだけを実装し、明示的return、early return、複数exit、CPS tail forwardingを検証する。次の段階で
`Y <- S`、handlerへ渡るresumption、multi-shot実行を加える。最後にone-shotと証明できるsiteのframe化をoptional optimizationとして
評価する。

実装着手前に次を決める必要がある。

- `*`のtoken、precedence、およびformatting
- `*[A, B]`と通常の直和値を一つのcontinuationへ渡す場合の導入・除去規則
- resumable handler clauseがresumption parameterを受け取るconcrete syntax
- CPS control valueのcapture範囲とextern transport禁止のdiagnostic
- `[]`と`[A]`の型・式上の用途
- resumptionが保持するmanaged ownerと各multi-shot invocationのshare規則

各段階はpositive、negative、evaluation order、managed capture、深い再帰をfocused testで固定する。resumable段階では同じresumptionを
0回、1回、複数回使うcaseと、各回のextern traceを追加する。
