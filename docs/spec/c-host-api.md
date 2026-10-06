# C host value API

Status: Accepted ABI 0x000b00 for mal v0.7

この文書はC runtime extensionが扱うcarrier、type form、responsibility、storage descriptor、公開primitiveを定める。buildとexternal
operationのcall boundaryは[C runtime extension ABI](c-host-abi.md)、source-level managed valueは[Engram](engrams.md)と
[`Buffer`](memory.md)を正とする。

## Public surfaceの分類

public surfaceは、記述上の役割とruntime contextの有無で次の四種類へ分ける。

| 種類 | 名前または形式 | `call` |
|---|---|---|
| 型マクロ | `mal_type`、`mal_product`、`mal_sum`、`mal_owned` | 取らない |
| 定数 | `mal_false`、`mal_true`、`mal_unit` | 取らない |
| 演算子 | `mal_storageof`、`mal_share`、`mal_move`、`mal_drop`、`mal_from_bits`、`mal_to_bits` | 取らない |
| Call API | runtime capability、Symbol構築、Buffer operation | 有効な`call`を常に先頭に取る |

Call APIでは、現在の実装がallocationやtrapを行うかによらず、runtimeまたはruntime-managed valueを扱うoperationへ同じ規則を
適用する。各系統と所属するoperationは次のとおりである。

| 系統 | operation |
|---|---|
| Runtime capability | `mal_allocate`、`mal_deallocate`、`mal_trap` |
| Symbol construction | `mal_symbol`、`mal_snapshot` |
| Buffer construction | `mal_buffer` |
| Buffer observation | `mal_data`、`mal_count` |
| Buffer element mutation | `mal_push`、`mal_replace`、`mal_fill` |
| Buffer range transfer | `mal_copy`、`mal_append` |
| Buffer extent | `mal_extend`、`mal_truncate`、`mal_reserve` |

## Runtime carrier

primitive、Symbol、Buffer、product、sum、external opaque typeはLLVM moduleと同じruntime carrier layoutを持つ単一のC carrierを使う。
fixed-width integerは`stdint.h`の対応幅、`Float32`はbinary32 `float`、`Float64`はbinary64 `double`、`ByteSize`と`USize`はtarget
pointer index幅の`size_t`を使う。Symbolはowner、active data、length、Bufferはstable runtime objectへのhandleである。

generated file headerは、そのfileのextern signatureから到達するconcrete runtime carrierと、宣言元fileが所有するclosed aliasと
file-local opaque representationを出す。実在するC identifierにはstructural fingerprintを使うが、host bodyはそのmanglingを組み立てない。
別fileの宣言追加やgraph load orderでfingerprintを変えない。

C type spellingはaliasとfile-local opaque representationを展開した後のruntime carrierを表し、Malのsurface type applicationを再現しない。
transparent generic aliasは展開するため`mal_generic`やaliasごとの`*_of`を生成しない。sourceで名前を与えたclosed aliasだけはcanonical
carrierと互換な`mal_type(Name)`として残す。すべての`Buffer<T>`は`mal_type(Buffer)`へ写し、`T`はBuffer生成時に渡すstorage
descriptorにだけ残す。

generated headerはpointerとindex幅、Symbolのsizeとfield offsetを`_Static_assert`し、aggregateは同じtarget C ABIのrecord layoutを
LLVM側のlayout planにも使う。compiler-facing bridgeとhost bodyの間に別のraw carrier、field-wise変換、return helperを置かない。
このlayoutを別artifact、network、永続storageのencodingとして使用しない。

## Type form

type formはCの型を書く位置で使い、評価時のlifecycle effectやruntime type metadataを持たない。

| form | 表すC carrier型 | 使用範囲 |
|---|---|---|
| `mal_type(Name)` | named closed Mal型 | builtin、source alias、file-local opaque type |
| `mal_product(T0, ...)` | ordered fieldを持つanonymous product | generated headerが公開したreachable structural product |
| `mal_sum(T0, ...)` | ordered variantを持つanonymous sum | generated headerが公開したreachable structural sum |
| `mal_owned(Name)` | `mal_type(Name)`にlexical cleanupを付けたlocal declaration | cleanupが生成されたnamed managed type |

型マクロと`mal_storageof`へ渡すpublic carrier type form `T`は次で構成する。

