# C host APIの利用scenario監査

Status: Historical design record; implemented in v0.7

この文書は[C host lifecycle操作を小さな語彙へまとめる案](c-host-lifecycle-ergonomics.md)をrepositoryのexampleへ適用し、
genericなMal abstractionを除く処理をC externへ移した場合に不足する表現を調べる。現在のABI規範ではなく、proposal採択前の
coverage記録であり、現行規範は[C runtime extension ABI](../../spec/c-host-abi.md)を正とする。

v0.7 ABIは未公開なので、この案を採択しても`MAL_C_ABI_VERSION`は`0x000a00`のままとする。旧helperは残さず置き換える。

## 判定の範囲

「C externへ移す」には二つの強さがある。closed concrete parameterとresultだけを持つ処理は同じsignatureのexternへ置き換えられる。
一方、function valueをparameterまたはresultに持つbindingは、genericでなくても現行extern admissionを通らない。Cへmal closureを渡すには
code、environment、responsibility、callbackからのresumeを定める別のexecution ABIが必要になる。

したがって全bindingの一対一置換は目標にしない。function-valuedな内部境界をC body内へ融合し、exampleの入力と最終resultを同じclosed
carrierで保つ置換をcoverage条件とする。これはgeneric externやcallback/reentryを暗黙に導入しない。

## 想定するsurface

代表形は次とする。

```c
mal_type(T)                 /* generated C carrier type */
mal_product(T, ...)         /* collected product carrier type */
mal_sum(T, ...)             /* collected sum carrier type */
mal_owned(T) value          /* optional managed lexical cleanup */
mal_storage(T)              /* static storage/lifecycle descriptor */

mal_symbol(call, bytes, length)
mal_buffer(call, mal_type(Element), capacity)

mal_share(call, value)
mal_move(value)
mal_drop(value)

mal_push(call, buffer, value)
mal_replace(call, buffer, index, value)
mal_fill(call, buffer, offset, count, value)
mal_copy(call, destination, destination_offset, source, source_offset, count)
mal_append(call, buffer, source, count)
mal_extend(call, buffer, count)
mal_truncate(buffer, count)
mal_reserve(call, buffer, capacity)
mal_data(buffer)
mal_count(buffer)
mal_snapshot(call, buffer)
```

`push`、`replace`、`fill`はowned operandをconsumeする。productとsumはC initializerで構成し、managed fieldへowned localを入れる場合は
`mal_move`、borrowを入れる場合は`mal_share`を明示する。
`copy`と`append`はsourceをborrowし、destinationが必要とするelement shareを作る。`extend`はtrivial elementだけに未初期化領域を作る。

## I/Oとmanaged bytes

`managed-bytes`、`resource-errors`、`json-query`、`compiler-pipeline`の入力は、一時host allocationと要素ごとのpushを必要としない。

```c
MAL_DEFINE_readStdin(call) {
    mal_owned(Buffer) bytes = mal_buffer(call, mal_type(UInt8), 4096);
    for (;;) {
        size_t start = mal_count(bytes);
        mal_type(UInt8) *tail = mal_extend(call, bytes, 4096);
        size_t read = fread(tail, 1, 4096, stdin);
        mal_truncate(bytes, start + read);
        if (read == 0) {
            if (ferror(stdin)) mal_call_trap(call, "cannot read stdin");
            return mal_move(bytes);
        }
    }
}
```

static byte arrayやstack bufferには`mal_append`を使う。immutable snapshotには`mal_snapshot(call, bytes)`を使う。これは
`Buffer<UInt8>`をborrowしてSymbolを返し、後のBuffer mutationから独立したbytesを保持する。単純実装は`mal_symbol(call,
mal_data(bytes), mal_count(bytes))`であり、unique storage再利用はobservable semanticsを変えないoptimizationである。

Symbol parameterはborrowのまま`data`と`length`をC libraryへ渡せる。NUL終端pathはC activation内に一時領域を作る。FILE、descriptor、
errnoはexternal opaque carrierとsum resultへ写し、`mal_drop`へresource closeを混ぜない。

## Aggregateとmanaged element

`extern-runtime`の`Buffer<Sample>`、`json-query`のparser state、`compiler-pipeline`のresult、concrete化したmap entryは同じ規則で扱える。

```c
mal_owned(Buffer) numbers = mal_buffer(call, mal_type(UInt64), 8);
mal_owned(Symbol) label = mal_symbol(call, "values", 6);
mal_owned(Entry) entry = {
    .field_0 = mal_move(numbers),
    .field_1 = mal_move(label),
};
mal_owned(Buffer) entries = mal_buffer(call, mal_type(Entry), 4);
mal_push(call, entries, mal_move(entry));
```

outer BufferはEntryのsize、alignment、share、dropをdescriptorから得る。Entryのglueはinner BufferとSymbolへ型再帰する。
`mal_type(Entry) *data = mal_data(entries)`で直接観測でき、growth後だけpointerを再取得する。

