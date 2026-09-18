# `Packed` indirectionによる再帰的mal-owned data

Status: Draft proposal; non-normative

この文書は、`Packed`を明示的indirectionとして使う再帰型を提案し、採択に必要な型とownershipの境界を定める。現在の型規則は
[型](../spec/types.md)、`Representable`とcanonical layoutは[external memory](../spec/memory.md)、`Packed`の値とoperationは
[`Region`と`Packed`](../spec/packed.md)、構築案は[`pack` proposal](packed-construction.md)、実装上の回収規約は
[managed value ownership](../implementation/ownership.md)を正とする。

採択されるまでは現在のprofileを変更しない。mutable graph、object identity、cycle、hostによるmanaged ownerの保持は対象外とする。

## 提案の範囲

言語と実装に対する外部reviewを再評価した結果、具体的なprogram制約と解決経路を確認できた変更候補をこのproposalへ集約する。
提案する変更は次に限る。

- `pack<T>`とscopedな`emit : T -> Unit`でmal-ownedなPackedを直接構築する。
- `Packed<T>`の内部elementにmanaged valueを認め、構築、index、slice、破棄のownership規則を定める。
- すべての型cycleがPacked descriptorで隔てられるguarded recursive aliasを認める。

記号、operatorのfixityとspelling、`@`によるplacement、applicationの各表記、receiver-first application、result binder、direct result
blockは変更しない。現在のoperationはoperand位置と型から静的に決まり、Region indexによって`(?region)@shape`へ戻す冗長な経路も
避けられる。`@`を含む表記の密度だけを理由に別構文を追加しない。

named fieldとvariant、mutual function recursion、標準library/package discovery、`Empty`とcompletion、`ByteSize`と`USize`、`Symbol`と
`Packed<UInt8>`、compiler IR、optimization plan、LLVM emission、test organizationも本proposalの変更対象にしない。これらは現在の
仕様または責務分離を維持し、独立した具体的な必要性が生じた場合だけ別proposalで扱う。

producerの評価、builder growth、failure、scoped emitの一般規則は[`pack` proposal](packed-construction.md)を正とする。同proposalの
`Representable(T)`制約をmanaged elementへ広げる部分だけは本proposalが所有する。既存ownerを編集する
[`Packed` editing proposal](packed-editing.md)は独立した変更候補として扱う。

## 対象program

候補となる最小例は、子を`Packed`越しに持つimmutable treeである。

現在の[JSON query example](../../examples/json-query/)は有限の`UInt64` frame encodingでnestingを制限し、
[external tree example](../../examples/external-tree/)はAddressとhost allocatorのlifetime contractでtreeを表す。どちらも現行仕様の
正当な解法だが、mal-ownedな再帰値を直接構築、共有、走査できないという制約の具体例になる。

```mal
RecursiveType<T> :: [
    Unit,
    (T, Packed<RecursiveType<T>>)
];
```

`Packed<RecursiveType<T>>`はowner pointer、offset、countを持つviewとして表せるため、各`RecursiveType<T>`のruntime valueへ
子nodeをinline展開する必要はない。空variantとnode variantのどちらも有限のruntime representationを持ち得る。

| Variant | Runtime payloadの概形 |
|---|---|
| empty | tagだけ |
| node | `T`のruntime valueと、固定幅のchildren Packed view |

再帰するのは型graphとowner間の到達関係であり、一つのnodeを格納するinline byte数ではない。

この性質は、次のunguarded aliasにはない。

```mal
Bad<T> :: [Unit, (T, Bad<T>)]; // payloadをinline展開すると終わらない
```

したがって検討対象は一般のrecursive aliasではなく、representationを有限にするconstructorが全cycleを隔てるguarded recursionである。

## 現在拒否される理由

現在は二つの独立した規則が候補を拒否する。

1. transparent aliasはrecursive dependencyを持てない。
2. `Packed<A>`は`Representable(A)`の場合だけwell-formedであり、`Representable`はmanaged ownerを含む`Packed`を再帰的に除外する。

現行の`pack<T>`案もこの境界を維持し、`emit(value)`を`T`のcanonical representationとしてflat byte ownerへstoreする。
したがって`pack`をそのまま採択しても`Packed<RecursiveType<T>>`は構築できない。

この拒否は`HostMappable`とは別である。候補型をMal内部だけで使うならpublic C carrierは不要だが、現在はinternal `Packed`の
型形成までexternal-memory `Representable`へ結合している。

## 必要な意味の分離

候補を成立させる最小の方向は、型形成、内部構築、external memory transferを分けることである。

