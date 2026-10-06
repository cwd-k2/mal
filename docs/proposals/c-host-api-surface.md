# C host API surfaceの分類とcall統一案

Status: Exploratory

この文書は、C host APIを型マクロ、定数、演算子、Call APIへ分類し、activeなruntime contextを必要とするoperationの
`mal_call_t *call`を統一する案を管理する。現行規則は[C runtime extension ABI](../spec/c-host-abi.md)と
[C host value API](../spec/c-host-api.md)を正とする。この提案は採択前のため、generated headerとhost C sourceがここで示す
名前またはsignatureへ依存してはならない。

## 問題

現行APIは`call`をruntime allocation、Share、trapに使うcapabilityと説明する一方、SymbolとBufferのoperationは、それらを内部で
行うかどうかにより`call`の有無が分かれる。たとえば`mal_count(buffer)`と`mal_truncate(buffer, count)`は`call`を取らず、
`mal_replace(call, buffer, index, element)`は範囲外をtrapできるため`call`を取る。利用者はsignatureを予測するために各operationの
実装可能なeffectを知る必要がある。

Shareが`call`を使う現在の理由は、Symbol byte ownerのreference count overflowをfatal failureへ写すことにある。Shareはallocationせず、
既存identityのresponsibilityを一つ増やすだけである。有効なresponsibilityに対するShareは失敗しないcontext-freeなlifecycle
operatorとし、representation固有のoverflowはpublicなcall capabilityを要求する理由にしない。

また、型を書くmacro、carrierだけを扱うoperator、runtime entry pointが同じ`mal_` namespaceに並び、名前から分類を判別できない。
`mal_storage(T)`はtype queryだが、名詞と同じ綴りのため`sizeof(T)`、`_Alignof(T)`、`offsetof(T, member)`に近いoperatorであることが
表面から分かりにくい。

## 分類規則

public surfaceを次の四種類へ分ける。

| 種類 | 判定規則 | `call` |
|---|---|---|
| 型マクロ | public carrier型またはmanaged local宣言を構成する | 取らない |
| 定数 | runtimeを参照しないcanonical carrier値を表す | 取らない |
| 演算子 | 型query、carrier表現、context-freeなlifecycleを扱う | 取らない |
| Call API | activeなruntime contextのもとでruntimeまたはruntime-managed valueを操作する | 常に先頭に取る |

この分類は、allocationまたはtrapが現在の実装で発生するかをcall siteごとに推測させない。Bufferの単純な観測や縮小もactiveな
runtime contextに属するBuffer operationなのでCall APIに含める。Share、Move、Dropは同じresponsibility語彙として演算子に置き、
opaque bits変換とtype queryもruntime contextから独立した演算子に置く。Call APIへ渡す`call`はactivationとの所属を表し、invalid
carrierの検査、新しいfailure、または新しいobservable effectを加えるものではない。

型マクロが受け渡すpublic carrier type form `T`は次で構成する。

```text
T ::= mal_type(Name)
    | mal_product(T, ...)
    | mal_sum(T, ...)
```

`mal_product`、`mal_sum`、`mal_storageof`にはこの`T`を渡す。`mal_owned(Name)`はcleanupが生成されたnamed managed typeだけを受ける
宣言形式であり、anonymous structural typeまたは任意のC型を受けるtype constructorではない。

## 提案するsurface

型マクロ、定数、演算子は次とする。

| 種類 | 名前または形式 | 役割 |
|---|---|---|
| 型マクロ | `mal_type(Name)` | named closed Mal型のC carrier型 |
| 型マクロ | `mal_product(T0, ...)` | structural productのC carrier型 |
| 型マクロ | `mal_sum(T0, ...)` | structural sumのC carrier型 |
| 型マクロ | `mal_owned(Name)` | lexical cleanupを持つmanaged local宣言 |
| 定数 | `mal_false` | `Bool`のfalse carrier |
| 定数 | `mal_true` | `Bool`のtrue carrier |
| 定数 | `mal_unit` | `Unit` carrier |
| 演算子 | `mal_storageof(T)` | carrier型のstorage descriptorを取得 |
| 演算子 | `mal_share(value)` | managed responsibilityを一つ追加 |
| 演算子 | `mal_move(value)` | responsibilityを移し、sourceをvacantにする |
| 演算子 | `mal_drop(value)` | managed responsibilityを一つ終了 |
| 演算子 | `mal_from_bits(T, bits)` | bitsからexternal opaque carrierを構成 |
| 演算子 | `mal_to_bits(value)` | external opaque carrierをbitsへ変換 |

Call APIは責務ごとに次の系統へ分ける。すべて有効な`call`を先頭に受ける。

