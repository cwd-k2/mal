# 型引数推論と型別operation family

Status: Partially accepted; generic implementation patternとhigher-kinded profileはExploratory

型引数推論とclosed canonical typeだけをkeyにするexact operation familyは採択済みである。現在の言語規則は
[parametric polymorphism](../spec/generics.md)と[operation family](../spec/operation-families.md)を正とする。この文書は採択理由と、
未採択のgeneric implementation pattern、higher-kinded constructor、lawful interface候補を管理する。

## 目的と設計境界

`loop`やcollection algorithmで、引数と期待resultから一意な型argumentを重ねて書かずに済ませる。同時に、`equal`、`hash`、
`format`、`add`など、一つのprincipal signatureを型ごとの実装へ結ぶoperationをgeneric codeから利用可能にする。

operation familyは同名で異なるsignatureを並べる一般的なoverloadではない。一つのfamily identityとgeneric signatureを持ち、
型argumentが確定した後に一つのimplementationへ解決する。implementation候補を型推論の情報源にしてはならない。型推論は
family signature、operand、期待resultだけから完了し、その後にimplementationを検索する。この順序により、別fileへimplementationを
追加しても既存callの推論結果を変えない。

選択はcompile-timeだけで行う。specialization後のprogramには通常のmonomorphic function referenceだけを残し、runtime dictionary、
type descriptor、dynamic dispatchを導入しない。

導入は、型引数推論、closed typeだけをkeyにするexact implementation、key内部に型parameterを持つgeneric implementation pattern、
higher-kinded constructorの順に分ける。後のprofileを前のprofileの採択条件にしない。stdlibの最初の土台は型引数推論とexact
implementationだけで構成でき、collection構造の再帰的なoperationとconstructor共通APIは利用例が揃ってから追加する。

## Source model

initializerを持たないgeneric value declarationはoperation familyを宣言する。

```mal
equal<T> :: (T, T) -> Bool;
hash<T> :: T -> UInt64;
add<T> :: (T, T) -> T;
```

既にfamilyとして宣言された名前にinitializerを置くとimplementationになる。初期profileではkeyの型argumentをすべてclosed typeとする。

```mal
equal<Symbol> :: (Symbol, Symbol) -> Bool :=
    (left, right) -> left == right;

add<Int32> :: (Int32, Int32) -> Int32 :=
    (left, right) -> left + right;
```

implementation annotationはfamily signatureへkeyの型argumentを代入したcanonical typeと一致しなければならない。parameterやresultを
狭める別signatureは、同じfamilyのimplementationとして受理しない。

同名familyが存在しないinitializer付き宣言は従来のgeneric bindingである。

```mal
double<T> :: T -> T := (value) -> add(value, value);
```

declarationはsource orderに従う。implementationは同じfileで先に宣言されたfamilyか、直接requireしたfileのfamilyへ追加する。
implementation item自身は新しいvalue名を導入しない。通常のbindingとfamilyは同じvalue namespaceで重複できない。

concrete grammarとして増えるのは、initializerを持たず`;`で終わるgeneric value declarationと、value declaration左辺で
`VALUE_IDENT`にtype-shapedな`<...>`を続けるheaderである。新しいtokenやdelimiterは増やさない。parserはheaderを未分類のまま保持し、
resolverがinitializerの有無と、既に解決済みのfamily identityから型parameter declaration、family argument、implementation patternへ
分類する。同名familyがなければ、initializer付きheaderの各項は従来どおり型parameter名でなければならない。

初期profileはkeyword、delimiter、requirement clauseを追加せず、上記header以外のsource formを増やさない。generic patternではfamilyが
宣言した型parameter名だけをpattern binderとして再利用する。例えば`equal<T>`のimplementation key内では`T`だけをbinderにできる。
未解決な別の型identifierを暗黙binderにしないため、型名のtypoが新しいparameterとして受理されることはない。この制限で表せないpatternは
専用binder構文を追加せず、拡張profileの対象外にする。higher-kinded signatureも既存のparameterとtype applicationの構文で記述し、
kind annotation構文は追加しない。

## 型引数推論

