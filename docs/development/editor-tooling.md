# editor tooling利用方法

Status: Current v0.6 development tooling

この文書はrepositoryが提供するeditor packageとlanguage serverの起動方法を定める。言語syntaxは
[`grammar`](../spec/grammar.md)、formatterのoutputは[formatting policy](formatting.md)を正とする。

## VS Code syntax

`editors/vscode/`は`.mal`のlanguage registration、TextMate grammar、commentとbracketの設定に加え、
`mal-lsp` processのlifecycleを扱う。repository rootから次の一commandでpinned Nix environmentへの移行、
server build、extension dependencyのinstall、利用可能なVS Code環境に応じた起動またはinstallを行う。

TextMate grammarはbyte literalをsingle-quoted string scopeの内側にcharacter scopeを持たせ、literal内のbracketを構文上のbracketから
隔離する。receiver-first applicationのcalleeはfunction、`.`はaccessor punctuationとして分類する。

```nu
nu scripts/dev.nu vscode
```

buildとdependency準備だけを確認するときは`--prepare-only`を指定する。

```nu
nu scripts/dev.nu vscode --prepare-only
```

scriptはextension development optionを実際にprobeする。通常のdesktop CLIではExtension Development Hostを起動する。
WSLの`remote-cli`ではこのoptionを利用できないため、server binaryを同梱したVSIXを生成してremote hostへinstallし、
repositoryを開く。`code`、`code-insiders`、WSL上のWindows user/system installationから自動検出できない配置では
`--code-command`でexecutableを指定する。

```nu
nu scripts/dev.nu vscode --code-command /path/to/code
```

GUIを開かずに選択結果まで確認する場合は`--dry-run`を使う。

repositoryの`.vscode/settings.json`はNix development environmentを選択し、root
のCargo workspaceを`rust-analyzer`へ明示する。保存時検査は全crateのall-target Clippyを`--locked`で実行する。
同じ設定はclangdにNixのClang wrapperをqueryさせ、`.clangd`はC sourceと生成headerの言語をCと明示して
backendと同じC11として解析する。
`.vscode/extensions.json`はこの環境選択、Rust、Cの各extensionを推奨する。設定を初めて受理した後、またはNix store pathが
flake更新で変わった後は、VS Codeをreloadする。個別に更新する場合は`rust-analyzer: Restart server`または
`clangd: Restart language server`を実行する。workspaceではdevelopment toolの内部listenerを自動公開しないよう
remote port auto-forwardingを無効にする。必要なportはPorts viewから明示的にforwardする。

手動で準備する場合は次を実行する。

```nu
cargo build -p mal-lsp --locked --release
cd editors/vscode
npm install
```

packageに同梱された`mal-lsp`以外を使う場合はVS Codeの`mal.server.path`へexecutable pathを指定する。desktop CLIでは
repository rootから次の形で開発用extensionを起動できる。`remote-cli`ではこのcommandを使わず、上のscriptを使う。

```nu
let extension_path = (pwd | path join editors/vscode)
run-external code $"--extensionDevelopmentPath=($extension_path)" .
```

## NeovimとHelix

`editors/tree-sitter-mal/`はNeovimとHelixが共有するparser source、highlight、indent、text object queryを所有する。
`grammar.js`から生成する`src/parser.c`、`src/grammar.json`、`src/node-types.json`はconsumerがgeneratorなしでparserを
buildできるようrepositoryへ含める。platform固有のshared libraryは`.artifacts/editor-runtime/`へ生成し、repositoryへ含めない。
生成時に同梱されるTree-sitter headerにはupstreamのMIT licenseを`third-party/tree-sitter/LICENSE`として添付する。このdirectoryは
外部由来のnoticeだけを所有し、grammarとquery自体にはrepository rootのMIT licenseを適用する。

repository flakeは対象system向けにcompileしたparserと同じrevisionのqueryを
`packages.${system}.editor-runtime`として公開する。別のflakeは`mal`をinputに置き、`malc`と`mal-lsp`と同様にこのpackageを
参照できる。consumerにTree-sitter CLI、Node.js、生成処理は要求しない。package layoutは次のとおりである。

```text
parser/mal.so
grammars/mal.so
queries/mal/highlights.scm
queries/mal/indents.scm
queries/mal/textobjects.scm
```

compiler、language server、editor runtimeをすべて使うconsumerは、三つを同じrevisionからまとめた
`packages.${system}.toolchain`をdevelopment shellへ追加できる。`toolchain/bin`は`malc`と`mal-lsp`、package rootは上記の
editor runtime layoutを持つ。compilerだけ、language serverだけ、またはruntimeだけが必要なconsumerには、対応する細粒度packageを
使用する。

