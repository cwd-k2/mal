# editor tooling利用方法

Status: Current v0.5 development tooling

この文書はrepositoryが提供するeditor packageとlanguage serverの起動方法を定める。言語syntaxは
[`grammar`](../spec/grammar.md)、formatterのoutputは[formatting policy](formatting.md)を正とする。

## VS Code syntax

`editors/vscode/`は`.mal`のlanguage registration、TextMate grammar、commentとbracketの設定に加え、
`mal-lsp` processのlifecycleを扱う。最初にserverをbuildし、extension dependencyをinstallする。

```nu
cargo build --manifest-path tools/mal-lsp/Cargo.toml --locked
cd editors/vscode
npm install
```

`mal-lsp`が`PATH`にない場合はVS Codeの`mal.server.path`へexecutable pathを指定する。repository rootから
開発用extensionを起動できる。

```nu
code --extensionDevelopmentPath (pwd | path join editors/vscode) .
```

## language server

`tools/mal-lsp/`はstdioでLSP JSON-RPCを扱う。開発環境では次のcommandで起動できる。

```nu
cargo run --manifest-path tools/mal-lsp/Cargo.toml --locked
```

full document sync、compiler diagnostic、document formattingに加え、hover、definition、references、rename、
document symbol、completion、semantic tokenを提供する。semantic requestはsource全体がparse、resolve、checkに
成功したときに利用できる。incremental analysisとinvalid sourceからの部分的semantic resultは対象外とする。
