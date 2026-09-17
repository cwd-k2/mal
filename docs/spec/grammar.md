# 字句と文法

Status: Accepted v0.6 profile

## Sourceとidentifier

source encodingはUTF-8、keywordとidentifierはASCIIで認識する。

```text
TYPE_IDENT  ::= "_"? [A-Z][A-Za-z0-9]*
VALUE_IDENT ::= "_"? [a-z][A-Za-z0-9]*
```

型名はPascalCase、値、parameter、external symbolはlowerCamelCaseである。先頭の`_`はtop-level declarationのprivate visibilityを
表せる。単独の`_`はwildcardである。空白はASCII space、tab、CR、LF、commentは`//`からline末尾までとする。
keywordは`require`、`extern`、`if`、`when`、`then`、`else`である。Unicode identifier、block comment、trailing commaはない。

型identifierはdeclaration、annotation、generic parameter/argumentなどtype grammarが要求する位置だけに現れる。expressionから
型を参照するtype-qualified primitiveとnumeric conversionはない。

## Numeric literal

```text
DEC_DIGITS ::= DEC_DIGIT ("_"? DEC_DIGIT)*
HEX_DIGITS ::= HEX_DIGIT ("_"? HEX_DIGIT)*
BIN_DIGITS ::= BIN_DIGIT ("_"? BIN_DIGIT)*
EXPONENT   ::= ("e" | "E") ("+" | "-")? DEC_DIGITS
INTEGER_SUFFIX ::= "i8" | "i16" | "i32" | "i64"
                 | "u8" | "u16" | "u32" | "u64"
                 | "bytes" | "count"
FLOAT_SUFFIX ::= "f32" | "f64"
```

`_`は各digit sequenceのdigit間に一つだけ置ける。radix prefix直後、小数点の隣、suffixの直前には置けない。
decimal floatは整数部と小数部の両方を必要とし、`.5`と`1.`はない。decimal point、exponent、`f32`/`f64` suffixのいずれかを
持つliteralをfloatとする。`bytes`と`count`はdecimal、hexadecimal、binary integer literalに使える。

## Declarationとtype

```text
program     ::= requireDecl* topItem*
requireDecl ::= "require" symbolLiteral ";"

topItem     ::= typeAlias ";"
              | externType ";"
              | externDecl ";"
              | genericBinding ";"
              | binding ";"

typeParameters ::= "<" TYPE_IDENT ("," TYPE_IDENT)* ">"
typeArguments  ::= "<" type ("," type)* ">"

typeAlias      ::= TYPE_IDENT typeParameters? "::" type
externType     ::= "extern" TYPE_IDENT
externDecl     ::= "extern" VALUE_IDENT "::" type
genericBinding ::= VALUE_IDENT typeParameters "::" type ":=" expression
binding        ::= pattern ("::" type)? ":=" expression

type         ::= functionType
functionType ::= atomicType ("->" functionType)?
atomicType   ::= TYPE_IDENT typeArguments?
              | builtinType typeArguments?
              | "(" type ")"
              | "(" type "," type ("," type)* ")"
              | sumType
sumType      ::= "[" "]" | "[" type "," type ("," type)* "]"
```

`Cursor`、`Region`、`Packed`はちょうど一つのtype argumentを要求する。他のbuiltin typeはtype argumentを受け取らない。
generic extern declarationはない。`>>` tokenはgeneric parameter/argument list内では二つのclosing `>`、expression内ではshiftである。

## Expression form

```text
lambda          ::= "(" lambdaParameter? ")" "->" expression
lambdaParameter ::= pattern ("," pattern)*
resultBlock     ::= "[" VALUE_IDENT ("," VALUE_IDENT)* "]" "=>" expression
bodyItem        ::= binding ";" | expression ";"
block           ::= "{" bodyItem* expression ";"? "}"

pattern        ::= VALUE_IDENT | "_" | productPattern
productPattern ::= "(" pattern "," pattern ("," pattern)* ")"

valueName      ::= VALUE_IDENT typeArguments?
callSuffix     ::= "(" argumentList? ")"
receiverSuffix ::= "." VALUE_IDENT "(" argumentList? ")"
continuationSuffix ::= "[" "]"
                     | "[" expression ("," expression)* "]"
conversionSuffix ::= "." ("i8" | "i16" | "i32" | "i64"
                           | "u8" | "u16" | "u32" | "u64"
                           | "f32" | "f64" | "bytes" | "count")
placementSuffix ::= "@" placementOperand
postfixAlign    ::= "!"

placementOperand ::= shape | VALUE_IDENT | numericLiteral | "(" expression ")"
shape ::= shapeAtom
        | "(" shape "," shape ("," shape)* ")"
        | "[" shape "," shape ("," shape)* "]"
shapeAtom ::= "unit"
            | "i8" | "i16" | "i32" | "i64"
            | "u8" | "u16" | "u32" | "u64"
            | "f32" | "f64"
            | "address" | "bytesize" | "count" | "bool"

product ::= "(" expression "," expression ("," expression)* ")"
unitApplication ::= "[" expression "]"

ifExpr   ::= "if" "(" expression ")" "then" expression "else" expression
whenExpr ::= "when" "(" expression ")" expression
```

