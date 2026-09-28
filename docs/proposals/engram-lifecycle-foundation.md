# Engram lifecycle loweringの共通基盤

Status: Partially accepted; backend-local lifecycle boundary implemented

この文書は、`Symbol`、`Buffer`、function closureとmanaged aggregateの値の保持と破棄を、Engram一般のlifecycle loweringとして
共通化する方針と後続候補を管理する。backend-localな共通化は採択済みであり、source-levelの型、operation、ownership syntaxは追加しない。
現在のauthorityは[EngramとExtern](../spec/engrams.md)、responsibilityの意味は
[D055](../history/decisions/active/D055.md)、実装policyは
[managed valueのownership](../implementation/ownership.md)を正とする。

## 問題

source programからretain、release、初期化状態は観測できないが、compilerとruntimeは既に次の処理を持つ。

- `execution/ownership`はmanaged responsibilityを`Fresh`、`Borrow`、`Share`、`Consume`、`Drop`として計画する。
- LLVM backendは`Symbol`、`Buffer`、function、product、sumごとにretainとreleaseを再帰的に出力する。
- managed `Buffer`の`get`と`put`はbackendで要素を保持、破棄し、`new`、`fill`、`copy`とBuffer破棄は
  program固有callbackを通じてruntimeで同じ処理を行う。
- closure environmentはtarget固有destructorを持ち、captureのreleaseとenvironment storageの解放を行う。

現在のLLVM backendは通常値、managed Buffer element callback、closure environment destructorから同じ
`retain_value`と`release_value`の型再帰をすでに使う。残る重複は、値の保持と破棄よりも、storage上の有効な値、
raw allocationのlifetime、slotやframeへのcopy、move、上書き、破棄を行うcarrier操作の境界にある。新しい
mal-owned Engram leafを追加したときに、共通の型再帰を迂回するdirect storeやreleaseを追加せずに済む境界が必要になる。

## 提案する境界

既存のownership planをEngram lifecycleのpolicy authorityとして維持し、backendに型別のlifecycle loweringを一つ置く。
概念上、concrete type `T`について次を出力できるものとする。

```text
share<T>(value)       既存responsibilityを保ったまま独立したresponsibilityを作る
drop<T>(value)        一つのresponsibilityを終了する
initialize<T>(place, value)
                      vacantなcarrierへowned valueを格納する
replace<T>(place, value)
                      liveな旧valueを終了してownedな新valueを格納する
vacate<T>(place)      ConsumeまたはDrop後のcarrierをlive valueとして扱えなくする
```

`share`と`drop`はsource operationではなく、ownership planの`Share`と`Drop`を物理表現へ変換する型別loweringである。
`Consume`はresponsibilityを複製も終了もせずdestinationへ移し、addressableなsourceを`vacate`する。`Borrow`はlifecycleを
変更しない。`initialize`、`replace`、`vacate`は値ではなくcarrierの状態遷移を表し、通常のmal valueへ未初期化状態を追加しない。

型別loweringは現在の分類を保つ。

- unmanaged scalar、`Unit`、`Address`、external opaque valueの`share`と`drop`は何もしない。
- `Symbol`はbyte owner、`Buffer`はbuffer object、functionはenvironment ownerを保持または解放する。
- productはmanaged fieldをsource orderで処理し、sumはactive payloadだけを処理する。
- static ownerとnull ownerに対する処理は安全なno-opのままにする。

この境界はreference countingをEngramのsource semanticsにしない。別の回収mechanismでもownership planが要求するresponsibilityを
保持できる限り、型別loweringとruntime representationを置き換えてよい。

## storage invariant

addressableなmanaged carrierはbackend lowering上で`Vacant`または`Live<T>`の一方として扱う。次を共通invariantとする。

