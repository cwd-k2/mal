# `pack`とpush producerによる`Packed`構築

Status: Draft proposal; non-normative

この文書は、要素数が事前に分からない計算からmal-ownedな`Packed<T>`を直接構築する案を検討する。現在の言語規則は
[`Region`と`Packed`](../spec/packed.md)、canonical representationは[external memory](../spec/memory.md)、managed ownerの
実装規約は[managed value ownership](../implementation/ownership.md)、機能を置く境界は[authority](../design/authority.md)を正とする。
既存Packedの一意なownerを再利用する案は[scoped capabilityによる`Packed`編集](packed-editing.md)で別に扱う。

## 提案する最小surface

このproposal群が追加するsource-level surfaceは、通常のapplicationで使うpredefined generic intrinsic `pack`と`edit`だけとする。
producerは説明上の用語であり、必要なprogramは`(T -> Unit) -> Unit`へ独自のtransparent aliasを付けられる。

repeat、generate、filter、map、concatなどの構築policyは`pack`へ渡す通常のMal functionとして書く。既存Packedの反復更新は`edit`、
Region admissionとstoreの一時Packed除去はcompiler optimizationとして導く。syscall、allocator、I/O、resource、threadは
Extern authorityを持つprogram固有の`extern` contractに置く。

## 目的

現在もexternal allocatorから得たstorageを`Region<T>`として初期化し、`<-region`で`Packed<T>`へadmitできる。しかしこの経路は
external allocationのfree contractを必要とし、完成時に別のmal-owned storageへcopyする。構築中storageを最初からMalのownerにすれば、
runtimeがcapacityを増やし、完成時に同じownerをimmutableな`Packed<T>`へ移せる。

emit capabilityは構築中storageについて次の性質を一つの境界に閉じる。

- builderがgrowth後の現在Addressを保持する。
- sourceへは要素単位のwrite capabilityだけを渡す。
- emit回数と順序が初期化済みprefixを定める。
- builderが構築中storageのownershipとlifetimeを保持する。

## interface

```mal
pack<T> :: ((T -> Unit) -> Unit) -> Packed<T>;
```

使用例は通常のlambdaとapplicationだけで書く。

```mal
packedInt32 := pack<Int32>((emit) -> {
    emit(10i32);
    emit(20i32);
    emit(30i32);
});
```

`emit`はfirst-class functionなので、producerは通常の関数分割、capture、分岐、再帰を利用できる。

```mal
emitRange :: ((USize -> Unit), USize, USize) -> Unit :=
    (emit, current, end) ->
        if (current == end)
        then ()
        else {
            emit(current);
            emitRange(emit, current + 1usize, end);
        };

values := pack<USize>((emit) -> emitRange(emit, 0usize, 1000usize));
```

このinterfaceはproducerがcontrolを保持してconsumer callbackへ値を渡すpush producerである。

## 暫定意味論

`pack<T>(producer)`は次の順序で評価する。

1. `producer`を一度評価する。
2. countが0で、Malがlifetime authorityを持つ空のbuilderを作る。
3. builderへ一要素を追加する`emit : T -> Unit`をproducerへ渡す。
4. 各`emit(value)`はvalueを一度評価し、必要ならcapacityを増やし、現在の末尾へcanonical representationをstoreしてからcountを増やす。
5. producerが`Unit`で正常完了するとbuilderをfreezeし、count要素のowned `Packed<T>`を返す。

要素順は`emit` applicationの完了順である。Packed resultはproducerの正常完了時にだけ成立する。growth factor、initial capacity、
余剰capacityの保持または縮小は、成功時に同じ値を作るimplementation detailとする。

count、`count * stride(T)`、owner allocation sizeがtargetで表現できない場合と、実際に必要となったallocationのfailureはtrapとする。
allocationを行う回数と時点はgrowth policyおよびowner再利用に従うimplementation detailである。
`Packed<Unit>`はemit回数をcount-only representationで保持できる。

## 型の境界

現行仕様では`Packed<T>`は`Representable(T)`の場合にwell-formedである。result型のwell-formednessから`pack`にも同じ制約を得る。
producerとemitはMal内部で評価する。

`Representable(T)`によりelementはmanaged ownerを含まない。この性質から、builderのgrowthは初期化済みbytesの移動、完成時の破棄は
flat byte ownerの解放になる。`Address`を含むTでは既存Packedと同じくPacked storageだけを所有する。

## authority、capability、lifetime

builder stateはowner、count、capacityを持つimplementation-only capabilityである。builderはproducerの実行中にowner responsibilityを
保持し、growth後のownerを更新する。正常完了時はownerをPacked resultへconsume-transferする。完成後のsliceは通常のPacked ownership規則で
ownerをshareする。

allocation identity、構築中storageの変更、freeze、完成後の回収はMal側が決める。emitはMalがauthorityを持つEngram constructionの
一部である。sourceからは要素列とcountだけを観測できる。

emitは通常のfunction valueとしてhelper、closure、recursive frameへ渡せる。producer resultの`Unit`、Mal内部に留まるfunction value、
immutable capture、result binderのcapture規則により、emitを保持するrootはproducerの正常完了までに終了する。したがって現在のprofileでは
producerの正常完了がcapability scopeの終端になる。mutable cellまたはhostへ渡せるfunction carrierを将来導入するときは、scoped emitter
typeまたはsealed stateをscope authorityとして再検討する。

## loweringの候補

`pack`をpredefined compiler intrinsicとし、resolve後のbinding identityで認識する。通常のnameとapplication規則を保ったまま、
backendはstableなbuilder stateを参照するemitter entryを生成し、runtimeのreserve operation後にdata pointerを再取得して
`T`のcanonical storeを行う。Packedのexact accessと同様、baselineはalignment 1のstoreを使う。

first-classな意味を保ったまま、既知のlambdaではcallback inline、emitter closure elimination、builder stateのstack配置、tail-recursive
producerの直接遷移をoptional optimizationにできる。baselineはmanaged closureとowner handoffで同じ結果を作る。

predefined環境への導入、重複、shadowは既存のpredefined valueと同じname resolution規則に従う。

## 先行例

Goの`iter.Seq[V]`は`func(yield func(V) bool)`というpush iteratorであり、`slices.Collect`はその値を新しいsliceへ収集する。
yield resultはconsumerによるearly stopを表す。Rust標準libraryはpull型`Iterator`と`collect::<Vec<_>>()`を中心にし、collection側が
preallocationまたはamortized growthを選ぶ。Rustの`Vec::spare_capacity_mut`とunsafeな`set_len`はRegion案に近く、初期化済み範囲の証明を
callerへ要求する。

- [Go `iter` package](https://pkg.go.dev/iter)
- [Go `slices.Collect`](https://pkg.go.dev/slices#Collect)
- [Rust `Iterator::collect`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect)
- [Rust `Vec::set_len`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.set_len)

## 検証境界

採択する場合は、少なくともempty、複数回growth、Unit element、product/sum element、再帰producer、helperへ渡したemit、allocation failure、
countとbyte sizeのoverflow、normal return後のlive allocation count 0をfocused testで固定する。managed elementの型検査、評価順、生成したPackedの
slice lifetimeをchecker、lowering、native artifactの各境界で検証する。
