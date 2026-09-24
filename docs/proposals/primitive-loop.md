# first-class primitive `loop`の導入計画

Status: Exploratory

この文書は、現在Mal sourceで定義しているcontinue-or-break反復を、通常のfunction valueとして使えるpredefined
primitive `loop`へ移す案と、その判断に必要な実装・検証順を管理する。現在の言語規則は[`spec/`](../spec/)、反復と
構造再帰を使い分ける方針は[最小性](../design/minimality.md#反復controlとdomain-stepを分ける)、現行の再帰実行計画は
[application control lowering](../development/application-control-lowering.md)を正とする。この提案は採択前のため、現在の
programが`loop`の存在へ依存してはならない。

## 目的と範囲

cursor、accumulator、early completionを持つ反復に一つの共通mechanismを与え、sourceで定義したself-recursive
combinatorをapplication graph、recursive region、control storageを経由して実行する必要をなくす。新しい構文は追加せず、
alias、capture、引数渡しを含む通常のfunction value規則を維持する。

この導入だけでは自己再帰を禁止しない。tree resultの合成、nested inputの解釈、post-order resource cleanupなど、recursive
callから戻った後に固有の処理が残るoperationは引き続き自己再帰で表してよい。自己再帰の範囲変更は、primitive `loop`の
採否と実装を確認した後の独立した提案とする。

## 提案するcontract

```mal
loop<A, B> :: (A, A -> [A, B]) -> B;
```

`loop<A, B>(initial, step)`は`initial`と`step`を通常のapplication規則で一度ずつ評価した後、次を繰り返す。

1. 現在のstateを引数として`step`を一度applicationする。
2. resultのvariant 0なら、そのpayloadを次のstateとして手順1へ戻る。
3. resultのvariant 1なら、そのpayloadを`loop` invocationのresultとして返す。

`step`のeffectとtrapは通常のfunction applicationと同じであり、選ばれなかったsum continuationを評価しない。variant 0を
返し続けるinvocationは停止しない。`loop`はtermination、fairness、iteration上限を追加で保証しない。

generic binding自体はruntime valueではないという既存規則を保つ。explicit type argumentを与えた`loop<A, B>`は、他の
specialized generic functionと同様に参照、alias、capture、引数渡しできる。function equalityはないため、builtin code
identityとsource functionの物理表現差は観測できない。

## Baseline実行形

各concreteな`(A, B)` specializationにcapture-freeなbuiltin function identityを与える。targetが直接分からないapplicationでも
このidentityを通常のcalleeとしてdispatchでき、baselineは概念上次のoutlined bodyを実行する。

```text
state = initial
loop:
    result = apply step(state)
    case result:
        0(next): state = next; goto loop
        1(value): return value
```

builtin bodyをself-recursive Mal functionへdesugarしない。専用back edgeはcontrol storageへframeを積まず、callback applicationだけが
通常のMal call規則とownership planを使う。callbackが別の`loop`を呼ぶ場合も、builtin bodyをpossible application graphの通常nodeへ
展開して見かけのrecursive SCCを作らない。

state、step closure、continue payload、break payloadのretain、release、moveは、同じsource contractを通常のsumとapplicationで
記述した場合と一致させる。特に`Unit`、managed aggregate、同じspecializationを使う複数callbackをbaseline admission caseに含める。

## 最適化段階

正しさはoutlined baselineだけで満たし、次のdecisionを互いに独立して追加できる境界を設計する。

- callee identityが`loop`と確定したcallをbuiltin invocationへdirect化する。
- step targetが一つならfunction code dispatchをdirect callへ変える。
- code size gateを満たすcall siteではloop bodyとstep bodyをcallerへ融合する。
- 融合後のstate productとcontinue-or-break sumをSSA fieldとbranchへscalarizeする。
- invocation全体で不変なstep captureとmanaged state fieldの不要なowner trafficを除去する。

alias追跡でidentityを確定できないcallはoutlined builtinへfallbackする。最適化の有無でresult、effect順、trap、owner lifetime、
bounded iteration stackを変えてはならない。採用条件と測定方法は
[generated program最適化policy](../development/generated-program-optimization.md)を正とする。

## 導入順

1. 仕様案で型、評価順、first-class利用、nontermination、ownership境界を固定する。
2. resolver、checker、editorへgeneric predefined value identityとdocumentationを追加する。
3. specialization後のbuiltin identityと、outlined baseline bodyを表すbackend非依存planを追加する。
4. LLVM backendへcallback call、sum branch、loop-carried state handoffを実装する。
5. baselineだけでconformance caseと既存exampleを実行する。
6. direct target、step devirtualization、fusion、scalarization、ownershipの順にoptimization decisionを追加する。
7. `generic-loop`、`csr-dijkstra`、`json-query`、`relation-views`に重複するsource `_loop`をpredefined `loop`へ置換する。
8. 利用箇所と生成物を再評価してから、自己再帰のscopeを変更するか別途判断する。

`tail-recursion`は自己再帰実行のconformance例として、tree、spreadsheet、Brainfuck、cleanupの構造再帰はdomain controlの例として、
primitive導入だけを理由に書き換えない。

## 検証境界

focused testは少なくとも次を固定する。

- scalar、product、managed value、`Unit`をそれぞれstateとresultに使う。
- zero step、one step、large iteration、early resultを実行する。
- stepのcapture、extern effect、trapについて評価回数と順序を保つ。
- `loop<A, B>`をalias、closure capture、higher-order argument経由でapplicationする。
- 同じ`(A, B)` specializationを異なる複数のstep targetから使う。
- nested `loop`と、callbackから通常のMal functionを呼ぶ経路を実行する。
- baselineと各optimizationの単独有効化およびproductionで同じobservable behaviorを得る。
- large iterationでnative stackとcontrol frame数がiteration countに比例しない。

LLVM cross-boundary testは生成moduleをClangでcompile・link・executeする。生成形状を検査する場合、outlined baselineがrecursive
regionとcontrol storageを使わないこと、融合時にclosure、state product、sum materializationを除去できることを別々のtestにする。

## 採択前の未決事項

- builtin specializationをclosure、control、executionのどのIRから専用identityとして保持するか。
- indirect builtin callのwrapperとdirect/fused callが共有するownership authorityをどのplanが所有するか。
- fusionとcallback inlineのcode size gateをどの測定から決めるか。
- predefined `loop`導入が、source generic functionとしての実装よりsystem全体のcontractと実装を実際に小さくするか。
- primitive導入後も自己再帰を現在のまま維持するか、direct self applicationへ狭めるか、禁止を別profileで検討するか。

これらを解決して仕様へ移すまでは、source `_loop`を削除せず、この文書を現在の言語contractとして参照しない。
