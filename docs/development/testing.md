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

## Boundary tests

| Change | Focused test | Cross-boundary test |
|---|---|---|
| source、span、diagnostic | byte位置、UTF-8、rendered diagnostic | CLIからの利用者向けerror |
| lexer | tokenとlexical error | lexerからparserへ渡す代表的source |
| parser | accepted ASTとsyntax rejection | parseからname resolutionまで |
| resolve、check | name、capture、typeのpositive/negative case | typed coreまでの代表的program |
| lowering | typed inputに対するevaluation orderと表現 | C emissionまでの代表的program |
| C backend、ABI | emitted unit/headerとABI rule | 生成Cを実際のClangでcompile/link/execute |
| driver、CLI | argument、filesystem、process failure、exit status | public `malc` command |

format boundaryでは実際のconverterを使う。特にC backendはCらしい文字列を比較するだけで完了とせず、
warningを有効にしたClangで生成物をcompileする。invalid inputのtestは、後段が失敗することではなく、
そのinputを所有するstageが拒否することを確認する。

## Commands

repository rootで`nix develop`へ入り、`compiler/`から次を実行する。

```nu
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

rootから実行する場合は`--manifest-path compiler/Cargo.toml`を指定する。Nix development environmentまたは
flake inputを変更した場合は、rootで`nix flake check`も実行する。
