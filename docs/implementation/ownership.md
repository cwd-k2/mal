# managed value ownership

Status: Current v0.6 implementation policy

この文書はLLVM execution backendとC host boundaryにおけるmanaged valueのlifetimeを定める。source-level lifetime authorityは
[Engram specification](../spec/engrams.md)、host carrierのcontractは[C host ABI](../spec/c-host-abi.md)を正とする。

## managed type

`Symbol`、`Buffer`、function closureはownerを持つ。productとsumはmanaged memberを再帰的に含む場合にmanagedである。数値scalar、
`Unit`、`Address`、`ByteSize`、`USize`、external opaque valueはownerを持たない。この分類は
`execution::ownership`が一箇所で提供する。

LLVM内の`Symbol`はowner pointer、active data address、byte countからなるviewであり、`Buffer<A>`はmanaged runtime objectへの
pointerである。`Symbol`と`Buffer<UInt8>`の変換は独立したsemantic valueを作るsnapshot copyである。closureはcode pointerとnullable environment pointerの組である。productは各field、sumはactive payloadだけについて同じ規則を
再帰的に適用する。literalのstatic byte ownerとnull environmentに対するretain/releaseは安全なno-opである。
完成したbyte ownerのdataはownerのlifetime中不変である。view構築時にdataを確定し、index、slice、比較はowner representationを再解釈しない。
storage再利用を判断するSymbol concatだけがownerに対するdataのoffsetを導出する。

## slotとoperation

managed local slotはzero状態で初期化する。owner successorと終了点は
[`D055`](../history/decisions/D055.md)に従い`execution::ownership`が`Borrow`、`Share`、`Consume`、`Drop`として決める。
LLVM backendはこれをretain、source carrierのzero、releaseとtyped storeへ変換し、last-useやcall modeを再推論しない。
memory primitiveの複数operandは通常のproduct構築ではない。execution ownershipは論理operandごとにeffectを決め、indexや
lengthのobservationへ引数伝達だけのaggregate responsibilityを作らない。snapshot conversionやC host copyはresultとoperandの
ownerを共有しない。

一つのtransactionではoperandを先に読み、必要な`Share`を完了し、`Consume`するsource carrierをzeroにした後に、
後継のないresponsibilityを`Drop`して格納をcommitする。owned resultを受け取るwildcardと
使われないpattern leafは保存せず直接`Drop`する。borrowed valueをそのようなplaceに渡す場合は何もしない。
`Atom(binding)`からpatternへ取り出したmanaged leafは、元のbindingがleafの全control lifetimeを包含する場合、ownerではないlocal
aliasとして保存する。aliasのlivenessはlenderをliveに保ち、frameは両carrierを運んでもaliasをretainまたはreleaseしない。
aliasが生きたproduct、sum、closure environment、returnなどのowner successorへescapeするときだけ`Share`する。owner successorを
持たずdiscardされる純粋なproductとsumは、`Atom`とlocal `Jump`によるadministrative handoffを越えて構成要素をborrowする。
closure生成はenvironment allocationと独立lifetimeを持つため、このpure aggregate規則の対象にしない。詳細は
[`D057`](../history/decisions/D057.md)を正とする。
control bindingはactivation内で一つのresponsibilityだけを初期化する。self-tailで同じcarrierを再利用する場合も、旧responsibilityは
引数への`Consume`またはedgeの`Drop`でentryへ戻る前に終了する。したがってpattern destinationはvacant carrierへの`Initialize`であり、
backendは格納時に旧値の存在を推測してreleaseしない。

control CFGのbackward livenessでbinding後にdeadとなるlocal ownerは、operation resultを保存してborrowを終えた直後にreleaseしてslotを
zeroにする。これはowner responsibilityの終了であり、optimization設定によらない。`backend/llvm/optimization/symbol_concat`が有効で、
`Symbol` concatのoperandがそのsiteでdeadなら、そのshareをreleaseの代わりにruntimeへmoveできる。runtimeはviewがowner全体を覆い、
reference countが1であるflat storageだけを再利用する。
techniqueが無効ならborrowするconcat後に通常どおりreleaseする。両operandが同じbindingならmoveせず、後続pathにuseがあるownerをreference
countから推測して消費しない。

