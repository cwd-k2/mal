# C host interface例

Status: Current ABI 0x000800 examples for the mal v0.6 development profile

この文書は[C host ABI](../spec/c-host-abi.md)を代表的なexternal operationへ適用する例を示す。public headerへ出るのは
[`HostMappable`](../spec/extern.md#host-mappable-type)な型だけである。すべてのbodyは`mal_call_t`を受け、型付きresult operationを
C `return` expressionで返す。

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

## Borrowed readable bytes

```mal
ReadableBytes :: (Address, USize);
extern writeBytes :: ReadableBytes -> USize;
```

```c
MAL_DEFINE_writeBytes(call, bytes) {
    size_t written = fwrite(bytes.field_0, 1, bytes.field_1, stdout);
    if (written == 0 && bytes.field_1 != 0 && ferror(stdout)) {
        mal_call_trap(call, "cannot write bytes");
    }
    return mal_USize_return(call, written);
}
```

`bytes.field_0`は少なくとも`bytes.field_1` bytesを読めるというoperation contractを持つ。hostはpointerをbody return後に保持せず、
変更も解放もしない。resultは消費したprefixの長さであり、mal側がremainderを再送するかを決める。

## Borrowed writable bytes

```mal
WritableBytes :: (Address, USize);
extern readBytes :: WritableBytes -> USize;
```

```c
MAL_DEFINE_readBytes(call, bytes) {
    size_t length = fread(bytes.field_0, 1, bytes.field_1, stdin);
    if (length == 0 && ferror(stdin)) {
        mal_call_trap(call, "cannot read bytes");
    }
    return mal_USize_return(call, length);
}
```

hostはcapacity以下のprefixだけを初期化する。mal側はresultをcapacity以下とするcontractを信頼し、そのprefixをRegionからPackedへ
admitしてから外部bufferを再利用できる。host-owned pointerを`Symbol` resultとして返さない。

## Product and sum

```mal
Packet :: (UInt64, Address, USize);
SendResult :: [USize, UInt32];
extern sendPacket :: Packet -> SendResult;
```

```c
MAL_DEFINE_sendPacket(call, packet) {
    Transfer sent = send_frame(packet.field_0, packet.field_1, packet.field_2);
    if (sent.error != 0) {
        return mal_SendResult_return_1(call, sent.error);
    }
    return mal_SendResult_return_0(call, sent.length);
}
```

productはsource orderのfieldを持つ。sum resultはvariant-specific terminal returnで構成し、host codeがtagを直接組み立てる必要を
なくす。Addressの範囲とpermissionはPacketの構造から推測せず、`sendPacket`のcontractが定める。

## Canonical memory

```mal
Sample :: (Int64, UInt8);
extern updateSample :: Address -> Unit;
```

```c
MAL_DEFINE_updateSample(call, address) {
    mal_Sample_t sample = mal_Sample_read(call, address, 0);
    sample.field_0 += 1;
    mal_Sample_write(call, address, 0, sample);
    return mal_Unit_return(call);
}
```

`address`がcanonical `Sample`列の先頭を指すというcontractは`updateSample`が定める。helperはunaligned access、field padding、sum tagを
compilerのtarget layoutに従って処理する。`mal_Sample_t *`へcastせず、extent、permission、lifetimeは別途保証する。

## External opaque capability

```mal
extern File;
OpenResult :: [File, UInt32];
PathBytes :: (Address, USize);
extern openReadOnly :: PathBytes -> OpenResult;
```

```c
MAL_DEFINE_openReadOnly(call, path) {
    char *terminated = copy_and_terminate(path.field_0, path.field_1);
    if (terminated == NULL) {
        mal_call_trap(call, "path allocation failed");
    }

    FILE *file = fopen(terminated, "rb");
    uint32_t error = file == NULL ? current_error() : UINT32_C(0);
    free(terminated);

    if (file == NULL) {
        return mal_OpenResult_return_1(call, error);
    }
    return mal_OpenResult_return_0(call, mal_File_from_bits((uintptr_t)file));
}
```

`File`の有効性、保持、close、failure mappingはExtern authorityに残る。`to_bits`と`from_bits`はresourceをallocate、clone、close、
freeしない。pathのAddressはcall-scopedだが、正常resultのFile capabilityはoperation contractが定める期間だけ有効である。

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

## Exampleから確認する性質

- bodyで特別扱いするcurrent-call objectは`mal_call_t`だけである。
- parameter、local、nested field、resultは同じ`mal_<T>_t`規則を使う。
- public aggregateとopaque型はHostMappableなextern surface、またはentry sourceのcanonical memory helper対象aliasから到達する。
- byte列はAddressと長さで借り、Symbol、Packed、Region、managed ownerをhost codeへ出さない。
- productは通常のC valueとしてcopy、変更、再構成できる。
- sumは`make_<variant>`と`return_<variant>`でvalid tagを構成する。
- Engramのadmission、Extern capabilityのtransfer、Extern cleanupを一つのownershipへ統合しない。