1. `initialize`は`Vacant`だけをdestinationにし、旧valueをdropしない。
2. `replace`は`Live<T>`だけをdestinationにし、新しいresponsibilityを成立させてから旧responsibilityを終了する。
3. `Consume`はsource responsibilityをdropせずに移し、source carrierを`Vacant`にする。
4. `Drop`はlive responsibilityを一度終了し、addressableなsource carrierを`Vacant`にする。
5. relocationはvalueの物理addressだけを変え、`share`または`drop`を発生させない。
6. typed payloadを所有するallocationは、全live payloadをdropした後にだけ解放する。

この状態を新しい汎用IRとして全stageへ直ちに追加しない。まずLLVM backendの共通helperとdebug validationで表し、実在する
carrierの誤った二重初期化、二重破棄、Consume後のreleaseを検出できる範囲から導入する。

初期実装ではlifecycleを独立したplan dataにしない。`execution/ownership`が既にtarget layout非依存のpolicyを保持しているため、
LLVM backend内の型再帰を一つのemitter APIへ集約する。新しいbackendまたは同じ型のglueを複数形式で生成する必要が実在した時点で、
そのAPIから共有できるplanを抽出する。carrierの`Vacant`と`Live`も新しいIR stateにせず、ownership planのvalidatorと
`initialize`、`replace`、`vacate`の呼出境界で検査する。

## runtime glue

型を知らないruntimeが要素を保持する場合、backendは共通lifecycle loweringからC-callableな`share<T>`と`drop<T>`のglueを
materializeする。managed Buffer element callbackはこのglueの利用者と位置づけ、Buffer専用の型再帰を持たせない。
closure environment destructorもcapture typeの同じ`drop` loweringを使う。

共通owner header、runtime type descriptor、dynamic dispatchはこの提案の必須条件にしない。現在のbyte ownerとenvironment ownerは
異なるrepresentationを維持できる。既存のallocation headerを統合すると実装と生成物が小さくなることを確認した場合だけ、
独立した変更として判断する。

Buffer element callbackとenvironment destructorのC signatureも統合しない。前者は一要素のshareまたはdrop、後者はowner payload全体の
破棄という異なるruntime contractを持つ。backend内で同じ型再帰を利用し、それぞれに必要な薄いwrapperを生成する。

## 新しいEngram leafに必要な定義

内部基盤の成立後も、新しいEngramは型名とdestructorだけでは追加しない。少なくとも次を局所的に定義する。

- valid valueを構成するtrusted operationとrepresentation invariant
- runtime value layoutとownerを取り出す方法
- `share`、`drop`、moveまたはrelocationの処理
- immutable valueかshared mutable identityか
- 保持する子Engramと、reference-count cycleを作らない根拠
- `Storable`、`Representable`、`HostMappable`の各property
- Externとの境界がある場合のadmission、observation、capability transfer

drop時の処理は子Engramとmal-owned storageの回収に限り、I/OやExtern resourceの`close`のような観測可能な作用を持たせない。
回収時点を観測可能にするとreference countingがsource semanticsになり、Engram仕様が許す別の回収方式を失うためである。

## 導入順

1. 現在のmanaged type分類、LLVMのretain/release、Buffer callback、environment destructorの対応をfocused testで固定する。
2. 既存の`retain_value`と`release_value`を型別の`share`と`drop`のauthorityとして明確化し、それを迂回する経路を除く。
3. slot初期化、上書き、Consume、Dropを`initialize`、`replace`、`vacate`の共通helperへ接続する。
4. 同じ型別loweringを使う既存のmanaged Buffer callbackで、`get`、`put`、`fill`、`copy`、破棄のresponsibilityを検証する。
5. 同じ型別loweringを使う既存のclosure environment destructorを保ち、control frame payloadのdirectな保持と破棄を調べる。
6. 重複が実際に残る場合だけ、owner headerまたはcallback ABIの統合を別途評価する。

各段階は生成物を実行できる状態で完了させる。将来のEngramだけを想定した空module、runtime registry、dynamic descriptorは作らない。

## Symbolを基準にしたextension境界

