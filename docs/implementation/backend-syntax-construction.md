# C / LLVM構文構築

この文書はgenerated CとLLVM IRを構築する内部DSLの境界、記法、coverageを定める。利用可能なmacro、constructor、
field labelは[内部DSL reference](backend-syntax-reference.md)から引く。backendが生成する意味とartifactの責務は
[compilerの責務境界](responsibilities.md)および[実行backend](execution-backend.md)を正とする。

## 構築経路

両backendはsource textを部品として合成しない。lowering policyはRustで実行し、`mal-backend-macros`のproc macroがrestricted
syntaxをtyped nodeのconstructorへ展開し、rendererだけが最後にtextへ変換する。CではRustの式、型、文、block、function、itemに自然に
対応する構造をproc macroが直接解析し、CのswitchはRustの`match`とarmへ対応させる。include、define、type alias、record、
function declaration / definition、static assertion、commentは`c_items!`でtranslation unitの断片として構築する。条件directiveと、
C preprocessor replacementが要求するrecord field、initializer、statementなどの型付き断片は`mal-backend`のconstructorで明示する。LLVM IR formもproc macroが直接解析し、
typed constructor呼び出しへ展開する。proc macro crateは構文のadmissionだけを所有し、typed node、validation、配置semantics、rendererは
`mal-backend`に置く。Rustの`proc-macro` crateは通常の型をexportできないため、syntax nodeをmacro crateへ移してはならない。

```text
checked interface / execution plan
              |
              v
       Rust lowering policy
              |
      +-------+--------+
      |                |
      v                v
 C proc macro      LLVM proc macro
      |                |
      v                v
 typed C nodes      typed LLVM nodes
      |                |
 TranslationUnit   emit_* + FunctionBuilder
      |                |
      +-------+--------+
              |
              v
        renderer (text化)
```

`TranslationUnit`、`Module`、`FunctionBuilder`はstateful rootである。順序、section間の空行、symbol重複、
basic blockの一意性、terminator、entry prefixを所有するため、通常のRust control flowで操作する。その内側の
type、item、declaration、signature、parameter、record、expression、statement、constant、instruction、terminator、
global、metadataはmacroから構築できる。LLVM function bodyでは`emit_instruction!`と`emit_terminator!`が、構築した
nodeを現在の`FunctionBuilder`へ登録する。entry blockの先頭へ遅延挿入する`entry_alloca`のように配置semanticsを
持つ操作だけは通常のemit経路と分ける。
backendのproduction call siteは、Rust-shapedなC構文とLLVM構文をmacroから構築する。C macroでは`{ expression }`と
`..{ iterator }`、LLVM grammarでは`{ expression }`と`..{ iterator }`で構築済みnodeを
渡す。Rust側で保持する`(Type, value)`列は
`TypedValue::from_pairs`のような名前付きhelperで一度だけ検証済みnode列へ正規化してからspliceする。個々のnodeの
constructorはmacro展開先、名前付き正規化helper、syntax自身の検証で使う。

## 共通記法

| 記法 | 意味 |
|---|---|
| `name: type` | 名前と型の対応 |
| `=` | alias、初期値、macro replacementなどの定義 |
| `=>` | switch caseのarm |
| `->` | function result |
| `,` | 同じ列または構造に属する要素の区切り |
| `;` | statementまたはitemの終端 |
| `(...)` | function callまたはIR固有formの引数 |
| `[...]` | source順を持つ構文列 |
| `{...}` | block、名前付き構造、switch armの集合 |
| `{ rust_expression }` | C / LLVM grammarへ構築済みscalarまたはnodeを一個挿入 |
| `..{ rust_iterator }` | C / LLVM grammarへ構築済みnode列をその位置へsplice |