| Operation | 候補となる条件 |
|---|---|
| `Packed<T>`の型形成、length、index、slice | `T`を内部ownerが保持・回収できる |
| `pack<T>`による構築 | `T`をemit先へownership transferできる |
| `<-Region<T>`、`Region<T> <- Packed<T>` | 現行どおり`Representable(T)` |
| extern parameter/result | 現行どおり`HostMappable(T)`。`Packed`は不可 |

この分離では`Packed<RecursiveType<T>>`自体をexternal storageへload/storeしない。Region transferを使える`Packed<T>`は
`Representable(T)` elementに限るが、Mal内部のindexとsliceはより広いelementを扱える。

## `Packed`への統一

再帰をguardするmal-owned indirectionは`Packed<T>`へ統一し、`Box<T>`、`Single<T>`、`OwnedSequence<T>`は追加しない。
一要素、二要素、または空であることはsource型のinvariantにせず、`#packed`と`packed # index`を使うprogram側のpreconditionとする。

binary treeなら、children Packedのcountを0または2とする規約で表せる。

```mal
Tree<T> :: [Unit, (T, Packed<Tree<T>>)];
```

`packed # index`は既存どおり`index < #packed`を未検査preconditionとする。tree arity、必須child、indexの役割はprogram固有であり、
languageは専用constructorやchecked conversionを追加しない。外部pointerをlayoutとpreconditionの下でprogramが解釈するのと同様に、
動的なmal-owned sequenceのshapeは利用側が管理する。

この判断で省けるsurfaceと、残る実装責務は分ける必要がある。count invariantはprogramが所有するが、Packed owner内のmanaged elementを
二重解放またはleakさせない責任はruntimeとcompilerに残る。index resultのowner share、builder growth時のmove、最後のrelease時の
element破棄はunchecked preconditionへ移さない。

| 不採用案 | 得られるもの | 不採用理由 |
|---|---|---|
| `Box<T>` | exactly one invariant、totalなunbox | Packedとowner、構築、破棄が重複する |
| `Single<T>` | count 1を型で表現 | count以外の意味がPackedと同じで、新しい型とoperationに見合わない |
| `OwnedSequence<T>` | external-flat Packedとの型上の分離 | length、index、slice、builderを重複させる |

必要なのは別のsource型ではなく、固定幅Packed descriptorでinline展開を止め、最後のrelease時にelementを破棄できることである。

新しい型形成条件を仮に`Packable(T)`と呼ぶ場合、最初のprofileではUnit、scalar、Address、ByteSize、USize、Symbol、
`Packable(A)`を満たす`Packed<A>`、全fieldが条件を満たすproductと二項以上のsumへ閉じ、function、Cursor、Region、
external opaque typeを除く保守的な案がある。functionを許すと、
`emit`をcaptureしたclosure自体をbuilderへemitしてscoped capabilityをresultへ逃がせるため、現在の`pack`案のscope根拠が崩れる。

```mal
// Packed<function>を許す場合に拒否または別のscope型が必要になる形
pack<Unit -> Unit>((emit) -> {
    emit(() -> emit(() -> ()))
});
```

正確な`Packable`集合は、再帰dataに必要な型だけから始め、operationを追加するたびに拡張する方が境界を短く保てる。

## 構築model

完成済みの子を先に作り、親へemitするbottom-up constructionなら、builder自身や未完成ownerをsourceへ公開しない。

```mal
branch<T> :: (T, RecursiveType<T>, RecursiveType<T>) -> RecursiveType<T> :=
    (value, left, right) -> [empty, node] =>
        node((value, pack<RecursiveType<T>>((emit) -> {
            emit(left);
            emit(right);
        })));
```

候補の型検査には`Packable(RecursiveType<T>)`をgeneric signatureから導く新しいrequirementが必要になる。

`emit`へ渡す時点でvalueは完成している。builderのowner identityを取り出せず、mutable cellとhost-visible closureがない範囲では、
新しい値から未完成builderへ戻るedgeを作れない。この条件なら構築できる値は有限のacyclic graphであり、reference countingで回収できる。
同じsubtreeを複数の親へ渡すDAGは許し、必要なowner shareを作る。

## Recursive aliasの選択肢

| 案 | Source surface | 型等価性 | 主なcost |
|---|---|---|---|
| guarded transparent alias | 新keywordなし | regular treeのbisimulation | alias cycle、generic substitution、diagnosticを有限graphで扱う |
| explicit recursive type constructor | `rec`または同等の新構文 | fold/unfoldまたはequi-recursive rule | 新しい型構文と操作を学ぶ必要がある |
| nominal recursive declaration | declaration identityで比較 | nominal | 現在の「nominal user typeなし」を変更する |

