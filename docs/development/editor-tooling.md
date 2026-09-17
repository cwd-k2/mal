# editor tooling利用方法

Status: Current v0.6 development tooling

この文書はrepositoryが提供するeditor packageとlanguage serverの起動方法を定める。言語syntaxは
[`grammar`](../spec/grammar.md)、formatterのoutputは[formatting policy](formatting.md)を正とする。

## VS Code syntax

`editors/vscode/`は`.mal`のlanguage registration、TextMate grammar、commentとbracketの設定に加え、
`mal-lsp` processのlifecycleを扱う。repository rootから次の一commandでpinned Nix environmentへの移行、
server build、extension dependencyのinstall、利用可能なVS Code環境に応じた起動またはinstallを行う。

semantic hoverはsymbolに対してmal形式の名前と型、symbol kindを表示する。型alias自身には右辺を一段だけ表示し、
型注釈を持つ値には注釈内のalias名を保った型を表示する。推論された型と名前を持たないtyped expressionにはcanonical
typeを使う。literalなど名前を持たないexpressionではsource expressionと型を表示し、hover rangeをそのexpressionへ限定する。
source declarationを持つsymbolでは宣言元fileからの相対pathと1始まりの行・columnも表示する。宣言の直前に空行を挟まず
連続する単独行の`//` commentはdocumentationとして表示し、各行の`//`直後にある一つのspaceと行末空白を除く。同じ行で
codeの後にあるcomment、宣言との間に空行があるcomment、predefined symbolと名前のないexpressionにはdocumentationを付けない。
byte literalはsingle-quoted string scopeの内側にcharacter scopeを持ち、literal内のbracketを構文上のbracketから隔離する。
TextMate grammarはreceiver-first applicationのcalleeをfunction、`.`をaccessor punctuationとして分類する。
semantic analysisではcalleeを通常のfunction referenceとして扱い、hover、definition、references、rename、
semantic tokenに同じdeclaration identityを使う。

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

repositoryの`.vscode/settings.json`はNix development environmentを選択し、`compiler`と`mal-lsp`の二つの
Cargo manifestを`rust-analyzer`へ明示する。保存時検査は両projectのall-target Clippyを`--locked`で実行する。
同じ設定はclangdにNixのClang wrapperをqueryさせ、`.clangd`はC sourceと生成headerをbackendと同じC11として解析する。
`.vscode/extensions.json`はこの環境選択、Rust、Cの各extensionを推奨する。設定を初めて受理した後、またはNix store pathが
flake更新で変わった後は、VS Codeをreloadする。個別に更新する場合は`rust-analyzer: Restart server`または
`clangd: Restart language server`を実行する。workspaceではdevelopment toolの内部listenerを自動公開しないよう
remote port auto-forwardingを無効にする。必要なportはPorts viewから明示的にforwardする。

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
成功したときに利用できる。`require`を含むsourceでは同じsource graphを解析し、definition、references、renameは
`.mal` file境界を跨ぐ。document symbolとsemantic tokenはrequest対象fileだけを返し、completionはそのfile自身の名前と
直接requireしたfileの公開名を返す。

`.`はcompletion triggerである。`receiver.`と`receiver.partialName`ではcurrent sourceのsyntax indexと直接requireしたfileから
lexical function候補を返す。receiverの型による候補探索や絞り込みは行わない。

`require "..."`内では`"`、`/`、`.`をcompletion triggerとして、source fileからの相対位置にあるdirectoryと`.mal`、`.c` fileを
候補にする。未閉じの引用符でもcursorまでのpath fragmentを使う。閉じたrequirement pathにはpath全体をrangeとするdocument linkを
返し、definition requestも対象fileの先頭へ移動する。どちらもsource全体のsemantic analysisが失敗していても利用でき、完全な
Symbol literalのescapeはdecodeしたpathへ対応付ける。存在しないfileはlink対象外とし、未閉じのpath fragmentにescapeがあれば
誤ったfilesystem pathを補完しない。

open中の`.mal` fileはdisk上の内容よりbufferを優先する。bufferのopen、change、close時にはopen documentのanalysisを
invalidateし、依存するsource graphを含めてdiagnosticを再生成する。変更されたdocumentには現在のversionを付けたdiagnosticを
publishし、修正後は空のdiagnosticをpublishして以前の表示を消す。再解析した他のopen documentについては、document versionと
diagnostic内容が直前のpublishから変わった場合だけpublishする。同一version、同一内容のdiagnosticは再送しない。
現在以下のversionを持つchange notificationは古いbuffer内容を復元しないよう無視する。

frontend analysisに失敗したversionでは、そのversionに対するsemantic requestをJSON-RPC errorにせず、hover、definition、renameは
結果なし、referencesとdocument symbolは空の結果として返す。semantic tokenはcurrent sourceのsyntax indexによる分類へfallbackし、
completionは上記のreceiver-first contextに限ってsyntax indexから候補を返す。一度失敗した同一versionをsemantic
requestごとに再解析しない。直前に成功したversionのsemantic indexは、編集やrequire先の変更で名前解決や型が変化している可能性が
あるため再利用しない。syntax fallbackは型、parameter identity、参照先を推測せず、現在のtokenとtop-level function declarationだけを
扱う。それ以上のpartial semantic resultには、parser、resolver、checkerがrecovery済み領域と依存関係を明示する別のadmitted表現を
導入する。
