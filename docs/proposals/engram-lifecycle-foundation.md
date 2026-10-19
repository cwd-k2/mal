# Engram lifecycle loweringの拡張境界

Status: Exploratory

この文書は、現在のbackend-localなmanaged value lifecycleを、新しいEngram leafや複数のglue形式へ拡張する場合の
採択条件を定める。現在のlifecycle ruleと実装境界は[managed valueのownership](../implementation/ownership.md)、
source semanticsは[EngramとExtern](../spec/engrams.md)を正とする。

## 解く問題

新しいmal-owned Engram leafを追加すると、通常値、runtime callback、closure environment destructorなど複数の利用箇所で
同じ型別の保持と破棄が必要になり得る。各利用者がowner field、retain、releaseを直接扱うと、ownership planが定めた
responsibilityをbackendやruntimeが別々に再解釈することになる。

拡張が必要になった場合も、`execution/ownership`をpolicy authorityとして保ち、backendはそのdecisionを物理表現へ変換する。
source languageへmanual `retain`、`release`、`drop`、未初期化valueを追加しない。

## 整理に使う概念境界

backend全体を一つのstore machineへ置き換えず、現在のstage境界を保ったまま次を区別する。

| 概念 | malでの意味 | authority |
|---|---|---|
| semantic value | `ValueId`で追跡するMalの値 | control program |
| carrier | 値をLLVM、C、runtime境界で運ぶ表現 | target layout lowering |
| place | slot、Buffer element、capture field、frame fieldなどcarrierを保持する位置 | 各storage owner |
| Engram identity | Buffer objectやclosure environmentなど共有観測される対象 | source semanticsとtrusted constructor |
| allocation | owner、payload、control storageを収容する回収単位 | runtime mechanism |
| responsibility | managed valueを生存させ、最後に終了する義務 | `execution/ownership` |
| authority | borrowが有効な期間を包含するresponsibility | ownership lender解析 |
| external referent | opaque carrierの外側にあるresource | extern contract |

同じpointer representationを使っていても、carrier、Engram identity、allocation、external referentを同一視しない。時間は独立した
event logではなく、確定済みcontrol programのbinding site、use site、control edge、parameter entryで表す。将来、source-level
referenceや非同期completionを導入してこのcontrol順序だけでは権限期間を表せなくなった場合に限り、pathやloanの独立identityを追加する。

`Borrow`は現在、独立したresponsibilityを作らずlenderの生存中にcarrierを使うことを意味する。排他的access、参照値の構築、
source-level loanを意味しない。これらを将来導入する場合は同じ語へ暗黙に追加せず、access permissionをresponsibilityから分離する。

## 現在の拡張圧力

### lifecycleとrelocation

現在の`Lifecycle = Trivial | Managed`は、型にshareとdropが必要かを十分に表す。一方、slot、Buffer growth、frame、C by-value carrierは、
すべてのadmit済みcarrierがbitwiseにrelocateできることを暗黙の前提にしている。新しいEngram leafがcarrier内部address、pin、
非同期native accessを持つ場合、`Managed`だけではadmissionを決められない。

必要になった時点で、少なくとも`share/drop`の有無と`bitwise relocatable`を別のpropertyとして扱う。現在の型がすべて同じ値を持つ間は、
値を返すだけのregistryや汎用descriptorを先行して追加しない。`Storable`、frame field、extern by-value carrierはそれぞれの境界で
このpropertyを要求する。

### planとloweringの証拠

ownership planはtypedな`StateId`、binding site、`UseId`、`EdgeId`を使い、backendはplanを再推論しない。このidentityは
別のordinalと取り違えられない型を保つ。新しいsynthetic useやdrop pointを追加するときも裸のtupleや共用integerへ戻さない。

LLVM loweringの`EmittedValue`は、source type、LLVM representation、`ValueResponsibility = None | Borrowed | Owned`を分けて持つ。
`None`はunmanaged type、`Borrowed`はplaceが保持するresponsibilityの観測、`Owned`はhandoffまたはreleaseが必要なcarrierを表す。
slot初期化とparameter handoffはtypeと状態の不一致を拒否し、shareだけが`Borrowed`を`Owned`へ変える。この状態はauthorityやlenderを
表さず、それらの決定は引き続きexecution ownership planに属する。

