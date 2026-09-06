# C host ABI

Status: Current v0.5 profile

この文書はmalのbackend非依存な意味論ではなく、v0.5 reference compilerのhost interfaceを定める。別backendはsource-level semanticsを保つ限り別のABIを使用できる。

## build model

reference compilerはmal sourceからC translation unitと、そのprogramが要求するextern symbolを宣言したC headerを生成する。利用者はheaderに対するC implementationまたはadapterを用意し、生成Cと同じtarget ABIでcompileする。

linker inputにはC source、object file、static archive、shared objectを指定できる。shared objectは通常のplatform linker/loaderでprocess開始時に解決し、v0.5 runtimeは`dlopen`、symbol discovery、plugin lifecycleを提供しない。

既存libraryのfunctionを任意の宣言で直接呼ぶことは保証しない。型やownershipが合わない場合は利用者が薄いC adapterを書く。

## generated header

headerは少なくともC11でcompileでき、同じprogramについて生成したC translation unitと対になる。v0.5は異なるcompiler versionが生成したheader間のbinary compatibilityを保証しない。shared objectは対象programのheaderに対してbuildする。

共通部分は概念上次を含む。

```c
#include <stdint.h>

#define MAL_C_ABI_VERSION 0x000500u

typedef struct MalContext MalContext;

typedef struct {
    uint8_t unused;
} MalUnit;

typedef struct {
    const uint8_t *data;
    uint64_t length;
} MalEngram;

typedef struct {
    uint8_t *address;
} MalPtr;

_Noreturn void mal_trap(MalContext *context, const char *message);

MalEngram mal_engram_copy(
    MalContext *context,
    const uint8_t *data,
    uint64_t length
);
```

`MalContext *`は各extern implementationの先頭parameterとして渡す。hostはcall終了後にcontextを保持してはならない。`mal_trap`と`mal_engram_copy`はreference runtimeが提供する。

generated headerは各external operationに`MAL_HAS_EXTERN_<name>`を値`1`で定義し、`MAL_DEFINE_<name>` macroも生成する。
前者は複数programで共有するhost adapterが、そのprogramにoperationが存在するかをpreprocessorで判定するために使う。
後者は先頭にcontextのidentifier、続いてsource-level parameterに対応するidentifierを受け取り、正しいC function
definition headerへ展開する。context parameterにはgenerated headerの`MAL_MAYBE_UNUSED`を付けるため、implementationが
runtime serviceを使わない場合にunused castを必要としない。macroを使わず、宣言された`mal_ext_<name>`を直接定義してもよい。

```c
#ifdef MAL_HAS_EXTERN_printInt32
MAL_DEFINE_printInt32(context, value) {
    /* ... */
}
#endif
```

`MalUnit`はaggregate内に現れる`Unit`の表現である。top-level parameterまたはresultそのものが`Unit`の場合は、後述のとおりC parameterを省略するか`void` resultにする。

`mal_engram_copy`はbytesをmal-ownedなprogram-lifetime storageへcopyする。allocation size overflowまたはfailureではtrapし、正常returnしたEngramはprogram終了まで有効である。`length == 0`では`data`をdereferenceしない。

## symbol naming

malのexternal symbol `name`に対応するC symbolは`mal_ext_name`とする。VALUE_IDENTがASCII alphanumericだけなので追加escapingは不要である。

```mal
extern printInt32 :: Int32 -> Unit;
```

```c
void mal_ext_printInt32(MalContext *context, int32_t value);
```

`mal_` prefixはgenerated/runtime symbol用に予約する。

## type mapping

numeric scalarは対応する`intN_t`、`uintN_t`、binary32 `float`、binary64 `double`でby-valueに渡す。targetが要求representationを満たさなければそのtargetにFloat32/64を提供しない。

`Bool`と構造的に同じ`[Unit, Unit]`は`uint8_t`でby-valueに渡し、index 0を`UINT8_C(0)`、index 1を
`UINT8_C(1)`で表す。generated Cが作る値はこの2値に限定する。host implementationもBool resultとして0または1だけを
返さなければならず、それ以外の値はextern contract違反である。このspecializationはtransparent aliasとしての
source-level semanticsを変えない。

