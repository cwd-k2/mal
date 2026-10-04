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
