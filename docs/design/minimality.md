# 「最小」とは何か

Status: Current design policy

「機能の一覧が短い」だけでは最小言語にならない。省いた概念が複雑な静的解析、暗黙の runtime、または未記述の host contract に移動しただけなら、system 全体は小さくなっていない。

mal では次の合計を小さくする。

```text
language surface
+ static semantics
+ dynamic semantics
+ runtime representation
+ compiler/backend
+ host contract
+ implicit behavior and cost
+ specification/API discovery surface
```

## 利用者から見た最小性

malのminimalismは、実装の行数だけでなく、利用者がprogramの挙動を把握するために調べる範囲を小さくする。

- capture、allocation、retain/release、暗黙変換を増やす場合は、対象と挿入規則を短く列挙できる境界に閉じる。
- policyを言語や巨大なstandard APIへ固定せず、可能な限り利用者が選べるmechanismまたは小さな`extern` contractに置く。
- sourceから依存と評価順を追えるようにする。
- surface sugarは、既存coreへの局所的で完全なdesugaringを短く説明できる場合に限る。
- 仕様から外した責務を未記述の慣習へ追い出しただけなら、最小化とは数えない。

便利な標準APIを多数用意すると、実装量だけでなく「どのAPIを選び、どの暗黙規約に従うか」という探索costが増える。malはmechanismを少数提供し、用途別policyをprogramまたはhost側へ残す。

memory operationを未検査にすること自体も、自動的に最小とはみなさない。bounds、permission、initialization、lifetimeを
callerまたはhost contractのpreconditionとして一貫して表現できること、その上で個別operationごとの例外や防御的挙動を
source contractへ増やさないことを基準にする。[D008](../history/decisions/D008.md)

controlは、利用者がすべてのmechanismをoperationごとに再定義することではない。policyを選択でき、選択後の
mechanismが一つの規則から予測できる状態を指す。minimalityの比較では、primitiveやsyntaxの個数だけでなく、
独立して発見、理解、検証しなければならないcontractと、各call siteで再判断する事項の数を小さくする。
変更規模や一度に追加する機能数も、それ自体ではminimalityの尺度にしない。基礎modelを理解した後に、個別の型や
operationの挙動を同じ規則から導けるなら、個別codecや例外を多数残すより広い一つのmechanismの方が小さくなり得る。

### 反復controlとdomain stepを分ける

自己再帰が次の状態を選ぶだけで、各stepの後に未完了の処理を残さない場合、再帰そのものをdomain operationへ
埋め込む必要はない。変化する値を明示的なstate、一回分の処理をstep、次状態と最終結果の選択を直和として分ければ、
同じmechanismから`upto`、`times`、`fold`、`any`、program固有のstate machineを定義できる。利用側にはmechanism名の
`loop`を常に露出させず、走査範囲、empty case、順序、早期終了を表す用途上の名前を選んでよい。

この分離は関数全体をloopへ変える規則ではない。前処理の後でloop expressionから値を得て後続処理へ戻る局所利用も
できる。採用するのは、終了判定やcursor更新の重複を一箇所へ集め、domain callbackを「現在の要素をどう扱うか」に
近づけられる場合である。closure、state product、追加の抽象名が直接再帰より多くの知識を要求するだけなら分離しない。
特にhost resourceのauthorityはoperation contractに明示する。このauthorityを反復callbackから隠す
抽象化は行わず、domain名を持つ直接再帰で引数とlifetimeを露出させる。

子から戻った後にも処理を行うtree traversal、入力のnesting自体を表す構造再帰、resource cleanupの順序を表す再帰は、
単純な反復へ置き換えない。明示work stackなど別の表現が必要なら、反復combinatorの導入とは別の設計判断として扱う。
実行例は[`generic-loop`](../../examples/generic-loop/)、parser state machineは
[`json-query`](../../examples/json-query/)、処理途中の局所利用は
[`resizable-buffer`](../../examples/resizable-buffer/)、carrier走査とrelation解釈の分離は
[`relation-views`](../../examples/relation-views/)に置く。

