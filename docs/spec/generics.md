# parametric polymorphism

Status: Accepted v0.6

この文書は型parameter、generic aliasとvalue binding、built-in型形成条件、specializationを定める。concrete syntaxは
[字句と文法](grammar.md)、Bufferの型形成は[AddressとBuffer](memory.md)を正とする。
型ごとのimplementation選択は[operation family](operation-families.md)を正とする。

## Declarationとapplication

type aliasとtop-level value bindingは明示的な型parameterを持てる。type applicationでは型argumentを明示するか、
value referenceの周辺型から推論させる。

```mal
Result<E, A> :: [E, A];
identity<A> :: A -> A := (value) -> value;

same :: Int32 -> Int32 := (value) -> identity(value);
```

generic value referenceのkind `Type`のargumentは、declarationのparameter型とoperand、result型と周辺の期待型をalias展開後に構造的に
unifyして決める。期待関数型から単独のreferenceも推論できる。直和continuation位置ではpayload型と、既知なら除去結果型を
期待関数型のconstraintにする。lambda argumentはparameter側が他のconstraintから確定した場合だけ
bodyを検査し、そのresultをconstraintに加える。周辺型が未確定なnumeric literalは他のconstraintを先に適用し、なお未確定なら
通常のliteral defaultを使う。constructor kindのargumentは構造から逆算せず、rigidな明示argumentとしてだけ使う。明示argumentは
declaration parameter列のprefixとして書け、残るkind `Type`のargumentは同じ局所solverが推論する。矛盾するconstraintと、解決後も
未確定なargumentはerrorであり、その箇所では型argumentを明示する。