generic bindingとoperation familyは同じ局所constraint solverを使う。solverは次からconstraintを集める。

- declarationのparameter型と実引数の型
- declarationのresult型とapplication全体の期待型
- parameter型が確定したlambdaのbody result
- direct result blockのresult binderへ渡したpayload
- 周辺型が未確定なnumeric literal

推論はapplicationではなくgeneric value referenceを起点に行う。したがって即時applicationのoperandだけでなく、family valueを
higher-order argumentとして渡す位置の期待型からも型argumentを決められる。referenceを単独で置き、周辺型からも決まらなければ明示する。

内部ではconcrete type、generic本体のrigid type parameter、call siteだけに存在するinference variableを区別する。rigid parameterを
inference variableとして具体化してはならない。alias展開後のcanonical typeをunifyし、occurs checkと型形成条件を満たさない解を拒否する。
constraint解決後も未確定の型argumentがあれば明示を要求し、numeric literalのdefaultingは残るconstraintを解決した後に行う。

```mal
identity(1i32)                         // identity<Int32>
values.mapBuffer((value) -> value.u64) // Aはreceiver、Bはlambda resultから推論
add(left, right)                       // operandからfamily argumentを推論
```

family parameterがparameterにもresultにも構造的に現れない`sizeOf<T> :: ByteSize`のようなfamilyは、implementation keyから逆向きに
推論しないため`sizeOf<Int32>`と明示する。利用可能なimplementation集合を増減しても型推論を変えない原則を、この場合も優先する。

standalone lambdaは引き続き期待関数型を必要とし、lambda parameter型をbodyのoperator候補から探索しない。generic applicationのargument
として期待される関数型のparameter側が他のconstraintから確定した場合だけ、そのlambda bodyを検査してresult側のinference variableを
拘束する。
`loop<A, B>`のstepがdirect result blockを返す場合は、`[A, ?B]`という一時的なsum型を作り、`break(value)`のpayloadから`B`を
拘束する。`B`項のresult binderを適用するreachable pathがなく、周辺の期待型もなければ`B`は未確定なので、明示型argumentを要求する。

## Operation requirement

generic本体でrigid parameterを持つfamily referenceは、implementationを選ばず型付きoperation requirementとして保存する。
これは現行仕様の型形成条件`Requirements(T)`とは別の集合であり、ここでは`Operations(binding)`と表す。

```mal
double<T> :: T -> T := (value) -> add(value, value);
```

この本体はfamily signatureにより一度型検査でき、次を導く。

```text
Operations(double<T>) = { add<T> }
```

別のgeneric bindingを参照する場合はcalleeのoperation requirementを型argumentで置換して伝播する。operation requirementsはbodyから
推論し、checked public APIの一部としてeditorとdiagnosticがsignatureとともに表示する。初期profileは明示的なrequirement構文を持たない。
body変更によるcontract変化をsource annotationで固定する必要が実例から確認された場合は、operation familyと切り離して再検討する。

concrete goalもsymbolic goalも型はfamily signatureだけで検査する。implementation tableの重複はprogram全体で拒否し、到達したgoalの
missing implementationはspecialization時に報告する。これにより未使用のgeneric bindingを成立させるためだけのimplementationを要求せず、
implementation候補を型検査の根拠にもしない。

```text
double<Int32>
  requires add<Int32>
  resolves to the unique implementation of add<Int32>
```

解決できない場合は、entryから不足したimplementationまでのgeneric specializationとoperation requirementのchainを診断する。

familyのresultはfunction型に限らない。value familyのimplementationには既存のtop-level initializer規則をそのまま適用する。

```mal
zero<T> :: T;

zero<Int32> :: Int32 := 0i32;
zero<Symbol> :: Symbol := "";
```

この規則はliteralやclosed aggregateを受理する一方、`make<Int32>(0usize)`や別の`zero<A>`をtop-levelで評価するinitializerを
拒否する。operation familyの導入だけを理由にglobal initialization、implicit thunk、constant evaluatorを追加しない。
constructionやfresh identityが必要なoperationは`empty<T> :: Unit -> T`のようにapplicationを明示する。

## 初期profile: exact implementation

