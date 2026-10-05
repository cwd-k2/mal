# C runtime extension guide

Status: Current v0.7 guide

この文書はgenerated file headerを使うC bodyの読み方と代表形を示す。型とlifecycleは
[C host value API](../spec/c-host-api.md)、buildとcall boundaryは[C runtime extension ABI](../spec/c-host-abi.md)、source-level semanticsは
[`extern`](../spec/extern.md)を正とする。C bodyは安全なFFIの外側にあり、runtime carrierを直接壊せるtrusted extensionである。

## C++とRustとの対応

次の対応は理解のための近似であり、C host APIがC++またはRustのobject modelや静的検査を実装することを意味しない。

| Mal C host API | C++で近いもの | Rustで近いもの | 主な違い |
|---|---|---|---|
| Borrow | reference、non-owning pointer | `&T`、`&mut T` | C compilerはlifetimeも排他性も検査しない |
| `mal_share` | `shared_ptr`のcopy、共有handleのcopy constructor | `Rc::clone`、`Arc::clone` | 同じmanaged identityのresponsibilityを増やし、deep cloneしない |
| `mal_move` | move construction、`std::move` | 通常のmove | sourceをzeroのvacant carrierにし、C compilerは再利用を拒否しない |
| `mal_drop` | destructor、`reset` | `drop`、`Drop` | 明示Dropはsourceをvacantにし、C compilerは二重Dropを拒否しない |
| `mal_owned(T)` | RAII local | owned binding | wrapper型ではなく、C cleanup attributeを付けるoptionalな宣言形 |
| `mal_storage(T)` | type-erased containerのlayoutとoperation table | `Layout`とclone/drop function table | type identity、reflection、dynamic dispatchを持たない |

trivial carrierはC assignmentでcopyでき、Rustの`Copy`やC++のtrivial copyに近い。managed carrierのbitsだけをcopyしてもresponsibilityは
増えず、Borrowとして元のlifetime内でしか使えない。独立して保持する場合は`mal_share`する。C++の一般的なmoved-from objectはvalidだが
値が未指定であるのに対し、`mal_move`後のMal carrierはvacantであり、再び値として使わない。

C++ exceptionやunwind modeのRust panicと異なり、`mal_call_trap`はstackをunwindしない。`mal_owned`のcleanupへtrap時のreleaseを依存させず、
trap前に必要なDropは明示する。

## Symbolをborrowして観測する

```mal
extern writeBytes :: Symbol -> Unit;
```

```c
MAL_DEFINE_writeBytes(call, bytes) {
    if (fwrite(bytes.data, 1, bytes.length, stdout) != bytes.length) {
        mal_call_trap(call, "write failed");
    }
    return mal_unit;
}
```

`bytes`はbody終了まで有効なborrowであり、`owner`をdropしない。call後にも保持する場合はbody中に
`mal_share(call, bytes)`し、保存したresponsibilityを後で`mal_drop`する。`data`だけを保存してもlifetimeは延びない。

## Bufferを構成してmoveする

```mal
extern readBytes :: Unit -> Buffer<UInt8>;
```

```c
MAL_DEFINE_readBytes(call) {
    mal_owned(Buffer) result = mal_buffer(call, mal_storage(mal_type(UInt8)), 4096);
    for (;;) {
        const int byte = fgetc(stdin);
        if (byte == EOF) {
            if (ferror(stdin)) {
                mal_drop(result);
                mal_call_trap(call, "read failed");
            }
            return mal_move(result);
        }
        mal_push(call, result, (mal_type(UInt8))byte);
    }
}
```

`mal_buffer`が返すresponsibilityはC bodyが所有する。`mal_owned(Buffer)`はnormalなscope終了とearly returnでcleanupし、
`mal_move`はresultへ渡したlocalをvacantにする。trapはstackをunwindしないので、trap前のcleanupが必要なら明示的に`mal_drop`する。
`mal_push`などgrowthし得るoperationの後は、以前`mal_data`で得たpointerを再利用しない。

## Bufferをborrowして変更する

```mal
Sample :: (Int64, UInt8);
Samples :: Buffer<Sample>;
extern adjust :: Samples -> Unit;
```

```c
MAL_DEFINE_adjust(call, samples) {
    if (mal_count(samples) == 0) {
        return mal_unit;
    }
    mal_type(Sample) *values = mal_data(samples);
    values[0].field_0 += 1;
    values[0].field_1 += 1;
    return mal_unit;
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
    char *terminated = mal_runtime_allocate(call, path.length + 1);
    memcpy(terminated, path.data, path.length);
    terminated[path.length] = '\0';
    FILE *file = fopen(terminated, "rb");
    mal_runtime_deallocate(terminated);
    if (file == NULL) {
        return (mal_type(OpenResult)){
            .tag = 1,
            .payload.variant_1 = (mal_type(UInt32))errno,
        };
    }
    return (mal_type(OpenResult)){
        .tag = 0,
        .payload.variant_0 = mal_from_bits(mal_type(File), (uintptr_t)(void *)file),
    };
}

MAL_DEFINE_close(call, file) {
    FILE *handle = (FILE *)(void *)mal_bits(file);
    if (fclose(handle) != 0) {
        mal_call_trap(call, "close failed");
    }
    return mal_unit;
}
```

external opaque typeはbitsのcopyだけを行い、resource lifecycleを自動化しない。valid bit pattern、close回数、failure mappingは各operationの
contractが所有する。

## Productとsum

sourceで名前を持つclosed aliasは`mal_type(Alias)`、anonymous productとsumは`mal_product(T, ...)`と`mal_sum(T, ...)`で表す。
productはfieldをsource順に持ち、sumは0-basedの`tag`と`payload.variant_<n>`を持つ。専用constructorはなく、上の例のようにCの
compound literalで直接構成する。managed payloadを入れる場合は、そのfieldへowned responsibilityを直接構成するか`mal_move`する。

`Text :: Buffer<UInt8>`のようなclosed aliasには`mal_type(Text)`も生成される。一方、transparent generic aliasは展開され、すべての
`Buffer<A>`は同じ`mal_type(Buffer)` carrierを使う。element型は
`mal_buffer(call, mal_storage(mal_type(Element)), capacity)`へ明示的に渡すstorage contractに残る。

## `mal_call_t`の範囲

`mal_call_t`は同期body中のruntime allocation、share、trapに使う。Cからmal closureを呼ぶexecution control、program continuation、
async completion tokenではない。pointerと内部stateをbody終了後に保持しない。runtime contextとmanaged carrierはthread-confinedであり、
worker threadを使う場合もbody return前にjoinし、managed resultは元のthreadで構成する。

repository内の実行可能例は[`managed-bytes`](../../examples/managed-bytes/)、
[`extern-runtime`](../../examples/extern-runtime/)、[`resource-errors`](../../examples/resource-errors/)に置く。