内部DSLはRustのliteral、call、operator、tuple、array、block、attribute、`let`、function、match armに相当する小さな構文だけを
使う。これはRust codeをmacro内で実行する仕組みではなく、typed syntax nodeを構築するquasiquoteである。bare identifierは
生成先のC / LLVM identifierまたはDSL keywordであり、Rustの値を渡す位置だけをbraceで囲む。命令などの名前付き構造を囲む
`{ field: value }`と値位置の`{ rust_expression }`はparserが文脈で区別する。単一挿入と列spliceを型から推測せず、列spliceには
Rustと同じ`..`を要求する。補間は一度だけ評価し、spliceはiterator順を保存する。長いRust expressionはlocalで構築し、
補間位置にはそのlocalを置く。

macro invocationが一行で完結するときは`c_type!(*mut void)`のように`()`を使う。二行以上になるときは
`c_function! {`の直後で改行し、本文を一段下げ、対応する`}`をinvocationの開始行と同じ深さへ置く。macro本文の
constructor、array、blockもRustのcall、array、blockと同じく、子を親より一段深くする。複数行のargumentと要素には
末尾commaを置く一方、短いconstructorを要素ごとに分割しない。delimiterの選択や改行位置に構築するnodeの種類という
別の意味は持たせない。

同じ意味のnodeを構築する糖衣は設けない。静的型はすべてtype DSLを通し、順序付きの子は`[]`で囲み、複数の
semantic fieldを持つLLVM nodeは位置引数ではなく`{ field: value }`で構築する。typed LLVM operandは
Rustと同じ`(type, value)`へ正規化し、Rustの`Option`をDSLへ露出しない。Cのexpression statementはexpressionと終端の
`;`から構築し、専用のcall statementを持たない。

C functionは`fn name(parameters) -> result { statements }`として記述する。translation unit内の宣言は末尾を`;`にする。
動的に組み立てたsignatureやbodyを結合する場合は、
`FunctionDefinition::from_signature`などのtyped constructorを使い、macro内に別の合成grammarを持たせない。

```rust
let body = c_block! {
    let count: size_t = 0;
    ..{ generated_statements }
    if count > 0 {
        return { dynamic_result };
    }
};

let signature = llvm_signature! {
    #[linkage(internal)] #[attributes(nounwind)] fn { name }(
        "%context": ptr,
        ..{ generated_parameters },
    ) -> int(32)
};

emit_instruction! {
    self;
    let { result } = call {
        tail: false,
        result_type: { result_type },
        callee: direct({ callee }),
        arguments: [
            (ptr, "%context"),
            { dynamic_argument },
            ..{ generated_arguments },
        ],
    };
};

emit_terminator! {
    self;
    branch {
        condition: { condition },
        then: "done",
        otherwise: { fallback_label },
    };
};
```

## Cのcoverage

| 構造 | 構築macro | rootとの関係 |
|---|---|---|
| type、parameter、signature | `c_type!`、`c_parameter!`、`c_signature!` | declarationとdefinitionで共有 |
| expression | `c_expr!` | call、operator、compound literalと動的列spliceを提供 |
| statement、block、switch | `c_statement!`、`c_block!`、`c_switch_cases!` | block内をsource順に構築し、`match` armをC caseへ写像 |
| function definition | `c_function!` | signatureとblockを結ぶ |
| translation unit item | `c_items!` | include、define、comment、assert、type alias、record、functionをsource順に構築 |
| record / field断片 | `c_record!`、`c_record_fields!` | preprocessor replacementにも同じrecord grammarを再利用 |
| initializer列 | `c_initializers!` | positional、designated、nested designated、macro invocationを構築 |
| macro invocation | `c_invocation!` | itemまたは型付きfragmentへ埋め込むinvocationを構築 |
| 型付きpreprocessor replacement | typed constructor | replacement categoryを明示 |
| translation unit、section spacing | なし | stateful rootとしてRustで操作 |

## LLVMのcoverage

