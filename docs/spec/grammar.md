# 字句と文法

Status: Current v0.5 profile

## source と identifier

source encoding は UTF-8。keyword と identifier の認識は ASCII に限定する。

```text
TYPE_IDENT  ::= "_"? [A-Z][A-Za-z0-9]*
VALUE_IDENT ::= "_"? [a-z][A-Za-z0-9]*
```

型名は PascalCase、値・parameter・external symbol・primitive は lowerCamelCase とする。先頭の`_`はtop-level declarationの
file-private visibilityを表せる。local identifierでも同じspellingを認めるがvisibilityの意味は持たない。単独の`_`は
wildcard専用でidentifierではない。

空白はASCII space、tab、CR、LFとする。commentは`//`からCR、LF、またはsource末尾までであり、block commentはない。
keywordはidentifier全体が`require`、`extern`、`if`、`when`、`then`、`else`のいずれかと一致するときだけ認識する。
Unicode identifierとtrailing commaは認めない。

## numeric separator

数値literalを構成する各digit sequenceでは、有効なdigitの間に一つの`_`を置ける。

```text
DEC_DIGITS ::= DEC_DIGIT ("_"? DEC_DIGIT)*
HEX_DIGITS ::= HEX_DIGIT ("_"? HEX_DIGIT)*
BIN_DIGITS ::= BIN_DIGIT ("_"? BIN_DIGIT)*
EXPONENT   ::= ("e" | "E") ("+" | "-")? DEC_DIGITS
INTEGER_SUFFIX ::= "i8" | "i16" | "i32" | "i64"
                 | "u8" | "u16" | "u32" | "u64"
FLOAT_SUFFIX ::= "f32" | "f64"
```

```mal
1_000
0xff_ffu32
0b1010_0001
1_000.25f64
```

`_`はdigit sequenceの先頭・末尾、連続位置、radix prefix直後、小数点の直前・直後、型suffixの直前には置けない。したがって`_1`、`1_`、`1__0`、`0x_ff`、`1_.0`、`1._0`、`1_u8`、`1_f32`はlexical errorである。separatorを除去したliteralと同じ値・型を持つ。

## 文法概要

```text
program     ::= requireDecl* topItem*

requireDecl ::= "require" symbolLiteral ";"

topItem     ::= typeAlias ";"
              | externType ";"
              | externDecl ";"
              | binding ";"

typeAlias   ::= TYPE_IDENT "::" type
externType  ::= "extern" TYPE_IDENT
externDecl  ::= "extern" VALUE_IDENT "::" type
binding     ::= pattern ("::" type)? ":=" expression

type        ::= functionType
functionType ::= atomicType ("->" functionType)?
atomicType  ::= TYPE_IDENT | builtinType | "(" type ")"
              | "(" type "," type ("," type)* ")"
              | sumType
sumType     ::= "[" "]" | "[" type "," type ("," type)* "]"

lambda      ::= "(" lambdaParameter? ")" returnBinderGroup? block
lambdaParameter ::= pattern ("," pattern)*
returnBinderGroup ::= "[" "]"
                    | "[" VALUE_IDENT ("," VALUE_IDENT)* "]"
bodyItem    ::= binding ";" | expression ";"
block       ::= "{" bodyItem* expression ";"? "}"

pattern     ::= VALUE_IDENT | "_" | productPattern
productPattern ::= "(" pattern "," pattern ("," pattern)* ")"

call        ::= expression "(" argumentList? ")"
receiverCall ::= expression "." VALUE_IDENT "(" argumentList? ")"
continuationApplication ::= expression "[" "]"
                          | expression "[" expression
                            ("," expression)* "]"
unitApplication ::= "[" expression "]"
product     ::= "(" expression "," expression
                ("," expression)* ")"
typeApplication ::= TYPE_IDENT "(" expression ")"
                  | expression "[" TYPE_IDENT "]"
typeQualifiedPrimitive ::= TYPE_IDENT "." VALUE_IDENT
symbolLength ::= "#" expression
symbolByteAccess ::= expression "#" expression

symbolLiteral ::= '"' (rawSymbolCharacter | symbolEscape)* '"'
symbolEscape  ::= "\\" ("\\" | '"' | "n" | "r" | "t" | "0"
                        | "x" HEX_DIGIT HEX_DIGIT)
byteLiteral ::= "'" byteUnit "'"
byteUnit    ::= printableAsciiExceptQuoteOrBackslash
              | "\\\\" | "\\'" | "\\n" | "\\r" | "\\t" | "\\0"
              | "\\x" HEX_DIGIT HEX_DIGIT

ifExpr      ::= "if" "(" expression ")"
                "then" block
                "else" block
whenExpr    ::= "when" "(" expression ")" block
```

