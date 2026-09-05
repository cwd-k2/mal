# 未決事項

Status: Discussion

優先度順。各項目の「暫定案」は docs を矛盾なく読めるように置いた仮決定で、合意後に `spec/` へ確定する。

capture付きlambdaについては解決済み。[D003](decisions.md#d003-v04-は-lexical-closure-を持つ)と[D007](decisions.md#d007-capture-listを明示する)を参照する。

floatの実行意味論については解決済み。[D009](decisions.md#d009-floatは-ieee-754-2019-の固定profileとする)を参照する。

`String`のlifetimeについては解決済み。[D010](decisions.md#d010-stringは-mal-ownedなprogram-lifetime-bytesとする)を参照する。

整数型間の変換とshift countについては解決済み。[D013](decisions.md#d013-整数型間の変換はdestination-widthでmoduloとする)と
[D014](decisions.md#d014-shift-countはleft-operandと同じ型とする)を参照する。

opaque resource safetyと`extern` ABIについては解決済み。[D015](decisions.md#d015-opaque-valueはcopyable-handleとする)と
[D016](decisions.md#d016-externはmal-c-abiとadapterを介する)を参照する。

## Q5. top-level initialization

**問い:** top-level RHS に任意の式や `extern` call を許すか。許す場合、file 間を含む実行順は何か。

**暫定案:** closed constant expression と lambda に制限し、`extern` は禁止。value は source order で scope に入り、annotated lambda の自己参照だけ例外とする。

## Q6. trap の観測

**問い:** trap を process exit、backend trap、host callback のどれにするか。

**暫定案:** 言語上は捕捉不能な異常終了だけを定義し、具体的な終了方法は embedding contract に置く。完了済み extern effect は巻き戻さない。

n-ary sum の記法と canonical form は解決済み。[D004](decisions.md#d004-直和型を-a-b-c-と書く) を参照する。

## Q10. `String` という名前

**問い:** arbitrary bytes なのに `String` と呼ぶか、`Bytes` と呼ぶか。

**暫定案:** 原案との連続性のため `String`。ただし利用者が UTF-8 invariant を期待する誤解は強い。v0.4 確定前なら `Bytes` への変更コストは低い。

## Q11. `return` は必要か

terminal にしか置けず core から消えるため、lambda body の最後の式だけでも意味は同じになる。

**暫定案:** 明示的な関数境界、statement と expression block の視認性を重視して残す。最小 token 数ではなく、読み手が意味を一意に把握するコストを優先する。

## Q12. lexical detail

line/block comment、Unicode identifier、trailing comma、keyword boundary、文字列中の不正 UTF-8 source の扱いが未定。
byte literalは[D006](decisions.md#d006-byte-literal-は-b--uint8-とする)、numeric separatorは
[D011](decisions.md#d011-numeric-separatorを認める)で解決済みである。

**暫定案:** identifier は ASCII、`//` line comment のみ、trailing comma はなし。機能追加前に lexer conformance test を作る。
