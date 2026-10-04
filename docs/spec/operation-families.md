# operation family

Status: Accepted v0.7

この文書は、generic signatureをcanonical typeごとのimplementationへcompile-timeで結ぶoperation familyを定める。
型argument推論の一般規則は[parametric polymorphism](generics.md)、構文は[字句と文法](grammar.md)を正とする。

## Declarationとimplementation

initializerを持たないgeneric value headerはfamilyを宣言する。familyは一つのidentityとprincipal signatureを持つ。

```mal
equal<A> :: (A, A) -> Bool;
zero<A> :: A;
```

既にfamilyとして宣言された同名headerへinitializerを置くとimplementationになる。implementationのheader項はkeyであり、型の
patternとして読む。型application `X<...>`の先頭以外に現れ、見える型名に解決しない名前はpattern変数であり、値のpatternが名前を束縛するのと
同じくそのimplementationのbinderになる。見える型名に解決する名前はその型を指す。pattern変数を含まないkeyはexact
implementation、含むkeyはgeneric implementation patternである。family declarationの型parameter名を再利用する必要はない。

```mal
equal<Int32> :: (Int32, Int32) -> Bool := (left, right) -> left == right;
equal<Buffer<A>> :: (Buffer<A>, Buffer<A>) -> Bool :=
    (left, right) -> #left == #right;
```

implementation annotationはfamily signatureへkeyを代入した型と一致しなければならない。patternはalias展開後のcanonical typeで
比較する。file-local opaque typeはrepresentationへ展開せず、declaration identityと型argumentをpatternへ残す。

family signatureはconstructor kindのparameterを持てる。implementation keyのconstructor-kind位置には、closed canonical constructor
termか、opaque typeまたは`Buffer`をpattern変数を含む引数へ部分適用したtermを置ける。後者は先頭のdeclaration identityで
一次に照合し、pattern変数は引数の位置にだけ現れる。constructor自体をpattern変数にするkeyと、transparent aliasを先頭に持つ
pattern変数入りのkeyは拒否する。aliasはidentityを持たず、展開後のtermから先頭を一意に決められないためである。kind `Type`の
位置には従来のexact keyとgeneric patternを認める。設計理由は[D094](../history/decisions/active/D094.md)に記録する。

```mal
Pair<A> :: (A, A);
opaque Either<E, A> :: [E, A];
first<F, A> :: F<A> -> A;
first<Pair, Int32> :: Pair<Int32> -> Int32 := (left, _) -> left;
first<Buffer, A> :: Buffer<A> -> A := (values) -> values.get(0usize);

pure<F, A> :: A -> F<A>;
pure<Either<E>, A> :: A -> Either<E, A> := (value) -> [failure, success] => success(value);
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
場合はpattern binderへのconcrete substitutionをimplementationのannotation、body、requirementへ適用する。implementationの
kind requirementと、keyを代入したsignatureから得る`Storable` requirementも、このsubstitutionで検査する
（[parametric polymorphism](generics.md#requirements)）。family signatureはopen application `F<A>`の中に`Storable`を要求できない
ため、Bufferを表現に持つkeyのimplementationはfamilyより多いrequirementを持ち得る。設計理由は
[D095](../history/decisions/active/D095.md)に記録する。到達したkeyにimplementationがなければcompile-time errorにする。
未到達のkeyへimplementationがなくてもerrorにしない。選択したimplementationは通常のmonomorphic bindingへ変換し、同じconcrete keyを
参照する箇所で共有する。

family resultはfunction型に限らない。value implementationも既存のclosed top-level initializer規則に従う。family valueは
期待型からargumentが決まれば通常のvalueとして渡せるが、function型でないfamily valueをapplicationできない。

specialization後はfamily declaration、implementation item、operation reference、requirementをすべて除去する。core以降は
operation family、kind、constructor term、型argument、dictionary、type descriptor、dynamic dispatchを受け取らない。
constructor pattern variable、user-defined kindとbound、runtime dispatchはこのprofileに含めない。
