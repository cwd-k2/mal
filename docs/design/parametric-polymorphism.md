# parametric polymorphismと型index付きprimitiveの試案

Status: Discussion draft (2026-09-15)

この文書は、型ごとに同じ構造と制御を複製する摩擦を減らすためのparametric polymorphismと、既存の
numericおよびmemory primitiveとの境界を記録する。現行の規範は[`types`](../spec/types.md)、
[`expressions`](../spec/expressions.md)、[`memory primitive`](../spec/memory.md)、
[`grammar`](../spec/grammar.md)を正とし、この試案だけを根拠にsourceやABIを変更しない。

## 目的

型parameterは、product、sum、functionと明示的に渡したoperationの配線を型ごとに書き直す摩擦を減らすために使う。
型からoperationを探索し、型固有の表現、能力、policyを暗黙に導く機能にはしない。

```mal
Result<E, A> :: [E, A];

identity<A> :: A -> A := (value) -> value;

mapResult<E, A, B> :: (Result<E, A>, A -> B) -> Result<E, B> :=
    (result, transform) -> {
        result[
            (error) -> [failure, success] => failure(error),
            (value) -> [failure, success] => success(transform(value))
        ];
    };
```

この記法は候補である。`<...>`はcompile-timeの型parameterまたは型application、`(...)`はvalue application、
`[...]`は従来どおりsum、result binder、continuation application、value-first applicationにだけ使う。
比較operatorとの構文境界、nested type applicationの`>>` token、およびformatter規則は採択前にgrammarとして固定する。

## ユーザー定義の多相性

最初のprofileは次に限定する。

- type aliasとtop-level value bindingだけが、明示した型parameterを持てる。
- 型parameterはすべて通常のsource typeを表し、kind、bound、constraintを持たない。
- generic本体はopaqueな型parameterのもとで一度検査する。specialization後に新しいoperationを許可しない。
- generic bindingは多相なruntime valueではない。使用時に具体化した単相valueだけを通常のfunction valueとして扱う。
- local bindingのgeneralization、first-class polymorphism、higher-kinded type、polymorphic recursionは認めない。
- self recursionは同じ型argumentを保つcallだけを認める。
- type reflection、type case、generic typeによるoverload resolutionは認めない。
- generic extern declarationは認めない。extern parameterとresultは従来どおりconcrete typeでなければならない。

型argumentは最初のprofileでは明示する。argument型または期待result型から一意に決まる型argumentの省略は、
call siteの摩擦と型推論規則を実例で比較してから別に判断する。

generic type aliasはtransparentであり、具体化してaliasを展開した型と同じ型になる。型parameterや型applicationに
runtime identity、descriptor、layoutを与えない。

## 明示的なoperation

型parameterは、その型に固有のoperationを導入しない。比較、算術、memory access、encodingが必要なgeneric関数は、
通常のfunctionまたはproduct valueとしてoperationを明示的に受け取る。

```mal
Equality<A> :: (A, A) -> Bool;

contains<A> :: (Equality<A>, A, A) -> Bool :=
    (equal, expected, actual) -> equal(expected, actual);
```

`Equality<A>`はtransparent aliasにすぎず、`Eq` class、instance、compiler-known dictionaryを宣言しない。
compilerはoperationを探索、構成、挿入しない。複数のoperationをproductへまとめても、それはprogramが明示的に構築し
引数として渡す通常のvalueである。

## 閉じたprimitive family

numeric operatorは既に、仕様が列挙するconcrete typeごとに意味を持つ閉じたprimitive familyである。
ここでの`Numeric`は仕様を簡潔に記述するmeta-level categoryであり、source type、kind、class、constraintではない。
programはそのmemberを追加できず、型parameterを`Numeric`として宣言できない。

```text
Numeric =
    Int8 | Int16 | Int32 | Int64
  | UInt8 | UInt16 | UInt32 | UInt64
  | Float32 | Float64
```

[`ByteSize`、`Count`、`Layout`、memory placementの試案](size-and-alignment.md)を同時に採択する場合は
`ByteSize`と`Count`も`Numeric`へ加える。
operatorごとの正確なdomainは引き続き個別に列挙し、例えばremainderとbit operatorをfloatへ拡張しない。

ユーザー定義のgeneric本体では、opaqueな型parameterへnumeric primitiveを適用できない。

