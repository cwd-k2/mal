# generic operation implementation

Status: Exploratory

この文書は、採択済みのexact operation familyを、canonical type patternに対するgeneric implementationへ拡張する案を管理する。
現在のdeclaration、型引数推論、operation requirement、exact implementation selectionは
[operation family](../spec/operation-families.md)と[parametric polymorphism](../spec/generics.md)を正とし、ここでは再定義しない。
higher-kindedな型parameterは[higher-kinded constructor abstraction](higher-kinded-types.md)で別に扱う。
[file-local opaque type](file-local-opaque-types.md)はhidden representationへ展開せず、opaque constructorをcanonical patternの
headとして扱う。

## 目的と範囲

exact profileでは、`equal<Buffer<Int32>>`と`equal<Buffer<Symbol>>`に別々のimplementationが必要である。この案は、型の構造から
一意に導けるoperationを要素型ごとに列挙せず定義できるようにする。

```mal
equal<Buffer<T>> :: (Buffer<T>, Buffer<T>) -> Bool :=
    (left, right) -> equalBufferBy(left, right, equal<T>);

format<[Unit, T]> :: [Unit, T] -> Symbol := (value) -> value[
    () -> "none",
    (item) -> "some(" + format(item) + ")"
];
```

family declarationの型parameter名をimplementation key内のpattern binderとして再利用する。別の暗黙binder構文は追加せず、
family parameter数を超える独立parameterへの分解と、argument列全体がbinderだけになるcatch-all patternは初期profileに含めない。

transparent aliasはkeyを作らない。`Array<T> :: Buffer<T>`なら`equal<Array<T>>`と`equal<Buffer<T>>`は同じcanonical patternである。
alias固有のinvariantを必要とするoperationは通常のnamed functionに置き、generic family implementationはcanonical structureから
一意に導けるbehaviorに限る。

opaque typeはtransparent aliasと異なりdeclaration identityをkeyに残す。hidden representationが`Pool<USize, T>`でも、
`equal<Buffer<T>>`と`equal<Pool<USize, T>>`は異なるpatternであり、宣言元fileのrepresentation viewによって重複しない。

## Matching、coherence、termination

generic implementationはcanonical typeに対するfirst-order pattern matchingだけを行い、初期profileでは次を要求する。

- pattern binderはfamily declarationの型parameter名に限り、annotationとbodyではrigid parameterとして扱う。
- familyごとのimplementation patternはpairwiseにunify不能であり、どのconcrete goalにも高々一つだけmatchする。
- exact implementationをgeneric patternより優先せず、両方が同じgoalへmatchするなら重複として拒否する。
- argument列全体がbinderだけになるcatch-all patternを認めず、具体constructor、product、sumなどの構造をkeyに残す。
- bodyから導く各operation requirementの型argumentは、implementation keyの対応する構造より真に小さいsubtermとする。

`equal<Buffer<T>>`から`equal<T>`への依存は小さくなる。`f<[Unit, T]>`から`f<[Unit, [Unit, T]]>`を要求する定義は拒否する。
同じconcrete keyへのdirect self referenceは通常bindingのdirect self recursionとして扱い、別keyへのcallはoperation requirementとして
縮小条件を検査する。相互再帰、同じ大きさのfamily間依存、specificity、exact overrideは実例が得られるまで追加しない。

## Compiler boundary

resolverはfamily identityに加えてpattern binderを分類する。checkerはimplementation annotationを検査し、canonical patternのoverlapと
requirementの縮小条件を判定する。specializerはconcrete goalをpatternへmatchし、得た置換を通常のgeneric specializationへ渡す。
選択後は通常のmonomorphic binding referenceへ置き換え、core以降へpattern、operation requirement、型argumentを残さない。

generic implementationから生成するinstanceも既存のspecialization node上限に数え、operation解決だけの無制限な展開graphを作らない。
未採択のpattern表現を現在のchecked ASTやbackendへ先行して追加しない。

## 採択条件と未決事項

`Buffer<T>`、product、sum、transparent aliasを使うnon-overlap caseと、overlap、非縮小requirement、暗黙binderを拒否するcaseを
focused testで固定する。exact implementationだけで書いた同じprogramとobservable behaviorが一致し、既存coreとbackendを変更せず
実行できることを確認する。

採択前に、exact profileの実例から型構造ごとの列挙が実際の重複になっていることを確認する。strictly-smaller規則で不足する実例、
specificityやtermination proof、bodyから推論したoperation requirementを固定する明示annotationは、必要性が確認された時点で
それぞれ独立に検討する。