| 構造 | 構築macro | rootとの関係 |
|---|---|---|
| type、parameter、signature、attribute | `llvm_type!`、`llvm_parameter!`、`llvm_signature!` | declarationとdefinitionで共有 |
| function declaration | `llvm_declaration!` | `Module::declare`へ渡す |
| typed value、constant | `(type, value)`を受ける親macro、`llvm_constant!`、`llvm_typed_constant!` | instruction operandまたはglobal plan。typed value単体のproduction用entryは持たない |
| instruction、terminator、switch case | `llvm_instruction!`、`llvm_terminator!` | function bodyでは`emit_instruction!`、`emit_terminator!`を介して`FunctionBuilder`へ渡す |
| byte-owner global | `llvm_global!` | `Module::add_global`へ渡す |
| metadata nodeとoperand | `llvm_metadata!`とその`operands` field | `Module::add_metadata`へ渡す。operand単体のentryはtest専用 |
| basic block、function definition | なし | block/terminator invariantを`FunctionBuilder`が所有 |
| moduleとitem ordering | なし | symbol uniquenessとsection順を`Module`が所有 |

## 返り値と補間型

| entry | 返り値 | 補間 |
|---|---|---|
| C leaf / fragment macro | 対応するtyped C nodeまたはnode列 | `{}`で単一値、`..{}`で同じ子nodeの列 |
| `c_items!` | `TranslationUnit` | `{}`で単一item、`..{}`で別の`TranslationUnit` |
| LLVM type、parameter、signature、declaration、metadata | 対応するtyped LLVM node | fieldが要求する検証済みnode / 同じ子nodeの列 |
| LLVM constant、typed constant、instruction、terminator、global | `Option<typed LLVM node>` | fieldが要求する検証済みnode / 同じ子nodeの列 |

LLVM instructionの`arguments`へspliceする列は`TypedValue`であり、Rust側の`(Type, value)`列は通常
`TypedValue::from_pairs`で一度だけ検証する。補間式は通常のRustのmove規則に従い、一度だけ評価される。splice後の順序は
iteratorの順序と一致する。

## 失敗の境界

LLVMの`Option`はtyped constructorのadmission結果を保存する。不正なoperand、alignment、結果を束縛した`void` call、空の
index列、不正な`phi`などは`None`になる。function bodyの`emit_instruction!`と`emit_terminator!`はその失敗とbuilderの拒否を
`FunctionEmitter::emission_failed`へ集約し、最終的なbody生成を失敗させる。`FunctionBuilder`は未開始または重複したblock、
terminator後のinstruction、重複terminator、未終端blockを拒否し、`Module`は重複symbolを拒否する。

C側の不正なidentifier、numeric token、include pathはcompiler内部のinvariant違反としてpanicする。user programの診断を
このDSLへ委ねてはならず、macroがvalidationを複製したり失敗を文字列へ変換して隠したりしてはならない。

## 構文を追加する手順

1. typed enumまたはconstructor、renderer、単体testを追加する。
2. `mal-backend-macros`のparserを追加し、真に独立したrootである場合だけentry macroを公開する。
3. [内部DSL reference](backend-syntax-reference.md)を更新する。
4. 静的要素と、そのgrammarが定める単一補間・列spliceを組み合わせたcomposition testを追加する。
5. 順序や一意性などのstateを持つ処理はmacroでなく既存のroot builderへ置く。

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
- 動的な単一値と列は、C grammarでは`{}` / `..{}`、LLVM grammarでは`{}` / `..{}`にだけ現れる。
- `=>`、`->`、`,`、`;`、`()`、`[]`、`{}`は共通記法で定めた意味以外に転用しない。
- 同じtyped nodeへ到達する別名やstatement専用のexpression糖衣を置かない。
- macroとbuilderのどちらを通っても同じtyped constructorとvalidationへ到達する。
- renderer以外に`format!`やline assemblyによるC/LLVM source構築を置かない。
- stateful invariantをmacro展開へ隠さず、root builderを唯一のownerに保つ。
- 静的構文のためにRustのconstructor chainを反復せず、動的policyのためにDSL内へ独自control flowを増やさない。
- C itemは`c_items!`のRust-shaped grammarから構築し、preprocessor replacementの型だけはconstructor境界で明示する。
- function bodyでinstructionとterminatorの1対1 forwarding methodを作らず、共通の`emit_*`境界を使う。
