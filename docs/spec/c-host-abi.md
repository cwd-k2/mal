# C host ABI

Status: Current v0.5 profile

この文書はmalのbackend非依存な意味論ではなく、v0.5 reference compilerのhost interfaceを定める。別backendはsource-level semanticsを保つ限り別のABIを使用できる。

## build model

reference compilerはmal sourceからC translation unitと、そのprogramが要求するextern symbolを宣言したC headerを生成する。利用者はheaderに対するC implementationまたはadapterを用意し、生成Cと同じtarget ABIでcompileする。

linker inputにはC source、object file、static archive、shared objectを指定できる。shared objectは通常のplatform linker/loaderでprocess開始時に解決し、v0.5 runtimeは`dlopen`、symbol discovery、plugin lifecycleを提供しない。

既存libraryのfunctionを任意の宣言で直接呼ぶことは保証しない。型やownershipが合わない場合は利用者が薄いC adapterを書く。

`main :: Unit -> Int32`にはCの`main(void)`を生成する。`main :: (UInt64, Ptr) -> Int32`には
`main(int argc, char **argv)`を生成する。`argv[1]`以降の各addressと終端NULを除いたlengthを、`MalType_Ptr`と
`uint64_t`をpaddingなしに並べた外部descriptor列へ置き、そのcountと先頭`Ptr`をsource-level `main`へ渡す。
argv bytesとdescriptor列は`main`のreturnまでread-onlyで有効であり、`loadSymbol`を呼ぶまでmal Symbolではない。

## generated header

headerは少なくともC11でcompileでき、同じprogramについて生成したC translation unitと対になる。v0.5は異なるcompiler versionが生成したheader間のbinary compatibilityを保証しない。shared objectは対象programのheaderに対してbuildする。

共通部分は概念上次を含む。

```c
#include <stdint.h>

#define MAL_C_ABI_VERSION 0x000500u

#define MAL_TYPE(name) MalType_##name
#define MAL_OPERATION(type, operation) mal_##type##_##operation
#define MAL_TAG(type, variant) MAL_##type##_TAG_##variant
#define MAL_EXTERN(name) mal_ext_##name

typedef struct MalContext MalContext;

typedef struct {
    uint8_t unused;
} MalType_Unit;

typedef uint8_t MalType_Bool;
typedef int8_t MalType_Int8;
typedef int16_t MalType_Int16;
typedef int32_t MalType_Int32;
typedef int64_t MalType_Int64;
typedef uint8_t MalType_UInt8;
typedef uint16_t MalType_UInt16;
typedef uint32_t MalType_UInt32;
typedef uint64_t MalType_UInt64;
typedef float MalType_Float32;
typedef double MalType_Float64;

typedef struct {
    const uint8_t *data;
    uint64_t length;
} MalType_Symbol;

typedef struct {
    uint8_t *address;
} MalType_Ptr;

#define MAL_FALSE ((MalType_Bool)UINT8_C(0))
#define MAL_TRUE ((MalType_Bool)UINT8_C(1))

_Noreturn void mal_trap(MalContext *context, const char *message);

MalType_Symbol mal_Symbol_copy_from_bytes(
    MalContext *context,
    const uint8_t *data,
    uint64_t length
);
```

`mal_ext_<name>`はraw host library functionそのものではなく、host operationとmal valueの間を変換するtrusted adapter
entryである。`MalContext *`はmal valueではなく、各adapterへ先頭parameterとして一時的に渡すruntime capabilityである。
adapterとhostはcall終了後にcontextを保持してはならない。`mal_trap`と`mal_Symbol_copy_from_bytes`はreference runtimeが
提供し、adapterは後者を通じてSymbol admissionをmalへ依頼する。helperを呼ぶauthorityはSymbolのownershipをadapterへ移さない。

malのpredefined type、source-level alias、external typeはすべてCで`MalType_<name>`と綴る。host implementationは
aliasとexternal typeを別の命名規則として記憶する必要がない。`MalRepr_Product_<id>`と`MalRepr_Sum_<id>`は
source-level nameを持たないstructural typeのgenerated representation名であり、`MalType_`の名前とは区別する。

