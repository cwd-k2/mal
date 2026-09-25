# 直和除去continuation導入後の残課題

Status: Exploratory

この文書は、[D072](../history/decisions/D072.md)で直和除去のlambda literal continuationをbranchとして扱った後に見つかった
不都合のうち、まだ対応していないものを管理する。example全体を早期脱出の形へ書き換えた後に、この順で対応する。
現在の言語規則は[`spec/`](../spec/)を正とする。

## editor

result binderのhoverは、適用するとblockを抜けることを示す。一方、`Value`か`Abrupt`かをexpression単位でeditorが示す機能はまだない。
continuationやbranchの位置に早期脱出が現れると、どのbranchが現在のpathを打ち切るかがsourceから読みにくい。`Abrupt`のbranchのhoverと、
後続がunreachableである理由の表示を、[editor tooling](../development/editor-tooling.md)の設計判断として検討する。

## 検証

`nu scripts/dev.nu check --fast`はVS Code packageとNix flakeを検証しない。完了判定として`--fast`なしの実行が要る。

## 別の提案

単一continuationとcallee位置のlambda literalへの拡張は[別の提案](immediate-lambda-redex.md)が扱う。