| 系統 | 名前または形式 | 役割 |
|---|---|---|
| Runtime capability | `mal_allocate(call, size)` | runtime allocatorからraw storageを確保 |
| Runtime capability | `mal_deallocate(call, allocation)` | runtime allocatorのraw storageを解放 |
| Runtime capability | `mal_trap(call, message)` | 回復不能failureとして終了 |
| Symbol construction | `mal_symbol(call, source, length)` | bytesをcopyしてowned Symbolを構成 |
| Symbol construction | `mal_snapshot(call, buffer)` | byte Bufferの独立したSymbol snapshotを構成 |
| Buffer construction | `mal_buffer(call, storage, capacity)` | owned Bufferを構成 |
| Buffer observation | `mal_data(call, buffer)` | element storageへのborrowed pointerを取得 |
| Buffer observation | `mal_count(call, buffer)` | element countを取得 |
| Buffer element mutation | `mal_push(call, buffer, element)` | elementを末尾へMove |
| Buffer element mutation | `mal_replace(call, buffer, index, element)` | elementを置換 |
| Buffer element mutation | `mal_fill(call, buffer, offset, count, element)` | rangeを同じelementで埋める |
| Buffer range transfer | `mal_copy(call, destination, destination_offset, source, source_offset, count)` | Buffer間でrangeをcopy |
| Buffer range transfer | `mal_append(call, buffer, source, count)` | pointer rangeを末尾へ追加 |
| Buffer extent | `mal_extend(call, buffer, count)` | trivial elementの未初期化領域を追加 |
| Buffer extent | `mal_truncate(call, buffer, count)` | 末尾のelementを取り除く |
| Buffer extent | `mal_reserve(call, buffer, capacity)` | element capacityを確保 |

`mal_storageof(T)`は`sizeof(T)`、`_Alignof(T)`、`offsetof(T, member)`と同じく、型から値を得るoperatorであることを綴りで示す。
`mal_storeof(T)`はstorage descriptorのqueryではなく値をstoreするoperationとも読めるため採らない。
`mal_to_bits`は入力をconsumeしないので、ownership transferを示唆する`mal_into_bits`を使わない。

`mal_share(value)`、`mal_move(value)`、`mal_drop(value)`は同じlifecycle operator群とする。Share callbackもcontext-freeとし、Shareは
allocationせず、有効なresponsibilityに対して失敗しない。reference count overflowは特定のrepresentationに由来するinternal fatal
failureであり、必要ならcontext-freeなruntime終了経路で扱う。CがShareのresult responsibilityを保存もDropもせず捨てた後の挙動は、
従来どおりlifecycle contract違反として保証しない。

## 現行surfaceからの変更

| 現行 | 提案 |
|---|---|
| `mal_storage(T)` | `mal_storageof(T)` |
| `mal_share(call, value)` | `mal_share(value)` |
| `mal_runtime_allocate(call, size)` | `mal_allocate(call, size)` |
| `mal_runtime_deallocate(allocation)` | `mal_deallocate(call, allocation)` |
| `mal_call_trap(call, message)` | `mal_trap(call, message)` |
| `mal_bits(value)` | `mal_to_bits(value)` |
| `mal_data(buffer)` | `mal_data(call, buffer)` |
| `mal_count(buffer)` | `mal_count(call, buffer)` |
| `mal_truncate(buffer, count)` | `mal_truncate(call, buffer, count)` |

ABIはversion間のsource compatibilityとbinary compatibilityを保証しないため、採択時は旧名を互換aliasとして残さず、
`MAL_C_ABI_VERSION`、common header、generated header、guide、fixtureを一つの変更として更新する。

## Public surfaceと内部entry point

host C bodyが使う上表のsurfaceと、generated lifecycle glueまたはruntime自身が使う低水準entry pointを分ける。
`mal_detail_*`、owner、bytes、Buffer storageの低水準functionはCall APIではなく内部ABIであり、publicな選択肢として文書化しない。
C headerから宣言を完全に隠せないentry pointも、予約prefixと責務を揃え、host向けのcanonical operationと競合させない。

## 採択前の未決事項

- `mal_deallocate(call, allocation)`により、raw allocationをcall activation外で解放する用途を意図的に閉じてよいか。現在の同期extern、
  thread confinement、unload protocolを持たない範囲で、必要な反例がないことを確認する。
- public名から`runtime`と`call`を外す`mal_allocate`、`mal_deallocate`、`mal_trap`が、内部entry pointとのnamespaceを明確に保てるか。
- `mal_data`と`mal_count`のような非失敗の観測へ`call`を要求する記述量と、全Call APIを一つの規則から予測できる利点をscenarioで比較する。

## 採択時の検証境界

- generated `mal.h`と保存済みcommon headerが同じ分類、名前、signatureを持つ。
- generated aggregateのcontext-freeなShare、Move、Drop、cleanup、storage descriptorが新しいpublic surfaceと同じresponsibilityを保持する。
- host fixtureをwarning付きClangでcompileし、Symbol、Buffer、opaque carrier、managed aggregateのresultとowner終状態を確認する。
- `mal_share`、`mal_move`、`mal_drop`が`call`なしで同じresponsibility遷移を行い、Share callbackがallocationまたはpublic Call APIを
  必要としないことを確認する。
- `mal_data`、`mal_count`、`mal_truncate`が有効な`call`を受け、旧signatureと旧名がcompileされないことを確認する。
- baselineとproductionでresult、effect order、trap、owner終状態、bounded native stackが変わらない。