```text
T ::= mal_type(Name)
    | mal_product(T, ...)
    | mal_sum(T, ...)
```

`mal_owned(Name)`はcleanupが生成されたnamed managed typeだけを受け、anonymous structural typeや任意のC型を受けるtype
constructorではない。

```c
mal_product(mal_type(Int64), mal_type(UInt8)) sample = {
    .field_0 = INT64_C(42),
    .field_1 = UINT8_C(7),
};

mal_sum(mal_type(Unit), mal_type(Symbol)) choice = {
    .tag = UINT32_C(0),
    .payload.variant_0 = mal_unit,
};
```

productはsource orderの`.field_N`、sumは0-basedの`.tag`と`.payload.variant_N`を持つ。専用constructorはなく、C initializerで直接構成する。
named closed aliasには`mal_type(Alias)`を優先し、anonymous formのargument列からgenerated identifierを推測しない。managed fieldへはowned
responsibilityを直接構成するかMoveする。

`mal_owned(Name)`はwrapper型ではない。builtin managed leafはcommon headerのcleanupを使い、generated headerはmanaged aggregateごとの
cleanupとnamed aliasからそれへのmappingを生成する。scope終了、early return、明示Drop、Moveを同じvacant規則で扱うが、
`mal_trap`はstack unwindingしない。external opaque resourceやtrivial typeへ`mal_owned`を提供しない。

`mal_false`と`mal_true`だけがvalidなBool carrierである。Cがinvalid Bool、sum tag、inactive payload、owner、Symbol viewを構成した後の挙動は
保証しない。

## External opaque carrier

external opaque typeはpublicな`.bits` fieldを持つone-machine-word carrierである。

| primitive | result / effect |
|---|---|
| `mal_from_bits(mal_type(T), bits)` | `uintptr_t`からopaque carrierを構成する |
| `mal_to_bits(value)` | opaque carrierを`uintptr_t`へ戻す |

変換はlosslessであり、generated headerは型別変換functionを生成しない。resourceのallocate、clone、close、free、valid bit patternは
operation固有contractが定める。carrier bitsのcopyはresource lifecycleを実行しない。

## Storage query operator

`mal_storageof(T)`はC carrier型`T`から`mal_storage_descriptor_t`値を返す。descriptorはsize、alignment、storage用Share / Drop callbackを持つが、
registry、型名、dynamic type equality、値、responsibilityを持たない。

```c
mal_storage_descriptor_t storage = mal_storageof(mal_type(Symbol));
```

これはCのoperatorではないが、利用上は`sizeof(T)`や`_Alignof(T)`に近いtype queryである。評価してもallocation、carrierの格納、Share、
Move、Dropを実行せず、storage objectや要素pointerも返さない。type-erased containerがdescriptorを値として保持し、後のoperationで
carrierを扱う。copyは`share`、place終了は`drop`を呼び、relocationのMoveはcallbackを呼ばずcarrierを移す。

typedな`mal_owned(T)` localではC compilerがcleanupを静的に選ぶため、local自身はstorage descriptorを保持しない。

## Borrowとlifecycle operator

Borrowにはoperatorがない。managed extern parameter、primitive表でborrowと記したoperand、responsibilityを持つ値の有効期間内だけ使う
carrier copyは、新しいresponsibilityを作らない通常のCの値として渡す。Share、Move、Dropだけが明示的なlifecycle operatorである。

| mode | C上の形 | sourceの状態 | resultのresponsibility |
|---|---|---|---|
| Borrow | 通常のparameter渡し、観測、carrier copy | liveのまま | 作らない |
| Share | `mal_share(value)` | liveのまま | 新しい一つを作る |
| Move | `mal_move(value)` | vacantになる | sourceの一つを移す |
| Drop | `mal_drop(value)` | vacantになる | 返さず一つを終了する |

Copyは独立したlifecycle modeではない。trivial carrierはC assignmentでbitsをcopyし、managed carrierを独立して保持する論理的なcopyは
Shareとして新しいresponsibilityを作る。Shareは同じmanaged identityを共有し、汎用的なdeep cloneを意味しない。Buffer primitiveの
`mal_copy`はrange名であり、汎用value copy operatorではない。