`Symbol`は新しいEngram leafに必要な構築、immutable value semantics、owner projection、share、drop、static/null ownerの例をすべて持つ。
ただし初期整理では別crateへ移さない。Symbolの規則はfrontend、ownership plan、LLVM value representation、runtime byte ownerへ跨っており、
先にcrate境界を置くと現在のcompiler内部APIをそのままextension contractとして固定するためである。

共通lifecycle loweringへ既存のSymbolを載せた後、次をすべて満たす場合にだけRust crateへの分離を独立して提案する。

- compiler本体がSymbol固有のowner field、retain、releaseを分岐せず、登録されたleaf lifecycleだけを使う。
- Symbol operation loweringが汎用compiler stateではなく、明示したvalue、target layout、artifact dependencyだけを入力にする。
- runtime C sourceとdeclarationの選択をcrateまたは静的なextension manifestから決定できる。
- crateを分離してもparser、checker、editor、public diagnosticsのlanguage authorityが分散しない。
- compilerとextensionのversion、artifact cache、reproducible buildのcontractを固定できる。

この条件はdynamic plugin ABIやthird-party extensionを約束しない。最初の候補は、compilerと同じworkspace、version、trusted boundaryで
静的にlinkするcrateとする。Symbolの分離が単にfile ownershipを移し、独立contractを減らさない場合は組み込みmoduleのままにする。

## 検証境界

focused testは少なくとも次を固定する。

- unmanaged scalar、`Symbol`、`Buffer`、function、managed product、managed sumの`Share`、`Consume`、`Drop`
- vacant slotへの初期化とlive slotの置換で、旧ownerのrelease回数が異なること
- duplicate field、self assignment、active sum payload、wildcard discardでresponsibilityが過不足なく終了すること
- Bufferのgrowthがelement owner数を変えず、`get`、`put`、overlapping `copy`、`fill`、破棄が正しいこと
- 空`Symbol`、static literal、null closure environmentへのshareとdropがallocationまたは不正なreleaseを生まないこと
- closure capture、control frame、tail handoff、returnを跨ぐ既存ownership testが同じ結果を保つこと

LLVM cross-boundary testは生成moduleをClangでcompile、link、executeし、allocation counterを使うtestでは終了時のlive allocationを
残さないことを確認する。generated IRの形だけでlifecycleの正しさを判定しない。

## 非目標

- `Pool<T>`、manual `init`、manual `drop`、`free`をsource languageへ追加しない。
- borrow checker、linear type、unique ownership、user-defined destructorを追加しない。
- `Buffer`の共有identity、auto-growth、`Symbol` snapshot、host copy semanticsを変更しない。
- `Storable`制限を緩和して任意のmanaged graphやcycleを許可しない。
- Engramの回収方式をreference countingとして仕様化しない。

## 実装で確認する事項

LLVM backendでは、型別retain/release、slotのload・initialize・vacate、managed placeのreplaceを`body/value`の共通helperへ集約済みである。
pattern destination、parameter handoff、frame resume、Buffer `put`、Symbol move、dead-slot cleanupはこの境界を利用し、
`execution/ownership`が引き続きresponsibility policyを所有する。独立したlifecycle plan、runtime descriptor、共通owner header、
source operationは導入していない。

残る検証と拡張候補は次のとおりである。

- 共通emitter APIが通常値、Buffer callback、environment destructorの型再帰を実際に一箇所へ閉じるか。
- operation-specific helperだけでcarrier stateの誤りを検出できるか、追加のbackend-local validationが必要か。
- byte ownerとenvironment ownerを別representationに保ったまま、Symbol固有分岐をleaf lifecycle境界へ閉じられるか。
- Symbolのcrate分離条件を満たす境界が得られるか、組み込みmoduleの方がsystem全体を小さく保つか。

これらの確認が終わるまでは、source仕様とpublic ABIを変更せず、現在の型固有runtime entryを削除しない。
