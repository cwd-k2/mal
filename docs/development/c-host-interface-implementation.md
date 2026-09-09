# C host interface実装手順

Status: Accepted implementation procedure; implementation pending

この文書は、採択済みの[C host interface再設計案](c-host-interface.md)をcurrent v0.5実装へ反映する順序、test、
切替条件を定める。host-facing behaviorは再設計案、現在の規範は[C host ABI](../spec/c-host-abi.md)、compiler内の
責務境界は[compilerの責務境界](../implementation/responsibilities.md)、検証方法は[test方針](testing.md)を正とする。
ここではそれらのruleを再定義せず、実装作業へ対応付ける。

## 完了時の状態

次を同じ公開ABI切替で成立させる。

- external operationは`MAL_DEFINE_<operation>`だけでhost bodyを定義し、bodyは`mal_call_t *`と
  `mal_<T>_t`を受け取る。
- compiler-facing `mal_ext_*`、raw `MalType_*`、parameter flattening、ownership helperはgenerated detailとなり、
  hostが直接使用する経路を残さない。
- resultはscalarと`Unit`を含めてtyped `return` operationでちょうど一度確定する。
- `Symbol`は`mal_Symbol_t`、`Symbol::to_bytes`、`Symbol::from_bytes`だけをhost-facing contractにし、観測しない
  Symbolをmaterializeしない。
- product、sum、external opaque type、`Ptr`を再設計案のmappingへ統一する。
- checked-in example headerと`host.c`が新しいsurfaceだけを使い、規範文書が実装済みのABIを記述する。
- ABI versionをv0.6へ上げ、v0.5とv0.6のpublic operationを混在させない。

## 作業規則

内部refactorは生成出力を変えないcommitとして先行できる。公開名、header、wrapper、example、spec、ABI versionの変更は一つの
cutoverとして扱う。途中のbranchで新旧両方をpublicにしたcompatibility layerやdeprecated aliasは追加しない。

各段階では先にfocused testを置き、その段階が所有する表現を直接検査する。C文字列のsnapshotだけで終えず、host boundaryを
変える段階ではwarningを有効にしたClangによるcompile、link、executeを行う。生成Cの構文は
`compiler/src/c_emit/syntax`のnodeとして構成し、複数宣言を出すmacroをraw stringで迂回しない。

## 1. Baselineを固定する

変更前に次を実行し、失敗があればABI実装と混ぜずに記録または解消する。

```nu
cargo fmt --manifest-path compiler/Cargo.toml --check
cargo clippy --manifest-path compiler/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path compiler/Cargo.toml
```

既存testから次のbaselineも記録する。

- Symbol flat、rope、passthrough、host bytes admissionのallocation、byte copy、materialization count
- representative exampleのClang `-O2` binary sizeと実行時間
- 終了時live allocation、ASan/UBSan pressure case

測定値は[性能測定履歴](../history/performance/)へ条件と共に保存する。未測定のoptimizationを実装条件にしない。

## 2. Raw signatureとhost body signatureを分離する

最初に`compiler/src/c_emit/host_signature.rs`の一つのsignature表現を、次の二つの責務へ分ける。この段階ではgenerated outputを
変えない。

- compiler signature: 現在の`mal_ext_*` result、`MalContext *`、raw parameter、top-level product flatteningを所有する。
- host body signature: source aliasを保つ`mal_<T>_t`、単一のtop-level product parameter、`mal_call_t *`を所有する。

`compiler/src/c_emit/header/mod.rs`のexternal declaration、definition macro、stub生成がどちらを要求しているかを型で区別する。
同じparameter listをflagで両用しない。`emit-header`と`emit-host`が同じ`core::ProgramInterface`から両signatureを導出することを
`compiler/tests/c_emit/host_interface.rs`と`compiler/tests/driver/artifacts.rs`で検査する。

## 3. Host value registryを導入する

`compiler/src/c_emit/types/mod.rs`の`TypeRegistry`にraw C typeとは別の`HostValue(T)` mappingを追加し、
`compiler/src/c_emit/types/collect.rs`で一度だけ収集する。

- source aliasは`mal_<Alias>_t`を使う。
- anonymous product/sumはraw registryと同じstructural identityから`mal_repr_<kind>_<id>_t`を得る。
- scalarは対応するC scalar、`Ptr`は`typedef void *mal_Ptr_t`、external opaque typeは`mal_<Type>_t`とする。
- product/sumはhost valueを再帰的にmemberへ適用する。
- `mal_Symbol_t`のfieldはgenerated implementation以外から参照しない。

