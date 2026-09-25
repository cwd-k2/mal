# 直和除去continuation導入後の残課題

Status: Exploratory

この文書は、[D072](../history/decisions/D072.md)で直和除去のlambda literal continuationをbranchとして扱った後に見つかった
不都合のうち、まだ対応していないものを管理する。example全体を早期脱出の形へ書き換えた後に、この順で対応する。
現在の言語規則は[`spec/`](../spec/)を正とする。

## 診断

- 全continuationが`Abrupt`の直和除去を束縛のinitializerに置くと`binding initializer must produce a value`と出る。原因が
  「すべてのcontinuationが抜ける」ことだと読み取りにくい。
- binder名のcontinuationの型不一致は一般の`type mismatch`で、どの位置のpayloadとbinderが合わないかを示さない。

## formatter

block直下の複数行のcontinuation listは、continuationと閉じ`]`を値と同じindentに置く。名前だけのlist
（`[\ncreated,\nfailed\n]`）では、どこまでが式か読み取りにくい。sourceで一行なら一行のまま保つので通常は現れないが、
改行したlistでの見え方を見直す。

## editor

`Value`か`Abrupt`かをeditorは利用者に示さない。早期脱出がcontinuationの位置に現れると、どのbranchが現在のpathを打ち切るかが
sourceから読みにくい。`Abrupt`のbranchのhoverと、後続がunreachableである理由の表示を、
[editor tooling](../development/editor-tooling.md)の設計判断として検討する。

## 検証

`nu scripts/dev.nu check --fast`はVS Code packageとNix flakeを検証しない。完了判定として`--fast`なしの実行が要る。

## 別の提案

単一continuationとcallee位置のlambda literalへの拡張は[別の提案](immediate-lambda-redex.md)が扱う。
