# 直和除去continuation導入後の残課題

Status: Exploratory

この文書は、[D072](../history/decisions/D072.md)で直和除去のlambda literal continuationをbranchとして扱った後に見つかった
不都合のうち、まだ対応していないものを管理する。example全体を早期脱出の形へ書き換えた後に、この順で対応する。
現在の言語規則は[`spec/`](../spec/)を正とする。

## 検証

`nu scripts/dev.nu check --fast`はVS Code packageとNix flakeを検証しない。完了判定として`--fast`なしの実行が要る。

## 別の提案

単一continuationとcallee位置のlambda literalへの拡張は[別の提案](immediate-lambda-redex.md)が扱う。