推論対象はcall siteごとの型argumentだけである。generic本体のopaqueな型parameterはrigidであり、concrete typeへ具体化しない。
implementation候補やspecialization済みinstanceを推論の情報源にしない。`Buffer<A>`のpredefined operation（`new`、`get`、`put`、
`fill`、`copy`、`into`、`#`）はgeneric bindingではなく、receiverまたはargumentから`A`を決める。`make`と`from`は期待される
`Buffer<A>` resultから`A`を推論でき、期待型がなければ`make<A>`、`from<A>`と明示する。
[AddressとBuffer](memory.md#buffer)に各operationの型を定める。

型parameterの名前は、そのdeclarationから見える型名、すなわちpredefined型、直接requireしたfileから導入した型、同じfileの
top-level型と同じであってはならない。型parameterは型名をshadowしないため、declaration内の型名は常に一つのものを指す。
設計理由は[D089](../history/decisions/active/D089.md)に記録する。

型parameterはkind inferenceにより通常のsource typeまたはtype constructorを表す。kind annotation、user-defined kind、bound、
constraintはない。generic aliasはtransparentであり、型argumentを
代入して展開したcanonical typeと同じ型になる。alias右辺のtype expressionに直接現れないparameterはphantomであり、
canonical type構成時にそのtype argumentを展開しない。

別のalias applicationのargument位置に現れたparameterは使用済みとする。参照先aliasのphantom parameterを調べてこの判定を
逆向きに変えない。phantom argumentだけを通る自己参照は認め、実際の表現に寄与するrecursive generic aliasは拒否する。

generic binding自体はruntime valueではない。non-generic codeはconcrete type argumentでspecializeした単相valueだけを参照、capture、
applicationできる。generic本体ではscope内の型parameterをtype argumentに使え、外側のspecializationでargumentがconcreteになった時点で
参照先もspecializeする。local generalization、first-class polymorphism、higher-rank kind polymorphism、polymorphic recursion、
type reflection、type case、generic typeによるoverload resolution、generic extern declarationはない。self recursionは同じ
canonical type argument列を保つcallだけを認める。

## 型検査

generic本体はopaqueな型parameterとsignatureから導いたbuilt-in型形成条件の下で一度検査する。specialization後に本体へ新しい
operationを許可しない。比較、算術、encodingなどを必要とするgeneric functionは通常のfunctionまたはproduct valueとして
operationを受け取るか、[operation family](operation-families.md)のrequirementを持つ。numeric operatorとconversionはconcrete type
だけを列挙するclosed primitive familyである。

```mal
Equality<A> :: (A, A) -> Bool;

contains<A> :: (Equality<A>, A, A) -> Bool :=
    (equal, expected, actual) -> equal(expected, actual);

double<A> :: A -> A := (value) -> value + value; // error
```

## Requirements

`Requirements(T)`はalias展開後の型`T`が要求するbuilt-in judgmentの有限集合である。product、sum、functionは要素のrequirementを
再帰的に合併する。`Buffer<A>`は`Storable(A)`を加え、type argument内のrequirementも加える。

compilerは`Storable(T)`をclosedな定義で正規化する。storableなconcrete base caseは消去し、productとsumは各要素へ
分解し、opaqueな型parameterまたはopen application `F<A>`だけをatomとして残す。既知の非storable型はdeclarationで拒否する。
`Buffer<(A, UInt64)>`から得るrequirementは`Storable(A)`である。

generic aliasはdefinitionのresult type、generic value bindingは明示signatureからrequirementを集める。本体内だけに現れ、signatureから
導けないrequirementを型parameterへ要求するoperationはerrorである。型applicationはconcrete argumentを代入し、aliasを展開した後に
全requirementを検査する。generic本体内のapplicationでは、代入後のrequirementがcaller bindingのrequirementから導けることを検査する。
満たさないapplicationはspecialization前のcompile-time errorである。

`from<A>`と`buffer.into`は`Representable(A)`を要求する。opaqueな型parameterはrepresentableと仮定できないので、generic本体は
型parameterの要素にこれらを使えない。callerがconcrete type argumentで`from`をspecializeすると、copy primitiveがstatic
representationを選ぶ。

```mal
readFirst<A> :: Buffer<A> -> A := (buffer) -> buffer.get(0usize);
writeFirst<A> :: (Buffer<A>, A) -> Unit := (buffer, value) -> buffer.put(0usize, value);
```

`Buffer<A>`は通常のgeneric argument、parameter、resultとして使える。`Storable(A)`はBufferが要素を保持できることだけを示す。
`Representable(A)`はC host copy representationの存在だけを示し、Address referentのextent、permission、initialization、
lifetime、valid representationを証明しない。

## Specialization

name resolutionとtype checkingは型parameter、型application、opaque type variable、requirement、generic binding identityを所有する。
type checking後、compilerはentry pointから到達する、明示または推論済みのconcrete applicationを起点にspecialization graphを構成する。

specialization keyはgeneric binding identityとalias展開後のclosed canonical type argument列であり、kind `Type`以外のconstructor termも
含む。同じkeyはfileを跨いで共有する。
file-local opaque typeはhidden representationへ展開せず、declaration identityとcanonical type argumentをkeyに残す。
各nodeはgeneric typed bodyへ型argumentを代入して単相typed coreを一度生成し、到達するgeneric applicationをgraphへ加える。
本体内で型parameterをargumentに使ったapplicationも、この代入後にはconcreteなkeyになる。self recursionは同じkeyへのedgeとして閉じる。
異なる型argumentで自分を呼ぶbindingはdeclarationで拒否する。

一つのprogramで生成するspecialization nodeは65,536個までとする。次のnodeを加えると上限を超える場合、source programを型不正とは
せず、展開元binding、type argument列、limitを示すartifact生成failureとする。型の物理表現上限とcompiler processの一般的な
resource failureは別の規則である。specializationはapplicationを正規化し、file-local opaque typeをhidden representationへ消去する。
ANF以降はkind、constructor term、generic declaration、type argument、opaque boundary、requirement、dictionaryを受け取らない。

## Host境界

generic bindingはmal source間だけで使える。generated C header、extern ABI、host adapterへgeneric binding、specialization、型parameterを
公開しない。generic aliasはconcrete argumentを代入して完全に展開した後、[`HostMappable`](extern.md#host-mappable-type)で判定する。
