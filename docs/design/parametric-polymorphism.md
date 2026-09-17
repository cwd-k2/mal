# parametric polymorphismと型index付きprimitiveの試案

Status: Discussion draft (2026-09-17)

この文書は型ごとに同じ構造とcontrolを複製する摩擦を減らすparametric polymorphism、built-in indexed typeの型形成、
specialization境界を定める。memory layoutは[`Address`、target size、layout、placementの試案](size-and-alignment.md)、
owned sequenceとのtransferは[`Region`と`Packed`の試案](region-and-packed.md)が所有する。現行の規範は
[`types`](../spec/types.md)、[`expressions`](../spec/expressions.md)、[`grammar`](../spec/grammar.md)であり、この試案だけを
根拠にsourceやABIを変更しない。

## 目的

型parameterはproduct、sum、functionと明示的に渡したoperationの配線を型ごとに書き直す摩擦を減らす。operationは通常の
functionまたはproduct valueとして明示的に渡し、representation capabilityはbuilt-in indexed typeから得る。

```mal
Result<E, A> :: [E, A];

identity<A> :: A -> A := (value) -> value;

mapResult<E, A, B> :: (Result<E, A>, A -> B) -> Result<E, B> :=
    (result, transform) -> {
        result[
            (error) -> [failure, success] => failure(error),
            (value) -> [failure, success] => success(transform(value))
        ];
    };
```

## Concrete syntax

`<...>`はtop-level declarationの型parameter、type expressionの型argument、generic value nameの明示的型argumentにだけ使う。
型argumentは最初のprofileですべて明示し、specializationした単相valueは通常のfunction valueとして扱う。

```mal
IntResult :: Result<Symbol, Int32>;
same :: Int32 -> Int32 := (value) -> identity<Int32>(value);
answer :: Unit -> Int32 := () -> identity<Int32>(42);
```

resolverはgeneric argument suffixを持つidentifierが対応するgeneric declarationであることを検査する。`<...>`とcomparison、
`>>`、applicationとの構文上の区別、結合順序、formatter規則は
[`generic memory surface syntax`](memory-syntax.md)を正とする。

## ユーザー定義の多相性

最初のprofileは次に限定する。

- type aliasとtop-level value bindingだけが明示した型parameterを持てる。
- 型parameterは通常のsource typeを表し、user-definedなkind、bound、constraintを宣言できない。
- generic本体はopaqueな型parameterとsignatureから導いたbuilt-in型形成条件の下で一度検査する。
- specialization後に本体へ新しいoperationを許可しない。
- generic binding自体はruntime valueではなく、具体化した単相valueだけを参照、capture、applicationできる。
- local generalization、first-class polymorphism、higher-kinded type、polymorphic recursion、type reflection、type case、
  generic typeによるoverload resolutionを認めない。
- self recursionは同じ型argument列を保つcallだけを認める。
- generic extern declarationを認めない。

generic aliasはtransparentであり、型argumentを代入して展開したcanonical typeと同じ型になる。parameter、application、
specializationにruntime identity、descriptor、layoutを与えない。recursive generic aliasは通常のaliasと同じく拒否する。

## 型形成条件

`Requirements(T)`をalias展開後の型`T`が要求するbuilt-in judgmentの有限集合とする。product、sum、functionは要素のrequirementを
再帰的に合併する。`Cursor<A>`、`Region<A>`、`Packed<A>`は`Representable(A)`を加え、それらのtype argumentに含まれる
requirementも再帰的に加える。他の最初のprofileの型はrequirementを加えない。

compilerは`Representable(T)`をclosedな定義で正規化する。representableなconcrete base caseは消去し、productとsumは各要素へ
分解し、opaqueな型parameterだけをatomとして残す。functionやindexed typeなど明らかに対象外の型が現れたgeneric declarationは
その場で拒否する。これにより`Cursor<(A, UInt64)>`から得るrequirementは`Representable(A)`になる。

generic aliasはdefinitionのresult type、generic value bindingは明示したsignatureからrequirementを集め、opaqueな型parameterと
そのrequirementの下で一度検査する。したがってparameter、result、nested product、展開したgeneric aliasのどこにindexed typeが
現れても同じ条件を導く。

```text
Representable(A)
────────────────────────
Cursor<A> type

Cursor<A>がsignatureに現れる
────────────────────────
generic本体でRepresentable(A)
```

