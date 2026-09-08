# generated program最適化計画

Status: Current work plan

この文書はreference C backendが生成するprogramの未解決なruntime性能課題、着手順、完了条件を管理する。
測定値と解決済みの経緯は[generated C performance](performance.md)、managed valueの正しさは
[Engram ownership](../implementation/ownership.md)、言語とhostのauthorityは
[EngramとExtern](../spec/engrams.md)を正とする。

## 制約

- operandとeffectの評価順を保持する。
- source preconditionを満たすprogramへ新しいfailureを加えない。
- `Symbol`とclosureのidentity、到達可能性、lifetime authorityをmalに残す。
- `Ptr`のregion、permission、alias、alignmentをbackendが推測しない。
- first-class function callとextern ABIに必要なgeneric representationを、direct callだけの測定から削除しない。
- narrow integer representationは、全operationの値域とwrap semanticsを証明できる場合だけ使う。
- wall-clock比較の両binaryは同じC compiler identityとoptionでbuildし、ambient `CC`を継承しない。
- pair比較はroundごとに先行順を反転し、runnerではなくHyperfineのprocess時間を使う。
- 最適化後のLLVM IR、deterministic counter、wall-clockの少なくとも二つで変更理由を確認する。

## 現在の課題

| 優先度 | 軸 | 観測したcost | Corpus |
|---:|---|---|---:|
| 1 | nested scanとcontrol flow | same-width化とsigned shift改善後にも残るscan cost | 063 |

新しいprofileで課題を追加する場合は、同じ意図を各言語で自然に記述した比較fixtureを使い、source上の記法差をcompiler差として
扱わない。

## 1. nested scanとcontrol flow

011と063の元の比率差は、そのままではcompilerの責務だけを測っていなかった。011をsame-widthに揃えたvariantは改善せず、
063をsame-widthかつportable popcountへ揃えたvariantは1.45倍から約1.27倍へ縮んだ。したがって011ではwidthを原因候補から外し、
063では元の差の一部をfixture差として除外する。

063に残っていたsigned right shiftのmask合成は、integer lowering全体でClangが`ashr`へ認識できるportable表現へ変更した。
独立したshift fixtureは改善したが063全体は変わらなかったため、残差をshift loweringへ帰属させない。

063のrow scanではcolumnの基準値を`sameRows`が毎回取得していた。基準値の選択を外側の`countColumns`へ移し、比較対象の
scalarだけを`sameRows`へ渡すと最適化後IRからrowごとの再loadが消えたが、wall-clockは同等だった。この責務分離はfixtureへ
反映するが、性能改善とは数えない。clear loopを直接`memset`にしたvariantも同等で、early exitをやめてdirect Cと同じfull scanへ
揃えると約27%退行した。typed/aligned accessは約4%だけ改善し、LTOとの併用に追加効果はなかった。したがってbulk fill、
full-scan rewrite、alignment、allocation visibilityのいずれも063の独立したbackend設計候補にしない。

最適化後IRとassemblyを比較すると、011のheap sortにある3-field record moveとbest値のvector reductionは通常buildと
LTO buildで同形だった。差はDPの内側loopにあり、通常buildは`dp`へのstoreが`jobs`をaliasし得るためjob durationを
反復ごとに再loadするが、LTO buildはhost allocatorの`calloc`まで見て異なるallocationを識別し、durationとrewardを
loop外へhoistしていた。source側でこの2値をjobごとに一度読みscalar parameterとして渡す診断variantも同じ再loadを除去し、
direct Cと同等になった。したがってflat record moveやbackend固有のDP rewriteは011の改善候補から外す。

011単体ではLTOが改善したが、全corpusでは中央値に効果がなく059を退行させたため、一律LTOは採用しない。現行extern contractは
allocationのfreshnessや呼び出し間のnon-aliasを保証しない。host実装がたまたま`malloc`を使うfixtureから属性を逆輸入せず、
その保証が必要ならlanguage/host contractの独立した要求を先に置く。011のsource variantはprogramが知る不変値を明示した
fixture訂正として扱い、全`Ptr`へalias仮定を加えるcompiler変更の根拠にはしない。063のnested scanは引き続き別のcostとして
分類する。

`popcount`の差だけを隠すbackend specializationや新しいprimitiveは導入しない。言語surfaceへbit-count operationを加える場合は、
Typical90 fixtureではなく独立した言語要求とhost contractを先に必要とする。

## 共通の完了条件

一つのcost modelごとにfocused generated-C test、native execution、通常のcompiler testを通す。managed lifetimeへ触れる変更は
[managed Engram性能](ownership-performance.md)の通常・sanitizer pressure suiteも通す。ABI変更はgenerated header、host helper、
repository内adapterを同時に更新する。

wall-clockでは同じinput、stdout、C compiler、optimization option、warmup/run数を揃え、5 ms未満のcaseを採否の主根拠にしない。改善が
focused caseだけに留まる場合は、複雑さとcode sizeに見合う独立した利用形状があるまで採用しない。
