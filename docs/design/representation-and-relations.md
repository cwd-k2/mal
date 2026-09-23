# 表現と関係を分ける

Status: Current design policy

この文書は、再帰的または相互参照を持つdomain structureをmalで表すときの設計方針を定める。個々の型とoperationは
[型](../spec/types.md)、[external memory](../spec/memory.md)、[`Buffer`](../spec/memory.md)を正とする。
carrierをoperationへ適用して解釈する全体の設計軸は[値、解釈、control](value-interpretation-and-control.md)に置く。

## 有限な表現から構造を得る

malでは、domain structureをrecursive typeとして値の物理構造へ埋め込むことを基本形にしない。有限な`Buffer`、product、sumなどを
carrierとし、その要素間の関係をindex、offset、tag、keyその他の有限値で表す。

型は値の表現構造とauthorityを定める。型だけでは、indexがchildであること、offsetがrecordを指すこと、特定の要素がrootであること、
edgeがacyclicであることまでは保証しない。このようなdomain上の意味は、carrierを解釈するoperationと、そのoperationが要求または保存する
invariantが定める。

```text
finite carrier + relation operations + invariants = domain structure
```

したがって構造の再帰はrecursive typeではなく、relationを辿る再帰、iteration、または明示的なwork stackとして現れる。型は形と
authorityを与え、operationが関係を与える。

## carrier、座標、relation

carrierはrow、column、byte、node recordなどを保持する有限な表現である。index、offset、`Address`は単独でdomain identityにはならず、
特定のcarrierに対する座標として意味を持つ。同じ`USize`値でも、別の`Buffer`を対象にすれば別の要素を指す。

carrierのcolumnが保持する値と、column間またはrow間のrelationを区別する。例えばheap-indexedなsegment indexのmaximum columnと
pending-assignment columnは、共通のnode IDをkeyとして結合されるpayloadであり、edgeやparentを保持するrelation indicatorではない。
親子relationは`2 * node`と`2 * node + 1`を解釈するoperationが与える。反対にCSRのoffset columnや明示的なparent columnは、別の
payloadへ到達する座標を保持するrelation indicatorである。隣接した物理配置や同じindexで読めることだけを、stored relationと呼ばない。

treeを`Buffer<NodeRow>`で表す場合、`Buffer`が保証するのは有限なrow列である。root位置、child fieldの解釈、child bounds、
reachability、acyclicityを加えたときに初めてtreeになる。databaseをbyte regionで表す場合も、record offset、active flag、key relation、
uniquenessをoperationが定める。

座標をcarrierから切り離して長期保持しない。別のcarrierへ座標を移すoperationは、同じ座標が有効である理由を持つか、明示的なremapを返す。
slice、sort、compaction、mergeなどが座標の意味を変える場合、その変換を呼出側の暗黙の了解にしない。

## invariantのauthority

transparent aliasはruntime identityもnominal proofも作らない。`Tree :: Buffer<NodeRow>`という名前は表現の役割を説明できるが、その値が
tree invariantを満たすことは証明しない。型で表さない条件は次のいずれかが所有する。

- admission後のvalidatorが、外部または未検査の表現をdomain operationへ渡せるか判定する。
- constructorが、完成時にinvariantを満たすcarrierだけを公開する。
- transformerが、入力に要求するinvariantと結果で保存するinvariantを契約にする。
- private helperが、上位のvalidatorまたはconstructorによって成立済みのpreconditionを引き継ぐ。

field位置、tag値、sentinel、header conventionを複数のcall siteへ散らさない。それらをrelation operationへ集約し、algorithmは可能な限り
domain上の問いとして記述する。型名やコメントだけをvalidationの代わりにしない。

## 論理構造とlayout

relationの分離は、常にrelationを別の配列へ置くことを意味しない。AoS、SoA、edge table、CSR、byte encodingのどれを選ぶかは、主要な
operationのaccess pattern、更新単位、ownership、host contractから決める。同じ論理relationを異なるlayoutで実装してよく、同じcarrierを
異なるrelationで解釈してもよい。

layout固有の知識を少数のoperationへ閉じ込め、domain operationの契約をlayoutから独立させる。あるlayoutが複数の解釈を支える場合、
carrierを複製せず、root、edge、group、orderingなどのrelationを別々に定義する。逆にaccess patternが異なるなら、論理上は同じstructureでも
専用のcarrierへ変換してよい。data-oriented designは特定のlayoutを選ぶ規則ではなく、観測するoperationからlayoutを選び、layout自体を
domain ontologyと同一視しない方針である。

## authorityとの直交

同じrelationをmal-ownedな`Buffer`とexternal storageへの`Address`のどちらに載せるかは、resource authorityの違いである。
tree、graph、tableであることとは別に判断する。`Buffer`のownerが生きていても、要素中の`Address`が指すreferentのlifetimeは延びない。
外部storageを読むrelation operationは、構造上のinvariantに加えてhost contractのlifetime、permission、initializationを要求する。

EngramとExternの境界判断は[authority](authority.md)を正とする。domain relationを導入するためだけにexternal ownershipや個別allocatorを
持ち込まず、recoverable allocation、shared external identityなど固有の要求がある場合だけ別のauthority modelを選ぶ。

## source designの確認事項

再帰的またはindexedな表現を追加するときは、次を確認する。

1. carrierは何を保持し、どのauthorityがそのlifetimeを支配するか。
2. 各fieldまたはcolumnはpayloadかrelation indicatorか。index、offset、tag、keyをどのrelationとして読むか。
3. raw representationとvalidなdomain structureの境界はどこか。
4. constructor、validator、transformerのどれが各invariantを所有するか。
5. 座標はどのcarrierに相対的で、変換時にremapが必要か。
6. traversalの深さとcycleを、再帰、iteration、work stack、visited relationのどれで扱うか。
7. operationのaccess patternに対してAoS、SoA、edge table、byte encodingのどれが適切か。
8. 別のlayoutまたは別のlogical interpretationへ差し替えても、domain operationの契約を保てるか。