初期profileのimplementation keyは、family identityとalias展開後のclosed canonical type argument列である。同じkeyをsource graph内に
二つ定義してはならない。transparent aliasは新しいkeyを作らない。

```mal
UserId :: Int64;
```

`equal<UserId>`と`equal<Int64>`は同じkeyになる。implementationはreachable source graph全体から収集し、familyを参照できる
local scopeだけでなく、specializationするprogram全体で一意にする。同じcanonical keyのimplementationは到達するgoalがなくても
拒否する。package境界が必要になるまではorphan ruleを導入しない。将来のpackage ruleはimplementation declarationのadmissionを
制限できるが、program内のlookupは引き続きcanonical keyごとに一意とする。

exact implementationだけでも、generic collectionはelement operationをoperation requirementとして利用できる。次はflatなcarrierの
説明用shapeであり、hash tableの採択済みrepresentationではない。

```mal
Array<T> :: Buffer<T>;
HashMap<K, V> :: (Buffer<UInt8>, Buffer<K>, Buffer<V>, USize);

arrayContains<T> :: (Array<T>, T) -> Bool :=
    (values, expected) -> containsBy<T>(values, expected, equal<T>);

hashMapGet<K, V> :: (HashMap<K, V>, K) -> [Unit, V] :=
    (table, key) -> {
        code := hash(key);
        _hashMapProbe<K, V>(table, key, code, equal<K>);
    };
```

ここで`containsBy`と`_hashMapProbe`はequality functionを受け取る通常のhelperとする。checkerは`arrayContains<T>`から
`equal<T>`、`hashMapGet<K, V>`から`hash<K>`と`equal<K>`を導く。`hashMapGet<Symbol, Int32>`をspecializeするprogramに
`hash<Symbol>`と`equal<Symbol>`のexact implementationがあれば、collection本体を型ごとに書き直さずに済む。

一方、collection自体を別のgeneric operationへ渡す実装は再利用できない。例えば初期profileで
`equal<Buffer<Int32>>`を定義できても、`Buffer<UInt8>`や`Buffer<Symbol>`には別のexact implementationが必要になる。

## 拡張profile: generic implementation pattern

次の拡張では、implementation keyの内部でfamily declarationの型parameter名をpattern binderとして再利用できる。family parameterの
個数を超えるbinderは導入できず、key全体が一つのbinderだけになるcatch-all patternは認めない。

```mal
equal<Buffer<T>> :: (Buffer<T>, Buffer<T>) -> Bool :=
    (left, right) -> equalBufferBy<T>(left, right, equal<T>);

format<[Unit, T]> :: [Unit, T] -> Symbol := (value) -> value[
    () -> "none",
    (item) -> "some(" + format(item) + ")"
];
```

ここで`equalBufferBy`はequality functionを受け取る通常のgeneric helperとする。最初のbodyから`equal<T>`、二つ目から
`format<T>`を導く。goal `equal<Buffer<Symbol>>`はpatternを
`T := Symbol`でmatchし、そのoperation requirement `equal<Symbol>`を続けて解決する。これによりcollectionやsum/product構造に対する
operationを要素型ごとに列挙せず定義できる。

`Array<T> :: Buffer<T>`のようなtransparent aliasをstdlibが提供しても、`equal<Array<T>>`はcanonicalには
`equal<Buffer<T>>`と同じpatternである。aliasはcollectionのnominal identityやimmutable invariantを作らない。同様に、
`HashMap<K, V>`へ書いたpatternはalias展開後のproduct shapeへ作用するため、HashMap固有のinvariantを必要とするoperationは
`hashMapGet`のような通常のnamed functionに置く。operation familyのgeneric patternは構造から一意に導けるbehaviorに限る。

## Matching、coherence、termination

generic implementationはcanonical typeに対するfirst-order pattern matchingだけを行う。initial extensionでは次を要求する。

