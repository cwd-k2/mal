# higher-kinded constructor abstraction

Status: Exploratory; source profile and static boundary specified

この文書は、型parameterをvalue typeだけでなくtype constructorとして扱う候補profileを定める。現在の規範的な言語は
[parametric polymorphism](../spec/generics.md)と[operation family](../spec/operation-families.md)を正とし、型parameterを
kind `Type`へ固定する。この文書の規則は未採択である。

## 目的と範囲

同じshapeのoperationをconstructorごとに定義し、generic codeからcompile-timeに選べるようにする。

```mal
fmap<F, A, B> :: ((A -> B), F<A>) -> F<B>;
pure<F, A> :: A -> F<A>;
flatten<F, A> :: F<F<A>> -> F<A>;
```

kind、type constructor、型applicationはcompile-timeだけに存在する。specialization後のcore、runtime representation、extern ABIへ
constructor value、kind、dictionary、type descriptorを渡さない。Functorなどのlaw、associated type、interface、runtime dispatch、
type case、dependent typeはこのprofileに含めない。

例に使う`fmap`、`pure`、`flatten`はpredefined operationを追加する提案ではない。どのconstructorにどのoperationとlawを与えるかは
program側のfamilyまたはnamed operationが所有する。特にmutable identityを持つ`Buffer`の`pure`や`flatten`にcanonicalなpolicyはない。

## Surface language

kind annotation、匿名のtype lambda、placeholder構文を追加しない。既存のgeneric declaration binderをabstraction、`T<A, ...>`を
applicationとして使う。

```mal
Id<X> :: X;
Apply<F, X> :: F<X>;
Compose<F, G, X> :: F<G<X>>;

Maybe<A> :: [Unit, A];
Result<E, A> :: [E, A];
Nested<F, G, A> :: F<G<A>>;
```

`C<P1, ..., Pn> :: T`は型levelでは`P1`から`Pn`までを順にabstractしたtermである。applicationのcomma区切りは左から適用する。
このためdeclarationのbinder数より少ないpartial applicationと、resultがconstructorである場合の追加applicationを認める。

```mal
Result<Symbol>                 // Type -> Type
Nested<Buffer, Maybe>          // Type -> Type
Apply<Result<Symbol>, Int32>   // Type
Id<Buffer, Int32>              // Type
```

既存文法はapplicationのheadを型名または型parameterに限定したままでよい。匿名abstractionがないため、application resultへの追加argumentは
同じ`<...>`へflattenできる。`(Id<Buffer>)<Int32>`は追加せず、`Id<Buffer, Int32>`と書く。

transparent aliasの右辺はkind `Type`に限定しない。opaque typeのrepresentation、value signatureを構成する各型、product、sum、function、
`Buffer`の要素は従来どおりkind `Type`を要求する。external typeはkind `Type`、`Buffer`はkind `Type -> Type`である。

## Kind inference

compiler内部のkindは次だけからなる。`->`は右結合とする。

```text
K ::= Type | K -> K | kind inference variable
```

各type declarationのparameterとresultへfreshなkind変数を与え、application、built-in type form、opaque representation、value
signatureから等式を集め、occurs check付きunificationでprincipal kindを求める。type名はfile全体から見えるため、dependency SCCへ
一度monomorphicなkind変数を割り当て、参照先を含むconstraintを解いてからacyclicな単位をgeneralizeする。正規化時にforceされるalias
cycleは後述の規則で拒否する。

declarationの環境に残らない未解決kind変数はgeneralizeし、参照ごとにfreshにinstantiateする。これはrank-1 kind polymorphismであり、
polymorphic constructor自体をparameterとして保持するhigher-rank kindは認めない。

概念上、次のkindをsource annotationなしで得る。

```text
Id      : forall k. k -> k
Apply   : forall k1 k2. (k1 -> k2) -> k1 -> k2
Compose : forall k1 k2 k3.
          (k2 -> k3) -> (k1 -> k2) -> k1 -> k3
fmap parameters:
          F : Type -> Type
          A : Type
          B : Type
```

使用されないparameterのkindを利用者が狭める手段は置かない。例えば`Phantom<F, A> :: A`の`F`は任意のkindを取る。不要な制約を
API contractとして宣言する用途だけがkind annotationなしでは表現できない。

kind inferenceはconstructorの形だけを保証する。`F<A>`が`Type`でも、その具体化が`Storable`、`Representable`などの型形成条件を
満たすかは既存judgmentで別に検査する。open termに対するrequirementは`Storable(F<A>)`のようにterm全体を保持し、substitutionと
正規化後に分解または判定する。

## 正規化と型等価

generic transparent aliasは型level abstractionとして展開し、applicationをbeta reduceする。alpha同値とeta同値もcanonicalizationへ
含める。したがって次のconstructorは同じcanonical termになる。

```mal
Maybe<A> :: [Unit, A];
Optional<A> :: Maybe<A>;
Forward<F, A> :: F<A>;
```

