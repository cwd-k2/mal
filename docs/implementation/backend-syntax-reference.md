# C / LLVM内部DSL reference

この文書はgenerated C / LLVM構文を組み立てるときの検索用indexである。記法、補間、責務境界、追加時の判断は
[C / LLVM構文構築](backend-syntax-construction.md)を正とする。各entryはそのmacro内だけで有効であり、同じ語が別の
entryで使えることを意味しない。動的なnodeは`#{}`、動的な列は`...#{}`で渡す。

## C

| 構築対象 | macro | 静的な形式 |
|---|---|---|
| comment | `c_comment!` | scalar payload |
| type | `c_type!` | `named(name)`、`struct(name)`、`const(named(name))`、`ptr(type)` |
| variable | `c_variable!` | `name: type`、`array name: type; size length` |
| parameter | `c_parameter!` | `name: type`、`_: type`、`#[maybe_unused] name: type` |
| parameter列 | `c_parameters!` | `[parameter, ...]`相当のcomma区切り列 |
| signature | `c_signature!` | `[attributes] fn name(parameters) -> type` |
| aggregate field | `c_aggregate_field!` | `name: type`、function pointer、nested `struct` / `union` |
| aggregate field列 | `c_aggregate_fields!` | comma区切りfield列 |
| aggregate | `c_aggregate!` | `struct tag { fields }`、`type alias = struct [tag] { fields }` |
| declaration | `c_declaration!` | function declaration、type alias、`static_assert` |
| initializer | `c_initializer!` | `positional(value)`、`field(name, value)`、`path(path, value)` |
| expression | `c_expr!` | 下記constructor |
| statement | `c_statement!` | `let`、`return`、`if`、`switch`、expression statement |
| block | `c_block!` | statement列 |
| switch case | `c_switch_case!` | `value => { statements }`、`_ => { statements }` |
| function definition | `c_function!` | static function構文、または下記の合成label |
| preprocessor directive | `c_directive!` | `include`、`define`、`if`、`ifndef`、`else`、`endif`、`define_items` |
| macro invocation | `c_macro_invocation!` | `name([arguments])` |

`c_expr!`のconstructorは次のとおりである。

| 種類 | constructor |
|---|---|
| atom | `id`、`number`、`string` |
| unary / access | `address`、`dereference`、`field`、`pointer_field`、`sizeof`、`cast` |
| call | `call`、`invoke` |
| binary | `add`、`subtract`、`multiply`、`assign`、`equal`、`not_equal`、`greater`、`logical_and` |
| aggregate | `conditional`、`initializer`、`compound` |

Cで名前付きfieldを取る形式は次のとおりである。

| form | field label |
|---|---|
| `c_function!`による構築済みnodeの合成 | 先頭に`signature`または`macro`。構築済みblockには`body` |
| `c_variable!`のarray | `size` |
| `c_directive!(define_items ...)` | `parameters`、`declarations`、`definitions`、`trailing` |

## LLVM

| 構築対象 | macro | 静的な形式 |
|---|---|---|
| type | `llvm_type!` | `void`、`ptr`、`float`、`double`、`int(bits)`、`array(length, type)`、`structure([types])` |
| parameter | `llvm_parameter!` | `name: type`、`_: type`、`#[immarg] name: type` |
| parameter列 | `llvm_parameters!` | comma区切りparameter列 |
| function attribute列 | `llvm_function_attributes!` | `nofree`、`noinline`、`nounwind`、`willreturn`、`memory_none`、`memory_argmem_read` |
| signature | `llvm_signature!` | `[#[linkage(internal)]] [#[attributes(...)]] fn name(parameters) -> type` |
| function declaration | `llvm_declaration!` | `[#[attributes(...)]] fn name(parameters) -> type;` |
| typed value | `llvm_value!` | `typed(type, value)` |
| constant | `llvm_constant!` | `atom`、`structure`、`get_element_ptr`、`unary`、`binary`、`cast`、`zero` |
| typed constant | `llvm_typed_constant!` | `typed(type, constant)` |
| instruction | `llvm_instruction!` | 下記form |
| terminator | `llvm_terminator!` | `branch`、`return`、`switch`、`unreachable` |
| byte-owner global | `llvm_global!` | `byte_owner { ... }` |
| metadata operand | `llvm_metadata_operand!` | `node`、`text`、`integer` |
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

`callee`は`direct(value)`または`indirect(value)`、LLVM operandは`typed(type, value)`で表す。`arguments`と
`operands`は静的要素、`#{}`、`...#{}`を混在でき、`cases`は静的armと`...#{}`を混在できる。`indices`と
`incoming`のように固定長arrayまたは構築済み列を受けるfieldでは、列全体を`#{}`で渡す。
