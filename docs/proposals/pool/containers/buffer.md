# pointerとIxPool上のdense sequence

Status: Exploratory support document

現行`Buffer`の規範は[`memory`](../../../spec/memory.md)にある。この文書はpointerとIxPoolの表現力を
検証するdense sequence sketchであり、predefined BufferやSymbol変換の変更を提案するものではない。

## representation

```mal
opaque Dense<A> :: (*USize, IxPool<A>);
```

invariantは次である。

- `load(count) <= capacity(values)`
- `[0, load(count))`はLive
- `[load(count), capacity(values))`はVacant

constructionはmetadata placeと要素poolを別に作る。

```mal
make<A> :: USize -> Dense<A> := requested -> {
    count := place(0usize);
    values := pool<A>();
    grow(values, requested);
    (count, values)
};
```

## 単一要素のoperation

以下はworking syntaxによる擬似codeである。

```mal
length<A> :: Dense<A> -> USize := (count, _) -> load(count);

append<A> :: (Dense<A>, A) -> Unit := ((count, values), value) -> {
    i := load(count);
    ensureCapacity(values, i + 1usize);
    store(values.at(i), value);
    store(count, i + 1usize);
};

get<A> :: (Dense<A>, USize) -> A := ((count, values), i) -> {
    // precondition: i < load(count)
    load(values.at(i))
};

put<A> :: (Dense<A>, USize, A) -> Unit := ((count, values), i, value) -> {
    // precondition: i < load(count)
    store(values.at(i), value)
};
```

`append`は要素をLiveにしてからcountを増やす。`put`は同じplaceの置換なので、incoming valueを保持してから
旧値をDropする。loadは値をShareする。

このDense APIは`values.at(i)`を公開しない。内部pointerを返すと利用者が`take`して`[0, count)`へ穴を作れ、
dense invariantを迂回できるためである。pointerを返す別APIを設計するなら、そのauthorityと有効期間を独立に定める。

末尾を削るoperationを持つなら、countを減らして公開範囲から外した後に`take`する順序を選べる。
Drop glueやcallbackの再入規則を変える場合は、このcommit順もspec化する。

## range operation

`fill`は必要な末尾までcapacityを確保し、既存Live範囲と末尾のVacant範囲へstoreした後、countを
一度だけ更新する。単一operandを複数placeへ保存するため、各要素用のresponsibilityをShareする。

`copy(source, sourceOffset, destination, destinationOffset, length)`は、同じidentityで範囲が重なる場合に
memmoveと同じ方向を選ぶ。

- destinationが後ろなら末尾から先頭へ進む。
- それ以外は先頭から末尾へ進む。
- sourceをloadし、destinationへstoreする。
- destinationのLive範囲が伸びるなら、全要素を確立してからcountを更新する。

この参照形は各要素でShare/Dropを行う。runtimeの一括primitiveは同じ最終responsibilityとoverlapの観測を
保つ限り、trivial carrierのmemmoveや型別loopへ特殊化してよい。

## 費用

素朴な表現はmetadata pointer、IxPool owner、要素accessごとのpointer carrier、occupancy bitmapを持つ。
現行dense Bufferはcountとstorageを一allocationへまとめ、occupancyを省ける。この差を測るための参照実装であり、
APIが同じだから物理表現も統一すべきだとは結論しない。