Neovimではpackage rootを`runtimepath`へ追加し、`.mal` filetype、`vim.treesitter.start`、commandを`mal-lsp`とするbuilt-in
LSP configをconsumer側で登録する。Helixではpackage rootを`HELIX_RUNTIME`で公開し、projectの`languages.toml`でgrammarを
`mal`、language server commandを`mal-lsp`とする。editorのuser configuration、workspace trust、root detection、起動directoryに
依存するlauncherはconsumerが所有し、mal repositoryの絶対pathや`tools/*/target`を参照しない。

C host adapterを編集するprojectは、対応する`.mal` sourceから`malc emit header -o`でheaderを生成する。NixのClang wrapperを使う
場合、`.clangd`でC11を指定し、clangdへ`--query-driver=/nix/store/*-clang-wrapper-*/bin/clang`を渡す。この設定はTree-sitterや
mal language serverとは別のC editor integrationである。

rootの`.nvim.lua`と`.nvim/lsp/mal.lua`はNeovim 0.11以降のproject-local filetype、Tree-sitter、built-in LSP設定である。
rootの`.helix/languages.toml`はHelixのproject-local languageと`mal-lsp`設定である。両editor用のserver、parser、queryを準備して
起動するにはrepository rootで次を実行する。各editorがproject-local設定を読む前に、内容を確認してworkspaceをtrustする。

```nu
nu scripts/dev.nu neovim
nu scripts/dev.nu helix
```

`--prepare-only`を指定するとeditorを起動せず生成と検証まで行う。Helixのlexical highlight、indent、text objectはTree-sitter queryを
使う。Helixは`mal-lsp`のsemantic tokenを利用しない。NeovimはTree-sitterによるlexical highlightへ`mal-lsp`のsemantic tokenを
重ねる。LSP機能の内容は次節を正とする。

## language server

`crates/mal-lsp/`はstdioでLSP JSON-RPCを扱う。開発環境では次のcommandで起動できる。

```nu
cargo run -p mal-lsp --locked
```

module責務は[`crates/mal-lsp/README.md`](../../crates/mal-lsp/README.md)を正とする。

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

documentのsemantic analysis cacheは`Stale`、`Failed`、`Ready`のいずれかであり、source graphとsemantic indexはその状態に
付随する。独立したfreshness flagとoptional resultの組合せは持たず、編集時には状態全体を`Stale`へ戻す。

frontend analysisに失敗したversionでは、そのversionに対するsemantic requestをJSON-RPC errorにせず、hover、definition、renameは
結果なし、referencesとdocument symbolは空の結果として返す。semantic tokenはcurrent sourceのsyntax indexによる分類へfallbackし、
completionは上記のreceiver-first contextに限ってsyntax indexから候補を返す。一度失敗した同一versionをsemantic
requestごとに再解析しない。直前に成功したversionのsemantic indexは、編集やrequire先の変更で名前解決や型が変化している可能性が
あるため再利用しない。syntax fallbackは型、parameter identity、参照先を推測せず、現在のtokenとtop-level function declarationだけを
扱う。それ以上のpartial semantic resultには、parser、resolver、checkerがrecovery済み領域と依存関係を明示する別のadmitted表現を
導入する。

### hover

semantic hoverはsymbolごとに次を表示する。

- mal形式の名前、型、symbol kind。
- 型aliasは右辺を一段だけ表示する。型注釈を持つ値は注釈内のalias名を保った型を、推論された型と名前を持たない
  typed expressionはcanonical typeを表示する。
- literalなど名前を持たないexpressionではsource expressionと型を表示し、hover rangeをそのexpressionへ限定する。
- source declarationを持つsymbolでは、宣言元fileからの相対pathと1始まりの行・column。
- documentation。宣言の直前に空行を挟まず連続する単独行の`//` commentを表示し、各行の`//`直後にある一つのspaceと行末空白を除く。
  同じ行でcodeの後にあるcommentと、宣言との間に空行があるcommentは対象外とする。predefined type、value、memory intrinsic、
  Buffer methodはcompilerのpredefined metadataにある英語reference documentationを表示し、signatureに加えてscope、offsetの単位、
  返り値、主要preconditionを説明する。completion itemにも同じdocumentationを付ける。名前のないexpressionにはdocumentationを付けない。

receiver-first applicationのcalleeは通常のfunction referenceとして扱い、hover、definition、references、rename、semantic tokenに
同じdeclaration identityを使う。