現行のtransparent aliasに最も近いのは第一案だが、単純なalias展開では実装できない。compilerはcycleを拒否する代わりに
canonical typeを有限graphとしてinternし、型比較を再訪済みnode pairで閉じる必要がある。guardedness判定はalias dependency cycle上に
有限representationのindirectionが必ずあることを検査する。

`Packed`だけをguardとするのか、将来の別owner constructorも含めるのかはtype grammarではなくruntime representation policyに属する。
初期案では`Packed`だけに閉じる方が規則と検証範囲を小さくできる。

## Runtimeとcompilerへの影響

現在のflat byte ownerはelement destructorを必要としない。managed elementを持つ`Packed<T>`には次が必要になる。

- `emit`がmanaged fieldのresponsibilityをbuilder slotへshareまたはconsumeする。
- growthが初期化済みelementを重複releaseせず新storageへmoveする。
- sliceはownerをshareし、index resultはelement内managed fieldの有効なshareを返す。
- 最後のowner releaseが全elementのactive managed fieldを破棄する。
- 深いtreeの最終releaseをnative stack深度へ比例させない。

実装案にはowner headerへprogram固有destructor entryを持たせる方法と、element型ごとにspecialized owner releaseを生成する方法がある。
前者は共通runtimeのheaderとindirect callを増やし、後者はbackend生成量とowner種別を増やす。どちらも現在の「byte ownerの解放」より
大きい変更であり、`execution/ownership`はPacked ownerだけでなくelement responsibilityも追跡する。

canonical external layout、public C ABI、Region permission/lifetimeは変更しない。`Packed<T>`の内部runtime representationを
canonical representationとして公開しないことが、この分離の条件である。

## 代案

| 代案 | 得られるもの | 失うもの |
|---|---|---|
| 現状維持しexternal treeを辿る | runtime変更なし、resource policyが明示的 | host allocationとlifetime contractが必要 |
| indexで結ぶflat `Packed<NodeRecord>` | `Representable`のまま任意深さを扱える | topologyとboundsをprogramが管理する |
| recursive closure encoding | 現行functionとself recursionを再利用 | data observation、layout、serializationが不自然 |

JSON parserのstackだけが目的なら、growableなflat `Packed<Frame>`で足り、recursive aliasは不要かもしれない。tree valueをAPI間で
受け渡し、subtreeを共有し、generic traversalを書くことが目的なら、本案の価値が現れる。対象programを区別して判断する。

## 暫定評価

| 観点 | 評価 |
|---|---|
| 表現可能性 | guarded cycleを有限type graphとして扱えば可能 |
| source追加 | guarded transparent aliasなら新しいconstructor syntaxは不要 |
| 型検査 | alias cycle、`Packable` requirement、coinductiveな型比較が必要 |
| runtime | managed element ownerとdestructorが必要で、変更は大きい |
| external互換性 | Region transferとHostMappableを分離すれば現行境界を維持可能 |
| 主な利益 | immutable tree/DAGの直接表現、subtree共有、generic traversal |
| 主な非利益 | parser stackやmutable graphを自動的には解決しない |

技術的には成立可能であり、recursive aliasが直ちに無限runtime layoutを意味するわけではない。最大のriskは型の再帰そのものより、
managed elementを持つPacked ownerの構築と深さ非依存の回収にある。採択判断の前に、型graphだけのprototypeとowner prototypeを
分けて測る価値がある。

## 検証条件

採択候補にする前に、少なくとも次をprototypeで固定する。

- guarded aliasを受理し、unguarded aliasとguardを持たないmutual cycleを宣言spanで拒否する。
- empty、leaf、branch、shared subtree、深さ10万のtreeを構築・走査・解放する。
- generic substitution後の型等価性、alias表示、specialization keyが有限時間で安定する。
- `Packed<RecursiveType<T>>`のindex、slice、owner終了後のresult lifetimeを検査する。
- non-`Representable` elementに対するRegion admission/storeとhost mappingを拒否する。
- allocation failure、途中でAbruptになるproducer、複数回growthで全ownerを回収する。
- emitterまたは未完成builderへのcapabilityがresultへescapeしない。

評価指標は、手書きindex encodingが減る量だけでなく、新しいtype judgment、owner operation、runtime header、backend生成量、
利用者が調べるcontractの合計とする。

## 未決事項

- `Packable(T)`を独立judgmentにするか、`Packed<T>`の内部表現可能性として名前を公開しないか。
- recursive aliasをtransparentに保つか、明示的またはnominalな再帰型にするか。
- `Packed` element destructorをruntime callbackとspecialized LLVMのどちらが所有するか。
- `pack`だけで構築を閉じるか、既知長のallocationを最適化として認識する必要があるか。
- v0.6の適用範囲を広げる利益が、現行のexternal treeとflat index表現を上回るか。
