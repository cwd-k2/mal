# `extern`をruntime extension境界にする提案

Status: Historical design record; implemented in v0.7

この文書は、`extern`をmanaged valueから隔離したpublic C value境界ではなく、同じcompiler/runtime revisionへ結合する
runtime extension境界として再定義した変更の背景と移行範囲を記録する。現在の規範は[`extern`](../../spec/extern.md)、
[C host ABI](../../spec/c-host-abi.md)、[`Buffer`](../../spec/memory.md)、[EngramとExtern](../../spec/engrams.md)を正とし、
採択判断は[D098](../decisions/active/D098.md)を正とする。

## 目的

現在の`HostMappable`境界は`Symbol`、`Buffer<A>`、managed memberを持つaggregateをCから隠し、可変長dataを
`Address`と長さで外部storageへ写す。この分離はhostからEngramのrepresentationとlifecycleを守る一方、hostを同じprocessへ
静的linkする現行buildでもcopy、allocation handle、operation固有のAddress contractを要求する。

malはhost implementationの誤りからmemory safetyを保証しない。そこで、C implementationをmal runtimeの一部として扱い、
runtimeがallocationとmanaged responsibilityを統括したまま、Cがmal valueを直接構築、観測、変更、保持できる境界へ置き換える。
これによりI/O、system call、codec、native collection、Pool試作をmanaged valueへ直接接続し、外部memoryだけのためにある
source-level mechanismを削除する。

## 提案するsource contract

`extern`のspellingは変えない。別の`unsafe` declaration、safe/unsafe二系統のextern、opt-in annotationは追加しない。

```mal
extern readLine :: Unit -> Buffer<UInt8>;
extern write :: Buffer<UInt8> -> Unit;
extern sort :: Buffer<UInt64> -> Unit;
extern tokenize :: Symbol -> Buffer<Symbol>;
```

external operationは通常のfunction value、application順序、effectの規則を維持する。違いはparameterとresultをpublic host
representationへadmit/observeするのではなく、specialization後のruntime carrierとしてC implementationへ渡すことである。

`HostMappable` judgmentは廃止する。compilerは実装可能性だけを表す内部判定を持ち、閉じたconcrete typeのscalar、external
opaque type、`Symbol`、`Buffer<A>`、file-local opaque type、product、sumを受理する。aliasとfile-local opaque typeはbackendへ
渡るrepresentationまで正規化する。`Buffer<A>`自身の`Storable(A)`条件は変えない。

generic extern declarationは引き続き認めない。open type parameterを一つのC implementationへ渡すにはruntime type descriptorか
specialization別のC definitionが必要であり、本変更だけではどちらも導入しない。functionを直接または再帰的に含む型も認めない。
Cからmal closureをapplicationして元のC activationへ戻るには、contextに存在しないprogram固有のexecution controlとcallback
trampolineが必要だからである。これらは安全性のための制限ではなく、未定義の実行mechanismを受理しないための制限である。

## Runtime extension ABI

toolchainはprogram非依存の`mal.h`を提供し、generated file headerはそれをincludeする。別のsafe ABIを持たないため、共通headerも
safe/unsafeに分割しない。`mal.h`は共通carrier、call context、allocation、lifecycle operationを宣言する。generated file
headerはprogram固有のaggregate、normalized opaque representation、extern signatureを宣言する。

このABIはSymbol viewとprogram固有carrierのfield、Buffer element storageを公開する。Buffer object、byte owner、managed owner headerの
物理layoutはruntime内部に留め、`mal.h`はretain、release、allocation、data access、reserve、growthをCから使えるoperationとして公開する。
helperは安全性を強制するfacadeではなく、正しいruntime invariantとlifecycleを実装するcanonical operationである。Cが公開pointerを
castして内部表現を破壊することも防がず、その後の挙動を保証しない。

generated headerはruntime carrierのC record、field、sum tagとpayloadを同じtarget ABIから出し、pointer/index幅とSymbol layoutを
`_Static_assert`でC compilerのlayoutと照合する。C implementationはfieldを直接参照でき、`mal.h`はBuffer data access、managed
carrier leafのshare/drop、result moveの小さなhelperを提供する。productとsumのlifecycle再帰はgenerated headerが型別に構成するが、
host implementationがhelperを迂回することも妨げない。

host bodyは現在と同じく`mal_call_t *`とsource-level parameter一個のruntime carrierを受け、productをflattenしない。parameter
carrier自体はCのby-value copyであり、managed leafはcallerが保持するidentityへのborrowである。resultはCのdirect returnで
runtime carrierとmanaged responsibilityをmoveする。この形によりscalarとaggregateのvalue semantics、Bufferの共有mutation、
Symbolのimmutable viewを同じsignature ruleから導く。