borrowしたcarrierはsourceのresponsibilityより長く使わず、`mal_move`、`mal_drop`、owned operandをconsumeするprimitiveへ渡さない。
call後に保存する場合やmanaged resultとして返す場合は、borrow中に`mal_share`する。carrier bitsだけをcopyしてもlifetimeは延びず、
そのcopyをowned valueとして扱えない。C compilerとruntimeはこの区別を追跡しない。

`mal_move`と`mal_drop`はowned lvalueだけを受け、一度だけ評価する。productではmanaged field、sumではactive payloadだけへgenerated glueが
再帰する。Cが保持したresponsibilityは同じthread上で後に`mal_drop`する。

`mal_share`はallocationせず、有効なresponsibilityに対して失敗しない。Share callbackもcontext-freeであり、representation固有の
reference count overflowはpublicなcall capabilityを必要とするfailureではなく、context-freeなinternal fatal failureとして扱う。

## Runtime capability primitives

| primitive | result / effect |
|---|---|
| `mal_allocate(call, size)` | runtimeと同じallocatorからraw storageを確保する |
| `mal_deallocate(call, allocation)` | `mal_allocate`のstorageを解放する |
| `mal_trap(call, message)` | 回復不能failureとしてprocessを終了し、returnしない |

allocationはmanaged valueやresponsibilityを構成せず、raw storageの用途と内容はhostが管理する。確保と解放はいずれもactiveなcall
activation内で行う。`mal_trap`はstackをunwindしない。

## Symbol primitive functions

| primitive | source responsibility | result / effect |
|---|---|---|
| `mal_symbol(call, source, length)` | `source`が指すbyte rangeをcall中だけborrow | byteをcopyした独立したowned Symbolを返す |
| `mal_snapshot(call, buffer)` | `Buffer<UInt8>`とそのactive elementをborrow | 後のBuffer mutationから独立したowned Symbolを返す |

どちらもsourceのlifetimeを延長せず、返したSymbolがcopy先のbytesを所有する。`mal_snapshot`の`buffer`は
`mal_storageof(mal_type(UInt8))`で構築されていなければならない。

## Buffer primitive functions

Buffer primitiveは生成時に明示されたstorage descriptorをobjectへ保持し、以後の要素operationで利用する。表中の`buffer`、`source`、
`destination`はBorrowであり、Buffer自体のresponsibilityを移さない。

| primitive | element responsibility | result / effect |
|---|---|---|
| `mal_buffer(call, storage, capacity)` | `storage`を値として保持 | count 0、指定capacityのowned Bufferを返す |
| `mal_data(call, buffer)` | なし | element storageへの`void *`をborrowする |
| `mal_count(call, buffer)` | なし | 現在のelement countを返す |
| `mal_push(call, buffer, element)` | owned `element`を末尾へMove | 追加したindexを返し、countを1増やす |
| `mal_replace(call, buffer, index, element)` | 既存要素をDropし、owned `element`をMove | countを変えず指定indexを置換する |
| `mal_fill(call, buffer, offset, count, element)` | owned `element`をconsumeし、必要なら追加分をShare。既存要素はDrop | rangeを同じ値で埋め、必要ならcountを伸ばす |
| `mal_copy(call, destination, destination_offset, source, source_offset, count)` | source rangeをborrowし、managed要素をShare。置換要素はDrop | Buffer間でrangeをcopyし、必要ならdestinationを伸ばす |
| `mal_append(call, buffer, source, count)` | pointer rangeをborrowし、managed要素をShare | rangeを末尾へ追加する |
| `mal_extend(call, buffer, count)` | trivial elementだけを許可 | countを増やし、未初期化の新しい末尾への`void *`を返す |
| `mal_truncate(call, buffer, count)` | 取り除くmanaged要素をDrop | countを縮め、現在値以上なら何もしない |
| `mal_reserve(call, buffer, capacity)` | なし | element capacityを確保し、countは変えない |

`mal_copy`の二つのBufferは同じstorage descriptorを持たなければならない。`mal_append`のsource pointerはdestinationのdescriptorと一致する
elementを`count`個指さなければならない。runtimeはdescriptorのdynamic equalityを検査しない。

growthし得るoperation後は以前のelement pointerを使用せず再取得する。descriptorと異なるpointer型でのaccess、managed elementのraw
overwrite、`mal_extend`で作った未初期化placeの観測、runtime以外によるBuffer object/data allocationのfreeはcontract違反である。