型applicationではconcrete type argumentを代入し、aliasを展開した後に全requirementを検査する。満たさないapplicationは
specializationを始める前のcompile-time errorである。本体内にだけ現れ、signatureから導けないrequirementを型parameterへ要求する
operationはerrorとする。この規則はclosedなbuilt-in型形成条件だけを扱い、user-defined constraint、dictionary、method lookupを
導入しない。

## 明示的なoperationとclosed primitive

比較、算術、encodingなどを必要とするgeneric functionは通常のfunctionまたはproduct valueとしてoperationを受け取る。

```mal
Equality<A> :: (A, A) -> Bool;

contains<A> :: (Equality<A>, A, A) -> Bool :=
    (equal, expected, actual) -> equal(expected, actual);

double<A> :: A -> A := (value) -> value + value; // error
```

numeric operatorとpostfix conversionは仕様がconcrete typeを列挙するclosed primitive familyである。`Numeric`はmeta-level categoryで、
source typeやconstraintではない。`ByteSize`と`Count`を採択する場合は各operatorのdomainへ個別に加える。opaqueな`A`へのnumeric
primitive、`.A` conversion、compilerによるoperation挿入を認めない。

## Memory indexed type

generic codeは裸の`Address`と`A`だけからlayoutを導けない。callerがconcrete shapeからCursorまたはRegionを作り、indexed typeが
保持するstatic layoutをgeneric functionへ渡す。

```mal
readCursor<A> :: Cursor<A> -> (A, Cursor<A>) :=
    (cursor) -> <-cursor;

writeCursor<A> :: (Cursor<A>, A) -> Cursor<A> :=
    (cursor, value) -> cursor <- value;

makeRegion<A> :: (Cursor<A>, Count) -> Region<A> :=
    (cursor, elementCount) -> cursor@elementCount;

packRegion<A> :: Region<A> -> Packed<A> :=
    (region) -> <-region;
```

Cursor loadのproductはpatternで分解するか、既存のvalue-first applicationで次の処理へ渡せる。

```mal
(<-cursor)[(value, next) -> use(value, next)]
```

indexed typeを受け取ることはlayoutの存在だけを示し、referentのextent、permission、initialization、lifetime、valid representationを
証明しない。これらはmemory operationの未検査preconditionである。ANF以降へopenな型parameter、requirement、runtime layout
descriptorを渡さない。

採択時には現行の`T.load`と`T.store`をCursor operationで置き換え、名前付き`load<T>`と`store<T>`は提供しない。
`Symbol`と`Packed<UInt8>`の変換、Region transferの規則は[`Region`と`Packed`](region-and-packed.md)が所有する。

## Specialization

name resolutionとtype checkingは型parameter、型application、opaqueな型変数、requirement、generic binding identityを所有する。
type checking後、compilerはentry pointから到達する明示的なconcrete applicationを起点にspecialization graphを構成する。

specialization keyはgeneric binding identityと、aliasを展開したcanonical concrete type argument列である。同じkeyはfileを跨いで共有する。
各nodeはgeneric typed bodyへ型argumentを代入して単相typed coreを一度生成し、そこから到達するgeneric applicationをgraphへ加える。
self recursionは同じkeyへのedgeとして閉じる。異なる型argumentで自分を呼ぶbindingはdeclarationの型検査で拒否し、graphは有限で
なければならない。

specialization数、compile memory、生成code sizeが実装のdocumented resource limitを超えた場合は、source programを型不正とはせず、
展開元binding、type argument列、limitを示すartifact生成failureとする。generic machine codeを別artifact向けbinary ABIとして
配布しない。

ANF以降はgeneric declaration、type argument、requirement、dictionaryを受け取らず、現在と同じconcrete typeとprimitiveだけを扱う。
managed valueのretain、transfer、releaseは単相coreの通常規則で完結する。

## Host境界

public generic bindingはmal source間だけで使える。generated C header、extern ABI、host adapterへgeneric binding、specialization、
型parameterを公開しない。generic aliasはconcrete type argumentを代入して完全に展開した後、閉じた
[`HostMappable`](region-and-packed.md#host-abi) judgmentで判定する。展開後に`Cursor`、`Region`、`Packed`が残る型は拒否する。

## 採択前に確認すること

- generic alias、generic function、明示的specialization、first-classな単相function value、cross-file共有、self recursion
- duplicate parameter、arity mismatch、未確定型へのprimitive、requirement不足、generic extern、polymorphic recursionのdiagnostic
- specialization graphの共有、有限性、resource failure、source span
- Cursor loadのproduct patternとvalue-first application、Region/Packed transferを使うgeneric positive case
- managed valueを含むspecializationが単相core以降のownership規則だけで完結すること