`Maybe`と`Optional`、`Forward<Maybe>`と`Maybe`はそれぞれ等しい。opaque type constructorはdeclaration identityを保持し、hidden
representationへ展開してconstructor equalityを変えない。

型levelの一般再帰、type case、polymorphic recursionは認めない。alias expansionが同じaliasを再びforceするcycleは拒否する。
phantom argumentは現在と同様に展開しないため、使われないargumentだけを通るcycleは値表現も型計算も再帰させず、許可できる。
この非strictなphantom消去後のsimply-kinded termは正規化を停止する。

## Value referenceの型argument推論

kind `Type`のparameterは現在と同じ局所的な構造unificationで推論する。constructor kindのparameterは、rigid constructorのargumentとして
term全体に一致する場合だけ推論してよい。未知parameterをcalleeとするflex applicationは逆算しない。

```text
opaque Witness<F> :: Unit;

Witness<F>  ~ Witness<Buffer> から F = Buffer は推論できる
F<A>        ~ Buffer<Int32>    から F は推論しない
```

後者は`F = Buffer`以外にconstant constructorも解となり、一般にはhigher-order unificationになる。未確定のconstructor argumentは
明示を要求する。

```mal
mapped := fmap<Buffer>(convert, values);
failed := fmap<Result<Symbol>>(convert, result);
```

value referenceへ書いたtype argumentはdeclaration parameterの先頭から対応し、残りを推論する。placeholderは導入しない。このためAPIは
通常constructor parameterを`Type` parameterより前へ置く。implementation候補やspecialization済みinstanceは引き続き推論に使わない。

## Operation family

family signatureはconstructor parameterを持てる。generic bodyのsymbolic referenceはconstructor termを含む
`OperationRequirement`として保存し、specialization時に全argumentがclosedになってからimplementationを選ぶ。

```mal
fmap<F, A, B> :: ((A -> B), F<A>) -> F<B>;

fmap<Maybe, A, B> :: ((A -> B), Maybe<A>) -> Maybe<B> := ...;
fmap<Buffer, A, B> :: ((A -> B), Buffer<A>) -> Buffer<B> := ...;
```

coherence判定をfirst-orderに保つため、初期profileではimplementation keyのconstructor-kind位置をclosed canonical constructor termに
限定する。constructor-kindのpattern variableをapplicationのcalleeやconstructor内部へ置くpatternは認めない。上例の`Maybe`、`Buffer`、
`Result<Symbol>`は使えるが、`Compose<F, G>`をpatternとして`F`と`G`を抽出するimplementationは使えない。

kind `Type`の位置には現在のnon-overlapping generic patternとproper structural subtermによる減少規則を維持する。constructor argumentを
含むrequirement keyはkind付きcanonical termとして比較し、同じfamilyの同じcanonical keyへの直接自己再帰だけを従来どおり認める。

## Specializationと境界

specialization keyのtype argumentはkind `Type`に限らず、closed canonical constructor termを含める。transparent alias identityはkeyへ
残さず、opaque constructor identityとそのcanonical argumentは残す。generic valueのphantom constructor argumentも現在のphantom
`Type` argumentと同様にkeyの一部とする。

generic bodyへclosed termを代入し、すべてのvalue typeと型形成requirementを正規化した後、到達するbindingとoperation familyを従来の
上限内でspecializeする。成功した単相programだけをcoreへ渡す。constructorのpartial applicationやopen termはC header、host adapter、
extern declarationに公開せず、kind `Type`まで飽和した型だけを既存の`HostMappable`規則で判定する。

## Diagnosticとeditor

kind mismatchは、kind構文をsourceへ要求せず、使用位置と推論した形を表示する。

```text
expected a value type, but `Result<Symbol>` is a type constructor
expected `Type -> Type`, but `F` is used with two arguments
cannot infer constructor argument `F`; write `fmap<Buffer>(...)`
```

editor hoverはparameterとconstructorの推論済みkindを表示してよい。completionはkindが一致するvisible constructorだけへ絞れるが、
kind inferenceやname resolutionの成功条件にはしない。

## 採択と実装の境界

採択時は`types.md`、`generics.md`、`operation-families.md`、`grammar.md`、`scope.md`を同時に更新し、HKTを部分的なsyntaxだけとして公開しない。
実装は次の境界で分ける。

1. resolverまでのtype applicationをargument数で拒否せず、kind inference用termとして保持する。
2. checkerへkind scheme、type term normalization、proper `Type`へのadmissionを導入する。
3. local inferenceをrigid constructor matchingまで拡張し、flex applicationをdiagnosticにする。
4. operation pattern、requirement、specialization keyをkind付きcanonical termへ拡張する。
5. editor表示、formatterの既存構文、focused positive/negative test、単相core境界を検証する。

最低限、partial/追加application、kind-polymorphic alias、kind occurs check、unsaturated proper type、constructor argument明示、rigid位置の推論、
operation keyのoverlap拒否、aliasのbeta/eta同値、forced recursion拒否、opaque identity、`Storable(F<A>)`、specialization後にconstructor termが
残らないことを個別testで固定する。