- pattern binderはfamily declarationに現れる型parameter名に限り、implementation annotationとbodyでは新しいrigid parameterとして扱う。
- familyごとのimplementation patternはpairwiseにunify不能であり、どのconcrete goalにも高々一つだけmatchする。
- exact implementationをgeneric patternより優先する規則は設けず、両方が同じgoalへmatchするなら重複として拒否する。
- argument列全体がbinderだけになるcatch-all patternは認めず、`Buffer<T>`、product、sum、具体constructorなどをkeyに残す。
- implementationから導く各operation requirementの型argumentは、implementation keyの対応する構造より真に小さいsubtermにする。

implementation自身と同じconcrete keyへのdirect self referenceは、通常bindingのdirect self recursionと同じ条件で再帰として扱い、
新しいoperation requirement edgeにしない。別keyへのcallと、direct lambda RHS以外から形成するcycleは通常のoperation requirementとして
検査する。

例えば`equal<Buffer<T>>`から`equal<T>`への依存は小さくなる。反対に`f<[Unit, T]>`から`f<[Unit, [Unit, T]]>`を要求する定義は
拒否する。この保守的な規則で表現できない相互再帰、同じ大きさのfamily間依存、exact overrideが実際に必要になった場合だけ、
termination graphまたはspecificity relationを別の拡張として設計する。

family declarationにない型identifierは通常の型名として解決し、見つからなければerrorにする。これにより`Symbol`のtypoを新しいpattern
binderとして受理しない。一つのfamily argumentを複数の独立parameterへ分解するpatternなど、このbinder数制限を越える用途は初期拡張で
表現できない。実例が十分に集まるまでは、そのためのbinder構文を追加しない。

## 後続profile: Higher-kinded constructor abstraction

generic implementation patternはhigher-kinded polymorphismではない。`equal<Buffer<T>>`は既知の`Buffer` constructorへmatchできるが、
任意のconstructor `F`を受け取って`F<A>`を形成することはできない。したがってこの拡張まででは、constructorごとのoperationを定義する。

```text
bufferMap<A, B> :: (Buffer<A>, A -> B) -> Buffer<B>
optionMap<A, B> :: ([Unit, A], A -> B) -> [Unit, B]
```

`fmap`、`pure`、`ap`、`bind`をconstructor共通のfamilyにするには、概念上次のsignatureが必要になる。

```mal
fmap<F, A, B> :: (A -> B) -> (F<A> -> F<B>);
pure<F, A> :: A -> F<A>;
ap<F, A, B> :: F<A -> B> -> (F<A> -> F<B>);
bind<F, A, B> :: F<A> -> ((A -> F<B>) -> F<B>);
```

`F<A>`と`F<B>`がvalue typeの位置に現れるため、`A : Type`、`B : Type`、`F : Type -> Type`というkind constraintを得られる。
source上で`F<_>`のようなarity annotationを要求する必要はない。現在は型parameterをkind `Type`へ固定しているが、この拡張では未確定のkindを
割り当て、constructor位置とargument位置から解決する。`Input`と`Output`を別の`Type` parameterへ平坦化すると、`fmap`のoutputを
implementation候補から決める必要が生じ、型を確定してからimplementationを選ぶ原則を崩す。

kind inferenceが確定するのはconstructorの形であり、すべての型argumentに対して適用可能であることではない。`Buffer`もkind上は
`Type -> Type`として扱い、具体的な`Buffer<A>`が形成可能かはspecialization後に既存のtype formation judgmentで検査する。
`Buffer`の生成や変換にuser codeから利用できない操作が必要なら、そのimplementationは現在のpredefined Buffer operationと同じ
compiler/runtime境界に置ける。higher-kinded abstractionはその能力を公開したり、kindへ符号化したりする必要はない。

この分離をしても、形成不能なtype applicationが有効になるわけではない。functionを格納できないなら`Buffer<A -> B>`を必要とする上記の
`ap`は`F := Buffer`でspecializeできない。familyの実装可能性は具体化したsignature全体で検査する。

初期のhigher-kinded profileはconstructorの部分適用構文を持たない。二引数constructorの一方を固定したい場合は、既存のgeneric aliasで
一引数constructorへ適応する。

```mal
SymbolMap<V> :: HashMap<Symbol, V>;
```

constructor位置の`SymbolMap`は`Type -> Type`と推論し、specialization keyではalias展開後のconstructorへcanonicalizeする。これで不足する
実例が確認されるまでは、`HashMap<K, _>`のようなplaceholder構文を導入しない。