現在のproduct carrierはleafごとの状態を保持せず、managed leafがすべて`Borrowed`またはすべて`Owned`の場合だけaggregateを構成する。
異なる状態を混在させる構成は、aggregate全体を`Owned`としてborrowed leafのresponsibilityまで暗黙に得たことにせず拒否する。
leafごとに異なるhandoffを通常値として保持する実需が生じた場合は、`ValueResponsibility`を構造化し、share、consume、dropを
その構造に対する型付き変換に限定する。

### 保存則の検証

exact validatorはauthorityからplanを再構成して集合の欠落と余分を検査する。独立した保存則validatorは、各stateのlive ownerから
binding、frame、terminator、case payload、control edgeのeffectをpathごとに適用し、同じplanning algorithmを複製せず次を検査する。

- 一つのlineageは`Fresh`または`Share`で始まり、pathごとに一度だけowned handoffまたは`Drop`で終わる。
- `Borrow`はresponsibilityを増減させず、そのuseを包含するlenderを持つ。
- `Consume`したsourceを同じpathで再利用または`Drop`しない。
- `Initialize`はvacant place、`Replace`はlive placeだけを対象にする。
- allocationは保持する全payloadのresponsibilityを終了した後にだけ解放する。

具体的実行traceはdiagnosticやtestの説明には使えるが、一つのtraceを全pathのvalidationの代わりにしない。

## ABIの分離

ABIは一つの層としてまとめず、少なくとも次を分ける。

| 境界 | 安定させるもの | 公開しないもの |
|---|---|---|
| public C extension ABI | carrier、extern signature、host lifecycle、Call API | LLVM calling convention、frame、owner実装 |
| generated C / LLVM bridge | context、argument storage、result storageの内部call | hostが直接呼ぶsurface |
| runtime mechanism ABI | allocation、owner、Buffer、control storageの生成物内接続 | source valueやextern resourceの意味 |

ABI 0x000c00では`mal_call_t`をopaqueなcapabilityとし、具体的な`MalContext` layoutとgeneric owner primitiveをgenerated shimとruntimeが
共有するprivate headerへ移した。`mal.h`に残すlifecycle entry pointはSymbolまたはBufferのaddressable carrierを受け、host helperが
representation固有のownerを直接操作しない。これによりcontrol storageとowner mechanismはpublic carrierやextern signatureを変えずに
変更できる。

一方、`mal_storage_descriptor_t`はtype-erased Buffer storageが実際に必要とするsize、alignment、share、dropだけを保持している。
dynamic type identity、allocation policy、access permissionを将来用に追加しない。bitwise relocationを満たさない型を扱う要求が出た場合は、
既存descriptorへ無条件にmove callbackを足すのではなく、その型をBufferとby-value ABIへadmitするかから再検討する。

## 再整理を再開する条件

現在の三値のresponsibility、bitwise relocation、静的runtime組み込み、opaqueなCall APIは、現在admitする型と実行方式には十分である。
次の観測可能な要求が現れた箇所から境界を再検討し、将来用のfield、callback、stageは先行して追加しない。

- carrier内部address、pin、または非同期native accessによってbitwise relocationできない型をadmitする場合は、relocation capabilityと
  Buffer、frame、extern by-value carrierのadmissionを分離する。
- 一つの通常aggregateでmanaged leafごとに異なるresponsibilityを境界越しに保つ必要がある場合は、`ValueResponsibility`とlifecycle変換を
  leaf構造にする。
- source reference、排他的access、shared access、または非同期completionがcontrol program上のlender生存期間だけでは表せない場合は、
  responsibilityとは独立したpermission、loan、path identityを導入する。
- 同じ型再帰のshare、drop、initialize、replace、vacateが独立した二つ以上のemitterまたはglue形式で必要になった場合は、typed lifecycle
  loweringを共有境界として抽出する。
- Engram実装をcompiler外から選択する要求が生じ、後述するcrate分離の条件とversion、cache、reproducible buildのcontractを満たせる場合に、
  静的extension manifestを検討する。
- public carrier、extern signature、またはCall APIを変える必要がある場合だけC ABIを改訂する。private context layout、owner mechanism、
  runtime内部接続だけの変更はpublic ABI改訂の理由にしない。

## 採択候補

具体的な重複が複数の利用者に現れた場合、concrete type `T`に対する次のbackend-local operationを共有境界として評価する。

