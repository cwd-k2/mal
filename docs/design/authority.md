# EngramとExternのauthority

Status: Current design policy

この文書はmemory、resource、extern境界を設計するときの判断軸を定める。規範的な型とoperationは
[EngramとExtern](../spec/engrams.md)を正とし、個別のABIやsyntaxをここへ重複させない。

## Ownershipより先にauthorityを問う

値やresourceに関する問題では、最初に「誰が所有するか」ではなく次の決定権がどこにあるかを問う。

- 何が有効な値か
- identityを誰が構成できるか
- いつまで有効か
- 誰が観測、変更、破棄できるか
- failureをどちらの規則で扱うか

ownership、borrow、copy、lifetime、permissionは、このauthorityを具体的なoperationへ落とした関係である。一つのmemory management
mechanismを言語全体の答えとして先に選ばない。

## Source semanticsとruntime extensionを分ける

Engramはmal sourceで意味が成立する値、Externは外部stateとresourceが成立する領域である。mal compilerとruntimeはEngramの
identity、共有、immutability、managed responsibilityを定める。external opaque valueをEngramへ包んでもreferentのauthorityは
Externに残り、carrier copyはresourceをcloneしない。

extern Cはこの意味上の分類の外側に隔離されたobserverではなく、mal implementationへ参加するruntime extensionである。generated
headerが公開するcarrierとlifecycle operationを使い、validなEngramを構築、観測、変更できる。Cがruntime invariantを破れることは、
invalid valueへ新しいsource semanticsを与えることを意味しない。contract違反後のbehaviorを保証しないことで、runtimeをhostから
防御する別のmemory modelを持たない。

## Lifecycleの分担

mal compilerはmanaged responsibilityの発生、share、move、終了を計画し、runtimeがallocationと最後のdropを実行する。source programは
manual retain、release、freeを持たない。extern parameterはcall中のborrow、managed resultはhostからmalへのowned moveである。

Cがmanaged valueをcall後も保持する場合はruntime shareで独立したresponsibilityを作り、後にdropする。carrier bitsだけのcopyは
lifetimeを延長しない。external resourceのclose、unmap、socket shutdownなどはexternal opaque carrierのcopy/dropへ暗黙に結合せず、
operation固有contractに残す。native resourceをmanaged dropへ結合するなら、新しいEngram leafとしてdestructor effectまで定める。

## Policyとmechanismを分ける

filesystem、network、process、device、system call、encoding、partial transferなどhost固有policyは型付きextern operationに置く。
Bufferのshared identity、Symbol snapshot、managed element lifecycleは言語とruntimeの共通mechanismに置く。C libraryやkernelへpointerを
渡すことはextern implementation detailであり、source-level pointerや汎用memory primitiveを要求しない。

| 問い | 置き場所 |
|---|---|
| runtime valueのlayout、share、dropか | compiler、generated header、runtime |
| resource固有の取得、解放、failure、permissionか | extern contract |
| program固有のaggregateまたはprotocol encodingか | malまたはCで書くcodec |
| 同期native APIがpointerを要求するか | C body内の一時borrow |
| call後もkernelやlibraryがstorageを保持するか | pinnedなresource固有typeとcompletion contract |

runtime carrier layoutは同じartifactのLLVMとC extensionが共有するが、file、network、永続storageのformatではない。protocol encodingと
runtime ABIを同一視しない。

## Controlをrepresentationから推測しない

function valueの参照と受け渡しはmal-controlledなEngramの操作である。extern applicationはmalからCへ同期的に入り、terminal resultか
trapで完了する。Cがclosure representationを知っても、現在のcontinuation、callback後のresume、suspended C activationをcontextから
構成できない。generic carrier layoutを公開しても、specialization後に消えたopen type parameterのruntime descriptorにはならない。

したがってgeneric externとfunction callbackは、安全性を理由に閉じるのではなく、必要なexecution mechanismを独立して設計するまで
拒否する。representation公開を未実装のcontrol authorityへ読み替えない。

## Mechanismを導入する条件

reference counting、tracing GC、region、borrow checking、finalizer、runtime type descriptorはauthorityそのものではない。次の場合にだけ
言語機能またはruntime mechanismの候補とする。

1. 現在のauthorityを保ったままでは必要な意味を表現できない。
2. sourceまたはextern contractから新しい制約とcostを追跡できる。
3. Engram回収とExtern resource破棄を混同しない。
4. 既存のconcrete specializationとgenerated lifecycle glueで表せない実例がある。

観測不能なstorage回収方式だけを変える場合はsource semanticsを増やさずimplementationで扱う。external resourceの破棄が必要な場合は
明示的なextern operationを基本とし、managed native typeはdestructorの観測可能性を別に判断する。

## 新しい境界機能への問い

新しいtype、extern ABI、callback、native collectionを提案するときは少なくとも次を答える。

1. source valueとexternal referentのauthorityはどこにあるか。
2. parameter borrow、host share、result moveのどれが必要か。
3. aggregateのmanaged leafへlifecycleを再帰適用できるか。
4. Cが保持できるものとdrop pointはどこか。
5. invalid representation後を非保証にするか、runtime検査が必要か。
6. 同期callだけで足りるか、callback、pin、completion、thread transferが必要か。
7. concrete generated glueを再利用せずdescriptorやregistryを増やす理由があるか。

この問いに短く答えられない機能はsurfaceだけを追加せず、value、lifecycle、controlのmodelから再検討する。採択理由は
[D098](../history/decisions/active/D098.md)を正とする。
