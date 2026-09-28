# operation family

Status: Accepted v0.6

この文書は、generic signatureをclosed canonical typeごとのimplementationへcompile-timeで結ぶoperation familyを定める。
型argument推論の一般規則は[parametric polymorphism](generics.md)、構文は[字句と文法](grammar.md)を正とする。

## Declarationとimplementation

initializerを持たないgeneric value headerはfamilyを宣言する。familyは一つのidentityとprincipal signatureを持つ。

```mal
equal<A> :: (A, A) -> Bool;
zero<A> :: A;
```

既にfamilyとして宣言された同名headerへinitializerを置くとexact implementationになる。初期profileのkeyはfamily identityと、
alias展開後のclosed canonical type argument列である。implementation annotationはfamily signatureへkeyを代入した型と
一致しなければならない。同じcanonical keyをprogram内へ二度定義できない。

```mal
equal<Int32> :: (Int32, Int32) -> Bool := (left, right) -> left == right;
zero<Int32> :: Int32 := 0;
```

declarationとimplementationはsource orderに従う。implementationは同じfileで先に宣言されたfamily、またはそのfileが直接
`require`したfileのpublic familyへ追加できる。implementation itemは新しいsource value名を導入しない。同名familyが存在しない
initializer付きheaderは従来のgeneric bindingであり、header項は型parameter名でなければならない。

exact implementationのkeyに型parameterを置けない。generic implementation pattern、specificity、overlap resolution、
higher-kinded constructor、user-defined kindとboundはこのprofileに含めない。numeric operatorとconversionは引き続きclosedな
primitive familyであり、user-defined operation lookupへ統合しない。

## 推論とrequirement

family referenceは通常のgeneric valueと同じ局所solverで型argumentを決める。solverはfamily signature、operand、期待resultだけを
使い、implementation集合をconstraintに使わない。型argumentが確定してからexact implementationを検索するため、別fileへ
implementationを追加しても既存referenceの推論結果は変わらない。

generic本体でrigid type parameterを含むfamily referenceは、型付き`OperationRequirement`としてそのbindingに保存する。
generic bindingを参照した本体には、参照先bindingのrequirementへ型argumentを代入して伝播する。requirementはfamily identityと
canonical argument列からなり、bodyをprincipal signatureだけで一度検査できる。明示的なrequirement構文とruntime dictionaryはない。

```mal
same<A> :: (A, A) -> Bool := (left, right) -> equal(left, right);
```

## Selection

specializationはconcreteになったfamily referenceごとにexact keyを検索する。到達したkeyにimplementationがなければcompile-time
errorにする。未到達のkeyへimplementationがなくてもerrorにしない。選択したimplementationは通常のmonomorphic bindingへ変換し、
同じkeyを参照する箇所で共有する。

family resultはfunction型に限らない。value implementationも既存のclosed top-level initializer規則に従う。family valueは
期待型からargumentが決まれば通常のvalueとして渡せるが、function型でないfamily valueをapplicationできない。

specialization後はfamily declaration、implementation item、operation reference、requirementをすべて除去する。core以降は
operation family、型argument、dictionary、type descriptor、dynamic dispatchを受け取らない。