```text
share<T>(value)       既存responsibilityを保ったまま独立したresponsibilityを作る
drop<T>(value)        一つのresponsibilityを終了する
initialize<T>(place, value)
                      vacantなcarrierへowned valueを格納する
replace<T>(place, value)
                      liveな旧valueを終了してownedな新valueを格納する
vacate<T>(place)      ConsumeまたはDrop後のcarrierをlive valueとして扱えなくする
```

この境界を独立したplan dataや汎用IRにするのは、同じlifecycle decisionを複数形式へ生成する利用者が実在する場合に限る。
単一backend内のhelperで閉じる間は、新しいstageやregistryを追加しない。

型を知らないruntimeが値を保持する場合は、同じloweringからC-callableな`share<T>`と`drop<T>`のglueを生成する案を評価する。
Buffer element callbackとclosure environment destructorはruntime contractが異なるため、signatureを統合せず、必要なら型再帰だけを
共有する。

## carrier invariant

共有境界を導入する場合、addressableなmanaged carrierはbackend lowering上で`Vacant`または`Live<T>`の一方として扱う。

1. `initialize`は`Vacant`だけをdestinationにし、旧valueをdropしない。
2. `replace`は`Live<T>`だけをdestinationにし、新しいresponsibilityを成立させてから旧responsibilityを終了する。
3. `Consume`はsource responsibilityをdropせずに移し、source carrierを`Vacant`にする。
4. `Drop`はlive responsibilityを一度終了し、addressableなsource carrierを`Vacant`にする。
5. relocationはvalueの物理addressだけを変え、`share`または`drop`を発生させない。
6. typed payloadを所有するallocationは、全live payloadをdropした後にだけ解放する。

この状態はsource valueの一部ではない。reference countingもsource semanticsにせず、ownership planが要求するresponsibilityを
保持できる限り別の回収mechanismを許す。

## 新しいEngram leafのadmission

新しいEngram leafには少なくとも次を定義する。

- valid valueを構成するtrusted operationとrepresentation invariant
- runtime value layoutとowner projection
- share、drop、moveまたはrelocation
- immutable valueかshared mutable identityか
- 保持する子Engramと、reference-count cycleを作らない根拠
- `Storable`とclosed extern carrierとしてのadmission
- Extern境界がある場合のborrow、share/drop、result move

drop時の処理は子Engramとmal-owned storageの回収に限り、I/OやExtern resourceの`close`のような観測可能な作用を持たせない。

## crate分離の条件

Engram固有実装を別crateへ分離するのは、次をすべて満たす場合に限る。

- compiler本体がEngram固有のowner field、retain、releaseを分岐せず、明示したleaf lifecycleだけを使う。
- operation loweringが汎用compiler stateではなく、value、target layout、artifact dependencyだけを入力にする。
- runtime sourceとdeclarationの選択を静的なextension manifestから決定できる。
- parser、checker、editor、diagnosticのlanguage authorityが分散しない。
- compilerとextensionのversion、artifact cache、reproducible buildのcontractを固定できる。

この条件はdynamic plugin ABIやthird-party extensionを約束しない。file ownershipを移すだけで独立contractが増えない場合は、
組み込みmoduleのままにする。

## 検証条件

共有境界または新しいleafを採択する変更では、次を検査する。

- unmanaged scalarと各managed leaf、managed product、managed sumの`Share`、`Consume`、`Drop`
- vacant slotへの初期化とlive slotの置換で異なる旧ownerのrelease
- duplicate field、self assignment、active sum payload、wildcard discardでのresponsibilityの過不足
- runtime callbackとclosure destructorが通常値と同じ型再帰を使うこと
- static owner、null owner、empty valueがallocationや不正なreleaseを生まないこと
- LLVM moduleをClangでcompile、link、executeしたときに終了後のlive allocationが残らないこと

generated IRの形だけでlifecycleの正しさを判定しない。

## 非目標

- manual lifecycle operation、borrow checker、linear type、unique ownership、user-defined destructor
- 共通owner header、runtime type descriptor、dynamic dispatchを先行して導入すること
- `Buffer`の共有identity、auto-growth、`Symbol` snapshot、host copy semanticsの変更
- `Storable`制限を緩和して任意のmanaged graphやcycleを許可すること
- 将来のEngramだけを想定した空module、runtime registry、dynamic descriptor
