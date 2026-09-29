# operation family

Status: Accepted v0.6

この文書は、generic signatureをcanonical typeごとのimplementationへcompile-timeで結ぶoperation familyを定める。
型argument推論の一般規則は[parametric polymorphism](generics.md)、構文は[字句と文法](grammar.md)を正とする。

## Declarationとimplementation

initializerを持たないgeneric value headerはfamilyを宣言する。familyは一つのidentityとprincipal signatureを持つ。

```mal
equal<A> :: (A, A) -> Bool;
zero<A> :: A;
```

既にfamilyとして宣言された同名headerへinitializerを置くとimplementationになる。closed canonical type argumentだけからなるheaderは
exact implementationである。familyの型parameter名を含むheaderはgeneric implementation patternであり、binderはfamily declarationの
型parameterを同じ名前で再利用する。新しい型parameter名は導入できず、familyの全型parameterをkey内で束縛しなければならない。

```mal
equal<Int32> :: (Int32, Int32) -> Bool := (left, right) -> left == right;
equal<Buffer<A>> :: (Buffer<A>, Buffer<A>) -> Bool :=
    (left, right) -> #left == #right;
```

implementation annotationはfamily signatureへkeyを代入した型と一致しなければならない。patternはalias展開後のcanonical typeで
比較する。file-local opaque typeはrepresentationへ展開せず、declaration identityと型argumentをpatternへ残す。

family signatureはconstructor kindのparameterを持てる。implementation keyのconstructor-kind位置はclosed canonical constructor
termだけを認め、constructor pattern variableを拒否する。kind `Type`の位置には従来のexact keyとgeneric patternを認める。

```mal
Pair<A> :: (A, A);
first<F, A> :: F<A> -> A;
first<Pair, Int32> :: Pair<Int32> -> Int32 := (left, _) -> left;
```

declarationとimplementationはsource orderに従う。implementationは同じfileで先に宣言されたfamily、またはそのfileが直接
`require`したfileのpublic familyへ追加できる。implementation itemは新しいsource value名を導入しない。同名familyが存在しない
initializer付きheaderは従来のgeneric bindingであり、header項は型parameter名でなければならない。

## Coherenceとtermination

一つのfamilyに属する二つのimplementation keyが一つでも同じconcrete argument列へunifyできる場合、programを拒否する。exactとgeneric
の間にも優先順位を設けず、source order、specificity、別fileの追加によって選択を変えない。すべてのargumentが型parameterそのものの
catch-all patternも拒否する。

generic implementationのbodyで生じる各`OperationRequirement`のargumentは、implementation keyのいずれかのargumentに含まれる
proper structural subtermでなければならない。`Buffer<A>`に対する`A`、`(A, B)`に対する`A`は減少するが、同じkeyやそれを包む型は
減少しない。ただし同じfamilyの同じkeyへの直接の自己再帰は通常のsame-key recursionとして認める。この規則により、選択と
requirement展開は有限なspecialization graphとして検査できる。

## 推論とrequirement

family referenceは通常のgeneric valueと同じ局所solverで型argumentを決める。solverはfamily signature、operand、期待resultだけを
使い、implementation集合をconstraintに使わない。型argumentが確定してからimplementationを検索するため、別fileへの非重複な
implementation追加で既存referenceの推論結果は変わらない。

generic本体でrigid type parameterを含むfamily referenceは、型付き`OperationRequirement`としてそのbindingに保存する。
generic bindingを参照した本体には、参照先bindingのrequirementへ型argumentを代入して伝播する。requirementはfamily identityと
canonical argument列からなり、bodyをprincipal signatureだけで一度検査できる。明示的なrequirement構文とruntime dictionaryはない。

```mal
same<A> :: (A, A) -> Bool := (left, right) -> equal(left, right);
```

## Selection

specializationはconcreteになったfamily referenceごとに、一意にmatchするexactまたはgeneric keyを検索する。generic keyがmatchした
場合はpattern binderへのconcrete substitutionをimplementationのannotation、body、requirementへ適用する。到達したkeyに
implementationがなければcompile-time errorにする。未到達のkeyへimplementationがなくてもerrorにしない。選択したimplementationは
通常のmonomorphic bindingへ変換し、同じconcrete keyを参照する箇所で共有する。

family resultはfunction型に限らない。value implementationも既存のclosed top-level initializer規則に従う。family valueは
期待型からargumentが決まれば通常のvalueとして渡せるが、function型でないfamily valueをapplicationできない。

specialization後はfamily declaration、implementation item、operation reference、requirementをすべて除去する。core以降は
operation family、kind、constructor term、型argument、dictionary、type descriptor、dynamic dispatchを受け取らない。
constructor pattern variable、user-defined kindとbound、runtime dispatchはこのprofileに含めない。
