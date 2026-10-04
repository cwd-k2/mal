# C host lifecycle操作を小さな語彙へまとめる案

Status: Exploratory

この文書は、runtime extensionとなったextern Cで型別lifecycleとBuffer操作を記述しやすくする後続案を記録する。
現在の規範は[C runtime extension ABI](../spec/c-host-abi.md)、compiler内の型再帰は
[Engram lifecycle loweringの拡張境界](engram-lifecycle-foundation.md)を正とする。本案はまだABIではない。
example全体へ適用した結果は[利用scenarioによる監査](c-host-api-scenarios.md)に記録する。

v0.7 ABIはまだ公開されていないため、採択時も`MAL_C_ABI_VERSION`は`0x000a00`のまま既存surfaceを置き換える。
旧spellingとの互換helperや段階的移行期間は設けない。

## 現在の摩擦

scalarとborrowedなSymbolの観測は短い一方、managed aggregateとmanaged element Bufferの構築には、型再帰した
retain/release callback、stride、`share`、`drop`、`new_managed_move`、`return_move`をC bodyが個別に選ぶ必要がある。
同じownership transferがoperation名のsuffixとして増えると、正しい組合せを利用者が再構成するAPIになる。

LLVM backendはすでにconcrete typeごとの再帰的なshare/dropを生成できる。これをprogram固有のC-callable glueとして出し、
generated headerはそのglueを使う薄いtyped APIだけを生成する。C macroへaggregate lifecycleそのものは実装しない。

## Carrier型のspelling

C headerはMalのsurface type applicationではなく、extern admissionがaliasとfile-local opaque representationを展開した後のruntime
carrierを公開する。named scalar、external opaque、Symbol、Bufferと、sourceで名前を与えたclosed typeは`mal_type(T)`で綴る。
これはdescriptor式ではなくCのtype tokenへ展開するsyntax-like macroである。

```c
mal_type(UInt64) number;
mal_type(Symbol) label;
mal_type(Buffer) bytes;
mal_type(Entry) entry;
```

source aliasはruntime identityを作らないが、source signatureとC bodyを対応させるspellingとして残す。たとえば
`Text :: Buffer<UInt8>;`には`mal_type(Text)`を生成し、`mal_type(Buffer)`と互換なtypedefへ展開する。Text自身を作るときのelementは
UInt8なので`mal_buffer(call, mal_type(UInt8), capacity)`を使い、`Buffer<Text>`を作る場合は
`mal_buffer(call, mal_type(Text), capacity)`を使う。

anonymous productとsumは、正規化後のcarrier型を引数に取る`mal_product(T, ...)`と`mal_sum(T, ...)`で綴る。これらはC compile時に
新しいruntime typeを合成せず、generated headerがextern signatureから収集したclosed carrierへ展開する。収集されていない組合せは
C compile errorになる。

```c
mal_product(mal_type(UInt8), mal_type(Int64)) pair;
mal_sum(mal_type(Unit), mal_type(UInt64)) result;
```

transparent generic aliasはCへ残さない。`Result<A> :: [Unit, A];`に対する`Result<UInt64>`は
`mal_sum(mal_type(Unit), mal_type(UInt64))`へ正規化する。名前が必要ならsourceに
`UIntResult :: Result<UInt64>;`とclosed aliasを置き、`mal_type(UIntResult)`を使える。`mal_generic`、aliasごとの`*_of`、
`mal_result(operation)`、`mal_parameter(operation)`は生成しない。

すべての`Buffer<T>`は同じhandle carrierなので、C type positionでは`mal_type(Buffer)`へ消去する。element型はBuffer objectが保持する
storage descriptorにだけ残る。このため`mal_buffer_of(T)`も型spellingには導入しない。productやsumに含まれるBuffer fieldも
`mal_type(Buffer)`であり、element型の整合性はunsafeなC bodyのcontractになる。

runtimeへstorage contractを値として渡す場合だけ`mal_storage(T)`を使う。これはsize、alignment、share、dropを持つ
`mal_storage_descriptor_t`への
pointerであり、registry、型名検索、dynamic type equalityを提供しない。通常の`mal_buffer`は受け取った`mal_type(T)`からdescriptorを
内部で選ぶため、host bodyが`mal_storage`を直接使うのはtype-erasedなC helperを書く場合に限る。`ops`はdescriptor内部とruntime実装の
用語に留め、`value`は実体carrierと区別するため、どちらもpublic descriptor macro名には使わない。

```c
static mal_type(Buffer) make_list(
    mal_call_t *call,
    const mal_storage_descriptor_t *element,
    size_t capacity
) {
    return mal_buffer_dynamic(call, element, capacity);
}

mal_type(Buffer) entries = make_list(call, mal_storage(Entry), 16);
```

Mal sourceのextern signatureでgeneric applicationにaliasを要求しない。たとえば`extern readLine :: Unit -> Buffer<UInt8>;`のC bodyは
`mal_buffer(call, mal_type(UInt8), capacity)`でresultを構築でき、`Text :: Buffer<UInt8>;`を追加する必要はない。Buffer carrierの
share、move、dropはelement型に依存せず、element descriptorは各Buffer objectが保持する。

