# managed value ownership

Status: Current v0.6 implementation policy

この文書はLLVM execution backendとC host boundaryにおけるmanaged valueのlifetimeを定める。source-level lifetime authorityは
[Engram specification](../spec/engrams.md)、host carrierのcontractは[C host ABI](../spec/c-host-abi.md)を正とする。

## managed type

`Symbol`、`Packed`、function closureはownerを持つ。productとsumはmanaged memberを再帰的に含む場合にmanagedである。数値scalar、
`Unit`、`Address`、`ByteSize`、`USize`、`Cursor`、`Region`、external opaque valueはownerを持たない。この分類は
`execution::ownership`が一箇所で提供する。

LLVM内の`Symbol`と`Packed<A>`はowner pointer、byte offset、element countからなるviewである。`Symbol`と`Packed<UInt8>`の変換は
共通のflat byte ownerをretainしてviewを組み替え、allocationもbyte copyも行わない。sliceは同じownerをretainしてoffsetとcountを
変える。closureはcode pointerとnullable environment pointerの組である。productは各field、sumはactive payloadだけについて同じ規則を
再帰的に適用する。literalのstatic byte ownerとnull environmentに対するretain/releaseは安全なno-opである。

## slotとoperation

managed local slotはzero状態で初期化する。owner successorと終了点は
[`D055`](../history/decisions/D055.md)に従い`execution::ownership`が`Borrow`、`Share`、`Consume`、`Drop`として決める。
LLVM backendはこれをretain、source carrierのzero、releaseとtyped storeへ変換し、last-useやcall modeを再推論しない。

一つのtransactionではoperandを先に読み、必要な`Share`を完了し、`Consume`するsource carrierをzeroにした後に、
後継のないresponsibilityとdestinationの旧値を`Drop`して格納をcommitする。owned resultを受け取るwildcardと
使われないpattern leafは保存せず直接`Drop`する。borrowed valueをそのようなplaceに渡す場合は何もしない。

control CFGのbackward livenessでbinding後にdeadとなるlocal ownerは、operation resultを保存してborrowを終えた直後にreleaseしてslotを
zeroにする。これはowner responsibilityの終了であり、optimization設定によらない。`backend/llvm/optimization/symbol_concat`が有効で、
`Symbol` concatのoperandがそのsiteでdeadなら、そのshareをreleaseの代わりにruntimeへmoveできる。runtimeはviewがowner全体を覆い、
reference countが1であるflat storageだけを再利用する。
techniqueが無効ならborrowするconcat後に通常どおりreleaseする。両operandが同じbindingならmoveせず、後続pathにuseがあるownerをreference
countから推測して消費しない。

function returnではresultをowned handoffし、後継のないactivation-local responsibilityをreleaseする。tail transitionでも
次argumentと次environmentの`Share`を完了し、`Consume`するsourceを失効させてから現在のlocalとenvironmentをreleaseする。

## parameter handoff

`execution/parameter`はfunction parameterの行先を`Bind(slot)`または`Discard`として決め、`execution::ownership`はその行先と
entryの由来からhandoffを計画する。region外のnative ABI entryはborrowedであるため、liveな`Bind`にだけ`Share`する。region内遷移と
`DirectSelfTail`はowned handoffであり、liveな`Bind`へ`Consume`、使われない`Bind`または`Discard`へ`Drop`する。
LLVM backendはentryの由来やparameterのlivenessを再推論しない。

## closure environment

capturing closureの生成時にtarget固有のenvironment storageをruntimeから確保し、capture fieldごとにownership shareを保存する。
environment headerはreference countとtarget固有destructorを持つ。最後のclosure shareをreleaseするとdestructorがmanaged captureを再帰的に
releaseし、environment storageを解放する。

capture-free closureはnull environmentを使う。self closureは実行中のactive environmentをborrowし、escapeする保存先でretainする。

## control frame

recursive regionのnon-tail callではresume live-inのmanaged fieldと次activationのargumentを、ownership planの`Share`または`Consume`に従って
frameとparameter handoffへ配布する。同じsourceに複数のowner successorがある場合は先行するsuccessorを`Share`し、最後の一つだけを
`Consume`する。共通regionでresume後もenvironmentが必要ならcaller environment ownerをframeへ渡す。

return時はcallee localとactive environmentをreleaseし、frame fieldとcaller environmentをresume activationへ移す。terminal returnでは
root result以外のlocal、active environment、control storageを解放する。tail edgeはframe shareを作らない。

## host boundary

extern parameterはcall中だけborrowされる。hostが保持する場合はpublic helperのcontractに従ってcopyする。managed resultはinternal
pointer/out-pointer bridgeがMal ownerへ変換する。`Symbol` resultはruntime ownership pointerとして受け入れ、aggregateとsumはactive fieldだけを
再帰的に変換する。invalid Boolまたはsum tagはpayloadを読む前にtrapする。

C shimがprocess argumentから作るdescriptorとargument bytesはborrowed external storageであり、`main`のreturnまでだけ有効である。
`Region<UInt8>`を`Packed<UInt8>`へadmitした時点でruntime-owned bytesとなり、`Symbol`への変換後もownerを保つ。

## 検証

- managed slot、aggregate、sum、closure capture、frame、extern bridgeの各境界でretain/releaseの対応を実行testで確認する。
- deep self recursionとfirst-class cycleでowner数がdepthに比例して残らないことを確認する。
- activeでないsum payloadへretain、release、readを行わない。
- allocation counterを使うfixtureはnormal return後にlive allocationがないことを確認する。
