# C / LLVM内部DSL reference

この文書はgenerated C / LLVM構文を組み立てるときの検索用indexである。記法、補間、責務境界、追加時の判断は
[C / LLVM構文構築](backend-syntax-construction.md)を正とする。各entryはそのmacro内だけで有効であり、同じ語が別の
entryで使えることを意味しない。Cでは動的なnodeを`{}`、動的な列を`..{}`で渡す。LLVMでは`{}`、`..{}`で渡す。

## C

| 構築対象 | macro | 静的な形式 |
|---|---|---|
| type | `c_type!` | `name`、`Struct<tag>`、`*const type`、`*mut type`。動的なtypeは`{ type }` |
| parameter | `c_parameter!` | `name: type`、`_: type`、`#[maybe_unused] name: type` |
| parameter列 | `c_parameters!` | `[parameter, ...]`相当のcomma区切り列 |
| signature | `c_signature!` | `[attributes] fn name(parameters) -> type`。attributeは`#[static]`、`#[inline]`、`#[noreturn]`の実装済み組合せ |
| expression | `c_expr!` | 下記のRust-shaped expression |
| statement | `c_statement!` | `let`、`return`、`if`、`match`、expression statement |
| block | `c_block!` | statement列 |
| switch case列 | `c_switch_cases!` | `value => { statements }`、`_ => { statements }`、`..{ cases }` |
| function definition | `c_function!` | `fn name(parameters) -> type { statements }` |
| record field列 | `c_record_fields!` | `name: type`、`name: struct { fields }`、`name: union { fields }`、`name: fn(parameters) -> type` |
| record definition | `c_record!` | `struct tag { fields }`、`union tag { fields }` |
| initializer列 | `c_initializers!` | `field: value`、`outer.inner: value`、`_0: value`、`{ initializer }`、`..{ initializers }` |
| macro invocation | `c_invocation!` | `name(arguments)`。nameとargumentは通常の補間・spliceを利用可能 |
| translation unit断片 | `c_items!` | 下記のRust-shaped item列 |

`c_expr!`が受ける通常構文は次のとおりである。

| 種類 | 構文 |
|---|---|
| atom | bare identifier、number、string、`{ rust_expression }`、`sizeof(expression)` |
| unary / access | `&value`、`*value`、`value.field`、`value.{ field }`、`value as type` |
| call | `callee(arguments...)` |
| binary | `+`、`-`、`*`、`=`、`==`、`!=`、`>`、`&&` |
| conditional | `if condition { then_value } else { else_value }` |
| aggregate | `[values...]`、`Type { field: value, _0: positional_value, ..{ initializers } }` |

call argumentとcompound initializerの列には`..{ iterator }`を挿入できる。`match value { label => { body }, _ => { body } }`は
Cのswitchへ写像し、arm列には`..{ iterator }`を挿入できる。labelはRust patternでなくC expressionとして解析する。
nested initializerも同じexpression grammarを使う。

`c_items!`が受けるitemは次のとおりである。各item位置では`{ item }`、item列では`..{ translation_unit }`を使える。

| 種類 | 構文 |
|---|---|
| include | `include_system!(stdio.h);`、`include_system!("stdio.h");`、`include_quoted!({ path });` |
| define | `define!(NAME);`、`define!(NAME = expression);`、`define!(NAME(parameters) = expression);` |
| comment / assertion | `comment!(text);`、`assert!(condition, message);` |
| type alias | `type name = type;` |
| record typedef | `type name = struct [tag] { fields };`、`type name = union [tag] { fields };` |
| tagged record | `struct tag { fields }`、`union tag { fields }` |
| function | `[attributes] fn name(parameters) -> type;`またはbody付きdefinition |
| conditional items | `if defined(NAME) { items } else { items }`、`if !defined(NAME) { items }` |

`{ ... }`は、itemやfieldを囲むDSL上のblockと、Rust値を一個挿入する位置の両方に現れる。parserは文脈から区別する。
旧来の`#{ ... }`は受理せず、列の挿入には常に`..{ ... }`を使う。