型に属するhost helperは`mal_<owner>_<operation>`と綴り、positional variantとfieldはoperationの後ろへsource orderで
付ける。`MAL_TYPE(name)`、`MAL_OPERATION(type, operation)`、`MAL_TAG(type, variant)`、`MAL_EXTERN(name)`は、
この規則からそれぞれC type、type operation、sum tag constant、external symbolを構成する。これらは通常のC identifierを隠す別interfaceではなく、展開後のidentifierも
直接使用できる。`mal_trap`のような型に属さないruntime operationと、`mal_ext_<name>`のようなprogram operationには
型ownerを補わない。

```c
MAL_TYPE(Symbol) value;
MAL_OPERATION(Ptr, from_address)(address);
MAL_OPERATION(Response, make_1)(memory, length);
MAL_TAG(Response, 1);
MAL_EXTERN(printInt32)(context, value);
```

generated headerは各external operationに`MAL_HAS_EXTERN_<name>`を値`1`で定義し、`MAL_DEFINE_<name>` macroも生成する。
前者は複数programで共有するhost adapterが、そのprogramにoperationが存在するかをpreprocessorで判定するために使う。
後者は先頭にcontextのidentifier、続いてsource-level parameterに対応するidentifierを受け取り、正しいC function
definition headerへ展開する。context parameterにはgenerated headerの`MAL_DETAIL_MAYBE_UNUSED`を付けるため、implementationが
runtime serviceを使わない場合にunused castを必要としない。macroを使わず、宣言された`mal_ext_<name>`を直接定義してもよい。
`MAL_HAS_EXTERN_<name>`は`#ifdef`のoperandとしてliteral identifierを要求し、`MAL_DEFINE_<name>`はprogram固有signatureを
保持するため、この二つはgeneric macroだけへ置き換えない。

```c
#ifdef MAL_HAS_EXTERN_printInt32
MAL_DEFINE_printInt32(context, value) {
    /* ... */
}
#endif
```

`MalType_Unit`はaggregate内に現れる`Unit`の表現である。top-level parameterまたはresultそのものが`Unit`の場合は、後述のとおりC parameterを省略するか`void` resultにする。

`mal_Symbol_copy_from_bytes`はbytesをmal-controlled storageへcopyしてSymbolをadmitする。allocation size overflowまたは
failureではtrapし、正常returnした値はmalのlifetime authorityに属する。`length == 0`では`data`をdereferenceしない。

## symbol naming

malのexternal symbol `name`に対応するC symbolは`mal_ext_name`とする。VALUE_IDENTがASCII alphanumericだけなので追加escapingは不要である。

```mal
extern printInt32 :: Int32 -> Unit;
```

```c
void mal_ext_printInt32(MalContext *context, MalType_Int32 value);
```

`mal_` prefixはgenerated/runtime symbol用に予約する。

## type mapping

numeric scalarは対応する`intN_t`、`uintN_t`、binary32 `float`、binary64 `double`をtypedefした
`MalType_<name>`でby-valueに渡す。targetが要求representationを満たさなければそのtargetにFloat32/64を提供しない。