この概要では左再帰を避ける expression grammar と lexer の詳細を省略している。実装は recursive descent と Pratt parser を想定する。

lambda bodyが参照する外側のlocal valueはlexically captureされる。詳細は[closure規則](execution.md#scope-と-closure)を参照する。

## operator precedence

高い順に次の通り。

| level | operator | associativity |
|---|---|---|
| application | `f(...)`、`value[...]`、`value.f(...)` | left |
| Symbol length | `#value` | right |
| Symbol byte access | `value # index` | non-associative |
| unary | `- ! ~` | right |
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

`+`と`-`は同じnumeric型同士に加えて、`Ptr`と`UInt64`の順のoperandにbyte offset operatorとして定義する。
`UInt64 + Ptr`、`Ptr + Ptr`、`UInt64 - Ptr`は定義しない。`|` は式中の bitwise OR だけに使用する。直和型は
`[]` で区切るため、型と式で`|`の意味を切り替えない。assignment operatorはない。

`TYPE_IDENT.VALUE_IDENT`は型で修飾したpredefined primitiveを表す一つのatomic expressionである。`.`の
前後に空白を置かない。この形はmoduleやnamespaceのqualified name、field access、method、user-defined
associated itemを導入しない。認める型とprimitiveの組は[memory primitive](memory.md)に定める。

`expression.VALUE_IDENT(arguments)`はreceiver-first applicationであり、`VALUE_IDENT(expression, arguments)`と
同じapplicationを表す。callee名は通常のlexical scopeだけから解決し、receiverの型によるmemberやfunctionの
探索は行わない。parenthesized argument listは必須であり、`expression.VALUE_IDENT`だけの形は認めない。

`T(value)`と`value[T]`は同じtype applicationである。numeric型`T`ならnumeric conversion、直和型`T`かつ
`value`がcompile-time integer literalなら該当indexのinjection functionを表す。後者へ通常のapplicationを
続けた`i[T](payload)`と`payload[i[T]]`は同じ直和値を構築する。

`#`はoperand数でSymbol lengthとbyte accessを区別する。標準の表記はprefixでは`#value`、binaryでは
`value # index`とする。binary `#`はchainできず、必要な場合は括弧で境界を明示する。

`[]`は空直和、`[A]`は不正である。`[A, B, C]` は n-ary sum、`[A, [B, C]]` は nested sum であり、両者は同じ型ではない。
return binder group、`when`、zero-continuation applicationの意味は[明示的returnとcompletion](control.md)に定める。

decimal float literalは`DEC_DIGITS "." DEC_DIGITS EXPONENT? FLOAT_SUFFIX?`、
`DEC_DIGITS EXPONENT FLOAT_SUFFIX?`、または`DEC_DIGITS FLOAT_SUFFIX`のいずれかである。
`.5`と`1.`は認めない。exponentの数値separatorも他のdigit sequenceと同じ規則に従う。

`Bool` は predefined `TYPE_IDENT`、`false` と `true` は predefined `VALUE_IDENT` として通常の identifier 規則で token 化する。`then` は `if` syntax の keyword である。

Symbol literalのraw source characterとescapeが表すbytesは[Symbol仕様](symbols.md#literal)に定める。

byte literal の raw character は ASCII `0x20` から `0x7e` のうち single quote と backslash を除く範囲とする。非ASCII source characterは、UTF-8 encoded lengthにかかわらずbyte literal内では認めない。

## 存在しない構文

v0.5は`let`、`var`、`mut`、`const`、`fn`、`case`、return statement、loop、`break`、`continue`、record、
class、method、field access、nominal enum constructor、typed pointer syntax、reference、generic、trait、interface、macro、
exceptionを持たない。

return binder applicationはstatementではなくcontrol expressionであり、`return`という予約語も存在しない。
