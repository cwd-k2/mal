# 型

Status: Accepted v0.7

## 型の構成

```text
T ::=
    Unit
  | Int8 | Int16 | Int32 | Int64
  | UInt8 | UInt16 | UInt32 | UInt64
  | Float32 | Float64
  | ByteSize | USize
  | Symbol
  | Buffer<T>
  | (T, T, ...)
  | []
  | [T, T, ...]
  | T -> T
  | ExternalType
  | OpaqueType<T, ...>
  | TypeAlias
  | TypeParameter
```

`Float32`と`Float64`は、それぞれIEEE 754-2019のbinary32とbinary64である。normal、subnormal、正負のzero、正負のinfinity、NaNを含む。詳細な演算規則は[実行意味論](execution.md#浮動小数点)に定める。

`Int`、`Long`、`Size`のようにhost C spellingへ依存する整数型はない。`ByteSize`と`USize`の幅はtargetのpointer index幅から
決まり、用途の異なる別の型である。subtypingとimplicit numeric conversionはない。source-defined opaque typeだけが
declaration identityを持つuser typeである。

`Byte` と `Char` という型はない。単一 byte は `UInt8` で表す。mal は Unicode character を primitive value として定義しない。

## Symbol

`Symbol`は言語組み込みのimmutableな有限byte値である。literal、値のlifetime、operatorは[Symbol](symbols.md)、
`Buffer<UInt8>`とのsnapshot変換は[`Buffer`](memory.md#symbol-conversion)を正とする。

## Buffer

`Buffer<T>`はmal-ownedなmutable有限sequenceへの共有参照である。型形成、operation、element lifecycleは
[`Buffer`](memory.md)に定める。mal codeはraw pointer型を持たない。

## Unit

`Unit` は空直積で、唯一の値は `()` である。

```mal
done :: Unit := ();
```

一要素 tuple は存在しない。`(A)` と `(a)` はそれぞれ括弧付きの型と式である。

## 直積

二要素以上の直積は tuple notation で表す。

```mal
Point :: (Float64, Float64);
origin :: Point := (0.0, 0.0);
xOf :: Point -> Float64 := (point) -> {
    (x, y) := point;
    x;
};
```

型、値、pattern の位置が対応する。record、field name、method はない。

## 直和

直和は角括弧で囲んだ順序付き n 項型で、各項は 0-based index を持つ。

```mal
MaybeInt32 :: [Unit, Int32];

none :: Unit -> MaybeInt32 := () -> [none, some] => [none];
some :: Int32 -> MaybeInt32 := (value) -> [none, some] => some(value);
```

同じ型を複数の項に置いてよい。

```mal
Choice :: [Int32, Int32];
a :: Int32 -> Choice := (value) -> [first, second] => first(value);
b :: Int32 -> Choice := (value) -> [first, second] => second(value);
```

直和値は選択した一項の0-based indexとその項型のpayloadを持つ。source-levelの構築は
[direct result block](control.md#direct-result-block)のresult binderで行う。

空直和`[]`は値を持たず、一項直和`[A]`は存在しない。二項以上の直和は各項の値を持つ。
`[]`のeliminationとcompletion規則は[result boundaryとcompletion](control.md#empty)に定める。

n-ary sum と nested sum は異なる型である。

```mal
Flat :: [A, B, C];
Nested :: [A, [B, C]];
```

直和値の除去は[直和の除去](expressions.md#直和の除去)に定める。

## predefined Bool

`Bool` は独立した primitive type ではなく、次の transparent alias と predefined binding である。

```mal
Bool :: [Unit, Unit];

false :: Bool := [() -> [cont0, cont1] => [cont0]];
true :: Bool := [() -> [cont0, cont1] => [cont1]];
```

このcode blockはpredefined nameの型と値をmal notationで示す意味上の擬似定義であり、どのsource fileにもtop-level
declarationとして含まれない。外側の`[lambda]`はlambdaへのUnit application、bodyの`[cont0]`または`[cont1]`は
選択したvariant binderへのUnit applicationである。compilerが同じidentityと値をpredefined scopeへ直接導入する。
`Bool`が期待されるlocalな式位置では、
`[cont0, cont1] => [cont1]`も同じ`true`値を作る。direct result blockの配置は[direct result block](control.md#direct-result-block)と
[top-level initializer](programs.md#top-level-item)の規則に従う。

`Bool`、`false`、`true`は他のpredefined名と同じく[top-levelで再宣言できない](programs.md#program-と-source-file)。local scopeでは
通常のshadowing規則により`false`と`true`をshadowできる。

`false` と `true` は keyword や専用 literal ではなく、型付きの immutable value である。

`Bool` は transparent なので `[Unit, Unit]` と同じ型である。比較演算は false の場合に index 0、true の場合に index 1 の値を返す。

## transparent aliasと型parameter

PascalCase identifier に型を対応させる。

```mal
Point :: (Float64, Float64);
Size :: (Float64, Float64);
```

`Point`、`Size`、`(Float64, Float64)` は同じ型である。alias は新しい runtime representation や nominal identity を作らない。
表現に寄与するrecursive aliasは認めない。表現に使われないphantom type argumentだけを通る参照はrecursive value typeを
作らないため認める。

aliasとtop-level value bindingは明示的な型parameterを持てる。

```mal
Pair<A> :: (A, A);
identity<A> :: A -> A := (value) -> value;
```

型parameterはdeclaration内でopaqueなtype-level termを表し、canonical type argumentで明示的にspecializeする。完全な規則は
[kindとtype constructor](type-constructors.md)と[parametric polymorphism](generics.md)に定める。

sum result binderのarityとparameter型は、期待result型のaliasを展開した直和型から決まる。alias自体にruntime identityは残らない。

## file-local opaque type

`opaque` declarationはdeclarationごとのcanonical type identityと、zero-costなhidden representationを導入する。

```mal
opaque Option<A> :: [Unit, A];
opaque PairBox<A, B> :: (A, B);
```

同じrepresentationを持つ二つのopaque type、opaque typeとそのrepresentationは、それぞれ異なるsource typeである。
型argumentもidentityの一部であり、transparent aliasのようにrepresentationへ展開して型等価にはしない。recursive
representationと、型argument数の不一致はdeclarationの使用有無にかかわらず拒否する。

宣言元source fileのtype checkingだけは、opaque typeとrepresentationを双方向にviewできる。このviewには専用の`pack`、`open`、
coercion syntaxを使わず、既存のproduct構築とpattern、sumのresult binderとelimination、function applicationなどをそのまま使う。
まずopaque identityのまま型を比較し、通常の構造と一致しないときだけ宣言元fileのrepresentation viewを使う。型の各位置では
一方の側だけを、宣言元fileのopaque層を必要なだけ辿ったrepresentationとして見る。したがって`opaque B :: A;`は`A`とも
`A`のrepresentationとも一致するが、同じrepresentationを持つ別々のopaque type同士を、このviewで変換することはできない。
primitive operatorとnumeric literalも、宣言元fileではoperandと期待型をこのviewで扱い、結果はrepresentationの型になる。
設計理由は[D088](../history/decisions/active/D088.md)に記録する。

publicなopaque名は通常のpublic typeと同様に直接`require`したfileへ導入されるが、representation viewのauthorityは導入されない。
representationにprivate typeを含めてもよい。別fileはopaque値をsignature、型argument、productやsumの要素、値の受け渡しに使えるが、
hidden representationによる構築、分解、operation適用はできない。

layout、lifecycle、`Storable`などの型形成条件はhidden representationから再帰的に導く。opaque wrapperによってrepresentationの
制約を迂回できない。extern signatureへ現れたfile-local opaque typeはspecialization後にhidden representationへ正規化し、宣言元fileの
generated headerがそのruntime carrierをC implementationへ公開する。

generic specialization keyと[operation family](operation-families.md)のkeyにはopaque declaration identityと型argumentを残す。
representation viewによってkeyを変えない。specialization完了後はopaque boundaryをrepresentationへ消去し、coreとbackendへ
新しいruntime wrapper、tag、metadataを渡さない。

## 関数型

```mal
Int32 -> Int32
(Int32, Int32) -> Int32
A -> B -> C
```

`->` は右結合なので、最後の例は `A -> (B -> C)` である。直和は `[]` で区切られるため、`[A, B] -> C` の domain は直和型全体になる。

複数 parameter の関数は意味上、一つの直積を受け取る。0 parameter の関数は `Unit` を受け取る。

関数型の値は lexical closure である。同じ関数型を持つ capture-free な関数と capture を持つ関数は、型として区別しない。closure の code や environment を観察する操作と、関数値の equality は存在しない。

## external opaque type

```mal
extern Mem;
extern File;
```

opaque type の内部表現を mal program から構成・分解・観察することはできない。値は通常の mal 値と同様に binding でき、複製・破棄できるものとして型検査する。

このため `File` や `Mem` のような resource の一意所有権は保証されない。resource correctness は [`extern` contract](extern.md) の責務であり、決定理由は[D015](../history/decisions/active/D015.md)に記録する。
