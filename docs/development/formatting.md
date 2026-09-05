# source formatting policy

Status: Current v0.5 tooling policy

この文書は`malc format`が生成するcanonical layoutを定める。受理するsyntaxは
[`grammar`](../spec/grammar.md)、command contractは[compiler usage](compiler-usage.md)を正とする。

## layout

- indentationはASCII space 4個とし、tabは出力しない。
- outputの改行はLFとし、file末に1つのLFを置く。
- `::`、`:=`、`->`とbinary operatorの両側、commaの後にspaceを置く。
- call、conversion、sum injection、delimiterの内側にspaceを置かない。
- blockのbraceと内容は別の行に置く。
- `if`のconditionの後で改行し、`then`と`else`を同じcontinuation indentに置く。
- `case`のscrutineeの後で改行し、すべてのarmを同じcontinuation indentに置く。

line commentのcontentsと順序を保持する。tokenと同じsource lineにあるcommentはそのtokenの後へ残し、
単独行のcommentは次のtokenと同じindentに置く。元のwhitespaceと空行数は保持しない。

numeric separator、suffix、byte/String escapeを含むliteralのbyte spellingは変更しない。formatterは
malformed sourceを補正せず、lexerまたはparserのstructured diagnosticを返す。