```mal
double<A> :: A -> A := (value) -> value + value; // error

doubleWith<A> :: ((A, A) -> A, A) -> A :=
    (add, value) -> add(value, value);
```

この区別により、ユーザー定義部分のparametricityと、言語primitiveの閉じたad-hoc polymorphismを混同しない。

## memory primitiveとの統合

memory representation、placement、alignmentの候補は
[`ByteSize`、`Count`、`Layout`、memory placementの試案`](size-and-alignment.md)を正とする。ここでは型parameterとの境界だけを定める。

型`A`だけからrepresentationを導くgeneric memory operationは認めない。

```mal
readUnknown<A> :: Ptr -> A := (pointer) -> <-pointer; // error
```

一方、`Layout<A>`は`A`のrepresentation capabilityを明示的に運ぶbuilt-in opaque valueである。callerがこれを渡す場合、
generic本体はpointerへlayoutを適用して`Address<A>`を作り、load/storeできる。

```mal
readWith<A> :: (Ptr, Layout<A>) -> A :=
    (pointer, layout) -> <-(pointer / layout);

writeWith<A> :: (Ptr, A, Layout<A>) -> Ptr :=
    (pointer, value, layout) -> (pointer / layout) <- value;
```

これは型から暗黙に探索または挿入されるdictionaryではない。`Layout<A>`を作るprimitive、product/sum layout operator、
または引数として受け取った通常のvalueだけがmemory capabilityを導入する。`Layout<A>`を渡しても`A`の比較、算術、encodingなど
無関係なoperationは導入されない。

`Span<A>`、`Address<A>`、`Region<A>`も同じ型indexを保存するため、genericな有限regionのaccessを記述できる。

```mal
readAt<A> :: (Region<A>, Count) -> A :=
    (region, index) -> <-(region # index);
```

`Region<A>`はbounds、ownership、lifetimeを新しく証明しない。`index`がspanのcount未満であり、元のhost contractがregionを
access可能にしていることは引き続きpreconditionである。

`Layout`、`Span`、`Address`、`Region`に対するoperatorは、型parameterへ任意のprimitiveを後付けする例外ではなく、
明示されたoperandの型indexを保存するbuilt-in primitive familyである。型検査後のspecializationではconcreteなlayoutと
value型が確定し、ANF以降へopenな型parameterまたは暗黙dictionaryを渡さない。

raw pointerへ異なるlayoutを順にstoreする場合は、値へlayoutを明示する。

```mal
end := pointer
    <- header * u8
    <- version * i32
    <- payloadPointer * pointerLayout;
```

採択時には[D037](../history/decisions/D037.md)の`T.load`と`T.store`を置き換え、名前付きの`load<T>`と`store<T>`は提供しない。
`Symbol.read`と`Symbol.write`はmanaged byte copyであり、canonical object representationのload/storeではないため統合しない。

## compiler境界

name resolutionとtype checkingは型parameter、型application、opaqueな型変数、およびgeneric bindingを所有する。
型検査後、到達するconcrete specializationを共有して単相のtyped coreを構成する。ANF以降はgeneric declaration、
type argument、dictionaryを受け取らず、現在と同じconcrete typeとprimitiveだけを扱う。

specialization graphは有限でなければならない。同じgeneric bindingとconcrete type argument列を一つのidentityとして共有し、
異なる型argumentで再帰するbindingを型検査で拒否する。code size上限とdiagnosticは実装着手前にresource limitとして定める。

public generic bindingはmal source間だけで利用できる。generated C header、extern ABI、host adapterへopenな型parameterを公開せず、
host interfaceは従来どおりconcrete typeだけから構成する。

## 採択前に確認すること

採択判断には、少なくとも次の実例と境界を固定する必要がある。

- concrete typeごとに複製されているsum変換、state threading、higher-order helperを実際に削減できること
- 明示的なoperation引数が、型別実装の複製よりcall siteとcontractの摩擦を減らすこと
- generic alias、generic function、cross-file use、self recursionのpositive case
- 未確定型へのprimitive適用、generic extern、polymorphic recursionのnegative case
- specializationの共有、code size、managed valueのretain、transfer、releaseが単相core以降で完結すること
- `<...>`、`#`、`&`、`|`、`*`、`/`、`<-`、`<~`とcomparison、shift、nested type applicationを曖昧なくparse、formatできること
- `Layout<A>`あり／なしのgeneric load、`Region<A>`のindexing、異なるlayoutを連ねたstore-and-advanceのpositive/negative case
