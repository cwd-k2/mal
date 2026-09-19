# C host ABI

Status: Accepted ABI 0x000800 for the mal v0.6 profile

この文書はmal v0.6のreference compilerが生成するC host interfaceを定める。`0x000800`はC ABI自体の
versionであり、source languageのversionではない。言語側のextern semanticsは
[`extern`](extern.md)、authorityは[`engrams`](engrams.md)、外部memoryは[`memory`](memory.md)を正とする。
別backendはsource-level semanticsを保つ限り別のABIを使用できる。

## Build model

reference compilerはmal sourceからprogram固有headerを生成し、`build`ではLLVM module、C shim、C runtimeを構成する。host implementationはheaderを
includeし、生成artifactと同じtarget ABIでcompileする。`.mal` sourceから推移的にrequireされた`.c` fileは`build`のlink入力に
なる。build時には今回生成したheaderをC translation unitへ先に読み込み、host sourceの隣にある保存済みheaderが生成物を
置き換えない。既存libraryには薄いC adapterを介して接続し、必要なlibrary、object、archive、include path、macroなどの
toolchain argumentはreference compilerの明示的なbuild optionから渡す。これはsource-level `require`の一部ではない。

generated headerと対応するbuild artifactは一組であり、異なるcompiler出力を組み合わせてはならない。ABI versionは次で判定する。

```c
#define MAL_C_ABI_VERSION 0x000800u
```

`main :: Unit -> Int32`は`main(void)`へ、`main :: (USize, Address) -> Int32`は`main(int, char **)`へlowerする。
後者ではshimが各argumentをcanonical shape `(address, bytesize)`のdescriptorへ変換する。C structのlayoutをsource memory layoutとして
reinterpretしない。descriptorとargument bytesは`main`のreturnまでread-onlyで有効である。

## Host operation

各external operationには`MAL_HAS_EXTERN_<name>`と`MAL_DEFINE_<name>`を生成する。host implementationは
`MAL_DEFINE_<name>`だけでbodyを定義し、compiler-facing wrapperを直接定義しない。

```mal
Counter :: UInt64;
extern increment :: Counter -> Counter;
```

```c
MAL_DEFINE_increment(call, value) {
    return mal_Counter_return(call, value + UINT64_C(1));
}
```

bodyのparameterは常に先頭の`mal_call_t *call`と、source-level parameterが`Unit`でない場合の一つのtyped valueである。
productもflattenせず一つのvalueとして渡す。source aliasがあれば`mal_<Alias>_t`、なければhost value mappingの型を使う。

bodyはresult型に対応するterminal return helperでちょうど一度完了する。helperの結果を保存したり、helper後に処理を
続けたりしてはならない。

- `Unit`: `mal_Unit_return(call)`
- scalarまたはproduct: `mal_<Type>_return(call, value)`
- sum: `mal_<Type>_return_<variant>(call, payload)`
- source aliasのないaggregate: `mal_repr_<kind>_<id>_return...`

`mal_call_t`はcall-scoped capabilityである。hostはcall終了後にpointerまたはその内部状態を保持してはならない。
recoverできないcontract違反には`mal_call_trap(call, message)`を使う。

## Host value mapping

| mal type | Host C type |
|---|---|
| `Unit` | `mal_Unit_t` |
| `Bool` | `mal_Bool_t` |
| `IntN` / `UIntN` | 対応する`mal_IntN_t` / `mal_UIntN_t` |
| `Float32` / `Float64` | `mal_Float32_t` / `mal_Float64_t` |
| `ByteSize` / `USize` | `mal_ByteSize_t` / `mal_USize_t`（`size_t`） |
| `Address` | `mal_Address_t`（`void *`） |
| external opaque type `T` | `mal_T_t` |
| named alias `T` | `mal_T_t` |
| anonymous product/sum | `mal_repr_product_<id>_t` / `mal_repr_sum_<id>_t` |

fixed-width numeric typeは`stdint.h`の対応幅、`Float32`はbinary32 `float`、`Float64`はbinary64 `double`を使う。
generated headerは`sizeof(size_t) * CHAR_BIT`がtargetのpointer index幅と一致することをcompile-time assertionで検証する。
floating-point environmentの要件は[D019](../history/decisions/D019.md)に定める。

`mal_false`と`mal_true`だけがvalidな`mal_Bool_t`である。`mal_Bool_return`はそれ以外をtrapする。

productはsource orderの`field_<index>`を持つ。二項以上のsumは`uint32_t tag`と`payload.variant_<index>`を持ち、tagは0始まりである。
sum helperは`mal_<Type>_tag_<variant>`、pure constructor `mal_<Type>_make_<variant>`、terminal
`mal_<Type>_return_<variant>`を生成する。parameterとして受けたsumのtagはvalidである。hostがresult内に直接構成した
nested sumはterminal loweringがactive payloadを読む前にtagを検査し、不正値をtrapする。
これらはC host ABIのrepresentation helperであり、source-levelのsum constructorや構築authorityを追加しない。