runtime extension ABIはcompiler/runtimeとのexact matchだけを保証する。layout、symbol、source compatibility、binary
compatibilityをversion間で保証せず、生成artifactとhost sourceはcompiler更新後に再compileする。長期安定する別のC ABI層を
設けない。version mismatchはcompile時またはlink時に拒否する。

## Responsibility contract

managed parameterはhost body完了までcallerが保持するborrowである。productとsumではactiveなmanaged leafへ再帰的に同じ規則を
適用する。CはborrowしたBufferの共有identityを変更できるが、borrow responsibility自体をreleaseしてはならない。

Cがparameterまたはそこから得たmanaged valueをcall後も保持する場合、型別`share`で独立したresponsibilityを作り、同じruntimeと
threadのcontractに従って後に`drop`する。C storageへcarrierだけをcopyしてもlifetimeは延長しない。

managed resultはhostが一つのowned responsibilityをterminal returnへmoveする。return後に同じresponsibilityを使用またはdropしては
ならない。aggregate resultはmanaged leafごとにこのhandoffを行う。Cはruntime allocationと型別initialize、replace、share、dropを
利用できるが、raw field writeで同じ状態を構成することも妨げない。

invalid tag、破損したowner、二重move、borrowのrelease、managed elementのlifecycleを無視した上書き、期限後のinterior pointer利用、
別threadからのruntime accessなど、extension contract違反後の挙動は保証しない。境界でrepresentation validation、rollback、
resource cleanupを追加しない。`mal_call_t`とcontextは同期call中のruntime accessに使え、program固有のcontinuationや現在のcontrol
stateを表さない。

## Mutationとcompiler effect

extern callは、引数から到達できるBuffer、Cが以前shareして保持したmanaged value、Cの外部stateを観測または変更し得る。
compilerはextern callをeffectfulな未知のmemory clobberとして扱い、callを越えてmanaged storageの内容、owner metadata、nested
identityの状態が不変だと仮定しない。C sourceとgenerated moduleをLTOする場合も、language-levelのeffect順序をC optimizerの
偶発的な解析結果へ委ねない。

`Symbol`のsource semanticsはimmutableのままである。Cが完成済みSymbol storageを書き換えるなどruntime invariantを破った場合、
そのprogramはextension contract外となる。Cが破壊できることと、破壊後の値へ新しいsource semanticsを与えることは区別する。

## `Address`とcanonical host memoryの廃止

source-level `Address`、`from<A>`、`Buffer<A>.into`を削除する。これらだけが要求した`Representable` judgment、canonical memory
layout、generated canonical read/write helperも削除する。`ByteSize`と`USize`はbyte量とcollection座標を区別する型として維持する。
`Storable`は維持する。Buffer elementはすべてspecialization後のruntime carrier layoutで保存し、LLVMとgenerated C headerが同じ
target layoutからそのsize、alignment、fieldを得る。現在canonical layoutを再利用するunmanaged elementもこのlayoutへ移し、host
copy専用representationを残さない。

C libraryや同期system callへpointerを渡す場合、C implementationがcall中に`Symbol`または`Buffer`からdata pointerを取得する。
pathのNUL終端、native structure、`iovec`などはC activation中の一時representationとして構成し、mal valueにraw pointerを残さない。

call後にも外部memoryが存在する場合は、resource固有のexternal opaque type、またはruntime-managed native objectへpointer、extent、
destructorを閉じ込める。`mmap`、registered I/O buffer、futex storageなどはそれぞれmapping lifetime、pin、completion、atomicityを持ち、
裸のpointerと長さだけではcontractを表せない。標準添付libraryは型付きextern operationとしてこれらを提供する。

任意のsystem call numberとmachine word列をmal sourceから組み立てるraw `syscall` primitiveは提供しない。それを提供してBufferの
interior pointerを引数にできるようにすれば、名前にかかわらず`Address`を再導入することになる。`mal.h`を使うC
implementationはraw syscallを自由に呼べる。

## External opaque typeとnative managed type

external opaque typeはhost resourceのnominalなcarrierとして維持する。初回変更では現在と同じcopyableなone-word carrierとし、
resourceのcloseやreleaseはoperation contractに置く。万能な`Pointer` opaque typeと汎用dereference operationは追加しない。

extension定義の新しいmanaged source typeは本変更の必須範囲に含めない。まず既存の`Symbol`、`Buffer`、aggregateとruntime内のPool
kernelをexternから利用できることを検証する。新しいmanaged leafを導入する場合は、source identity、runtime layout、share/drop、
Storable、cycle、destructor effectを[lifecycle loweringの拡張境界](../../proposals/engram-lifecycle-foundation.md)に従って別途定める。

## Authority model

