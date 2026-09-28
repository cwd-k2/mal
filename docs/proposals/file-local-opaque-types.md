# file-local opaque type

Status: Exploratory

この文書は、malで実装したrepresentationを宣言元fileだけから観察できるabstract typeの候補を管理する。現在の
transparent aliasと[external opaque type](../spec/types.md#external-opaque-type)は現行仕様を正とし、ここで扱う`opaque`宣言は
未採択である。Poolをcontainer policyの背後へ隠す利用例は[Poolとopaque型によるcontainer基盤](pool/README.md)に分ける。

## Declarationとidentity

候補syntaxを次に示す。

```mal
opaque Option<T> :: [Unit, T];
opaque PairBox<A, B> :: (A, B);
```

opaque型はdeclarationごとのcanonical type identityを持ち、hidden representationへalias展開しない。同じrepresentationを持つ
二つのopaque型、opaque型とそのrepresentationは異なるsource typeである。別fileはpublicなopaque名をsignature、型argument、
bindingに使えるが、representationを使って値を構築、分解、観察できない。

identityはcompile-timeのabstraction boundaryであり、runtime tag、wrapper、allocation、copy、別のEngram identityを追加しない。
opaque値のlayout、lifecycle、型形成に必要なbuilt-in requirementはhidden representationから導く。したがってzero-costであっても、
transparent aliasのようにcanonical typeをrepresentationへ置き換えるものではない。

## 宣言元fileのrepresentation view

宣言元source fileの通常のtype checkingでは、opaque型とhidden representationを双方向にviewできる。専用の`pack`、`open`、coercion
functionはsourceへ導入せず、既存のexpression、期待型、pattern、直和除去とresult binderをそのまま使う。

```mal
opaque Option<T> :: [Unit, T];
opaque PairBox<A, B> :: (A, B);

none<T> :: Unit -> Option<T> := () -> [returnNone, returnSome] => returnNone();

isSome<T> :: Option<T> -> Bool := (value) -> value[
    () -> false,
    (_) -> true
];

first<A, B> :: PairBox<A, B> -> A := (value) -> {
    (first, _) := value;
    first;
};
```

representation viewは宣言元fileのauthorityを使うtype-checking judgmentであり、一般のimplicit conversionではない。型argument推論は
まずopaque型をatomとしてunifyし、通常のoperandまたは期待型の構造と一致しない場合だけrepresentation viewを使う。別fileへ伝播せず、
requireしてもauthorityを得ない。opaque値をcapture、productやsumへ格納、returnするときはopaque identityを保つ。

hidden representationが別のnamed type constructorでも同じ規則を使う。例えば
`opaque Buffer<T> :: Pool<USize, T>`の宣言元fileでは、`Buffer<T>`を`Pool` operationへ直接渡し、`Pool<USize, T>`を
`Buffer<T>`が期待される位置から返せる。これらはcarrier responsibilityをそのまま受け渡し、Share、Consume、Dropを追加しない。

## Canonicalizationとoperation family

representation viewはtype equality、generic specialization key、operation family implementation keyを変更しない。

```mal
opaque Buffer<T> :: Pool<USize, T>;
```

このとき`Buffer<Symbol>`と`Pool<USize, Symbol>`は異なるcanonical typeであり、`equal<Pool<USize, T>>`のgeneric
implementation patternは`equal<Buffer<T>>`へmatchしない。例えば`equal<A> :: (A, A) -> Bool`へBufferを渡すと、atomのまま
`A = Buffer<T>`と推論する。representation固有のsignatureへ宣言元fileから適用した場合はviewによって型argumentを推論できるが、
確定したfamily keyを後からhidden representationへ展開しない。

この区別により、generic implementationのcoherenceとspecialization keyをfile identityから独立に保つ。opaque constructor自体を
generic implementation patternに置くことはでき、その場合もhidden representationのpatternとは重複しない。generic patternの候補規則は
[generic operation implementation](generic-operation-implementations.md)で管理する。

## Visibilityとrepresentationの公開

opaque declarationがpublicなら、名前と形成条件は通常のpublic typeと同様にrequire元へ導入する。hidden representationはname lookup
から隠すのではなく、そのopaque型に対する構築・観察authorityだけを宣言元fileへ限定する。representationにprivate型を含めても、
利用側はopaque値をpublic operationの引数やresultとして受け渡せる。

宣言元fileはpublic operationを通じてrepresentation由来の情報や値を意図的に公開できる。型機構は、誤って同一視することを防ぐが、
宣言元file自身が不変条件を破るoperationを書くことまでは防がない。public signatureにhidden representationを直接現すことを禁止するかは、
必要な実例が得られるまで本proposalの採択条件にしない。

## Compiler boundary

resolverはopaque declaration identity、宣言元file identity、型parameter、hidden representationを関連付ける。checkerは通常の
canonical typeをopaque identityのまま保ち、宣言元fileに限ってrepresentation viewを使う。generic requirementとspecializerは
hidden representationではなくopaque canonical typeをkeyにする。

coreへ渡す前にrepresentation viewをrepresentation-preservingな型境界として消去する。core以降はpack/open operationやruntime coercionを
受け取らず、hidden representationから導いたlayout、lifecycle、built-in requirementだけを使う。

## 採択前に固定する検査

- 宣言元fileでproduct pattern、sum elimination、sum construction、representationを受けるoperationを利用できる。
- 別fileではrepresentationによる構築、分解、operation適用を拒否し、opaque値の受け渡しは許可する。
- 同じrepresentationを持つ二つのopaque型を混同できない。
- opaque型とrepresentationのgeneric specialization keyおよびoperation family keyが一致しない。
- representation viewがallocation、copy、Share、Consume、Drop、runtime identityを追加しない。
- hidden representationの型形成条件とlifecycleをopaque型へ伝播し、wrapperで既存propertyを迂回できない。

具体syntax、diagnosticでhidden representationをどこまで表示するか、public signatureへrepresentationを明示する制限は、focused testと
Pool上のcontainer例を通して採択前に決める。