空直和`[]`のC carrierは`uint32_t tag`だけを持ち、payload、constructor、terminal return helperを持たない。validなtagは存在せず、
hostから`[]`を返す正常完了も存在しない。carrierを宣言できることは値を構築するauthorityをhostへ与えない。

source aliasはtransparentであり、新しいruntime representationを作らない。extern declarationとalias定義に明記された
alias spellingだけをhost signatureとmember helperへ保存し、構造的一致から別名を推測しない。

## External opaque typeとAddress

external opaque type `T`は一machine wordのcopyable handleである。hostは
`mal_T_from_bits(uintptr_t)`と`mal_T_to_bits(value)`でlosslessに変換する。この操作はresourceのallocate、clone、close、freeや
追加authorityを伴わない。resource contractは各operationが定める。

`mal_Address_t`は`void *`であり、null以外をvalidな`Address`とする。terminal return helperはAddressを含むresultを再帰的に
検査し、nullをtrapする。変換helperは設けない。指すregion、permission、alignment、lifetimeはoperation固有のcontractであり、
境界通過によって変化しない。`Region`、`Packed`、`Buffer`はpublic C ABIへ出せない。

public headerはHostMappableなbuiltin carrierとhelper、extern signatureから到達できるHostMappableなaggregateとopaque型、および
後述するcanonical memory accessの対象aliasを生成する。`Symbol`、`Region`、`Packed`、`Buffer`、function、およびそれらを含む
aggregateの型名、内部carrier、ownership helperを宣言しない。

可変長bytesはoperation固有のHostMappableなproductとして`Address`と`USize`または`ByteSize`を渡す。読み出しではhostは
指定範囲をcall中だけborrowし、書き込みではmalが用意した範囲のうちcontractが定めるprefixだけを初期化する。hostはAddressを
call後に保持せず、mal-owned valueのidentity、owner、連続表現を観測しない。長さ、permission、初期化、partial transferの
postconditionは[`Region`と`Packed`](packed.md#partial-io)とoperation固有のcontractを正とする。

## Canonical memory access

entry sourceで宣言されたpublicなnongeneric type alias `T`が`HostMappable(T)`と`Representable(T)`をともに満たす場合、generated headerは
次のhelperを生成する。require先で宣言されたaliasは、同名の独立したmodule APIが衝突しないよう、extern signatureから要求される場合を除いて
entry programのC surfaceへ自動的に再公開しない。

```c
mal_T_t mal_T_read(mal_call_t *call, mal_Address_t address, mal_USize_t index);
void mal_T_write(
    mal_call_t *call,
    mal_Address_t address,
    mal_USize_t index,
    mal_T_t value
);
```

helperは`address`を先頭とするcanonical `T`列の`index`番目を、public C carrierとの間でfieldごとに変換する。compilerが同じ
target layoutからstride、product field offset、sum tag幅、payload offsetを生成するため、unaligned locationでも利用できる。
`read`はpaddingと非選択payloadを読まず、`write`はpaddingと非選択payloadを書かない。`Unit`のstrideは0であり、storageを
dereferenceしない。

`read`はcanonical `Bool`、sum tag、`Address`のvalidityを検査し、`write`はhost carrier内の同じ値を検査する。不正値は
`mal_call_trap`で終了する。これらはC boundaryで内部corruptionを防ぐadmissionであり、source-level memory primitiveへ検査済み
semanticsを追加しない。

extent、permission、initialization、lifetime、および`index * stride(T)`のoverflowがないことはcallerとoperation固有contractの
preconditionである。helperはallocation、retain、releaseを行わず、Addressのauthorityを変更しない。public C carrierのlayoutは
canonical memory layoutではないため、`mal_T_t *`へのcast、`sizeof(mal_T_t)`によるstride推定、field addressの直接対応は保証しない。
private aliasと、二つのjudgmentのどちらかを満たさないaliasにはhelperもcarrierも追加公開しない。

## Failureとconcurrency

allocation failure、target sizeで表現できないlength、不正なBool/tag/Address、またはhost adapterが検出したoperation contract違反はtrapする。
sum loweringはtag検査前にpayloadを読まない。terminal conversion中にallocation failureが起きる現在のruntimeではtrapが
processを終了するためrollback frameを設けない。

reference runtimeのcontextとmanaged valueはthread-confinedである。同じcall capabilityへ複数threadから同時にaccessしては
ならない。hostがAddressの範囲を別threadで処理する場合もbody return前にjoinし、operation固有のpermissionを守る。

## Reserved namesとcompatibility

`mal_`と`MAL_` prefixはgenerated header/runtime用に予約する。uppercase `MAL_`はpreprocessor macro、lowercase `mal_`は型、
function、constantに使う。`MAL_DEFINE_<name>`が展開するcompiler-facing declarationと`mal_detail_` memberは実装detailであり、
host contractとして直接参照してはならない。

generated headerおよびlinked artifactのsource compatibilityまたはbinary compatibilityを異なる`malc` version間で保証しない。host sourceと`.mal`
sourceをauthorityとし、compiler更新後には生成物を組で再生成する。
