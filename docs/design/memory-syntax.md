# generic memory surface syntaxの試案

Status: Discussion draft (2026-09-17)

この文書はparametric polymorphism、memory placement、`Region`、`Packed`を同時に採択するprofileのexpression syntax、
operator precedence、formatter規則、numeric conversion移行を定める。各operationの型と意味は
[`Address`、target size、layout、placementの試案](size-and-alignment.md)、
[`Region`と`Packed`によるmemory transferの試案](region-and-packed.md)、
[parametric polymorphismと型index付きprimitiveの試案](parametric-polymorphism.md)を正とする。現行の規範は
[`expressions`](../spec/expressions.md)と[`grammar`](../spec/grammar.md)であり、この試案だけを根拠にsourceやcompilerを変更しない。

## Numeric conversion

numeric conversionは`.i8`、`.i16`、`.i32`、`.i64`、`.u8`、`.u16`、`.u32`、`.u64`、`.f32`、`.f64`、
`.bytes`、`.count`のclosed postfix familyへ統一する。通常のliteralは既存suffixを使い、conversionは既存のrounding、
precondition、integer modulo規則を保つ。

```mal
300u8          // range error
(300i64).u8    // 44
```

現行の`T(value)`と`value[T]`は同時に廃止し、compatibility syntaxを残さない。型identifierはdeclaration、annotation、
generic type argumentなどtype grammarが要求する位置だけに現れ、numeric conversionやmemory operationを含むexpressionの
operatorとして使わない。

## Expression precedence

既存operatorとmemory、Packed、conversion operatorを高い順に次の表へ統合する。

| Level | Form | Associativity |
|---|---|---|
| primary/name suffix | literal、name、product/sum、block、`name<T, ...>` | — |
| postfix chain | `f(...)`、`value[...]`、`value.f(...)`、`.i8`等のconversion、`@operand`、postfix `!` | left |
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

postfix chainはsource orderで左から適用するため、`address@u64@elementCount!`は
`(((address@u64)@elementCount)!)`になる。`@`の直後にあるclosed shape spellingはshape operandとしてparseする。
それ以外のname、literal、parenthesized expressionはCount operandであり、operatorを含むCountは`cursor@(count + 1count)`と書く。
これにより`@count`は`Count` shape、`@elementCount`はvalue operandとして構文だけで区別できる。

prefixとpostfixで共有する`!`、prefixとbinaryで共有する`#`、prefixとbinaryで共有する`*`と`<-`はoperand位置で区別する。
postfix conversionはparenthesized argumentを伴わないclosed suffix、receiver-first applicationは`.name(...)`なので互いに曖昧に
ならない。generic argument suffixの`<...>`はnameの一部としてcomparisonより先に認識し、generic list内の`>>`は二つの
closing `>`として扱う。

## Formatting

formatterはprefixとpostfixをoperandへ密着させ、binary operatorの両側へ空白を置く。postfix chainを改行する場合は継続行を
一段indentする。binary `<-`のchainはoperatorから始まる継続行にできる。

```mal
end := address@u8
    <- first
    <- second;
```

## 採択時に検証すること

- 各levelの隣接組合せ、prefix/postfix/binaryでtokenを共有するoperator、parenthesized Count operand
- `<...>`とcomparison、shift、nested generic applicationを含むlexer、parser、formatter corpus
- 旧conversion構文とexpression位置の型identifierを所有stageで拒否するdiagnostic
