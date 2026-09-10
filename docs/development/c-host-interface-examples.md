# C host interface再設計例

Status: Current v0.6 examples

この文書は[C host ABI](../spec/c-host-abi.md)を代表的なexternal operationへ適用する例を示す。すべてのbodyは
`mal_call_t`を受け、`mal_<T>_t`を通常のC valueとして扱い、型付きresult
operationをC `return` expressionで返す。説明には`T::operation`というabstract notationを使い、C code blockには対応する
`mal_<T>_<operation>` spellingを示す。

## Scalar

```mal
extern increment :: Int64 -> Int64;
extern write :: Int64 -> Unit;
```

```c
MAL_DEFINE_increment(call, value) {
    return mal_Int64_return(call, value + INT64_C(1));
}

MAL_DEFINE_write(call, value) {
    if (printf("%" PRId64 "\n", value) < 0) {
        mal_call_trap(call, "cannot write value");
    }
    return mal_Unit_return(call);
}
```

scalarと`Unit`も他のresultと同じterminal `return`規則を使う。inline後のconversionはidentityへ消える。

## Product and lazy Symbol observation

```mal
Packet :: (UInt64, Symbol);
extern sendPacket :: Packet -> UInt32;
extern sequenceOf :: Packet -> UInt64;
```

```c
MAL_DEFINE_sendPacket(call, packet) {
    mal_span_t payload = mal_Symbol_to_bytes(call, packet.field_1);
    uint32_t error = send_frame(
        packet.field_0,
        payload.data,
        (size_t)payload.length
    );
    return mal_UInt32_return(call, error);
}

MAL_DEFINE_sequenceOf(call, packet) {
    return mal_UInt64_return(call, packet.field_0);
}
```

どちらも`mal_Packet_t`を受ける。`sequenceOf`はSymbol bytesを要求しないためbyte viewを取得しない。

## Sum observation

```mal
ReceiveResult :: [Packet, UInt32];
extern inspect :: ReceiveResult -> UInt32;
```

```c
MAL_DEFINE_inspect(call, result) {
    switch (result.tag) {
        case mal_ReceiveResult_tag_0:
            return mal_UInt32_return(
                call,
                (uint32_t)result.payload.variant_0.field_0
            );

        case mal_ReceiveResult_tag_1:
            return mal_UInt32_return(
                call,
                result.payload.variant_1
            );
    }
    mal_call_trap(call, "invalid ReceiveResult tag");
}
```

inputはvalidなMal sumから構成される。switch末尾はUBを仮定する`unreachable`でなくtrapにする。

## Symbol passthrough and duplication

```mal
Pair :: (Symbol, Symbol);
extern identity :: Symbol -> Symbol;
extern duplicate :: Symbol -> Pair;
```

```c
MAL_DEFINE_identity(call, value) {
    return mal_Symbol_return(call, value);
}

MAL_DEFINE_duplicate(call, value) {
    mal_Pair_t result = {
        .field_0 = value,
        .field_1 = value,
    };
    return mal_Pair_return(call, result);
}
```

hostはclone、retain、dropを扱わない。terminal loweringがresult fieldごとに必要なEngram shareを作る。

## Symbol from host bytes

```mal
extern receive :: Unit -> Symbol;
```

```c
MAL_DEFINE_receive(call) {
    uint8_t storage[256];
    size_t length = receive_bytes(storage, sizeof(storage));

    mal_Symbol_t result = mal_Symbol_from_bytes(
        (mal_span_t){
            .data = storage,
            .length = (uint64_t)length,
        }
    );
    return mal_Symbol_return(call, result);
}
```

`from_bytes`はpure constructionである。result operationがC activation中にbytesをMal-controlled storageへcopyする。

## Input copy and product modification

```mal
Packet :: (UInt64, Symbol);
extern resequence :: Packet -> Packet;
```

```c
MAL_DEFINE_resequence(call, packet) {
    mal_Packet_t result = packet;
    result.field_0 += UINT64_C(1);
    return mal_Packet_return(call, result);
}
```

host valueのcopyとproduct field変更は元のMal valueを変更しない。result operationが新しいresultを確定する。

## Nested result and recoverable failure

```mal
extern Socket;
Packet :: (UInt64, Symbol);
ReceiveResult :: [Packet, UInt32];
extern receivePacket :: Socket -> ReceiveResult;
```

```c
MAL_DEFINE_receivePacket(call, socket) {
    uint8_t storage[MAX_PAYLOAD_SIZE];
    ReceivedFrame received = receive_frame(
        mal_Socket_to_bits(socket),
        storage,
        sizeof(storage)
    );

    if (received.error != 0) {
        return mal_ReceiveResult_return_1(call, received.error);
    }

    mal_Packet_t packet = {
        .field_0 = received.sequence,
        .field_1 = mal_Symbol_from_bytes(
            (mal_span_t){
                .data = storage,
                .length = received.length,
            }
        ),
    };
    return mal_ReceiveResult_return_0(call, packet);
}
```