Floatを使うprogramのC adapterはround-to-nearest, ties-to-evenのfloating-point environmentを保持し、
flush-to-zeroまたはdenormals-are-zeroを有効にしたままreturnしてはならない。完全なtarget条件は
[D019](../design/decisions.md#d019-decimal-float-syntaxとc-target-profileを固定する)に定める。

top-level parameter型が`Unit`ならC側parameterを追加しない。top-level result型が`Unit`ならC resultは`void`とする。top-level parameter型がproductなら、その直下の要素をsource orderでC parameterへflattenする。nested productとsumにはgenerated header内のprogram固有structを用いる。aggregate resultはgenerated structをby-valueで返す。

```mal
extern writeInt32 :: (Mem, UInt64, Int32) -> Unit;
```

は概念上次の形になる。

```c
void mal_ext_writeInt32(
    MalContext *context,
    MalOpaque_Mem memory,
    uint64_t offset,
    int32_t value
);
```

各external opaque typeは一machine wordのcopyable handleとして生成する。

```c
typedef struct {
    uintptr_t bits;
} MalOpaque_Mem;
```

host resourceが一wordに収まらない場合はhost側でboxする。zero bit pattern、copy、dropには言語組み込みの意味を与えず、個々のhost contractが定める。
generated headerは各opaque typeについて`mal_<Type>_from_bits`と`mal_<Type>_bits`を生成する。このhelperは
`.bits` fieldと同じbit patternを構成・取得するだけであり、resource contractやownershipを追加しない。

`MalEngram`はmal EngramをC境界で運ぶABI carrierであり、hostが独立して所有するbyte buffer型ではない。
Engram parameterは`MalEngram`で渡し、hostはcall終了後に`data`を保持しない。Engram resultを返すhost
implementationは、一時byte bufferを`mal_engram_copy`へ渡して作った`MalEngram`を返す。

Engram descriptorのmemory load/store表現はCの`MalEngram` object representationそのものではない。
`MalPtr`のobject representationと`uint64_t`のlengthをこの順でpaddingなしに置く。hostがこの表現を書く場合も、
data pointerとlengthは既存の有効なmal Engramから取得し、C structのpaddingを含む`sizeof(MalEngram)` bytesを
そのままcopyしてはならない。

`Ptr`は`MalPtr`でby-valueに渡す。hostは`address`が指すlive region、read/write permission、lifetimeを
operation固有のcontractとして定める。reference runtimeのnumeric scalarおよびpointer accessは`memcpy`相当であり、
alignmentを要求しない。異なるscalar型で同じbytesを観測した場合はtarget C scalarのobject representationに従う。
pointer accessは`MalPtr`のobject representationをcopyし、必要なstorage sizeとrepresentationはtarget ABIに従う。
hostは`mal_ptr_from_address`と`mal_ptr_address`で`MalPtr`を構成・参照できる。このhelperはregion、permission、lifetimeを
検査または延長しない。

productと一般のsumのfield order、tag、paddingを含む正確なC declarationはgenerated headerを正とする。一般のsumのtagは
0-based `uint32_t`である。`[Unit, Unit]`には前述のBool specializationを適用し、sum structを生成しない。

extern signatureのABI表現に現れるsource-level aliasには、generated headerで`MalType_<Alias>`という`typedef`を生成する。
extern declarationはsourceの対応位置に明記されたaliasを`MalType_<Alias>`として保持する。同じunderlying typeを表す
aliasが複数あっても、構造的一致から別のaliasを推測しない。top-level product parameterをflattenするときは、そのproduct
aliasの定義に明記された直下要素のaliasを各C parameterに保持する。flattenによってC declarationに現れない外側のproduct
aliasと、そのためだけのproduct structはheaderへ生成しない。どの`typedef`も新しいnominal identityやruntime
representationを作らない。

extern境界から到達できるproduct aliasには`mal_make_<Alias>`と位置ごとの`mal_get_<Alias>_<index>`を生成する。
一般のsum aliasには`MAL_TAG_<Alias>_<index>`、`mal_tag_<Alias>`、`mal_is_<Alias>_<index>`、
`mal_make_<Alias>_<index>`を生成する。payload取得helperはvariantが一致しなければ`mal_trap`を呼ぶ。product payloadの
取得helperは`mal_get_<Alias>_<variant>_<field>`とする。これらはC記述用のconvenience APIであり、source-levelの
positional product/sum semanticsを変更しない。

## closure exclusion

function型を直接またはproduct/sum内に含む型はextern signatureに使用できない。したがってこのABIはcallback function pointer、closure environment、hostによるclosure retentionを定義しない。

## failure

回復可能なhost failureは明示的なsum resultとしてAPIに表す。ABI共通のhidden error channel、`errno` mapping、exception translationは持たない。回復不能なcontract violationは`mal_trap`を呼べる。
