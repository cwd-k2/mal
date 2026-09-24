# `malc`利用contract

Status: Current v0.6 development contract

この文書は`malc`のcommand、対応toolchain、生成物を利用者向けに定める。言語の意味は[`spec/`](../spec/)、
Cとの型・lifetime対応は[C host ABI](../spec/c-host-abi.md)、repository内の検証手順は[test policy](testing.md)を
正とする。

## 対応環境

検証し対応する環境は、repositoryの`flake.lock`で固定した`x86_64-linux` development
environmentと、そこに含まれるClangである。repository rootから`nix develop`を使うと同じRust compiler、
Cargo、Clangへ入れる。

C shim、runtime、generated header、host sourceはC11を要求する。generated headerは浮動小数点の要件を`_Static_assert`で検査し、
満たさないtargetをcompile-timeに拒否する（要件は[C host ABI](../spec/c-host-abi.md#host-value-mapping)に定める）。
他のOS、architecture、C compilerは検証対象外である。

## Nix flake

repository flakeは`x86_64-linux`向けに`packages.default`と`packages.malc`を同じcompiler packageとして公開する。
packageは`malc`が`build`時に起動するpinned Clangをruntime closureと`PATH`に含むため、呼出し側が別途Clangを用意する必要はない。
`apps.default`と`apps.malc`はそのpackageの`malc`を起動する。

repository rootでは次の形でpackageをbuildまたは実行できる。

```nu
nix build .#malc
nix run .#malc -- --help
nix run . -- build source.mal --output program
```

別のflakeはこのflakeをinputに置き、`inputs.mal.packages.x86_64-linux.malc`をpackageとして参照できる。`checks.malc`は同じ
derivationをbuildし、Cargo testを含むpackage検証を`nix flake check`へ接続する。対応systemは上記の対応環境と同じ
`x86_64-linux`だけであり、他system向けoutputを暗黙に宣言しない。

## Command

```nu
malc check source.mal
malc build source.mal -o program
malc build source.mal -o program --optimization baseline
malc build source.mal -o program --artifact-dir artifacts
malc build source.mal -o program --clang-arg '-lm'
malc emit header source.mal -o program.mal.h
malc emit host source.mal -o host.c
malc emit host source.mal --header custom.h
malc emit atcoder source.mal -o Main.cpp
```

生成物を出す`emit`は、`-o`（`--output`）がなければstdoutへ出し、あればそのpathへ書く。optionはsourceの前後どちらにも置け、
各commandは自身のoptionだけを受け取る。formatterは別commandの`mal-fmt`である（[formatting policy](formatting.md)）。

### `check`

sourceを型検査する。成功時には生成物を作らない。

### `emit header`と`emit host`

`emit header`はhost implementation用のgenerated headerを出す。`extern` interfaceが型検査できればよく、実行可能な`main` bindingは要求しない。
host sourceが`program.mal.h`をincludeする前提で、`emit header -o program.mal.h`のように保存する。

`emit host`は各external operationを`MAL_DEFINE_<name>`で定義したC stubを出す。stubは`program.mal.h`をincludeし、未実装のoperationを
`mal_call_trap`させるため、そのまま保存して実装の開始点にできる。別名のheaderを生成した場合は、`--header name`でstubのquoted include名を
合わせる。`emit header`と同様に`main` bindingは要求しない。

### `build`

LLVM module、C shim、C11 runtimeをtemporary directoryに作り、pinned Clangでlinkした実行可能fileだけを指定先へ残す。`-o`は必須である。
root sourceから推移的にrequireされた`.c` fileもcompileしてlinkする。

| Option | 効果 |
|---|---|
| `--optimization production` | 既定。正しさと採用gateを満たしたexecution、LLVM emission、toolchain techniqueをすべて有効化する |
| `--optimization baseline` | debugと差分検証のため、optional techniqueを外す |
| `--artifact-dir directory` | 通常temporaryな生成物とruntime入力を指定directoryへ書き、build後とClang失敗時も保持する。同名fileは置き換え、他のfileは変更しない |
| `--clang-arg argument` | 追加のClang argumentを一つ渡す。必要な数だけ繰り返せる |

同じoptionは一度だけ指定できる。ただし`--clang-arg`は繰り返せる。

### `emit atcoder`

mal module、C host、C shim、runtimeをx86_64 assemblyへまとめ、そのassemblyをglobal `asm`で運ぶ単一のC++ sourceを出す。
提出内容を読めるよう、rootと推移的にrequireしたmal moduleのsourceを先頭へ行commentとしてそのまま置く。AtCoderでは
`C++23 (GCC)`または`C++23 (Clang)`を選び、生成した`Main.cpp`全体を提出する。`build`と同じ`--optimization`、`--artifact-dir`、
`--clang-arg`を受け取る。

## 実行と出力

生成した実行可能fileのcommand-line argumentは、source-level `main`が`(USize, Address) -> Int32`型なら実行ファイル名を除いた
countとC hostの`argv + 1`として渡される。`Unit -> Int32`型の`main`はargumentを受け取らない。entry pointの正確なcontractは
[program specification](../spec/programs.md#entry-point)に定める。

親directoryは必要に応じて作成し、同名の出力は置き換える。出力の更新はatomicではなく、filesystemまたはprocess failureの後に一部の
既存・生成済みartifactが残る場合がある。

## Build toolchain

`build`はpinned `clang`から取得したtarget tripleとdata layoutをLLVM moduleへ設定し、generated C shim、checked-in C11 runtime、
requireされたhost C sourceと同じ`clang`でcompile、linkする。extern callはinternal pointer/out-pointer bridgeを通してpublic headerの
C ABIへ変換する。ambient `CC`は参照せず、compiler自体を差し替えるCLIはない。

`baseline`はoptionalなcompiler techniqueを使わず、Clangへ`-O0`を渡してLTO unitを作らない。Nix toolchainがambientに指定する
`_FORTIFY_SOURCE`の`-O0` warningだけは`-Wno-error=#warnings`でerrorから外す。その他のwarningは`-Werror`のままである。

既定の`production`は各artifactを`-O2 -flto`でcompileし、generated LLVM module、C shim、C11 runtime、requireされたhost C sourceを
一つのlink-time optimization unitにする。これはprogram固有のLLVM IRとprogram非依存のC mechanismのsource責務を保ったまま、境界上の
小さいhelper callを最適化する生成物policyである。`baseline`と`production`のどちらも`-fno-fast-math`、`-ffp-contract=off`、`-frounding-math`、
`-fexcess-precision=standard`を渡し、言語semanticsをC optimizer固有のundefined behaviorへ依存させない。

追加の`--clang-arg`はgenerated inputとrequireされたC sourceの後、compilerが所有する最後の`-o`より前に、指定順で渡す。
したがって`--clang-arg '-lm'`、`--clang-arg '-L/path' --clang-arg '-lname'`、追加のobjectまたはarchive、Cのinclude pathや
macro optionを利用できる。これは明示的なexternal build authorityであり、argumentのtarget compatibility、順序、外部fileの
lifetime、およびlanguage semanticsを変えるoptionを渡さない責任は呼出し側が持つ。追加argumentはMal sourceのrequire graphや
別のbuildへ伝播しない。

`emit atcoder`は同じtargetと入力を`-flto`で一つのassemblyへまとめるため、pinned LLDの`--lto-emit-asm`を使う。
生成するC++ sourceはx86_64 LinuxのC ABIに依存し、別architecture向けのportable sourceではない。libcやlibmのように
assemblyから未定義symbolとして参照するlibraryは提出先のC++ link環境にも必要である。任意のlocal shared libraryを
提出fileへ埋め込む機能ではない。

Clangを起動できない場合と、compilerまたはlinkerがnon-zeroで終了した場合、`malc`は失敗し、診断を
stderrへ出す。後者ではtoolchainのstderrも保持する。

## Host adapterとshared object

host C sourceは対象programが生成した`program.mal.h`をincludeし、LLVM moduleとshimと同じtarget ABIでcompileする。対応する
`.mal` fileからhost C sourceをrequireする。
新しいadapterは`malc emit host source.mal -o host.c`で雛形を作成できる。既存fileを置き換えるcommandなので、
編集済みの`host.c`に対して再実行してはならない。
`build`は生成直後のheaderを各C translation unitへpreincludeし、同名の隣接headerが今回の生成物を置き換えないようにする。
host sourceの明示的な`#include "program.mal.h"`は単独でのeditor supportとcompileのために維持する。

Mal sourceのrequirementとしてのshared object、`dlopen`、実行時symbol discovery、plugin lifecycleは提供しない。
link時に必要なshared libraryは`--clang-arg`で明示する。
`Address`を受け渡すadapterはnullを返してはならず、live region、permission、lifetimeを
[memory contract](../spec/memory.md)に従ってoperation固有のcontractに定める。

## 生成物policy

generated headerとbuild artifactのsource compatibilityまたはbinary compatibilityを異なる`malc` version間で保証しない。
配布や調査のため保持してよいが、source of truthは`.mal` sourceとhost adapterであり、compiler更新後には組で
再生成する。`examples/`ではhost sourceのeditor supportと生成例を兼ねて`program.mal.h`をversion controlに含め、testで
compiler出力との一致を検査する。`build`のtemporary artifactはcommandが所有し、成功・失敗のどちらでも終了時に削除する。
`--artifact-dir`を指定した場合は`program.ll`、`program-shim.c`、`program.mal.h`、`runtime.h`、`core.c`、`control.c`を保持し、
Symbolまたは`Buffer`を使うprogramでは`bytes.c`、`bytes_internal.h`、`symbol.c`も保持する。`emit atcoder`では`program-atcoder.lto.s`も保持する。これらはtoolchainとtargetに依存する
inspection用artifactであり、version間の互換性を保証しない。

CLIの終了statusは成功が`0`、source・compile・toolchain errorが`1`、command grammarのusage errorが`2`である。
mal programのtrapはstderrへ理由を出して異常終了するが、portableなprocess exit codeは定めない。
