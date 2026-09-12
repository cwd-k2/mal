# Compiler test policy

Status: Current development policy

この文書はcompiler変更の検証layer、test方針、完了時のcommandを定める。featureは、その規則を直接検査する
focused testと、影響する境界を代表するtestが通ったときに完了とする。

## 原則

1. 規則をbehaviorとして観測できる最小のdeterministic boundaryでtestする。
2. privateなcall orderや偶発的な内部構造ではなく、stageの公開された結果をtestする。
3. 新しい規則にはfocusedなpositive caseとnegative caseを置く。
4. 再発し得るdefectの修正にはregression testを置く。
5. stageやexternal toolの境界を跨ぐ変更では、各stageを直接testし、代表的なcross-boundary pathを一つ加える。
6. end-to-end caseは少数に保ち、[`spec/`](../spec/)のcontractから選ぶ。
7. testが作るtemporary file、directory、process、生成物はtestが所有し、必ずcleanupする。
8. totality、idempotence、layoutの組合せは、固定入力から生成する有限でdeterministicなcorpusで検査する。

optional optimizationは空集合の`baseline`を通常のcorrectness pathとする。各techniqueは単独のdecision testを持ち、採用済み集合は
`production` profileとしてbaselineと同じobservable result、effect order、trap、owner終状態、bounded native stackを保つことを
representativeなcross-boundary testで確認する。performance固有のresource上限や生成形状を検査するtestだけがproductionを明示する。

大きいintegration test targetは、共通helperとprocess起動回数を管理できるようtarget自体は維持しつつ、検査する
behaviorの領域ごとにchild moduleへ分ける。source fileと同様、行数だけを満たす分割や番号付きfileは作らない。

## Boundary tests

| Change | Focused test | Cross-boundary test |
|---|---|---|
| source、span、diagnostic | byte位置、UTF-8、rendered diagnostic | CLIからの利用者向けerror |
| lexer | tokenとlexical error | lexerからparserへ渡す代表的source |
| parser | accepted ASTとsyntax rejection | parseからname resolutionまで |
| resolve、check | name、capture、typeのpositive/negative case | typed coreまでの代表的program |
| lowering | typed inputに対するevaluation orderと表現 | LLVM artifact生成までの代表的program |
| LLVM backend、C ABI | emitted LLVM module/headerとABI rule | LLVM module、C shim、C runtimeを実際のClangでcompile/link/execute |
| driver、CLI | argument、filesystem、process failure、exit status | public `malc` command |

format boundaryでは実際のconverterを使う。特にLLVM backendはIRらしい文字列を比較するだけで完了とせず、
warningを有効にしたClangで生成物をcompileする。invalid inputのtestは、後段が失敗することではなく、
そのinputを所有するstageが拒否することを確認する。

## Commands

repository全体の完了判定は、rootでpinned environmentから一つのcheck scriptを実行する。

```nu
nix develop
nu scripts/check.nu
```

このscriptはcompiler、language server、VS Code extension、VSIX package、Nix flakeを順に検証する。VS Codeの
`node_modules`は`package-lock.json`から`npm ci`で再構成し、VSIXを`/tmp/mal-language-support-test.vsix`へ生成する。

compilerだけを変更中にfocused verificationを行う場合は、`compiler/`から次を実行する。

```nu
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

development shellは`rust-analyzer`も含む。editor上のRust diagnosticは同じCargo projectとtoolchainを使い、
完了判定では上記の全targetに対するClippyをwarning-freeにする。

rootから実行する場合は`--manifest-path compiler/Cargo.toml`を指定する。

`tools/mal-lsp/`を変更した場合はrepository rootで次も実行する。

```nu
cargo fmt --manifest-path tools/mal-lsp/Cargo.toml --check
cargo clippy --manifest-path tools/mal-lsp/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path tools/mal-lsp/Cargo.toml --locked
```

`editors/vscode/`のfocused verificationは次で行う。完了時のrepository checkではdependency install、server同梱、
package生成も常に実行する。

```nu
nu scripts/vscode-dev.nu --prepare-only
cd editors/vscode
npm test
npm exec -- vsce package --out /tmp/mal-language-support-test.vsix --allow-missing-repository
```
