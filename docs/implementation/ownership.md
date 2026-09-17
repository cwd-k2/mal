# managed value ownership

Status: Current v0.5 implementation policy

この文書はLLVM execution backendとC host boundaryにおけるmanaged valueのlifetimeを定める。source-level lifetime authorityは
[Engram specification](../spec/engrams.md)、host carrierのcontractは[C host ABI](../spec/c-host-abi.md)を正とする。

## managed type

`Symbol`とfunction closureはownerを持つ。productとsumはmanaged memberを再帰的に含む場合にmanagedである。数値scalar、`Unit`、`Ptr`、
external opaque valueはownerを持たない。この分類は`execution::ownership`が一箇所で提供する。

LLVM内の`Symbol`はruntime allocationへのpointer、closureはcode pointerとnullable environment pointerの組である。productは各field、sumは
active payloadだけについて同じ規則を再帰的に適用する。literalのstatic `Symbol`とnull environmentに対するretain/releaseは安全な
no-opである。

## slotとoperation

managed local slotはzero状態で初期化する。borrowed atomをslot、aggregate、return、frame、または次のactivationへ保存するときは先にretainし、
slotの旧値をreleaseしてから新しいshareを格納する。operationが新しいownerを返す場合はそのshareを直接移せる。wildcardがowned resultを
捨てる場合は直ちにreleaseする。

control CFGのbackward livenessでbinding後にdeadとなるlocal ownerは、operation resultを保存してborrowを終えた直後にreleaseしてslotを
zeroにする。これはowner responsibilityの終了であり、optimization設定によらない。`backend/llvm/optimization/symbol_concat`が有効で、
`Symbol` concatのoperandがそのsiteでdeadなら、そのshareをreleaseの代わりにruntimeへmoveできる。runtimeは一意なflat storageを再利用する。
techniqueが無効ならborrowするconcat後に通常どおりreleaseする。両operandが同じbindingならmoveせず、後続pathにuseがあるownerをreference
countから推測して消費しない。

function returnではresult shareを確保してからactivation-local slotをreleaseする。tail transitionでも次argumentと次environmentを先に
確保し、その後に現在のlocalとenvironmentをreleaseする。aliasを早く解放しないため、この順序を変えてはならない。

## parameter handoff

`execution/parameter`はfunction parameterの行先を`Bind(slot)`または`Discard`として一度だけ決める。LLVM backendはこのdestinationを受け、
parameter patternのbinding有無を再解釈しない。

region外のnative callではcallerがargument ownerをcallのreturnまで保持する。calleeの`Bind` prologueはmanaged argumentをretainしてlocal
slotへ保存し、`Discard`は新しいshareを作らない。region内遷移と`DirectSelfTail`では、次のactivation用argument shareをcaller cleanupより
先に確保する。`Bind`はそのshareをslotへ移し、`Discard`は一度releaseする。

## closure environment

capturing closureの生成時にtarget固有のenvironment storageをruntimeから確保し、capture fieldごとにownership shareを保存する。
environment headerはreference countとtarget固有destructorを持つ。最後のclosure shareをreleaseするとdestructorがmanaged captureを再帰的に
releaseし、environment storageを解放する。

capture-free closureはnull environmentを使う。self closureは実行中のactive environmentをborrowし、escapeする保存先でretainする。

## control frame

recursive regionのnon-tail callではresume live-inのmanaged fieldをretainしてframeへ保存する。共通regionでresume後もenvironmentが必要なら
caller environment ownerをframeへ移す。calleeへ渡すargumentとenvironmentを確保してからcaller localをcleanupする。

return時はcallee localとactive environmentをreleaseし、frame fieldとcaller environmentをresume activationへ移す。terminal returnでは
root result以外のlocal、active environment、control storageを解放する。tail edgeはframe shareを作らない。

## host boundary

extern parameterはcall中だけborrowされる。hostが保持する場合はpublic helperのcontractに従ってcopyする。managed resultはinternal
pointer/out-pointer bridgeがMal ownerへ変換する。`Symbol` resultはruntime ownership pointerとして受け入れ、aggregateとsumはactive fieldだけを
再帰的に変換する。invalid Boolまたはsum tagはpayloadを読む前にtrapする。

C shimがprocess argumentから作るdescriptorとargument bytesはborrowed external storageであり、`main`のreturnまでだけ有効である。
`Symbol.read`を呼んだ時点でruntime-owned bytesへcopyする。

## 検証

- managed slot、aggregate、sum、closure capture、frame、extern bridgeの各境界でretain/releaseの対応を実行testで確認する。
- deep self recursionとfirst-class cycleでowner数がdepthに比例して残らないことを確認する。
- activeでないsum payloadへretain、release、readを行わない。
- allocation counterを使うfixtureはnormal return後にlive allocationがないことを確認する。