header declarationとraw/host変換は`compiler/src/c_emit/types/host/`が所有する。既存の
`product.rs`、`sum.rs`をconstruction、observation、terminal loweringの責務で組み替え、必要なら同directory内で分割する。
raw lifetimeのcopy/destroyは`compiler/src/c_emit/types/lifetime.rs`に残す。generic value graph、frame、owner listは導入しない。

この段階のtestは少なくともalias、anonymous structural type、nested product/sum、同型memberの重複、`Ptr`、external opaque typeについて、
一意な宣言順、C type spelling、rawとの往復を検査する。

## 4. Generated wrapperとcall completionを実装する

`compiler/src/c_emit/syntax/preprocessor/mod.rs`と`render.rs`へ、body macroが複数の宣言を安全に生成できるstructured nodeを追加する。
`compiler/src/c_emit/syntax/preprocessor/tests.rs`でindent、改行、parameterなし、aggregate parameterを直接検査する。

`compiler/src/c_emit/header/mod.rs`の`MAL_DEFINE_<operation>`は概念上、次を一度に生成する。

1. raw result carrierを返す`static` host bodyのforward declaration
2. compiler signatureを持つ`mal_ext_<operation>` wrapper definition
3. host body signatureを持ち、利用者の`{ ... }`が続く`static` function header

wrapperはstack上に`mal_call_t`を作る。初期実装のcall stateは`MalContext *`だけとし、動的cleanup stateを追加しない。raw argumentを
host valueへ変換して直ちにbodyを呼び、bodyが返したraw resultをcompilerへ返す。top-level source productもhost bodyでは一つの
`mal_<Alias>_t` parameterに戻す。

各result型にterminal `return` operationを生成する。productは全field、sumはactive payload、Symbolは全leafを再帰的にlowerする。
Boolは`mal_false`または`mal_true`だけ、nested sumは既知tagだけを受理し、それ以外は`mal_call_trap`する。top-level sumには
variant-specific `return_<variant>`を生成する。`Unit`のprivate bodyはinternal Unit carrierを返し、raw `void` wrapperがその式を
評価して破棄する。

terminal helperがraw resultを完成させてからbodyを終了する形だけを生成し、helper後の処理やstack spanへの参照を許すcarrierを
公開しない。trapとallocation failureを含む既存のnon-returning contractを維持する。

## 5. Symbolをlazy host valueへ切り替える

`mal_Symbol_t`は少なくとも「Mal由来のraw designation」と「host bytes由来のspan」を区別できるprivate complete representationにする。
field名はdetail prefixを持たせ、host contractにしない。実装前に`sizeof`、alignment、aggregate引数に含めたClang ABIと`-O2` codeを
小さなprobeで確認し、結果を性能測定履歴へ残す。

次を`compiler/src/c_emit/types/host/`と`compiler/src/c_emit/runtime/core/symbol/`の既存責務に沿って実装する。

- `mal_Symbol_from_bytes(mal_span_t)`はdescriptorだけを作り、allocationもcopyもしない。lengthが非zeroならdataはnonnullという
  preconditionは、call authorityがあるterminal loweringで検査する。
- `mal_Symbol_to_bytes(call, symbol)`はbytes由来ならspanをそのまま返し、Mal由来なら既存の
  `mal_symbol_materialize`を必要時だけ呼ぶ。
- terminal loweringはMal由来ならraw ownership shareをfieldごとに一つ作り、bytes由来ならallocation一回とcopy一回でraw Symbolを作る。
- 同じhost Symbolを複数fieldへ返す場合もhostにclone operationを要求しない。

この変換をwrapperが受け持てる時点で、`compiler/src/c_emit/body/expression/mod.rs`のextern call直前にある再帰的な
`materialize_symbols`を除く。内部のMal primitiveや別のboundaryが必要とするmaterializationまで一括削除しない。

focused testはflat Symbol、rope Symbol、bytes未観測、`to_bytes`観測、Mal由来passthrough、同一Symbolの複数field返却、stack上の
bytes返却、zero-length/null、nonzero-length/null rejection、allocation failureを含める。counterで次を確認する。

- bytes未観測のropeはmaterialization zero
- Mal由来passthroughはbyte copy zero、追加share一回以内
- host bytes返却はallocation一回、byte copy一回
- 完了またはtrap後のlive allocation zero

## 6. 残るhost value operationを揃える

