# editor tooling利用方法

Status: Current v0.5 development tooling

この文書はrepositoryが提供するeditor packageとlanguage serverの起動方法を定める。言語syntaxは
[`grammar`](../spec/grammar.md)、formatterのoutputは[formatting policy](formatting.md)を正とする。

## VS Code syntax

`editors/vscode/`は`.mal`のlanguage registration、TextMate grammar、commentとbracketの設定に加え、
`mal-lsp` processのlifecycleを扱う。repository rootから次の一commandでpinned Nix environmentへの移行、
server build、extension dependencyのinstall、利用可能なVS Code環境に応じた起動またはinstallを行う。

```nu
nu scripts/vscode-dev.nu
```

buildとdependency準備だけを確認するときは`--prepare-only`を指定する。

```nu
nu scripts/vscode-dev.nu --prepare-only
```

scriptはextension development optionを実際にprobeする。通常のdesktop CLIではExtension Development Hostを起動する。
WSLの`remote-cli`ではこのoptionを利用できないため、server binaryを同梱したVSIXを生成してremote hostへinstallし、
repositoryを開く。`code`、`code-insiders`、WSL上のWindows user/system installationから自動検出できない配置では
`--code-command`でexecutableを指定する。

```nu
nu scripts/vscode-dev.nu --code-command /path/to/code
```

GUIを開かずに選択結果まで確認する場合は`--dry-run`を使う。

手動で準備する場合は次を実行する。

```nu
cargo build --manifest-path tools/mal-lsp/Cargo.toml --locked --release
cd editors/vscode
npm install
```

packageに同梱された`mal-lsp`以外を使う場合はVS Codeの`mal.server.path`へexecutable pathを指定する。desktop CLIでは
repository rootから次の形で開発用extensionを起動できる。`remote-cli`ではこのcommandを使わず、上のscriptを使う。

```nu
let extension_path = (pwd | path join editors/vscode)
run-external code $"--extensionDevelopmentPath=($extension_path)" .
```

## language server

`tools/mal-lsp/`はstdioでLSP JSON-RPCを扱う。開発環境では次のcommandで起動できる。

```nu
cargo run --manifest-path tools/mal-lsp/Cargo.toml --locked
```

full document sync、compiler diagnostic、document formattingに加え、hover、definition、references、rename、
document symbol、completion、semantic tokenを提供する。semantic requestはsource全体がparse、resolve、checkに
成功したときに利用できる。incremental analysisとinvalid sourceからの部分的semantic resultは対象外とする。
