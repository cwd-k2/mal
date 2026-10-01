# IxPoolの所有権primitive

Status: Exploratory support document

この文書は、IxPool、Arena、`ImPool`、run primitiveの所有権上の意味を、placeに対する二つの遷移とexecution ownershipの
operand effectへ分解する。storage、precondition、failureは[lifecycle contract](lifecycle-contract.md)、local slotとcall conventionの
現行規則は[managed valueのownership](../../implementation/ownership.md)を正とする。

## place

IxPoolのslot、IxPoolのState、Arenaのentryは、どれも`Vacant`または`Live`のplaceである。Liveなplaceはちょうど一つの
responsibilityを持つ。これは[local slot](../../implementation/ownership.md#slotとoperation)のinitialize、vacate、replaceと同じ状態であり、
違いはplaceがIxPool identityの中にあり、実行時のcoordinateで選ばれることだけである。

## 核になる遷移

所有権を動かす遷移は次の二つだけである。

| 遷移 | place | responsibility |
|---|---|---|
| `init(place, value)` | Vacant → Live | operandのresponsibilityをplaceへ移す |
| `take(place)` | Live → Vacant | placeのresponsibilityをresultへ移す |

どちらもresponsibilityを移すだけで、型別の`share`も`drop`も呼ばない。`init`へ渡すresponsibilityは、call site の
execution ownershipが、operandを後で使うなら`Share`し、last useなら`Consume`して用意する。`take`のresultは通常のowned resultであり、
使われなくなった時点でexecution ownershipが`Drop`する。

したがってsourceへ一般の`drop(value)`を公開する必要はない。値のDropはlast useの後に自動で起き、placeの値を捨てることは
`take`のresultを使わないことと同じである。

## 派生operation

他のIxPool operationは`init`と`take`の合成として定義する。primitiveとして残すのは占有状態の往復やcarrierの移動を省く性能のため、
および`get`のようにstorageを共有するIxPoolで派生形の`init`が共有storageのcopyを起こすのを避けるためであり、意味はこの分解と同じである。
`share`と`drop`の回数と順序も分解と一致しなければならない。

```text
get(place)          = v := take(place); init(place, v); v      // vを二度使うのでinitのoperandはShare
put(place, value)   = old := take(place); init(place, value); drop(old)
drop(place)         = drop(take(place))
state(pool)         = get(pool.state)
setState(pool, s)   = put(pool.state, s)
```

`put`の分解は、新valueをplaceへ成立させてから旧valueをDropする順序をそのまま与える。同じmanaged valueを読み出して書き戻しても、
旧valueのDropが新valueのreferentを解放しない。

| operation | 分解 | primitive内のshare | primitive内のdrop |
|---|---|---|---|
| `initAt` | `init` | なし | なし |
| `takeAt` | `take` | なし | なし |
| `getAt`、`state` | `get` | 1 | なし |
| `putAt`、`setState` | `put` | なし | 旧value 1 |
| `dropAt` | `drop` | なし | 1 |
| `writeRange`（n slot） | slotごとに`init`または`put` | n − 1 | Liveだったslot数 |
| `copyRange`（n slot） | sourceのn回の`get`の後、destinationへ`init`または`put` | n | Liveだったslot数 |
| IxPoolの終了 | Stateと全Live slotの`drop` | なし | Live place数 |
| `reserve` | 遷移なし。全carrierを移動するだけ | なし | なし |

`writeRange`はcall siteから一つのresponsibilityを受け取り、残りのslotのためにruntimeが`share`する。n = 0なら
受け取ったresponsibilityをruntimeが`drop`する。
`copyRange`はsourceを全て`get`してから書くので、同じidentityで範囲が重なってもsourceの値を先に失わない。

## 拡張operation

writable successor、IdPool、Arena、`Id<T>`も同じ語彙で表せる。

- `writableSuccessor(pool)`は、inputが唯一のresponsibilityならそのidentityをresultへ移す。共有中なら新しいIxPoolを作り、
  Stateと各Live slotを`get`して`init`し、inputのresponsibilityをDropする。
- `ImPool`の更新はwritable successorの後に`put`または`init`を行い、successorを返す。
- IdPoolの`idInsert`は要素を空いた場所へ`init`し、`idRemove`は`take`し、`idGet`は`get`する。
- `arenaAdd`は新しいIxPoolをentryへ`init`し、`arenaRemove`はentryを`drop`し、`arenaGet`はentryを`get`してIxPoolを`Share`する。
- `Id<T>`は所有権を持たないdataであり、どの遷移も起こさない。

## compilerとruntimeの分担

compilerのexecution ownershipに新しく要るのは、operand effectの`Store`だけである。

- `Store`は、operandのresponsibilityをprimitiveが保持することを表す。`init`、`put`、`writeRange`のvalue、`makeIxPool`と
  `setState`のState、`arenaAdd`のIxPool、writable successorのinputが該当する。
- use planは`Store`を`Share`または`Consume`へlowerする。D083の保持解析は、`Store`へ渡るparameterをreturnやcaptureと同じく
  保持として扱い、Bufferの`put`のようなmal wrapperをowned native entryにする。
- IxPool handle、index、lengthは`Borrow`である。resultは全てownedであり、これは現行のprimitive resultと変わらない。

現行のBuffer operandは全て`Borrow`で、保存に必要なretainはruntimeが行う。これは`Store`を常にruntime内の`Share`として扱うことに
当たる。`Store`を導入すると、last useのvalueを`Consume`してruntime内のretainとcall site側のreleaseを省ける。

runtimeが型ごとに必要とするglueは、上の表で「primitive内」に数えたものだけである。

- `share<T>`は、primitiveが一つのresponsibilityから複数を作るときに使う。`get`、range operation、共有時のwritable successorが該当する。
- `drop<T>`は、primitive内で終わるresponsibilityに使う。`put`の旧value、`drop`、IxPoolの終了が該当する。
- relocationと`init`、`take`はcarrierを移動するだけでglueを呼ばない。

glueは失敗せず、I/O、host resourceの`close`、別IxPoolの更新など観測可能な作用を持たない。IxPool終了時のStateとslotのdrop順は
sourceから観測できず、container algorithmはその順序へ依存しない。pluginが新しいIxPool相当operationを追加する場合も、operandごとの
`Borrow`または`Store`と、上の分解を宣言する。

## 所有権を持たないprimitive

`reserve`、`capacity`、`isLive`はplaceの遷移を起こさない。[Buffer実装](buffer-implementation.md)の
`load`、`store`、`symbol`、`loadSymbol`は、`Representable`な型か`UInt8`だけを扱う。これらの型は
managed valueを含まないため、書き込みは形式上`init`または`put`でも`share`と`drop`はno-opであり、所有権解析へ入力を持たない。

## 検証

- 分解を直接実行するtest用runtimeと比べ、各派生primitiveの`share`と`drop`の回数と順序が一致する。
- `Store`へ渡るparameterを持つmal wrapperがowned native entryになり、last-use argumentを`Consume`する。
- 同じvalueを`put`で書き戻す場合と、範囲が重なる`copyRange`で、Drop済みのreferentを読まない。