同じ規則はnested Bufferにも適用する。`Buffer<Buffer<UInt8>>`のouter BufferはelementとしてBuffer carrierのdescriptorだけを持ち、
各inner Bufferが自身のUInt8 descriptorを持つ。

```c
mal_owned(Buffer) inner = mal_buffer(call, mal_type(UInt8), 4096);
mal_owned(Buffer) outer = mal_buffer(call, mal_type(Buffer), 4);
mal_push(call, outer, mal_move(inner));
```

C carrier上では`Buffer<UInt8>`と`Buffer<Symbol>`を静的に区別しない。異なるelement descriptorを持つinner Bufferを誤って入れることも
できるが、externは安全性を提供しないruntime extensionなので、その整合性はC bodyのcontractとする。

aliasが読みやすさに効くのは、同じstructural carrierをdomain名で繰り返し使うときである。aliasがなくても
`mal_product`と`mal_sum`を使えるが、`Entry :: (Buffer<UInt64>, Symbol);`のようにsource名を与えれば`mal_type(Entry)`と書ける。

```mal
extern f :: Unit -> (UInt8, Int64);
```

```c
MAL_DEFINE_f(call) {
    return (mal_product(mal_type(UInt8), mal_type(Int64))){
        .field_0 = UINT8_C(7),
        .field_1 = INT64_C(42),
    };
}
```

## 最小のownership語彙

C surfaceで直接使うownership動詞は次の三つを候補とする。

- `mal_share(call, value)`はborrowまたはowned valueを保ったまま、新しいowned responsibilityを返す。
- `mal_drop(value)`はowned lvalueのresponsibilityを終了し、元をvacantにする。
- `mal_move(value)`はowned lvalueからresponsibilityを取り出し、元をvacantにする。

parameterとprojectionは既定でborrowなので`mal_borrow`は設けない。`retain`と`release`はruntime実装名に留め、host APIでは
responsibilityの意味を表す`share`と`drop`を使う。`initialize`と`replace`もtyped place APIが必要になるまで公開しない。

container operationはownershipごとの派生名を増やさず、owned argumentをconsumeする一形にする。

```c
mal_type(Buffer) words = mal_buffer(call, mal_type(Symbol), 2);
mal_push(call, words, mal_symbol(call, "mal", 3));
mal_push(call, words, mal_share(call, borrowed));
mal_push(call, words, mal_move(owned));
```

これにより`push_move`と`push_share`、`put_move`と`put_share`の組を作らずに済む。resultもcarrierを統一できればCの
`return`自体をtransfer境界とし、automatic cleanupを使うlocalだけ`return mal_move(result);`と書ける。Boolやsum tagの検査は
generated wrapper側へ置き、ownership transferのためだけの`return_move`を増やさない案を評価する。

## Cでのspelling

portable C11の`_Generic`は共通builtinだけなら使えるが、generated headerが後から型を追加できない。typedefは新しい型を作らず、
すべての`Buffer<T>`も現在は同じpointer carrierなので、element型を使うdispatchにもならない。

target toolchainをpinned Clangに限定している現在の方針では、`__attribute__((overloadable))`を型別glueの同名surfaceとして使う案が
素直である。別々のgenerated headerから同じ`mal_detail_share`、`mal_detail_drop`、`mal_detail_move`へoverloadを追加できる。
`mal_drop(x)`と`mal_move(x)`は引数を一度だけ評価してlvalueのaddressを渡すsyntax-like macroとし、実処理はoverloadされた
inline functionまたはLLVM functionに置く。

```c
#define mal_drop(value) mal_detail_drop(&(value))
#define mal_move(value) mal_detail_move(&(value))

mal_type(Symbol) kept = mal_share(call, borrowed);
mal_drop(kept);
```

macroが受け取るのはlvalueだけとする。任意の式を受けてbitsだけcopyする`move`では元を失効できず、automatic cleanupとの合成も
できないためである。overloadableを採用しない場合は、同じ意味を持つ型別関数名を基盤に残し、短い総称spellingは提供しない。

## Optional automatic cleanup

Clangの`cleanup` attributeを使う`mal_owned(T)`のような宣言macroは、failure pathが多いC bodyの補助として検討できる。

```c
mal_owned(Symbol) temporary = mal_symbol(call, data, length);
return mal_move(temporary);
```

`cleanup` attributeはoverload集合を直接受け取れないため、generated headerは`mal_owned(T)`から選ばれる型別の薄いcleanup wrapperを
生成する。scope終了時にはそのwrapperがlvalueをdropする。明示的な`mal_drop`と`mal_move`が元をvacantにするため、cleanupは正常return、
early return、途中の明示dropに同じ規則を適用できる。ただし`mal_call_trap`はprocessを終了しstack unwindingしないので、trap時の
cleanup保証には使わない。宣言macroはClang固有syntaxを局所化し、型名と変数名の読みにくさ、debugger表示、clangd補完を実物で
評価してから採択する。必須の正しさはautomatic cleanupへ依存させない。