`pure`はinput carrierを持たないため、higher-kinded導入後も`pure(value)`だけでは`F`を決められない。`Buffer<Int32>`などの期待result型か
明示constructor argumentを要求する。Functor、Applicative、Monadのlawは型検査から自動的に得られず、family contractとtestが所有する。
特にmutable identityを持つ`Buffer`の`pure`と`bind`、`ap`のzipと直積の選択は一意な構造規則から決まらないため、global familyへ
canonical policyを置くか、`bufferMap`、`zipApply`などのnamed operationへ残すかを個別に判断する。

higher-kinded constructor自体を型引数推論とexact operation familyの採否から独立した後続proposalとする。その初期profileでは
kindをconstructorの使用位置から推論し、具体化後のtype formationを別に検査する。constructorの部分適用はunary generic
aliasで適応し、それで不足する実例が得られるまでplaceholder構文を追加しない。

associated type、operation群とlawを一つのlawful instanceとして束ねるinterfaceは、higher-kinded constructorとも独立した
後続proposalとする。collection APIの重複と必要なlawを具体例で確認するまでは現在の提案へ含めない。

## Compiler boundary

parserは未分類のdeclaration header、resolverはfamilyとimplementationのidentity、checkerは推論済みgeneric referenceと型付きoperation
requirementを所有する。checkerはfamily signatureとimplementation annotationの一致、keyの重複とpatternのoverlapをprogram全体で検査し、
concreteとsymbolicの両方のoperation goalをchecked ASTへ残す。specializerはgeneric bindingの型parameterを置換し、到達したgoalが
concreteになった時点でimplementation tableを検索する。解決したreferenceは通常のbinding identityへ置き換える。ANF以降はoperation family、
operation requirement、inference variable、implementation patternを受け取らない。

operation解決はspecialization中のcompile-time選択であり、operand、family value、applicationの既存評価順を変えない。implementationの
function body、extern effect、trapは、解決後の通常bindingを同じsource位置で参照した場合と同じ時点に発生する。

型argumentを選択済みのfamily valueは通常のvalueと同様に参照、capture、higher-order argumentとして使える。value familyもlocalな式では
同じ規則で参照できるが、function型でなければapplicationできない。symbolicなfamily valueはoperation requirementを運び、specialization後に
同じconcrete implementation identityへ解決する。generic patternから生成するinstanceも既存の65,536 specialization node上限に数え、
operation解決だけで別の無制限な展開graphを作らない。

`make`と`from`はuser-defined generic bindingではないため、期待される`Buffer<T>`から`T`を推論する専用intrinsic規則を持つ。
resultにもoperandにもelement型が現れないcallは従来どおり明示型argumentを要求する。

## 導入状況と後続の検証境界

局所inference variableとstructural unification、lambda resultとdirect result block payloadのconstraint、exact familyの
parse・resolve・check、operation requirementの収集と伝播、specialization時のexact lookup、value familyは実装済みである。
推論形と明示形のspecialization key共有、canonical keyの重複、missing implementation、operation referenceがbackendへ残らないことを
focused testと既存backendを通る実行caseで検査する。

generic implementation patternを追加する場合は、exact profileの利用例と衝突を先に評価する。`Buffer<T>`、product、sumの
non-overlap、strictly-smaller requirement、alias展開をfocused testで固定し、代表programがexact profileと同じ既存core/backendで
実行されることを採択条件とする。

## 後続profileの未決事項

- strictly-smaller規則で不足する実例があるか、specificityやtermination proofを追加する価値があるか。
- exact profileの実例によって、bodyから推論したoperation requirementを固定する明示annotationが必要となるか。
- higher-kinded constructorまたはlawful interfaceが、constructor別のnamed APIよりsystem全体のcontractを小さくする実例が得られるか。

primitive numeric operatorはclosed primitive familyとして保ち、operation familyのimplementation bodyから利用する。型引数推論と
exact profileは、generic implementation pattern、higher-kinded constructor、lawful interfaceの未決事項に依存せず採否できる。