C macroに含めないC固有nodeは次のtyped constructorから構築する。

| 構築対象 | constructor |
|---|---|
| 型付きpreprocessor replacement | `Directive::*_define` |

`Directive::*`を直接使うproduction call siteは、preprocessor replacementがexpression、record、record field、initializer、
statement、switch case、function itemのどれであるかを指定する箇所に限る。この分類はrendererが推測できないため、汎用の
`define!` grammarへ統合しない。別々に動的構築したsignatureとbodyの結合も`FunctionDefinition::from_signature`で明示する。

## LLVM

| 構築対象 | macro | 静的な形式 |
|---|---|---|
| type | `llvm_type!` | `void`、`ptr`、`float`、`double`、`int(bits)`、`array(length, type)`、`structure([types])` |
| parameter | `llvm_parameter!` | `name: type`、`_: type`、`#[immarg] name: type` |
| parameter列 | `llvm_parameters!` | comma区切りparameter列 |
| function attribute列 | `llvm_function_attributes!` | `nofree`、`noinline`、`nounwind`、`willreturn`、`memory_none`、`memory_argmem_read` |
| signature | `llvm_signature!` | `[#[linkage(internal)]] [#[attributes(...)]] fn name(parameters) -> type` |
| function declaration | `llvm_declaration!` | `[#[attributes(...)]] fn name(parameters) -> type;` |
| constant | `llvm_constant!` | `atom`、`structure`、`get_element_ptr`、`unary`、`binary`、`cast`、`zero` |
| typed constant | `llvm_typed_constant!` | `(type, constant)` |
| instruction | `llvm_instruction!` | 下記form |
| terminator | `llvm_terminator!` | `branch`、`return`、`switch`、`unreachable` |
| byte-owner global | `llvm_global!` | `byte_owner { ... }` |
| metadata | `llvm_metadata!` | `{ id: ..., distinct: ..., operands: [...] }` |

function bodyでは`llvm_instruction!` / `llvm_terminator!`を直接登録せず、同じgrammarを受け取る
`emit_instruction!` / `emit_terminator!`を使う。

LLVMの名前付き形式とfield labelは次のとおりである。

| form | field label |
|---|---|
| `alloca` | `ty`、`alignment` |
| `load` | `ty`、`pointer`、`alignment`、`metadata` |
| `store` | `value`、`pointer`、`alignment`、`metadata` |
| `call` | `tail`、`result_type`、`callee`、`arguments` |
| instruction `unary` | `operator`、`value` |
| instruction `cast` | `operator`、`value`、`to` |
| `extract_value` | `aggregate`、`indices` |
| instruction `binary` | `operator`、`ty`、`left`、`right` |
| `compare` | `kind`、`predicate`、`ty`、`left`、`right` |
| instruction `get_element_ptr` | `inbounds`、`element_type`、`pointer`、`indices` |
| `insert_value` | `aggregate`、`element`、`indices` |
| `phi` | `ty`、`incoming` |
| unconditional `branch` | `target` |
| conditional `branch` | `condition`、`then`、`otherwise` |
| `switch` | `cases`、`default` |
| constant `get_element_ptr` | `element_type`、`pointer`、`indices` |
| constant `unary` | `operator`、`operand` |
| constant `binary` | `operator`、`left`、`right` |
| constant `cast` | `operator`、`operand`、`to` |
| `byte_owner` | `name`、`bytes`、`alignment` |
| metadata | `id`、`distinct`、`operands` |

`callee`は`direct(value)`または`indirect(value)`、LLVM operandは親macro内の`(type, value)`で表す。
instruction fieldとtyped operand内の静的typeは`int(32)`や`ptr`をそのまま書き、macro実装上の都合による追加の括弧で囲まない。
typed valueおよびmetadata operandの`node`、`text`、`integer`は独立したproduction用entry macroではない。`arguments`と
`operands`は静的要素、`{}`、`..{}`を混在でき、`cases`は静的armと`..{}`を混在できる。`indices`と
`incoming`のように固定長arrayまたは構築済み列を受けるfieldでは、列全体を`{}`で渡す。
