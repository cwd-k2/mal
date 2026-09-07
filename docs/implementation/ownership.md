# C backendのEngram ownership

Status: Current implementation contract

この文書はreference C backendがmanaged Engramの保持と解放を生成する規約を定める。source-levelの意味と
authorityは[Engram仕様](../spec/engrams.md)、hostとの受け渡しは[C host ABI](../spec/c-host-abi.md)を正とする。

## 現在の状態

v0.5が現在受理するprogramとtrusted C adapter contractの範囲では、ownership correctnessに必要なcopy、transfer、
cleanupは実装済みである。ownershipに関する残件は、現行contractを変えないretain/release除去、last-use move、
escape analysis、region化などの最適化であり、正しさを成立させるための未実装要件ではない。

将来、managed cycle、thread間共有、host resourceの自動解放などを言語またはABIへ追加する場合は、その新しい範囲に
対するownership設計を別途行う。これは現在のv0.5 ownership実装の未完成部分ではない。

## 対象

`Symbol`とcaptureを持つfunction valueはruntime storageを参照する。productとsumはfieldを再帰的に調べ、これらを
含む場合だけmanaged valueとして扱う。`Unit`、numeric scalar、`Ptr`、external opaque value、およびcapture-free
function valueは個別に解放するstorageを持たない。

Extern resourceのownershipはこの仕組みに含めない。`Ptr`のreferentやexternal opaque handleをcloseまたはfreeする
責務はoperation固有のhost contractに属する。

## generated Cの規約

値の受け渡しは次の二つへ統一する。

- function parameter、capture fieldの読取り、既存bindingへの参照、case payloadはborrowである。
- function result、runtime operation result、bindingが保持するmanaged valueはownである。

borrowを別のbinding、aggregate field、capture、branch result、function resultへ保存するときは、型ごとのcopy operationで
ownership shareを一つ増やす。binding、branch-local pattern、closure environment、top-level storageの終端では、生成と逆順に
型ごとのdestroy operationを呼ぶ。wildcardへ渡したowned resultも直ちにdestroyする。

expression emitterは各`Operation`のC式と`ResultOwnership`を同じinterfaceで返す。分類は`Operation`、
`MemoryPrimitive`、managed resultを作り得るprimitiveをwildcardなしで列挙し、新しいvariantの分類漏れをRustの
exhaustiveness checkで拒否する。structured operationもstatement emitterでowned resultを作る規約を明示する。

現在のemitterはlast-use moveを解析せず、保存時に保守的なcopyを生成する。したがって正しさは変数の最終使用位置に依存しない。
型付きclosure-converted IRのlexical blockとpatternからcleanupを生成でき、lexerやparserへlifetime解析を追加しない。

## 型ごとのoperation

| 型 | copy | destroy |
|---|---|---|
| `Symbol` | ownership pointerをretain | ownership pointerをreleaseし、最後ならbytesを解放 |
| captureを持つfunction | environmentをretain | environmentをreleaseし、最後ならcaptureを逆順にdestroyして解放 |
| product | managed fieldをsource orderでcopy | managed fieldを逆順にdestroy |
| sum | active payloadだけをcopy | active payloadだけをdestroy |
| その他 | C value copy | no-op |

`Symbol` literalはstatic storageを参照しownership pointerを持たない。runtime生成Symbolはallocation headerのreference countを
共有する。空文字との連結が既存descriptorを返す場合も、result contractを満たすためretainする。

closure valueはcode pointer、environment pointer、environment destructorの組である。destructorはcapture型を知る生成function
であり、generic reference-count runtimeはenvironment layoutを解釈しない。

## programとhost境界

top-level initializerの一時値は各initializerの終了時にdestroyし、保存したtop-level値は`main`のreturn後に逆順でdestroyする。
argument descriptor列のruntime allocationもsource-level `main`のreturn後に解放する。

extern parameterはcall中だけborrowされる。managed resultの各fieldはownership shareを一つmalへtransferしなければならない。
Symbol resultは`mal_Symbol_copy_from_bytes`で作る。malはextern resultをowned valueとして受け取り、通常のbinding cleanupへ接続する。
C adapter内のclone、move、dropとaggregate constructor/accessorの規約は[C host ABI](../spec/c-host-abi.md)を正とする。

generated C自身は`MAL_CLONE`、`MAL_MOVE`、`MAL_DROP`を内部ownership primitiveとして使わない。compilerは
typed IR上のborrow/ownを静的に知り、hostへ公開されないanonymous aggregateとclosureも含めて内部copy/destroyへ
直接loweringする。ABI macroはその静的情報を持たない手書きadapterへ同じ意味契約を提供する境界APIである。

## 最適化との境界

immutabilityによりcopyはreferentの複製ではなくretainでよく、cleanup順序によって値の内容は変わらない。将来はlast-use move、
escape analysis、region化でretain/releaseを除去できるが、この文書のborrow/result contractを変えずに行う。

rope、slice、hash cache、operation memoizationは値表現または計算量の最適化であり、ownershipの正しさとは分離する。導入する場合も
各nodeやcache entryが同じcopy/destroy contractへ従う。descriptor addressの同一性はsourceから観測できず、再利用可能性もあるため、
memoization keyのsource-level意味には使わない。
