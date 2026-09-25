# source formattingの方針

Status: Current v0.6 policy and implementation

この文書は`mal-fmt`が生成するcanonical layoutを定める。受理するsyntaxは[`grammar`](../spec/grammar.md)を正とする。

## command

```nu
mal-fmt source.mal
mal-fmt --write source.mal
```

既定ではcanonical source全体をstdoutへ出す。`--write`（`-w`）を指定した場合はstdoutへ出さず、同じdirectoryのtemporary fileを介して入力fileを
atomicに置き換える。syntax errorまたは書き込み失敗では元の入力を保持する。終了statusは成功が`0`、sourceまたはfile errorが`1`、
command grammarのusage errorが`2`である。

## layout

- indentationはASCII space 4個とし、tabは出力しない。
- outputの改行はLFとし、file末に1つのLFを置く。
- `::`、`:=`、`->`とbinary operatorの両側、commaの後にspaceを置く。
- keywordと同じ行に後続tokenがある場合は、その間にspaceを置く。
- application、numeric conversion、delimiterの内側にspaceを置かない。
- sourceで一行のblockは、body item、nested block、commentを持たなければ一行に置き、result直後の`;`を
  省く。sourceで複数行のblockは、単純なresultだけでも複数行のままにする。
- それ以外のblockはbraceと内容を別の行に置き、resultを含む各行を`;`で終える。
- 別のexpressionが値を要求する位置に埋め込まれ、sourceでconditionと両branchを一行に置いた短い`if`は一行のまま
  整形する。sourceで改行した`if`は改行を保ち、`then`と`else`の一方だけを同じ行に
  残さない。block直下のexpressionとして行頭から始まる`if`では`then`と`else`を`if`と同じindentに置き、bindingや
  lambda bodyなどのRHSにある`if`では一段深いcontinuation indentに置く。
- 複数continuationのapplicationは、`[`から閉じ`]`までがsourceで一行にあり、含まれるblockがすべて一行のまま置けるなら
  一行のまま整形する。sourceで改行した場合と、blockが複数行に展開される場合はvalueの後で改行する。block直下のexpressionとして
  行頭から始まる場合はcontinuationと閉じ`]`をvalueと同じindentに置き、bindingなどのRHSにある場合はcontinuationだけを一段深くし、
  閉じ`]`をbindingと同じindentへ戻す。各lambda bodyのblockは通常のlambdaと同じ規則で整形する。
- sourceで空行に分けたtop-level groupは1空行を保つ。lambdaを直接initializerに持つfunction bindingは
  前後のitemと1空行で分け、連続するそれ以外のbindingへformatterだけを理由とする空行を追加しない。
- `::`、`:=`、`->`、binary operator、delimiterで区切られた要素の前後にsource改行があれば、構文上
  曖昧にならない位置ではcontinuation改行として保つ。`:=`の前後で改行したinitializerはbinding終端まで
  一段深くし、それ以外のcontinuation行も一段深くする。
- receiver-first applicationとconversionを含むpostfix chainのsuffix直前にsource改行があれば、
  一段深いchain継続として保つ。

line commentのcontentsと順序を保持する。tokenと同じsource lineにあるcommentはそのtokenの後へ残し、
単独行のcommentは次のtokenと同じindentに置く。top-levelの単独行commentは直後のitemと同じgroupに置く。
上記の意味を持つ改行以外のwhitespaceと空行数は保持しない。

numeric separator、suffix、byte/Symbol escapeを含むliteralのbyte spellingは変更しない。formatterは
malformed sourceを補正せず、lexerまたはparserのstructured diagnosticを返す。

generic parameterとargumentではidentifierと`<`、comma以外のtype argument、closing `>`を密着させる。

## application notation

通常のfunction applicationは`f(a)`を標準styleとする。意味論上同じ`a[f]`は、値をcontinuationへ渡すことが主題の箇所、または
`value[normalize][measure]`のように動詞的な変換を左から右へ並べるpipelineで使う。receiverを処理の主題として保つ
domain operationや複数引数のpipelineでは`value.transform(option)`のreceiver-first形を使える。この形はfield accessではなく、
parenthesized argument listを必須とする。formatterはapplication表記を相互変換せず、sourceが選んだ向きと明示的な
receiver-first chain改行を保持する。
