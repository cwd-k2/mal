# C runtime extension実装例

Status: Current v0.7 examples

この文書はgenerated file headerを使うC bodyの代表形を示す。型、lifecycle、failureの規範は
[`extern`](../spec/extern.md)と[C runtime extension ABI](../spec/c-host-abi.md)を正とする。C bodyは安全なFFIの外側にあり、
runtime carrierを直接壊せるtrusted extensionである。

## Symbolをborrowして観測する

```mal
extern writeBytes :: Symbol -> Unit;
```

```c
MAL_DEFINE_writeBytes(call, bytes) {
    if (fwrite(bytes.data, 1, bytes.length, stdout) != bytes.length) {
        mal_call_trap(call, "write failed");
    }
    return mal_Unit_return(call);
}
```

`bytes`はbody終了まで有効なborrowであり、`owner`をdropしない。call後にも保持する場合はbody中に
`mal_Symbol_share(call, bytes)`し、保存したresponsibilityを後で`mal_Symbol_drop`する。`data`だけを保存してもlifetimeは延びない。

## Bufferを構成してmoveする

```mal
extern readBytes :: Unit -> Buffer<UInt8>;
```

```c
MAL_DEFINE_readBytes(call) {
    mal_Buffer_t result = mal_Buffer_make(call, sizeof(mal_UInt8_t), 4096);
    for (;;) {
        const int byte = fgetc(stdin);
        if (byte == EOF) {
            if (ferror(stdin)) {
                mal_Buffer_drop(result);
                mal_call_trap(call, "read failed");
            }
            return mal_Buffer_return_move(call, result);
        }
        const mal_UInt8_t value = (mal_UInt8_t)byte;
        mal_Buffer_new(call, result, &value, sizeof(value));
    }
}
```

`mal_Buffer_make`が返すresponsibilityはC bodyが所有する。正常resultでは`return_move`へ渡し、それ以後は使用もdropもしない。
trap前に解放したいtemporaryは明示的にdropする。`mal_Buffer_new`などgrowthし得るoperationの後は、以前
`mal_Buffer_data`で得たpointerを再利用しない。

## Bufferをborrowして変更する

```mal
Sample :: (Int64, UInt8);
Samples :: Buffer<Sample>;
extern adjust :: Samples -> Unit;
```

```c
MAL_DEFINE_adjust(call, samples) {
    if (mal_Buffer_count(samples) == 0) {
        return mal_Unit_return(call);
    }
    mal_Sample_t *values = mal_Buffer_data(samples);
    values[0].field_0 += 1;
    values[0].field_1 += 1;
    return mal_Unit_return(call);
}
```

Buffer handleはshared identityを指すため、この変更はmal側のaliasから観測できる。Cはgenerated headerの型とruntimeが決めたstrideを
使う。別のwire layoutやhost copy layoutはない。

## External opaque resource

```mal
extern File;
OpenResult :: [File, UInt32];
extern openReadOnly :: Symbol -> OpenResult;
extern close :: File -> Unit;
```

```c
MAL_DEFINE_openReadOnly(call, path) {
    char *terminated = mal_runtime_allocate(call->mal_detail_context, path.length + 1);
    memcpy(terminated, path.data, path.length);
    terminated[path.length] = '\0';
    FILE *file = fopen(terminated, "rb");
    mal_runtime_deallocate(terminated);
    if (file == NULL) {
        return mal_OpenResult_return_1(call, (mal_UInt32_t)errno);
    }
    return mal_OpenResult_return_0(
        call,
        mal_File_from_bits((uintptr_t)(void *)file)
    );
}

MAL_DEFINE_close(call, file) {
    FILE *handle = (FILE *)(void *)mal_File_to_bits(file);
    if (fclose(handle) != 0) {
        mal_call_trap(call, "close failed");
    }
    return mal_Unit_return(call);
}
```

external opaque typeはbitsのcopyだけを行い、resource lifecycleを自動化しない。valid bit pattern、close回数、failure mappingは各operationの
contractが所有する。

## Productとsum

source aliasから生成された`mal_<Alias>_t`はfieldをsource順に持つ。sumは`tag`と`payload.variant_<n>`を持ち、generated
`mal_<Alias>_make_<n>`と`mal_<Alias>_return_<n>`を使ってactive variantを構成できる。managed payloadを含むresultでは、active
payloadのresponsibilityもterminal helperへmoveされる。

## `mal_call_t`の範囲

`mal_call_t`は同期body中のruntime allocation、share、trapに使う。Cからmal closureを呼ぶexecution control、program continuation、
async completion tokenではない。pointerと内部stateをbody終了後に保持しない。runtime contextとmanaged carrierはthread-confinedであり、
worker threadを使う場合もbody return前にjoinし、managed resultは元のthreadで構成する。

repository内の実行可能例は[`managed-bytes`](../../examples/managed-bytes/)、
[`extern-runtime`](../../examples/extern-runtime/)、[`resource-errors`](../../examples/resource-errors/)に置く。
