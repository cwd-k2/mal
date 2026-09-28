# higher-kinded constructor abstraction

Status: Exploratory

この文書は、型parameterをvalue typeだけでなくtype constructorとして扱う後続profileと、それをoperation familyで利用する候補を管理する。
現在のparametric polymorphismは型parameterをkind `Type`へ固定する。[parametric polymorphism](../spec/generics.md)と
[operation family](../spec/operation-families.md)が現行規則であり、この文書の構文とoperationは未採択である。
canonical type patternだけを拡張する案は[generic operation implementation](generic-operation-implementations.md)で別に扱う。

## 必要になる型

constructorごとの`bufferMap`や`optionMap`を一つのoperationへまとめるには、概念上次のsignatureが必要になる。

```mal
fmap<F, A, B> :: (A -> B) -> (F<A> -> F<B>);
pure<F, A> :: A -> F<A>;
ap<F, A, B> :: F<A -> B> -> (F<A> -> F<B>);
bind<F, A, B> :: F<A> -> ((A -> F<B>) -> F<B>);
```

`F<A>`と`F<B>`の使用位置から`A : Type`、`B : Type`、`F : Type -> Type`を推論する案とし、初期profileではkind annotation構文を
追加しない。kind inferenceが保証するのはconstructorの形だけである。具体化した`F<A>`がvalue typeとして形成可能かは、既存の
type formation judgmentで別に検査する。例えばfunctionを格納できないconstructorでは`F<A -> B>`を要求する`ap`を利用できない。

constructorの部分適用構文も初期profileには加えない。二引数constructorの一方を固定する場合は既存のgeneric aliasで一引数へ適応する。

```mal
SymbolMap<V> :: HashMap<Symbol, V>;
```

constructor位置のaliasは展開後のconstructorへcanonicalizeする。これで不足する実例が得られるまでは、`HashMap<K, _>`のような
placeholder構文を導入しない。

## Operation familyとの境界

`pure(value)`だけでは`F`を決められないため、期待result型か明示constructor argumentを要求する。implementation候補から型argumentを
逆算せず、型を確定してからimplementationを選ぶ現行原則を維持する。

Functor、Applicative、Monadのlawはkindと型検査から自動的には得られない。mutable identityを持つ`Buffer`では`pure`と`bind`、
`ap`のzipと直積の選択もcanonical structureから一意に決まらない。global operation familyへpolicyを置くか、`bufferMap`や
`zipApply`のようなnamed operationへ残すかを個別に判断する。

associated type、operation群、lawを一つのinstanceとして束ねるinterfaceはhigher-kinded typeそのものと分ける。constructor別APIの
重複と必要なlawを具体例で確認するまでは、lawful interfaceとruntime dictionaryをこのproposalへ含めない。

## 採択前の未決事項

- constructor別のnamed APIよりsystem全体のcontractを小さくする実例があるか。
- kindを使用位置だけから一意に推論でき、diagnosticとeditor表示を簡潔に保てるか。
- unary generic aliasだけでは表しにくく、constructor部分適用構文を正当化する利用例があるか。
- canonical constructor key、type formation、specialization node上限を既存のcompile-time selection境界内で扱えるか。