extern CはEngramから隔離されたExtern observerではなく、mal implementationへ参加するruntime extensionである。Engramのsource-level
identityと意味はmalが定め、allocationとresponsibilityはruntimeが統括するが、CはvalidなEngramを構築し、既存Engramを観測、変更、
破壊できる。host contractを満たすprogramにだけmalのsource semanticsを適用する。

filesystem、socket、mappingなどの外部resource authorityは引き続きoperation固有contractに属する。Engram/Externという意味上の
区別は維持するが、その区別をC ABIへ出せる型の制限には使わない。admission、observation、capability transferをpublic ABIの
mechanism分類として要求しない。

## Build、header、配布

`.c` requirement、静的なlink input、同じtarget ABIとC11 compilerを使うbuild modelは維持する。runtime extensionをdynamic load、
unload、別compiler revisionと共有するplugin ABIは提供しない。保存したgenerated headerはeditor supportと生成例には利用できるが、
buildは今回生成したheaderをauthorityとする。

host stub生成はmanaged parameter/resultを含むsignatureと、正しいshare、drop、return moveの例を出す。stubを再生成して編集済みsourceを
上書きしない現行規則は維持する。

## 移行範囲

採択時は次を一つのlanguage/ABI変更として更新する。

- `extern`、Engram/Extern、Buffer、generic、型、grammar、program構造、scope、C ABIの規範
- authority、minimality、compiler responsibility、ownership、execution backendの現行policy
- checkerの`HostMappable`と`Representable`、predefined `Address`、`from`、`into`
- canonical layoutを使うBuffer loweringのruntime carrier layoutへの移行と、C memory helper generatorの削除
- generated header、LLVM/C bridge、runtime headerとfeature selection
- compiler usage、C interface例、conformance matrix、optimization corpus
- 全example、formatter/parser fixture、LSP documentation、Tree-sitter corpus、editor snippets
- Addressとcanonical memoryを前提にするactive decisionのsupersession、新しい採択decision
- Pool proposalとlifecycle proposalに残る旧境界の参照

過去の判断はhistoryから削除せず、新しいdecisionから置換範囲を明示する。active documentationからは旧ABIの説明、移行途中の
互換性記述、obsoleteなexampleを残さない。

## 実装と検証の順序

1. 本proposalをreviewし、採否、ABI responsibility、削除範囲を固定する。
2. 採択decisionを記録し、`spec/`とdesign authorityを新しいcurrent truthへ全面更新する。
3. frontend property、checked interface、specialization後のextern surfaceを更新する。
4. runtime extension header、program固有carrier/lifecycle glue、LLVM/C bridgeを実装する。
5. Address、canonical memory、不要なruntime sourceとgeneratorを削除する。
6. focused testとClang compile/link/execute fixtureを新contractへ移す。
7. example、compiler usage、conformance、editor、全`.mal` corpusを整理する。
8. Pool、syscall、managed bytesのscratch programで拡張性と生成物を検証する。
9. repository全体のcheck、Address/HostMappable/Representable残存参照の監査、性能比較を行う。

検証はscalar、Symbol、plain Buffer、managed-element Buffer、nested aggregate、sumのparameter borrow、result move、host share/drop、
Buffer mutation、growth後のpointer再取得、invalid lifecycleによる非保証範囲を分ける。baselineとproductionで同じ有効programの結果と
effect順序を確認し、Address削除後のread/write、file input、process argument、resource error、Pool kernelを代表的なend-to-end
caseにする。

## 非目標

- host implementationのmemory safety、resource safety、representation validation
- ABI version間のsource/binary compatibility
- generic extern、runtime type reflection、type descriptor registry
- Cからmal functionへのcallback、reentry、continuation suspension
- cross-threadなmanaged value共有、async task runtime
- dynamic plugin discovery、hot reload、extension unload
- arbitrary external memoryをmal sourceからdereferenceするpointer型

## 採択時に固定した事項

初回変更はparameter borrow、host share/drop、result moveを必須にする。これらを欠くとmanaged resultの構築、hostによる明示的な保持、
保持終了を同じresponsibility modelで説明できない。

`mal.h`とgenerated headerはruntime structのfieldを公開する。`MalContext`もexact-match ABIの一部に含めるが、contextに存在しない
program固有execution stateやcontinuationを合成しない。内部を公開することは未定義のcallback/reentry mechanismを追加しない。

external opaque typeはone-word trivial carrierのまま維持する。Address、Representable、canonical memoryは移行用の二重境界を残さず
同じreleaseで完全削除する。Pool scratchは既存kernelの直接利用までをこの変更の検証に含め、新しいmanaged source typeの一般化は
実際に必要なsource identityとlifecycleが明らかになった後続判断にする。