primary expressionはliteral、valueName、Unit、parenthesized expression、product、unit application、lambda、block、result block、
`if`、`when`からなる。primaryの後へcall、receiver、continuation、conversion、placement、postfix align suffixをsource orderで
0個以上適用する。`@`直後のclosed shape spellingはshape、それ以外のname、literal、parenthesized expressionはCount operandである。
したがって`@count`はCount shape、`@elementCount`はvalueである。shape atomと同じspellingのvalueをCount operandに使う場合と、
operatorを含むCountには括弧を使い、`@(count)`、`@(elementCount + 1count)`と書く。

prefix `#`の直後もclosed shape spellingならstride query、それ以外はvalue length queryである。shape atomと同じspellingの
valueをlength queryに使う場合は`#(count)`のように括弧を使う。shapeにuser-defined aliasとTYPE_IDENTは現れない。
lexerは`<-`を`<`と`-`より先に一つのtokenとして認識する。

Symbol literalとbyte literalは次の形を持つ。

```text
symbolLiteral ::= '"' (rawSymbolCharacter | symbolEscape)* '"'
symbolEscape  ::= "\\" ("\\" | '"' | "n" | "r" | "t" | "0"
                        | "x" HEX_DIGIT HEX_DIGIT)
byteLiteral ::= "'" byteUnit "'"
byteUnit    ::= printableAsciiExceptQuoteOrBackslash
              | "\\\\" | "\\'" | "\\n" | "\\r" | "\\t" | "\\0"
              | "\\x" HEX_DIGIT HEX_DIGIT
```

byte literalのraw characterはASCII `0x20`から`0x7e`のうちsingle quoteとbackslashを除く。非ASCII characterは認めない。

## Operator precedence

高い順に次の通り。

| Level | Form | Associativity |
|---|---|---|
| primary/name suffix | primary、`name<T, ...>` | — |
| postfix chain | call、continuation、receiver、conversion、`@operand`、postfix `!` | left |
| prefix | `-`、prefix `!`、`~`、`#`、`?`、prefix `<-`、prefix `*` | right |
| indexed access | binary `#` | non-associative |
| multiplicative | `* / %` | left |
| additive | `+ -` | left |
| shift | `<< >>` | left |
| relational | `< <= > >=` | non-associative |
| equality | `== !=` | non-associative |
| bit AND | `&` | left |
| bit XOR | `^` | left |
| bit OR | `|` | left |
| logical AND | `&&` | left |
| logical OR | `||` | left |
| store/observe | binary `<-` | left |

postfix chainは左から適用する。prefix/postfix `!`、prefix/binary `#`、prefix/binary `*`と`<-`はoperand位置で区別する。
conversion suffixはparenthesized argumentを伴わず、receiver suffixは必ず`.name(...)`なので曖昧にならない。binary `#`、relational、
equalityは同levelでchainできない。assignment operatorはない。

`expression.VALUE_IDENT(arguments)`はreceiver-first applicationであり、calleeをlexical scopeから解決する。field、property、method
lookupを導入しない。`expression.VALUE_IDENT`だけの形はconversion suffix以外には存在しない。

`[]`はempty sum type、`[A]`は不正である。`[k]`はUnitを`k`へ渡すapplication、`[k] => expression`はresult blockである。
空の`[] => expression`はない。詳細は[result boundary](control.md)に定める。

## Formatting boundary

identifierとgeneric `<`、comma以外のtype argument、closing `>`の間にspaceを置かない。prefixとpostfixはoperandへ密着させ、
binary operatorの両側へspaceを置く。postfix chainの継続行は一段indentする。binary `<-`のchainはoperatorから始まる継続行にできる。

## 存在しない構文

`let`、`var`、`mut`、`const`、`fn`、`case`、return statement、loop、`break`、`continue`、record、class、method、field access、
nominal enum constructor、typed pointer、reference、user-defined constraint、trait、interface、macro、exceptionはない。
