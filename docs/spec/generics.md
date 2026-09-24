# parametric polymorphism

Status: Accepted v0.6

この文書は型parameter、generic aliasとvalue binding、built-in型形成条件、specializationを定める。concrete syntaxは
[字句と文法](grammar.md)、Bufferの型形成は[AddressとBuffer](memory.md)を正とする。

## Declarationとapplication

type aliasとtop-level value bindingは明示的な型parameterを持てる。型argumentはすべて明示する。

```mal
Result<E, A> :: [E, A];
identity<A> :: A -> A := (value) -> value;

same :: Int32 -> Int32 := (value) -> identity<Int32>(value);
```

型parameterは通常のsource typeを表す。user-defined kind、bound、constraintはない。generic aliasはtransparentであり、型argumentを
代入して展開したcanonical typeと同じ型になる。recursive generic aliasは拒否する。

generic binding自体はruntime valueではない。non-generic codeはconcrete type argumentでspecializeした単相valueだけを参照、capture、
applicationできる。generic本体ではscope内の型parameterをtype argumentに使え、外側のspecializationでargumentがconcreteになった時点で
参照先もspecializeする。local generalization、first-class polymorphism、higher-kinded type、polymorphic recursion、type reflection、
type case、generic typeによるoverload resolution、generic extern declarationはない。self recursionは同じ型argument列を保つcallだけを認める。

## 型検査

generic本体はopaqueな型parameterとsignatureから導いたbuilt-in型形成条件の下で一度検査する。specialization後に本体へ新しい
operationを許可しない。比較、算術、encodingなどを必要とするgeneric functionは通常のfunctionまたはproduct valueとして
operationを受け取る。numeric operatorとconversionはconcrete typeだけを列挙するclosed primitive familyである。

```mal
Equality<A> :: (A, A) -> Bool;

contains<A> :: (Equality<A>, A, A) -> Bool :=
    (equal, expected, actual) -> equal(expected, actual);

double<A> :: A -> A := (value) -> value + value; // error
```

## Requirements

`Requirements(T)`はalias展開後の型`T`が要求するbuilt-in judgmentの有限集合である。product、sum、functionは要素のrequirementを
再帰的に合併する。`Buffer<A>`は`Representable(A)`を加え、type argument内のrequirementも加える。

compilerは`Representable(T)`をclosedな定義で正規化する。representableなconcrete base caseは消去し、productとsumは各要素へ
分解し、opaqueな型parameterだけをatomとして残す。既知の非representable型はdeclarationで拒否する。
`Buffer<(A, UInt64)>`から得るrequirementは`Representable(A)`である。

generic aliasはdefinitionのresult type、generic value bindingは明示signatureからrequirementを集める。本体内だけに現れ、signatureから
導けないrequirementを型parameterへ要求するoperationはerrorである。型applicationはconcrete argumentを代入し、aliasを展開した後に
全requirementを検査する。generic本体内のapplicationでは、代入後のrequirementがcaller bindingのrequirementから導けることを検査する。
満たさないapplicationはspecialization前のcompile-time errorである。

generic codeは裸のAddressと`A`からC host representationを導けない。callerがconcrete type argumentで`from`をspecializeすると、
copy primitiveがstatic representationを選ぶ。

```mal
readFirst<A> :: Buffer<A> -> A := (buffer) -> buffer.get(0usize);
writeFirst<A> :: (Buffer<A>, A) -> Unit := (buffer, value) -> buffer.put(0usize, value);
```

`Buffer<A>`は通常のgeneric argument、parameter、resultとして使える。`Representable(A)`はelement storageとC host copy representationの
存在だけを示し、Address referentのextent、permission、initialization、lifetime、valid representationを証明しない。

## Specialization

name resolutionとtype checkingは型parameter、型application、opaque type variable、requirement、generic binding identityを所有する。
type checking後、compilerはentry pointから到達するexplicit concrete applicationを起点にspecialization graphを構成する。

specialization keyはgeneric binding identityとalias展開後のcanonical concrete type argument列であり、同じkeyはfileを跨いで共有する。
各nodeはgeneric typed bodyへ型argumentを代入して単相typed coreを一度生成し、到達するgeneric applicationをgraphへ加える。
本体内で型parameterをargumentに使ったapplicationも、この代入後にはconcreteなkeyになる。self recursionは同じkeyへのedgeとして閉じる。
異なる型argumentで自分を呼ぶbindingはdeclarationで拒否する。

一つのprogramで生成するspecialization nodeは65,536個までとする。次のnodeを加えると上限を超える場合、source programを型不正とは
せず、展開元binding、type argument列、limitを示すartifact生成failureとする。型の物理表現上限とcompiler processの一般的な
resource failureは別の規則である。ANF以降はgeneric declaration、type argument、requirement、dictionaryを受け取らない。

## Host境界

generic bindingはmal source間だけで使える。generated C header、extern ABI、host adapterへgeneric binding、specialization、型parameterを
公開しない。generic aliasはconcrete argumentを代入して完全に展開した後、[`HostMappable`](extern.md#host-mappable-type)で判定する。
