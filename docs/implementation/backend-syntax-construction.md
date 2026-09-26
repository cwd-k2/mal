# C / LLVM構文構築

この文書はgenerated CとLLVM IRを構築する内部DSLの境界、記法、coverageを定める。backendが生成する
意味とartifactの責務は[compilerの責務境界](responsibilities.md)および
[実行backend](execution-backend.md)を正とする。

## 構築経路

両backendはsource textを部品として合成しない。lowering policyはRustで実行し、macroはrestricted syntaxの
typed nodeを構築し、rendererだけが最後にtextへ変換する。

```text
checked interface / execution plan
              |
              v
       Rust lowering policy
              |
      +-------+--------+
      |                |
      v                v
 C template macro  LLVM template macro
      |                |
      v                v
 typed C nodes      typed LLVM nodes
      |                |
 TranslationUnit   Module + FunctionBuilder
      |                |
      +-------+--------+
              |
              v
        renderer (text化)
```

`TranslationUnit`、`Module`、`FunctionBuilder`はstateful rootである。順序、section間の空行、symbol重複、
basic blockの一意性、terminator、entry prefixを所有するため、通常のRust control flowで操作する。その内側の
type、declaration、signature、parameter、aggregate、expression、statement、constant、instruction、terminator、
global、metadataはmacroから構築できる。
backendのproduction call siteはstateful root以外をmacroから構築する。動的policyは`{}`と`{{}}`で構築済みnodeを
渡し、call siteでconstructorを直接組み合わせない。既存nodeのconstructorはmacro展開先とsyntax自身の検証で使う。

## 共通記法

| 記法 | 意味 |
|---|---|
| `name: type` | 名前と型の対応 |
| `=` | alias、初期値、macro replacementなどの定義 |
| `=>` | switch case、typed value、metadata IDなど左右の対応 |
| `->` | function resultまたは変換先 |
| `[...]` | source順を持つ構文列 |
| `{ rust_expression }` | 構築済みnodeを一個挿入 |
| `{{ rust_iterator }}` | 構築済みnode列をその位置へsplice |

`{}`と`{{}}`の中だけが明示的なRust interpolationである。`rust`、`extend`、`typed_extend`という補助keywordは
使わない。補間は一度だけ評価し、列のspliceはiterator順を保存する。

再帰nodeは一個のtoken treeとして子macroへ渡せるよう、静的な子を`(...)`で囲む。これはCやLLVMの
出力上の括弧を表すものではなく、`macro_rules!`が任意のRust expressionと再帰DSLを曖昧なく分離する境界である。
constructor名と引数の見通しを悪くするだけの括弧は追加しない。

```rust
let body = c_block!(
    (var (c_type!(named("size_t"))) ("count") = (number 0)),
    {{ generated_statements }},
    (if (greater (id "count"); (number 0)); [
        (return { dynamic_result }),
    ]),
);

let signature = llvm_signature!(internal fn { name }(
    "%context": ptr,
    {{ generated_parameters }},
) -> int(32); attributes [nounwind]);

let call = llvm_instruction!(call
    Some("%result"), false, int_type, direct callee; [
        (typed pointer_type => "%context"),
        { dynamic_argument },
        {{ generated_arguments }},
    ]
);
```

## Cのcoverage

| 構造 | 構築macro | rootとの関係 |
|---|---|---|
| type、parameter、signature | `c_type!`、`c_parameter!`、`c_signature!` | declarationとdefinitionで共有 |
| variable declaration | `c_variable!` | statementとaggregate fieldの下位node |
| declaration、aggregate、field | `c_declaration!`、`c_aggregate!`、`c_aggregate_field!` | `TranslationUnit::push`へ渡す |
| initializer、expression | `c_initializer!`、`c_expr!` | 再帰構築と動的列spliceを提供 |
| statement、block、switch case | `c_statement!`、`c_block!`、`c_switch_case!` | block内をsource順に構築 |
| function definition | `c_function!` | signatureとblockを結ぶ |
| preprocessor、macro invocation | `c_directive!`、`c_macro_invocation!` | directive順は`TranslationUnit`が所有 |
| comment | `c_comment!` | 検証済みの単一payloadをitemへ変換 |
| translation unit、section spacing | なし | stateful rootとしてRustで操作 |

## LLVMのcoverage

| 構造 | 構築macro | rootとの関係 |
|---|---|---|
| type、parameter、signature、attribute | `llvm_type!`、`llvm_parameter!`、`llvm_signature!` | declarationとdefinitionで共有 |
| function declaration | `llvm_declaration!` | `Module::declare`へ渡す |
| typed value、constant | `llvm_value!`、`llvm_constant!`、`llvm_typed_constant!` | instruction operandまたはglobal plan |
| instruction、terminator、switch case | `llvm_instruction!`、`llvm_terminator!` | `FunctionBuilder`へ渡す |
| byte-owner global | `llvm_global!` | `Module::add_global`へ渡す |
| metadata nodeとoperand | `llvm_metadata!`、`llvm_metadata_operand!` | `Module::add_metadata`へ渡す |
| basic block、function definition | なし | block/terminator invariantを`FunctionBuilder`が所有 |
| moduleとitem ordering | なし | symbol uniquenessとsection順を`Module`が所有 |

LLVM macroが返す`Option`はtyped constructorのadmission結果を保存する。macroがvalidationを複製したり、失敗を
文字列へ変換して隠したりしてはならない。

## 分割とoptimization

logical LLVM moduleを複数の`.ll` fileへ物理分割してimport相当のlinkを行うことはdelivery policyであり、syntax
構築の責務ではない。現在は一つの`Module`を一つの`program.ll`へrenderする。分割する場合もfeature selection、
declaration deduplication、target triple/data layoutの一致をmodule compositionが先に確定し、各fileを独立の文字列
templateとして管理しない。

optimization passはtyped syntax構築より前の`execution` decision、または構築後のLLVM/Clang optimizationとして
位置付ける。rendererやmacroは最適化判断を持たず、選択済みの構造を保存する。

## 監査基準

変更時は次を同時に満たす。

- call siteから生成される構造とsource順が読める。
- CとLLVMで型、列、補間、対応関係の記法が同じ意味を持つ。
- macroとbuilderのどちらを通っても同じtyped constructorとvalidationへ到達する。
- renderer以外に`format!`やline assemblyによるC/LLVM source構築を置かない。
- stateful invariantをmacro展開へ隠さず、root builderを唯一のownerに保つ。
- 静的構文のためにRustのconstructor chainを反復せず、動的policyのためにDSL内へ独自control flowを増やさない。
- production call siteでstateful root以外のsyntax constructorを直接呼ばない。