failure variantではMal allocationを始めない。success result operationだけがnested valueを再帰的にlowerする。

## External opaque capability

```mal
extern File;
OpenResult :: [File, UInt32];
extern openReadOnly :: Symbol -> OpenResult;
```

```c
MAL_DEFINE_openReadOnly(call, pathValue) {
    mal_span_t path = mal_Symbol_to_bytes(call, pathValue);
    char *terminated = terminate_path(path);
    if (terminated == NULL) {
        mal_call_trap(call, "path allocation failed");
    }

    FILE *file = fopen(terminated, "rb");
    uint32_t error = file == NULL ? current_error() : UINT32_C(0);
    free(terminated);

    if (file == NULL) {
        return mal_OpenResult_return_1(call, error);
    }
    return mal_OpenResult_return_0(
        call,
        mal_File_from_bits((uintptr_t)file)
    );
}
```

`File`の有効性、保持、close、failure mappingはExtern authorityに残る。`File::to_bits`と`File::from_bits`はMal storageへ
触れないpure operationである。

## External cleanup before failure

```mal
extern Socket;
SocketPair :: (Socket, Socket);
CreateResult :: [SocketPair, UInt32];
extern createSocketPair :: Unit -> CreateResult;
```

```c
MAL_DEFINE_createSocketPair(call) {
    int sockets[2];
    if (socketpair(AF_UNIX, SOCK_STREAM, 0, sockets) != 0) {
        return mal_CreateResult_return_1(call, current_error());
    }

    if (!configure_socket(sockets[0]) || !configure_socket(sockets[1])) {
        uint32_t error = current_error();
        close(sockets[1]);
        close(sockets[0]);
        return mal_CreateResult_return_1(call, error);
    }

    mal_SocketPair_t pair = {
        .field_0 = mal_Socket_from_bits((uintptr_t)sockets[0]),
        .field_1 = mal_Socket_from_bits((uintptr_t)sockets[1]),
    };
    return mal_CreateResult_return_0(call, pair);
}
```

result transfer前のexternal resourceはadapterが片付ける。generic `mal_call_t` cleanup stackへ移さない。

## Nested sum construction

```mal
Status :: [Unit, UInt32];
Envelope :: (UInt64, Status);
extern statusEnvelope :: UInt32 -> Envelope;
```

```c
MAL_DEFINE_statusEnvelope(call, error) {
    mal_Status_t status = error == 0
        ? mal_Status_make_0()
        : mal_Status_make_1(error);

    mal_Envelope_t envelope = {
        .field_0 = UINT64_C(42),
        .field_1 = status,
    };
    return mal_Envelope_return(call, envelope);
}
```

`make`はvalidなnested sum host valueを作るpure operation、`return`はMal resultを確定するoperationである。

## Lifetime composition

```mal
extern Handle;
Mixed :: (Symbol, Handle, UInt64);
extern inspectMixed :: Mixed -> UInt64;
```

```c
MAL_DEFINE_inspectMixed(call, mixed) {
    mal_span_t bytes = mal_Symbol_to_bytes(call, mixed.field_0);
    remember_handle_if_contract_allows(mal_Handle_to_bits(mixed.field_1));
    return mal_UInt64_return(
        call,
        mixed.field_2 + bytes.length
    );
}
```

`bytes`とSymbol valueはcallを越えて保持しない。scalarは通常のcopyであり、Handleの保持可否はそのExtern contractが定める。
Handleを保持してもreferent lifetimeは自動では延長しない。

## Optional direct Symbol output

基本経路のcopyが独立したcost centerだと測定された場合だけ、result-owned storageへ直接書くadvanced pathを検討する。この経路は
unfinished outputのcleanup stateを必要とするため、基本のstateless `mal_call_t`とは分ける。

## Exampleから確認する性質

- bodyで特別扱いするcurrent-call objectは`mal_call_t`だけである。
- parameter、local、nested field、result descriptionは同じ`mal_<T>_t`規則を使う。
- authorityを必要とするoperationだけが`mal_call_t *`を受け取る。
- scalar、product、sum、Symbolのresultは同じC `return` patternを使う。
- Symbolのprovenance、raw carrier、storage表現、reference count、admission stateをhost codeへ出さない。
- productは通常のC valueとしてcopy、変更、再構成できる。
- sumは`make_<variant>`と`return_<variant>`でvalid tagを構成する。
- Symbol viewとExtern capabilityで異なるlifetimeをaggregateのleafごとに適用する。
- Engram result constructionとExtern cleanupを統合しない。
- `frame`、`Observe` namespace、専用return carrier、generic owner listを要求しない。
