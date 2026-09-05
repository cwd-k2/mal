# 未決事項

Status: Discussion

優先度順。各項目の「暫定案」は docs を矛盾なく読めるように置いた仮決定で、合意後に `spec/` へ確定する。

capture付きlambdaについては解決済み。[D003](decisions.md#d003-v04-は-lexical-closure-を持つ)と[D007](decisions.md#d007-capture-listを明示する)を参照する。

floatの実行意味論については解決済み。[D009](decisions.md#d009-floatは-ieee-754-2019-の固定profileとする)を参照する。

`String`のlifetimeについては解決済み。[D010](decisions.md#d010-stringは-mal-ownedなprogram-lifetime-bytesとする)を参照する。

immutable byte sequenceの名称については解決済み。[D017](decisions.md#d017-immutable-byte-sequenceの型名はstringとする)を参照する。

整数型間の変換とshift countについては解決済み。[D013](decisions.md#d013-整数型間の変換はdestination-widthでmoduloとする)と
[D014](decisions.md#d014-shift-countはleft-operandと同じ型とする)を参照する。

opaque resource safetyと`extern` ABIについては解決済み。[D015](decisions.md#d015-opaque-valueはcopyable-handleとする)と
[D016](decisions.md#d016-externはmal-c-abiとadapterを介する)を参照する。

top-level initializationの制限と自己参照例外は解決済み。[D018](decisions.md#d018-top-level-initializationは作用のないclosed-valueに限定する)を参照する。

## Q6. trap の観測

**問い:** trap を process exit、backend trap、host callback のどれにするか。

**暫定案:** 言語上は捕捉不能な異常終了だけを定義し、具体的な終了方法は embedding contract に置く。完了済み extern effect は巻き戻さない。

n-ary sum の記法と canonical form は解決済み。[D004](decisions.md#d004-直和型を-a-b-c-と書く) を参照する。

## Q11. `return` は必要か

terminal にしか置けず core から消えるため、lambda body の最後の式だけでも意味は同じになる。

**暫定案:** 明示的な関数境界、statement と expression block の視認性を重視して残す。最小 token 数ではなく、読み手が意味を一意に把握するコストを優先する。

## Q12. lexical detail

line/block comment、Unicode identifier、trailing comma、keyword boundary、文字列中の不正 UTF-8 source の扱いが未定。
byte literalは[D006](decisions.md#d006-byte-literal-は-b--uint8-とする)、numeric separatorは
[D011](decisions.md#d011-numeric-separatorを認める)で解決済みである。

**暫定案:** identifier は ASCII、`//` line comment のみ、trailing comma はなし。機能追加前に lexer conformance test を作る。