`compiler/src/c_emit/types/host/product.rs`と`sum.rs`で、productは通常のC struct、nested sumは
`mal_<T>_make_<variant>`、tagは`mal_<T>_tag_<variant>`として生成する。Mal由来sumのobserverはvalid tagを前提にできるが、generated
stub/exampleの`switch`はdefaultで`mal_call_trap`する。

external opaque type helperを`mal_<T>_to_bits` / `mal_<T>_from_bits`へ統一し、どちらもcall authorityを取らない。
`Ptr`はraw wrapperで`MalType_Ptr.address`と`void *`をwrap/unwrapし、public conversion operationを生成しない。pointerのbounds、
alignment、permission、nullability、referent lifetimeは個別Extern contractへ残す。

旧`MAL_TYPE`、`MAL_OPERATION`、`MAL_TAG`、`MAL_EXTERN`、`MAL_CLONE`、`MAL_MOVE`、`MAL_DROP`とSymbol admission carrierへの
host依存をtestとexampleから除く。raw runtimeで同等処理が必要ならdetail implementationとして残し、公開helperとして再輸出しない。

## 7. Public ABIを一括で切り替える

前段のfocused testが通ったcommitで、次を同時に行う。

1. `compiler/src/c_emit/header/prefix.rs`を新しいtypedef、call、span、macro/detail layeringへ変更し、
   `MAL_C_ABI_VERSION`を`0x000600u`へ上げる。
2. `compiler/src/c_emit/header/mod.rs`のdeclaration、definition macro、`emit-host` stubを新surfaceへ切り替える。
3. `compiler/tests/c_emit/host_interface.rs`、`compiler/tests/driver/artifacts.rs`と関連する`memory.rs`、`numeric.rs`、`symbol.rs`、
   `calls.rs`、`performance.rs`、`semantics.rs`を新contractへ更新する。
4. 全`examples/*/host.c`を新surfaceへ書き換え、compilerから全`program.mal.h`を再生成する。
5. `docs/spec/c-host-abi.md`をv0.6の規範へ更新し、`docs/development/compiler-usage.md`、
   `docs/implementation/ownership.md`、`docs/development/conformance.md`から退役したpublic helperを除く。
6. 採択理由を新しいdecision recordとして追加し、historicalな`D034.md`は書き換えず後継をlinkする。
7. 再設計案と本書のstatusをimplementedへ変更する。

checked-in headerは`compiler/tests/driver/artifacts.rs`の`checked_in_example_headers_match_the_compiler`で生成器と一致させる。
旧public identifierを許すcompatibility testは残さず、generated detail以外から旧名が消えたことを`rg`で確認する。

## Acceptance matrix

最低限、次の一行ごとにfocused testまたはnative C testを対応させる。

| Case | 確認する結果 |
|---|---|
| `Unit`、全scalar、alias | typed bodyとtyped terminal returnがcompile、link、executeする |
| top-level product | bodyでは一parameter、raw wrapperでは既存flatteningになる |
| nested product/sum | fieldごとの再帰変換とactive payloadだけのaccessになる |
| duplicate member type | declaration、field変換、cleanupが欠落または重複しない |
| invalid Bool/tag | payloadを読む前にtrapする |
| Symbol flat/rope/no observation | bytesが同じで、未観測ropeをmaterializeしない |
| Symbol passthrough/duplication | copy countとshare countが上限内になる |
| Symbol from stack bytes | terminal return中に一度だけcopyし、return後にstackを参照しない |
| invalid span | nonzero-length/nullをtrapする |
| external opaque type | `to_bits` / `from_bits`がlosslessでauthorityを要求しない |
| `Ptr` | `void *`がround-tripし、referent lifetimeを変更しない |
| trap/allocation failure | cleanup後のlive allocationがzeroになる |
| generated stub | 新surfaceだけでwarningなしにcompileする |

## 最終検証

repository rootの`nix develop`内で次をすべて通す。

```nu
cargo fmt --manifest-path compiler/Cargo.toml --check
cargo clippy --manifest-path compiler/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path compiler/Cargo.toml
```

加えて、全exampleのheader一致、host adapterのClang compile/link/execute、ASan/UBSan pressure caseを実行する。baselineと同条件で
`-O2`を測定し、scalar-only externにallocation、retain、release、materializationが増えていないこと、wrapper overhead、binary size、
Symbolの各counterを記録する。optional direct Symbol outputは基本経路のcopyが独立したcost centerだとこの測定で示された場合だけ、
別の設計変更として検討する。

完了判定はtest成功だけではなく、公開headerとexampleに新旧surfaceの混在がないこと、specが実装と一致すること、生成detailを
host authorが理解しなくても全exampleを書けることまで含む。
