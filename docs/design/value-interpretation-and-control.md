# 値、解釈、control

Status: Current design policy

この文書は、malにおける値、application、continuation、domain上の意味の関係を定める設計上の判断軸である。
評価順序と実行結果は[実行意味論](../spec/execution.md)、applicationの型と構文は
[式とbinding](../spec/expressions.md)、resource境界は[EngramとExternのauthority](authority.md)を正とする。

## 中心命題

dataは理解されるためにあり、dataだけでは理解にならない。値は理解可能な区別を保持する再利用可能なcarrierであり、
operationがその区別を解釈し、applicationが一回の解釈を実行する。その実行から先のdynamic controlは複製せず、
高々一度だけ進める。

```text
reusable data
    -> interpretation by application
    -> affine control consequence
```

ここでaffineとは、dynamic continuationを適用せず捨てることはあっても、同じcontinuationを複数回resumeしないことをいう。
値の再利用、function valueの再適用、同じalgorithmの再実行は、進行中のcontrolを複製することではない。

## 語彙

| 概念 | 役割 |
|---|---|
| type | 値が保持できる区別と、利用できるlanguage operationを定める |
| value | 現在成立している区別を保持するcarrier |
| operation | 値を特定の関係または目的の下で解釈する規則または計算 |
| function / continuation template | 値を受け取る再利用可能なfunction value。bodyはapplicationまで実行しない |
| application | source上で値とtemplateを接続するexpression |
| invocation | applicationの評価によって始まる一回のdynamic execution |
| dynamic continuation | invocationのresultを受け取る残りのcontrol |
| authority | 値またはreferentを構成、観測、変更、破棄できる範囲を定める |

`a[f]`と`f(a)`は同じapplicationである。application自体はinvocationではなく、その評価が新しいinvocationを始める。
通常のfunction value `f`は複数回applicationできるが、各invocationのdynamic continuationは別であり、それぞれ高々一度だけ進む。

```text
a : A               reusable value
f : A -> B          reusable template
a[f]                application
evaluate a[f] once  fresh invocation
use its B result    affine dynamic continuation
```

result binderは通常のfunction valueではなく、block invocationに属するnon-first-classなlexical continuation targetである。
compilerがcontrol frameへ保存するresume targetもsource valueではない。語の混同を避けるため、source上の再利用可能なfunction、
lexical target、実行中のdynamic continuation、backendのresume targetを無修飾の「continuation」だけで同一視しない。

## 値と理解

値の構成はstrictである。productは要素をすべて評価してから一つの値になり、closureは構築時点のcapture値を保持する。
確立したvalue carrierはbindingから繰り返し利用できる。ただし、carrierの再利用は、その内部の`Address`やexternal opaque
valueが指すreferentの複製、lifetime延長、再利用可能性を意味しない。

型とcarrierはdomain上の意味をすべて内包しない。`Buffer<NodeRow>`がtreeであるにはroot、edge、bounds、acyclicityを解釈する
operationとinvariantが必要であり、`(Address, USize)`がreadable bytesであるには範囲、permission、initialization、lifetimeを
定めるoperation contractが必要である。carrier、relation、invariantの分担は[表現と関係を分ける](representation-and-relations.md)
を正とする。

同じ値を異なるoperationへ適用すれば、同じcarrierを異なる関係または目的の下で理解できる。構造的一致だけからdomain relation、
permission、resource policyを推測しない。authorityは意味そのものではなく、どの解釈を誰がいつ実行してよいかを制約する。

## 選択とaffine control

lambda bodyはapplicationまでsuspendする。二つ以上のcontinuationによるsum eliminationは、active variantに対応するtemplateだけを
評価して適用し、非選択templateを評価しない。これはmemoizationを伴うcall-by-needではなく、controlによる選択まで実行を
保留する規則である。選択後のinvocationは通常どおりstrictに評価する。

dynamic continuationはsource valueとして構成、capture、clone、resumeできない。現在のcontrolは次のいずれかで進む。

- invocationが値で正常完了し、残りのdynamic continuationを一度進める。
- result binder applicationが現在位置へ戻らず、対応するlexical targetへ移る。
- sum eliminationが一つのbranchを選び、非選択branchを捨てる。
- empty eliminationまたは発散により、正常な後続へ到達しない。
- trapがprogramのactive controlを破棄する。

複数の可能性を扱う場合は、進行中のcontinuationを複製せず、候補またはstateを値として構成し、それぞれに新しいapplicationを
行う。この分担により、再利用可能な記述と、外部作用を含み得る一回の現実の進行を分ける。

## Host境界

external functionもMAL内では通常の再利用可能なfunction valueであり、そのapplicationごとに一回host operationを実行する。
hostへ渡す`mal_call_t`はdynamic continuationではなく、現在のextern invocationを規定されたresultで一度完了させる
call-scoped capabilityである。hostはこれを保持して後からresumeせず、完了後にどのcontrolへ進むかを選ばない。

```text
MAL chooses an external operation
    -> host determines this operation's result
    -> MAL continues with that result
```

この非対称性を変えるcallback、suspension、exception、library entry、concurrent rootを検討する場合は、便利な呼出し形だけでなく、
dynamic continuationを誰が保持し、何回、どのthreadから、どのlifetimeで進められるかを新しいcontrol authorityとして定める。

## 設計時の確認事項

新しいdataまたはcontrol mechanismは次を短く答えられる場合だけ候補にする。

1. carrierはどの区別を保持するか。
2. どのoperationとinvariantがcarrierへdomain上の意味を与えるか。
3. applicationは何を評価し、どの時点で新しいinvocationを始めるか。
4. 再利用できるのはcarrier、referent、templateのどれか。
5. dynamic continuationは正常完了、lexical transfer、discardのどれで進むか。
6. control targetはfirst-classか、どのscopeへescapeできるか、何回適用できるか。
7. 値またはreferentのauthorityはどこにあり、control scopeがそのlifetimeをどう囲むか。
8. hostまたはruntimeが保持するものはdata、operation identity、completion capability、dynamic continuationのどれか。