external operationのcontractは宣言に置き、applicationごとに同じ分類を再記述しない。external functionも通常の
function valueと同じ参照、shadowing、application規則に従い、境界transportだけを宣言されたidentityから決める。

例えばexternal storageでは、allocation、deallocation、region、permission、lifetimeをprogram固有の`extern`
contractに残し、canonical memory representationとのload/storeを共通primitiveに固定できる。これによりallocation
policyを選ぶcontrolを保ちつつ、型ごとにwidth、alignment、failureをhost APIから調査する必要をなくす。
canonical layoutを持つ型の範囲は、一度理解した配置規則からsize、alignment、padding、valid representationを
再帰的に導けるかで決める。program固有のencodingはmalで書くcodecとしてsourceに残し、runtime representation、
public host ABI、wire formatをcanonical memory layoutへ暗黙に結合しない。正確な配置規則は
[external memory](../spec/memory.md#canonical-layout)が所有し、[authority](authority.md#policyとmechanismを分ける)は
配置規則とresource policyを分ける判断軸だけを所有する。

memory、resource、host境界では、[EngramとExternのauthority](authority.md)から必要なadmission、observation、
capability transferを導く。reference compilerのEngram回収は
[managed ownership](../implementation/ownership.md)のresponsibility規約に閉じ、Extern resourceのpolicyへ拡張しない。採択理由は
[D033](../history/decisions/D033.md)と[D055](../history/decisions/D055.md)を正とする。

## 意味論上の核

言語の性質を説明するための核は次とする。値、application、dynamic continuationの関係は
[値、解釈、control](value-interpretation-and-control.md)を正とする。

```text
variable, lambda, application
Unit
product, sum
primitive scalar
```

`f(a)`と`a[f]`は同じapplicationであり、その評価が一回のinvocationを始める。直和の除去は複数continuation templateから
active variantに対応する一つを選ぶapplicationである。`Bool`は
`[Unit, Unit]`、`if`はBoolへのcontinuation application、`binding`はlambda/applicationへ消去できる。
blockの末尾式はlambdaの結果である。`fix`/自己再帰を足すと停止性を失い、`extern`を足すとhostとの
観測可能な作用が生まれるため、純粋な核とは分けて考える。

productとsumは数学的にさらにencodingできる場合があるが、そのencodingをgeneric codeで繰り返してもcanonical layout、
sum construction authority、host mappingは自動的に得られない。これらを一つの構造規則から導くため、両方をprimitiveとして残す。

現在のprofileに含む具体的な型とoperationは[言語の範囲](../spec/scope.md)だけに列挙する。実装の変更履歴はGitを参照する。

## 機能を加える判定基準

新機能は、次のすべてに答えられる場合だけ候補にする。

1. 既存の lambda、application、product、sum、primitive、extern の組合せでは何が不自然か。
2. surface syntax だけでなく型規則と評価規則を一段落で説明できるか。
3. reference compilerのexecution representationで説明でき、別backendを不必要に妨げないか。
4. hidden allocation、GC、lifetime analysis を要求しないか。要求するなら、それを言語の責務として認めるか。
5. 一つ以上の conformance test で境界を固定できるか。
6. value、storage、resourceのauthorityと、境界を越えるoperationを説明できるか。
7. system全体で独立contractと局所的な再判断を減らすか。

「頻繁に使う」「短く書ける」だけでは追加理由にしない。一方、仕様から外した結果、すべての利用者が危険な独自 ABI を発明するなら、外したコストも数える。

## 最小性の acceptance criterion

各profileの機能について、次が揃った時点を「仕様に存在する」とする。

- parse できる concrete syntax
- name resolution と型規則
- value/evaluation/trap の規則
- extern を跨ぐ場合の contract
- reference backend での lowering 方針
- valid/invalid/edge case の test

この定義なら、文法に一行だけ存在する未実装機能を「小さい」と数えずに済む。