## Bufferのgeneric C surface

Cにはsource-level genericsを再現せず、Buffer生成時にelementのstorage descriptorを一度渡す。descriptorはsize、alignment、share、dropを持ち、
Bufferは以後のpush、replace、truncate、destructionにそれを使う。

```c
mal_type(Buffer) words = mal_buffer(call, mal_type(Symbol), 16);
mal_push(call, words, mal_symbol(call, "mal", 3));

mal_type(Symbol) *elements = mal_data(words);
size_t count = mal_count(words);
```

`mal_push`はBufferが保持するdescriptorからstrideとlifecycleを得るので、element型ごとのfunction名を必要としない。rvalue constructorと
`mal_share`のresultはそのままconsumeでき、owned localは`mal_move`して渡す。`mal_data`は`void *`を返し、Cの代入先でelement pointer型を
選ぶ。このABIは型安全を提供しないため、Buffer descriptorと異なるpointer型への変換をruntimeで検査しない。

`mal_append`はhost arrayから要素をshareして一括追加する。`mal_extend`はtrivial elementだけに未初期化の末尾領域を作り、その領域を
埋めた後に実際の長さへ`mal_truncate`できる。これによりbyte inputを一要素ずつappendさせない。`mal_fill`、`mal_copy`、
`mal_replace`はBufferが保持するdescriptorからmanaged element lifecycleを選ぶ。
Buffer allocationはdescriptorのalignmentを使い、pointer alignmentだけに依存しない。Poolも同じ情報を必要とする場合はstorage descriptorを共有するが、
descriptor registry、型名検索、dynamic type equalityは導入しない。

nominalな`mal_Buffer_Symbol_t` wrapperを型ごとに生成すればClang overloadで静的なelement型を維持できるが、nested型のC名を増やし、
source carrierとhost carrierを再び二重化する。安全性を目的にしないruntime extensionでは既定案にしない。

## Constructorとaggregate

型固有のconstructorは長い型別動詞ではなく短い名詞で表す。

```c
mal_owned(Symbol) text = mal_symbol(call, bytes, length);
mal_owned(Buffer) values = mal_buffer(call, mal_type(Entry), capacity);
```

productとsumはC compound literal、または型が宣言から明らかなinitializerで直接構成する。product fieldは`.field_N`、sumは`.tag`と
`.payload.variant_N`を安定したpublic spellingとする。専用のproduct constructorやsum injection helperは設けない。不正なtag、inactive
payload、managed responsibilityを構成できることは、externをunsafeなruntime extensionとする方針の一部である。

```c
mal_owned(Entry) entry = {
    .field_0 = mal_move(values),
    .field_1 = mal_move(text),
};
return (mal_sum(mal_type(Buffer), mal_type(UInt32))){
    .tag = 0,
    .payload.variant_0 = mal_move(bytes),
};
```

scalarはC valueを直接returnし、Unitは`mal_unit`をreturnする。managed localは`return mal_move(value)`、その場で構成したowned rvalueは
直接returnする。型別`return_move`は公開しない。Boolとsum tagのartifact整合性検査が必要ならgenerated extern wrapper側で行い、
bodyごとのterminal helperにはしない。

external opaque carrierは`mal_from_bits(mal_type(File), bits)`と`mal_bits(value)`で接続する。`mal_drop`はmal-managed responsibilityだけを
終了し、external resourceの`close`や`free`を行わない。resource cleanupは引き続きoperation固有contractに置く。

## 採択前に確かめること

- `overloadable`な宣言をrequire graph上の複数headerから合成でき、aliasと同一structural representationを重複定義しないこと
- product、sum、Symbol、Bufferについてshare、drop、moveがLLVM内部の型再帰と同じlifecycle planから生成されること
- `mal_drop`と`mal_move`が引数を一度だけ評価し、moved/vacant valueのcleanupがno-opになること
- descriptorを保持するgeneric Buffer operationがtemporary、borrowのshare、owned localのmoveを同じsignatureで受けること
- automatic cleanupの有無でobservable resultと最終live allocationが一致すること
- carrier統一、result validation、Buffer alignmentを先に解決したとき、C bodyとgenerated headerが実際に短くなること
- source aliasとanonymous nested carrierのtype keyがrequire graphや宣言追加で不安定にならないこと
- direct sum initializer、external opaque conversion、bulk Buffer operationを含むscenario監査が型別の公開動詞を要求しないこと

[D034](../history/decisions/superseded/D034.md)にも`clone`、`take`、`drop` macroの旧案がある。当時と異なり現在はextern Cを
exact-matchのruntime extension、target toolchainをpinned Clangとしているため、portable C11だけを理由に総称surfaceを避ける必要は
ない。一方、旧案と同じくCの型systemがborrowed、owned、movedを完全に検査するとはみなさない。