`Bool`と構造的に同じ`[Unit, Unit]`は`uint8_t`をtypedefした`MalType_Bool`でby-valueに渡し、index 0を
`MAL_FALSE`、index 1を`MAL_TRUE`で表す。generated Cが作る値はこの2値に限定する。host implementationもBool resultとして0または1だけを
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
    MalType_Mem memory,
    MalType_UInt64 offset,
    MalType_Int32 value
);
```

各external typeはC hostにとって一machine wordのcopyable named handleとして生成する。`opaque`はmal側から
representationを操作できないことを表すlanguage-side propertyなので、C type名には含めない。

```c
typedef struct {
    uintptr_t bits;
} MalType_Mem;
```

host resourceが一wordに収まらない場合はhost側でboxする。zero bit pattern、copy、dropには言語組み込みの意味を与えず、個々のhost contractが定める。
generated headerは各external typeについて`mal_<Type>_from_bits`と`mal_<Type>_bits`を生成する。このhelperは
`.bits` fieldと同じbit patternを構成・取得するだけであり、resource contractやownershipを追加しない。

`MalType_Symbol`はmal SymbolをC境界で運ぶABI carrierであり、hostが独立して所有するbyte buffer型ではない。
Symbol parameterは`MalType_Symbol`で渡し、hostはcall終了後にdataを保持しない。dataとlengthは
`mal_Symbol_data`と`mal_Symbol_length`で取得できる。Symbol resultを返すhost implementationは、一時byte bufferを
`mal_Symbol_copy_from_bytes`へ渡して作った`MalType_Symbol`を返す。hostがstruct literalなどで独自のdata pointerを
持つ`MalType_Symbol`を直接作って返すことはcontract違反である。

`MalType_Symbol`はextern call中のABI carrierであり、source-level memory表現ではない。hostがstructやdata pointerを
外部memoryへ保存しても、後からmal Symbolとして復元できない。`loadSymbol`は外部のraw bytesとlengthを受け取り、
新しいSymbolへcopyする。`storeSymbol`はSymbolのraw bytesだけを外部memoryへcopyする。

`Ptr`は`MalType_Ptr`でby-valueに渡す。hostは`address`が指すlive region、read/write permission、lifetimeを
operation固有のcontractとして定める。reference runtimeのnumeric scalarおよびpointer accessは`memcpy`相当であり、
alignmentを要求しない。異なるscalar型で同じbytesを観測した場合はtarget C scalarのobject representationに従う。
pointer accessは`MalType_Ptr`のobject representationをcopyし、必要なstorage sizeとrepresentationはtarget ABIに従う。
hostは`mal_Ptr_from_address`と`mal_Ptr_address`で`MalType_Ptr`を構成・参照できる。このhelperはregion、permission、lifetimeを
検査または延長しない。

productと一般のsumのfield order、tag、paddingを含む正確な`MalRepr_` declarationはgenerated headerを正とする。一般のsumのtagは
0-based `uint32_t`である。`[Unit, Unit]`には前述のBool specializationを適用し、sum structを生成しない。

extern signatureのABI表現に現れるsource-level aliasには、generated headerで`MalType_<Alias>`という`typedef`を生成する。
extern declarationはsourceの対応位置に明記されたaliasを`MalType_<Alias>`として保持する。同じunderlying typeを表す
aliasが複数あっても、構造的一致から別のaliasを推測しない。top-level product parameterをflattenするときは、そのproduct
aliasの定義に明記された直下要素のaliasを各C parameterに保持する。flattenによってextern function declarationに現れない
外側のproduct aliasも`MAL_TYPE(<Alias>)`で参照できるよう、そのtypedef、representation、helperをheaderへ生成する。
どの`typedef`も新しいnominal identityやruntime
representationを作らない。

extern境界から到達できるproduct aliasには`mal_<Alias>_make`と位置ごとの`mal_<Alias>_get_<index>`を生成する。
一般のsum aliasには`MAL_<Alias>_TAG_<index>`、`mal_<Alias>_tag`、`mal_<Alias>_is_<index>`、
`mal_<Alias>_make_<index>`を生成する。payload取得helperは`mal_<Alias>_expect_<variant>`とし、variantが一致しなければ
`mal_trap`を呼ぶ。product payloadの取得helperは`mal_<Alias>_expect_<variant>_<field>`とする。
`get`は必ず成功するproduct projectionだけに、`expect`はtrapし得るsum projectionだけに使う。これらはC記述用のconvenience APIであり、source-levelの
positional product/sum semanticsを変更しない。

## closure exclusion

function型を直接またはproduct/sum内に含む型はextern signatureに使用できない。したがってこのABIはcallback function pointer、closure environment、hostによるclosure retentionを定義しない。

## failure

回復可能なhost failureは明示的なsum resultとしてAPIに表す。ABI共通のhidden error channel、`errno` mapping、exception translationは持たない。回復不能なcontract violationは`mal_trap`を呼べる。

adapterはresult capabilityを正常returnした時点でhostからmalへのtransferをcommitする。それ以前にtrapする場合、または
capabilityを含まないfailure variantを返す場合、adapterがそのcall内で取得した一時allocationや未transfer resourceは
adapter自身が解放する。argument resourceと以前にtransfer済みのresourceはこのcleanupの対象ではない。
`mal_Symbol_copy_from_bytes`と`mal_trap`はreturnしない場合があるため、その前にcleanup不能な一時resourceを残してはならない。
