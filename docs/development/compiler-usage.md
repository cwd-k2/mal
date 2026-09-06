# reference compiler利用contract

Status: Current v0.5 development contract

この文書は`malc`のcommand、対応toolchain、生成物を利用者向けに定める。言語の意味は[`spec/`](../spec/)、
Cとの型・lifetime対応は[C host ABI](../spec/c-host-abi.md)、repository内の検証手順は[test policy](testing.md)を
正とする。

## 対応環境

v0.5 development profileで検証し対応する環境は、repositoryの`flake.lock`で固定した`x86_64-linux` development
environmentと、そこに含まれるClangである。repository rootから`nix develop`を使うと同じRust compiler、
Cargo、Clangへ入れる。

generated CとheaderはC11を要求する。Floatを使うprogramはさらにbinary32 `float`、binary64 `double`、
subnormal、`FLT_EVAL_METHOD == 0`を要求し、generated Cが満たさないtargetをcompile-timeに拒否する。
他のOS、architecture、C compilerはv0.5 development profileの検証対象外である。

## Command

```nu
malc check source.mal
malc format source.mal
malc emit-header source.mal
malc emit-host source.mal
malc emit-host source.mal --header custom.h
malc emit-c source.mal --output generated/program.c
malc build source.mal --output program --link host.c
```

- `check`はsourceを型検査し、成功時には生成物を作らない。
- `format`はsyntaxを検査し、commentとliteral spellingを保持したcanonical sourceをstdoutへ出す。
  入力fileは書き換えない。
- `emit-header`はhost implementation用のgenerated headerだけをsourceと同じdirectoryの`program.mal.h`へ生成する。
  `--output path`で出力先を変更できる。`extern` interfaceが型検査できればよく、実行可能な`main` bindingは要求しない。
- `emit-host`は各external operationを`MAL_DEFINE_<name>`で定義したC stubをstdoutへ出す。stubは
  `program.mal.h`をincludeし、未実装のoperationを`mal_trap`させるため、そのまま保存して実装の開始点にできる。
  `emit-header --output`で別名のheaderを生成した場合は、`--header name`でstubのquoted include名を合わせる。
  `emit-header`と同様に`main` bindingは要求しない。
- `emit-c`は指定したC translation unitと、同じdirectoryの固定名`program.mal.h`を生成する。
- `build`はgenerated C/headerをtemporary directoryに作り、C compilerでlinkした実行可能fileだけを指定先へ残す。
- `--link`は複数回指定でき、C source、object、static archive、shared objectを指定順にC compilerへ渡す。

生成した実行可能fileのcommand-line argumentは、source-level `main`が`(UInt64, Ptr) -> Int32`型なら
`Ptr`と`UInt64`からなる外部descriptor列として渡される。`Unit -> Int32`型の`main`はargumentを受け取らない。entry pointの正確な
contractは[program specification](../spec/programs.md#entry-point)に定める。

親directoryは必要に応じて作成し、同名の出力は置き換える。二つの`emit-c`出力を同じdirectoryへ置くと
`program.mal.h`が衝突するため、programごとにdirectoryを分ける。Cとheaderは一組として扱い、一方だけを
別の生成結果と組み合わせない。出力の更新はatomicではなく、filesystemまたはprocess failureの後に一部の
既存・生成済みartifactが残る場合がある。

## C compilerと`CC`

`build`は`CC`があればその値をC compilerの実行ファイル名またはpathとして使い、なければ`clang`を使う。
Nushellで一回だけ切り替える例は次のとおり。

```nu
with-env { CC: /path/to/clang } {
    malc build source.mal --output program --link host.c
}
```

`CC`は一つの実行ファイルを表し、optionを含むshell commandとして分割・評価しない。代替compilerは`malc`が
渡すC11、warning、`-O2`、strict floating-point optionを受理し、Clangと同じtarget ABIで全linker inputを扱う必要が
ある。v0.5にはcompiler optionを追加するCLIはない。

`build`はgenerated CとC source形式のlinker inputを`-O2`でcompileする。これはpublic buildの生成物policyであり、
言語semanticsがC optimizer固有のundefined behaviorに依存することを許可しない。`-fno-fast-math`、
`-ffp-contract=off`、`-frounding-math`、`-fexcess-precision=standard`は`-O2`と同時に渡す。
`emit-c`はC sourceだけを生成するため、利用者がcompileするときに同じstrict floating-point profileを保つ必要がある。

C compilerを起動できない場合と、compilerまたはlinkerがnon-zeroで終了した場合、`malc`は失敗し、診断を
stderrへ出す。後者ではtoolchainのstderrも保持する。

## Host adapterとshared object

host C sourceは対象programが生成した`program.mal.h`をincludeし、generated Cと同じtarget ABIでcompileする。
新しいadapterは`malc emit-host source.mal | save host.c`で雛形を作成できる。既存fileを置き換えるcommandなので、
編集済みの`host.c`に対して再実行してはならない。
`build`はtemporary header directoryをinclude pathへ加えるため、`--link host.c`はそのheaderを直接includeできる。

shared objectは`--link`で通常のlinker inputとして渡す。`malc` runtimeは`dlopen`、実行時symbol discovery、
plugin lifecycle、loader search pathを提供しない。必要なsoname、rpath、`LD_LIBRARY_PATH`、install locationは
target platformと利用者のbuild/deploymentが管理する。shared objectも対象programのheaderに対してbuildする。
`Ptr`を受け渡すadapterは、live region、permission、lifetimeを
[memory contract](../spec/memory.md)に従って定める。

## 生成物policy

generated C/headerのsource compatibilityまたはbinary compatibilityを異なる`malc` version間で保証しない。
配布や調査のため保持してよいが、source of truthは`.mal` sourceとhost adapterであり、compiler更新後には組で
再生成する。`examples/`ではhost sourceのeditor supportと生成例を兼ねて`program.mal.h`をversion controlに含め、testで
compiler出力との一致を検査する。`build`のtemporary artifactはcommandが所有し、成功・失敗のどちらでも終了時に削除する。

CLIの終了statusは成功が`0`、source・compile・toolchain errorが`1`、command grammarのusage errorが`2`である。
mal programのtrapはstderrへ理由を出して異常終了するが、portableなprocess exit codeは定めない。