`Buffer<Buffer<UInt8>>`もouterを`mal_buffer(call, mal_type(Buffer), capacity)`で作り、UInt8 descriptorを保持するinner Bufferを
`mal_push`できる。innerのelement型はC carrierから消えるため、この組合せの整合性はunsafeなC bodyが負う。

sumはgenerated carrierのtagとactive payloadを直接初期化する。たとえばanonymousな`[Unit, UInt64]`は
`(mal_sum(mal_type(Unit), mal_type(UInt64))){ .tag = 1, .payload.variant_1 = value }`と書ける。dropとmoveはactive payloadだけを
辿る。Malのsum variantには宣言上の名前がないため、C APIもoperation-localな`failure`などを型の恒久名として生成しない。

## Indexed storage

`indexed-graph`はscalar Bufferのdata pointerを使うだけでも実装できる。proposalの一般APIとしては次を区別する。

- `mal_share`は同じBuffer identityへのresponsibilityを増やす。
- `mal_copy`は既存destinationのrangeへelementをcopyし、managed elementならshareする。
- 新しいidentityへのsnapshotは`mal_buffer`と`mal_copy`の組で表す。
- `mal_fill`は一つのowned operandをconsumeし、必要な個数のresponsibilityを内部で作る。
- `mal_replace`は旧elementをdropして新しいowned operandをconsumeする。

これでCSR columnのsnapshot、distance/visited columnのfill、frame stackのpush/replace/truncateを同じsurfaceで表せる。座標の意味、
offset単調性、joined column lengthなどdomain invariantはC implementationが所有し、type descriptorへ入れない。

## Example別coverage

| Example | Cへ移せる範囲 | 必要なsurfaceまたは境界 |
|---|---|---|
| `managed-bytes` | 同じextern signatureとmain相当の処理 | byte bulk append/extend、snapshot、Symbol borrow |
| `extern-runtime` | 全処理 | product carrier、typed descriptor、push、data、replace |
| `resource-errors` | 全処理 | external opaque bits、sum carrier、明示close、bulk byte input |
| `indexed-graph` | generic `_copyBuffer`をconcrete化した全処理 | scalar data、fill、copy、replace、複数Bufferを持つopaque carrier |
| `json-query` | generic loopをspecializeし、renderer選択をC内で融合した全処理 | Symbol view、sum/product/opaque lifecycle、frame Buffer、byte input |
| `compiler-pipeline` | compilerとI/Oを同じclosed resultのexternへ移せる | growable byte Buffer、snapshot、sum result、external I/O failure |
| `control-and-iteration` | concrete recursionと各specialized scenario | generic callback境界は除外し、C loopへ融合する |
| `language-tour` | numeric、sum、aggregate、Bufferのscenario | `makeAdder`のclosure resultは同じsignatureでextern化せず、生成とapplicationを融合する |
| `generic-map` | concrete specializationのstorage処理だけ | generic family選択はMalに残し、concrete entryはsum lifecycleで扱える |
| `monads-and-comonads` | concrete scenarioを一つのC bodyへ融合する場合だけ | generic familyとfunction-valued carrierはextern境界に出さない |

## 監査で加わった要件

最初のshare/drop案だけではexample coverageを満たさない。採択には次も必要になる。

- named closed typeは`mal_type(T)`、anonymous aggregateは`mal_product(T, ...)`と`mal_sum(T, ...)`でcarrierを参照できる。
- generic alias applicationはCへ残さず、aliasとopaque representationを展開したclosed carrierだけを公開する。
- `Buffer<T>`のcarrierは`mal_type(Buffer)`へ消去し、TはBuffer生成時のstorage contractにだけ使う。
- storage descriptorへalignmentを含め、Buffer allocationとdata pointerがelement alignmentを満たす。
- Bufferがstorage descriptorを保持し、managed/trivial別の公開operation名をなくす。
- bulk append、trivial extend、truncate、reserveを提供する。
- productとsumはpublic fieldを持つC initializerで直接構成できる。
- aggregateを含むshare、move、dropを一つのLLVM lifecycle planから生成する。
- Symbol snapshotをbyte Bufferから直接作れる。
- external opaqueのbits変換を型tokenで書けるが、resource destructorは自動化しない。
- extern wrapperのresult validationとbodyのownership transferを分離し、型別return helperをなくす。

## 結論

型別function名を増やさず、上記surfaceでrepositoryのnon-generic data workloadは表現できる。特にnested managed aggregate、sum、
byte I/O、indexed columns、external resource failureは同じdescriptorとownership語彙へ収まる。

表現できないのはfunction valueを同じsignatureのままCへ渡す境界である。これはC syntaxやlifecycle helperの不足ではなく、Cからmalへ戻る
execution controlが存在しないためであり、本proposalへcallback ABIを混ぜない。example全体をCへ移す試験では、その境界をC body内へ
融合するか、function-valued部分をMalに残す。