function returnではresultをowned handoffし、後継のないactivation-local responsibilityをreleaseする。tail transitionでも
次argumentと次environmentの`Share`を完了し、`Consume`するsourceを失効させてから現在のlocalとenvironmentをreleaseする。

## parameter handoff

`execution/parameter`はfunction parameterの行先を`Bind(slot)`または`Discard`として決め、`execution::ownership`はapplication target、
call mode、recursive regionとparameterが作るmanaged responsibilityからhandoffを計画する。native ABI callのcallerがcall完了まで
authorityを保持し、self-tailまたはregion内遷移も外側のinvocationが同じauthorityを保持できる場合、parameter bindingはownerを
複製せずborrowする。pureな`Atom`、product、sumによるargument構成graphも同じcall boundaryまでborrowできる。分解aliasの
provenanceは記述順で即決せず、parameter、case payload、pure constructionから得たauthorityを収集してから依存関係を解く。

recursive controlがfresh managed valueを作る、managed resultを返す、managed call resultを受け取る、またはregion外targetを含む
dispatchへ同じargumentを渡す場合は外側のauthorityだけで全pathを包含できない。そのregionのparameterは従来どおり、native ABI
entryで`Share`し、owned handoffで`Consume`または`Drop`する。borrowed parameterからclosure capture、return、その他の独立ownerへ
escapeするuseも`Share`する。edge dropは通常livenessを再計算せずborrow provenanceで閉じたlivenessを使い、aliasが最後に使われる
edgeでlender responsibilityを終了する。詳細は[`D058`](../history/decisions/D058.md)を正とする。

LLVM backendはentryの由来、call target、parameterのlivenessを再推論しない。

## closure environment

capturing closureの生成時にtarget固有のenvironment storageをruntimeから確保し、capture fieldごとにownership shareを保存する。
environment headerはreference countとtarget固有destructorを持つ。最後のclosure shareをreleaseするとdestructorがmanaged captureを再帰的に
releaseし、environment storageを解放する。

capture-free closureはnull environmentを使う。self closureは実行中のactive environmentをborrowし、escapeする保存先でretainする。

## control frame

recursive regionのnon-tail callではresume live-inのmanaged fieldと次activationのargumentを、ownership planの`Share`または`Consume`に従って
frameとparameter handoffへ配布する。同じsourceに複数のowner successorがある場合は先行するsuccessorを`Share`し、最後の一つだけを
`Consume`する。共通regionでresume後もenvironmentが必要ならcaller environment ownerをframeへ渡す。
borrowed local aliasのframe fieldはcarrierだけを保存し、同じresume lifetimeを包含するlender fieldがresponsibilityを保持する。

return時はcallee localとactive environmentをreleaseし、frame fieldとcaller environmentをresume activationへ移す。terminal returnでは
root result以外のlocal、active environment、control storageを解放する。tail edgeはframe shareを作らない。

## host boundary

extern parameterはcall中だけborrowされる。hostが保持する場合はpublic helperのcontractに従ってcopyする。managed resultはinternal
pointer/out-pointer bridgeがMal ownerへ変換する。`Symbol` resultはruntime ownership pointerとして受け入れ、aggregateとsumはactive fieldだけを
再帰的に変換する。invalid Boolまたはsum tagはpayloadを読む前にtrapする。

C shimが渡すargument pointer列とargument bytesはborrowed external storageであり、`main`のreturnまでだけ有効である。
`from<UInt8>`がcopyを完了した時点でruntime-owned Bufferとなる。`Symbol`への変換は別のownerへsnapshotする。

## 検証

- managed slot、aggregate、sum、closure capture、frame、extern bridgeの各境界でretain/releaseの対応を実行testで確認する。
- nested aggregateの分解と再構成で同じauthorityが保たれ、中間carrierがownerを複製しないことを確認する。
- deep self recursionとfirst-class cycleでowner数がdepthに比例して残らないことを確認する。
- activeでないsum payloadへretain、release、readを行わない。
- allocation counterを使うfixtureはnormal return後にlive allocationがないことを確認する。
