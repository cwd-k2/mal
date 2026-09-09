# EngramとExternのauthority

Status: Current design policy

この文書は、memory、resource、host境界を設計するときの判断軸を定める。規範的な型とoperationは
[EngramとExtern](../spec/engrams.md)を正とし、ここでは個別のABIやsyntaxを重複させない。

## ownershipより先にauthorityを問う

値やresourceに関する問題では、最初に「誰が所有するか」ではなく、次の決定権がどこにあるかを問う。

- 何が有効な値か
- identityを誰が構成できるか
- いつまで有効か
- 誰が観測、変更、破棄できるか
- failureをどちらの規則で扱うか

ownership、borrow、copy、lifetime、permissionは、このauthorityを具体的なoperationへ落とした関係である。
したがって一つのmemory management mechanismを言語全体の答えとして先に選ばない。

## 非対称な境界

Engramはmal内部で意味が成立する側、Externは外部のstate、storage、resourceが成立する側である。両者は対称な
value categoryではない。malはEngramを構成してExternへ観測させられるが、Externはmal内部のidentityやrootを
直接構成しない。Externから得たrepresentationはmalによるadmissionを経てEngramになる。

一方、`Ptr`やexternal opaque handleはEngramへ変換されるdataではなく、Externへのcapabilityとして運ばれる。
mal valueに包まれてもreferentのauthorityは移らない。この非対称性により、internal valueの回収とexternal
resourceのclose/freeを同じlifetime mechanismへ結合せずに済む。

## 境界では動詞を選ぶ

境界機能を設計するときは、型を一括して「渡せる」とする前にoperationを分類する。

- admission: external representationからmal valueを構成する
- observation: mal valueを一時borrowまたはcopyして外部から見る
- capability transfer: external referentへの権限を運ぶ

product、sum、closureなどの構造はfieldごとに分類する。外側のaggregateがmal-controlledでも、内側のcapabilityが
指すresourceまでmal-ownedとは限らない。境界をaggregate全体の単一ownershipとして扱わない。

## policyとmechanismを分ける

Externにauthorityがあることと、source-level operationをprogram固有のexternal operationにすることは同じではない。
resourceを取得または破棄するoperationと、region、permission、lifetime、failureなどhost固有のpolicyを決める
operationは`extern` contractに置く。一方、既に渡されたcapabilityを使い、言語がcanonical representationを
定めた値を固定規則でadmitまたはobserveするoperationはlanguage primitiveに置く。

| 問い | 置き場所 |
|---|---|
| resource policyまたはhost固有の意味を決めるか | program固有の`extern` contract |
| canonical representationとの固定された変換か | language primitive |
| program固有のaggregateまたはprotocol encodingか | malで書くcodec |
| hostにしか検査、構成、実行できないencodingか | 型固有の`extern` contract |

この規則では、allocationとdeallocationは`extern`、`Ptr`のbyte offsetとscalar、pointer、Symbol bytesの
load/storeはprimitiveになる。`Ptr`はExtern-owned storageへの組み込みcapabilityであり、referentをmal-ownedに
変えない。primitiveはstorage policyを決めず、`Ptr`を提供したcontractが定めるregion、permission、lifetimeを
引き継ぐ。

productとsumにはcanonical memory representationを与えない。したがって型だけから`loadTree`や`storeTree`は
導けない。external storage上のtag、field offset、pointer graph、invalid representationをprogramが定める場合は、
mal関数がscalarと`Ptr`のprimitiveを組み合わせてcodecを実装する。host library固有のrepresentationやatomicityが
必要な場合だけ、同名のoperationを個別の`extern` contractに置く。

この分担はcontrolを失わず、operationごとにwidth、alignment、admissionを再定義するcontractの増殖を避ける。
同時に、bounds、allocation、lifetimeを追跡するmemory systemを暗黙に言語へ追加しない。

external function valueの参照と受け渡しはmal-controlledなEngramの操作であり、それだけでは境界を越えない。
そのfunction valueのapplicationがhost operationを実行するときに限り、parameterとresultの各leafへadmission、observation、
capability transferを適用する。source上の専用call markerではなく、宣言されたfunction identityがcontractを選ぶ。

## mechanismを導入する条件

reference counting、tracing GC、region、borrow checking、finalizerはauthorityそのものではなく実装または検査の
mechanismである。次の場合にだけ言語機能の候補とする。

1. 現在のauthorityを保ったままでは必要な意味を表現できない。
2. sourceから新しい制約とcostを追跡できる。
3. Engramの回収とExtern resourceの破棄を混同しない。
4. host contractを隠すだけでなく、system全体の調査面積を減らす。

観測不能なstorageの回収方式だけを変える場合は、source semanticsを増やさずimplementationで扱う。external resourceの
破棄が必要な場合は、暗黙のfinalizerより明示的なhost contractを基本とする。

## 新しい境界機能への問い

新しいtype、ABI、memory primitive、callbackを提案するときは、少なくとも次を答える。

1. valueまたはreferentのauthorityはEngramとExternのどちらにあるか。
2. crossingはadmission、observation、capability transferのどれか。
3. aggregateに含まれる各fieldへ分類を再帰適用できるか。
4. crossing後に誰が何を保持でき、何がlifetimeを延長するか。
5. invalid representationとfailureを誰が検査するか。
6. trusted adapterだけで十分か、runtimeによる強制が必要か。
7. 固定mechanismを再利用せず、新しい独立contractを増やす理由があるか。

この問いに短く答えられない機能は、surfaceだけを追加せず境界modelから再検討する。採択に至った経緯は
[D031](../history/decisions/D031.md)に記録する。
