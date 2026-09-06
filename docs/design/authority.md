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

この問いに短く答えられない機能は、surfaceだけを追加せず境界modelから再検討する。採択に至った経緯は
[D031](decisions.md#d031-engramをmal内部のlifetime-authorityとする)に記録する。

