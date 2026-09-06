# `extern` 境界

Status: Current v0.5 profile

## 目的

I/O、memory、allocation、filesystem、network、clock、randomness、process、thread は mal の意味論へ個別に取り込まず、すべて `extern` call の外側に置く。

```mal
extern Mem;
extern alloc :: UInt64 -> Mem;
extern print :: Symbol -> Unit;
```

call site には必ず `extern` を書く。

```mal
mem := extern alloc(128);
extern print("hello");
```

external symbol は first-class value ではない。`f := extern print;` は不正である。

## transportable type

v0.5では、extern declarationのparameter型とresult型はfunction型を直接または再帰的に含んではならない。aliasは展開して判定する。
`externTransportable`は境界を運べる型shapeだけを表し、memory safetyやresource safetyを意味しない。
各leafがadmission、observation、capability transferのどれになるかは
[EngramとExtern](engrams.md#境界のoperation)に従う。

```text
externTransportable(Unit)         = true
externTransportable(scalar)       = true
externTransportable(Symbol)       = true
externTransportable(Ptr)          = true
externTransportable(ExternalType) = true
externTransportable((T...))       = all externTransportable(T)
externTransportable([T...])       = all externTransportable(T)
externTransportable(A -> B)       = false
```

```mal
extern print :: Symbol -> Unit;
extern choose :: [Int32, Symbol] -> Int32;
```

上の二つはvalidである。次はfunction型を含むためinvalidである。

```mal
extern register :: (Int32 -> Unit) -> Unit;
extern wrapped :: [Unit, Int32 -> Int32] -> Unit;
extern makeCallback :: Unit -> (Int32 -> Int32);
```

この制約はmal内のfirst-class closureを制限しない。callback ABIとhostによるclosure保持をv0.5から除外する。決定理由は[D016](../design/decisions.md#d016-externはmal-c-abiとadapterを介する)に記録する。

## source-level semantics

external declaration は mal 側の型だけを宣言する。call の引数は通常の式と同じく左から右へ評価される。
host operationが返り、resultのEngram部分のadmissionとExtern capabilityのtransferが完了した後、宣言された型の
mal valueを得たものとして評価を続ける。

mal は effect system を持たず、通常の関数型は pure/impure を区別しない。

```mal
printValue :: Int32 -> Unit := \(x :: Int32) {
    extern printInt32(x);
    ();
};
```

この型は単に `Int32 -> Unit` である。

## host contract

型の宣言だけでは ABI、ownership、lifetime、failure を定義できない。各 backend または embedding は少なくとも次を別途定義しなければならない。

- symbol の名前解決と calling convention
- scalar、product、sum、`Symbol` の表現
- opaque value の size、alignment、copy/drop の意味
- host 側の一時byte bufferの取得方法とcopy後の解放
- host failure を trap、process termination、戻り値のどれへ写像するか
- host が保持してよい引数と、mal が保持してよい戻り値
- result capabilityをmalへtransferするcommit pointと、それ以前にhostが取得した一時resourceのcleanup

## boundary transport

v0.5のEngram observationとadmissionは次とする。

- mal から host へ渡す `Symbol` は call 中だけ borrow され、host は return 後に参照を保持しない。
- host側の一時byte bufferはSymbolではない。Symbolを返すsource-level operationは、callが完了する前にbytesをmal-controlled storageへcopyし、その時点で新しいSymbolを作る。返されたSymbolはhost側bufferを参照しない。
- 返された値の意味とlifetime authorityはmalに属し、hostはmanaged identityやrootを生成しない。
- Symbol copyのallocation sizeを表現できない場合とallocation failureはtrapする。

このcopy規則は一時host bufferから返る値を安全にする。unboundedなstreaming inputでは、program固有の`extern`が
再利用可能な`Ptr` regionへbytesを書き、mal側へlengthを返し、保持する値だけを`loadSymbol`する形を使える。

host側bufferの具体的な取得、copy完了までの有効期間、copy後の解放はbackend adapter contractが定める。hostの後続変更や
解放がSymbolへ影響してはならない。境界operationの一般則は[Engram仕様](engrams.md#境界のoperation)に、決定理由は
[D031](../design/decisions.md#d031-engramをmal内部のlifetime-authorityとする)に記録する。

opaque value は copyable/droppable な handle bit pattern として振る舞い、resource の close/free 多重実行を言語は防がない。
決定理由は[D015](../design/decisions.md#d015-opaque-valueはcopyable-handleとする)に記録する。

host operationがresult capabilityを正常returnする前にtrapするか、capabilityを含まないfailure resultを返す場合、
そのoperationだけが取得し、callerにもresultにも属さない一時resourceはadapterが解放する。正常resultへ含めた
capabilityのtransferはreturn時にcommitする。この規則はargumentとして受け取ったresource、以前のcallでtransfer済みの
resource、またはtrap時の一般的なstack unwindingをcleanupしない。trapし得るruntime helperを呼ぶadapterは、helperより前に
取得した一時resourceを残さない構成にするか、operation固有のcleanup手段を用意する。

`Ptr`を返すoperationは、pointerが指すlive region、permission、lifetimeをhost contractに定める。`Ptr`の複製は
storageを複製せず、lifetimeを延長しない。詳細は[memory primitive](memory.md)に定める。

## ABI と adapter

`extern` 宣言を任意の C function declaration と同一視しない。特に product、sum、`Symbol` は target ABI によって引数・戻り値の渡し方が異なる。

reference C backendはmal用の一貫したC representationを生成し、必要に応じて手書きまたは生成した小さなC adapterを介して
host APIを呼ぶ。`mal_ext_*` adapterはtrusted computing baseに含まれるが、raw host resourceそのものではない。
runtime contextを一時的に借りてadmissionを依頼できても、Engramのownershipやlifetime authorityは得ない。
C header parserやC type systemはmalに導入しない。

reference compilerはprogram固有のC headerを生成する。利用者はそのheaderに対するC source、object、static archive、shared objectを
linker inputとして渡す。symbolはlink時に解決し、runtime `dlopen`やplugin discoveryは行わない。正確なmappingは
[C host ABI](c-host-abi.md)に定める。

## 安全性の境界

正しく型付けされた mal program であっても、contract に違反する host implementation から保護されない。例えば不正な tag の sum、copy完了前に無効となるhost側byte buffer、二重解放可能な handle を host が与えれば、言語の型安全性は維持できない。

したがって `extern` implementation は trusted computing base に含まれる。
