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
} MalString;

typedef struct {
    uint8_t *address;
} MalPtr;

_Noreturn void mal_trap(MalContext *context, const char *message);

MalString mal_string_copy(
    MalContext *context,
    const uint8_t *data,
    uint64_t length
);
```

`MalContext *`は各extern implementationの先頭parameterとして渡す。hostはcall終了後にcontextを保持してはならない。`mal_trap`と`mal_string_copy`はreference runtimeが提供する。

`MalUnit`はaggregate内に現れる`Unit`の表現である。top-level parameterまたはresultそのものが`Unit`の場合は、後述のとおりC parameterを省略するか`void` resultにする。

`mal_string_copy`はbytesをmal-ownedなprogram-lifetime storageへcopyする。allocation size overflowまたはfailureではtrapし、正常returnしたStringはprogram終了まで有効である。`length == 0`では`data`をdereferenceしない。

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

String parameterは`MalString`で渡し、hostはcall終了後に`data`を保持しない。String resultを返すhost implementationは`mal_string_copy`で作った`MalString`を返す。

`Ptr`は`MalPtr`でby-valueに渡す。hostは`address`が指すlive region、read/write permission、lifetimeを
operation固有のcontractとして定める。reference runtimeのscalar accessは`memcpy`相当であり、alignmentを
要求しない。異なるscalar型で同じbytesを観測した場合はtarget C scalarのobject representationに従う。

productと一般のsumのfield order、tag、paddingを含む正確なC declarationはgenerated headerを正とする。一般のsumのtagは
0-based `uint32_t`である。`[Unit, Unit]`には前述のBool specializationを適用し、sum structを生成しない。

## closure exclusion

function型を直接またはproduct/sum内に含む型はextern signatureに使用できない。したがってこのABIはcallback function pointer、closure environment、hostによるclosure retentionを定義しない。

## failure

回復可能なhost failureは明示的なsum resultとしてAPIに表す。ABI共通のhidden error channel、`errno` mapping、exception translationは持たない。回復不能なcontract violationは`mal_trap`を呼べる。
