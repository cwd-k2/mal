# scoped capabilityによる`Packed`編集

Status: Draft proposal; non-normative

この文書は、既存の`Packed<T>`から同じ長さの新しい値を構築し、source semanticsをimmutableに保ったまま一意なbacking ownerを
再利用できる編集operationを検討する。現在の言語規則は[`Region`と`Packed`](../spec/packed.md)、managed valueのhandoffは
[managed value ownership](../implementation/ownership.md)、機能を置く境界は[authority](../design/authority.md)を正とする。この案は
draft proposalである。要素数が未知の新しいPackedを構築する案は[`pack`とpush producer](packed-construction.md)で別に扱う。

## 目的

既存Packedをexternal Regionへstoreして編集し、再びadmitすれば現在の機能だけでも新しいPackedを得られる。しかしexternal storageの
allocation、free、lifetime contractと往復copyが必要になる。Packedのbacking ownerが一意で、入力値のresponsibilityをresultへ移せる
siteでは、compilerとruntimeが同じstorageを再利用できる余地がある。

get/set capabilityは編集について次の性質を一つの境界に閉じる。

- sourceとそのslice、および`T = UInt8`の場合のSymbol viewは編集前の要素列を保持する。
- callbackはindexとtyped valueだけを観測する。
- callbackの正常完了がresultをfreezeする。
- Packedの連続表現とalignmentはimplementation detailに留まる。

## interface

現在値を読む`get`と新しい値を書く`set`をfirst-class functionとしてcallbackへ渡す。

```mal
edit<T> ::
    (Packed<T>, ((USize -> T), ((USize, T) -> Unit)) -> Unit)
    -> Packed<T>;
```

```mal
updated := edit<Int32>(original, (get, set) -> {
    current := get(3usize);
    set(3usize, current + 1i32);
    set(10usize, 42i32);
});
```

`get`は構築中bufferの現在値を返すため、同じcallback内で先行する`set`を観測する。同じindexへ複数回書けば最後の値がresultに残る。
callbackが何も書かなければresultはoriginalと同じ要素列になる。長さは変化しない。

`get`と`set`の組がread-your-writesを一つの規則で定める。

`edit`は`pack`と同じく通常のapplicationで使うpredefined generic intrinsicとする。

## 暫定意味論

`edit<T>(source, callback)`は次の順序で評価する。

1. `source`と`callback`を通常のargument順で一度ずつ評価する。
2. sourceと同じ要素列とcountを観測するimplementation-only builderを用意する。最初のwriteまではsource ownerを変更しない。
3. builderだけを観測する`get`と変更する`set`をcallbackへ渡す。
4. callbackが`Unit`で正常完了するとbuilderをfreezeし、owned `Packed<T>`を返す。

sourceとresultは独立したimmutable valueとして振る舞う。sourceとそのslice、および`T = UInt8`の場合にownerを共有するSymbolは
edit前と同じ値を返す。Packed resultはcallbackの正常完了時にだけ成立する。

`get(index)`と`set(index, value)`は、既存の`Packed # USize`と同じく`index < #source`を未検査preconditionとして要求する。
防御的trapはimplementation detailである。`Packed<Unit>`ではstorageを変更せず、boundsとcountだけを扱える。

現行仕様どおり`Packed<T>`のwell-formednessから`Representable(T)`を要求する。flat byte owner上のelementを編集対象とする。

## capabilityのscope

getとsetはMal内部のfunction valueである。callback resultの`Unit`、immutable capture、result binderのcapture規則がcallbackの正常完了を
capability scopeの終端にする。callbackはcapabilityをhelper、nested closure、recursive frameへ渡せる。

この根拠は[`pack`のemit capability](packed-construction.md)と同じであり、現在のprofileではcallbackの正常完了がcapability scopeの終端になる。
mutable cellまたはhostへ渡せるfunction carrierを将来導入するときは、scoped typeまたはsealed stateをscope authorityとして再検討する。

sourceとresultの値、callback中の変更、owner再利用の可否はMal側のauthorityに閉じる。sourceは要素列とcountを観測する。
source responsibilityの後継はexecution ownership plan、物理storageの再利用はruntime representationが判断する。

## copyとowner再利用

correctness baselineは最初の`set`でsource viewの全要素を新しいownerへcopyしてから編集する。`set`が一度もなければsource ownerを
shareしたresultを返せる。owner再利用は同じobservable semanticsを持つoptional optimizationとし、次の二段階で判断する。

1. execution ownership planがsource responsibilityの唯一の後継をedit resultとしてconsume-transferする。
2. runtimeが変更可能なflat storageとreference count 1を確認する。

両方を満たす場合はsource ownerをbuilderへ移して直接編集し、finish時にresultへ移す。source responsibilityに複数の後継がある場合、
またはruntime ownerに複数のshareがある場合はcopy-on-writeを使う。

この分担により、受理可否と値の意味はすべてのoptimization profileで一致する。offsetを持つview、余剰capacity、static ownerを
再利用する条件と、no-opでownerをshareする方法はimplementation detailとし、baselineのcopy-on-writeがfallbackになる。実際に必要となった
mal-owned allocationのfailureはtrapする。

## effectとしての見方

`edit`は構築中Packedに対する二つのoperationを処理するscoped state handlerと見なせる。

```text
Get : USize -> T
Set : (USize, T) -> Unit
```

型parameterでscopeを表すならRustのexclusive borrowや`runST`型のrank-2 stateに近い。この案ではget/set capabilityを通常のfunctionとして
渡し、現在のtype systemでscopeを構成する。handlerは各operation後にcallbackの計算を通常のcall/returnで継続する。

## 検証境界

採択する場合は、no-op、単一更新、同一indexの反復更新、read-your-writes、emptyとUnit、product/sum、有効範囲の先頭と末尾をfocused testで固定する。
sourceがlive、dead、slice共有、Symbol共有、static ownerの各場合に同じ値を返すことを確認し、allocation counterではowner再利用と
normal return後のlive allocation count 0を検証する。
