# 型

Status: Current v0.5 profile

## 型の構成

```text
T ::=
    Unit
  | Int8 | Int16 | Int32 | Int64
  | UInt8 | UInt16 | UInt32 | UInt64
  | Float32 | Float64
  | Engram
  | Ptr
  | (T, T, ...)
  | [T, T, ...]
  | T -> T
  | ExternalType
  | TypeAlias
```

`Float32`と`Float64`は、それぞれIEEE 754-2019のbinary32とbinary64である。normal、subnormal、正負のzero、正負のinfinity、NaNを含む。詳細な演算規則は[実行意味論](execution.md#浮動小数点)に定める。

`Int`、`Long`、`Size` のような platform-dependent な整数型はない。subtyping、generic、implicit numeric conversion、nominal user type はない。

`Byte` と `Char` という型はない。単一 byte は `UInt8` で表す。mal は Unicode character を primitive value として定義しない。

## Engram

`Engram`は言語組み込みのimmutableな有限byte値であり、array、buffer、encoded textではない。値はcopyableで、そのbytesはprogram終了まで有効で変更されない。Engram値を複製してもbytes自体を複製する必要はない。source-levelの個別解放操作は存在しない。

Engram literalはnumeric literalと同じく組み込み値を表すnotationであり、そのbytesはprogram imageの静的storageに置いてよい。host側の一時byte bufferはEngramではなく、`extern`境界でmal-owned storageへcopyされた時点でEngramになる。literal、operator、storageの完全な規則は[Engram](engrams.md)に定める。

mutable byte bufferはEngramではなく、`Ptr`とlength、または必要に応じてexternal opaque typeで表す。v0.5は
組み込みのarray、slice、`ByteBuffer`型を持たない。

## Ptr

`Ptr`は型なしのdata address型である。要素型、length、ownershipは持たず、memory accessにはpredefinedな
numeric scalar、pointer、およびEngram descriptor operationを使う。完全な規則は[memory primitive](memory.md)に定める。

## Unit

`Unit` は空直積で、唯一の値は `()` である。

```mal
done :: Unit := ();
```

一要素 tuple は存在しない。`(A)` と `(a)` はそれぞれ括弧付きの型と式である。

## 直積

二要素以上の直積は tuple notation で表す。

```mal
Point :: (Float64, Float64);
origin :: Point := (0.0, 0.0);
(x, y) := origin;
```

型、値、pattern の位置が対応する。record、field name、method はない。

## 直和

直和は角括弧で囲んだ順序付き n 項型で、各項は 0-based index を持つ。

```mal
MaybeInt32 :: [Unit, Int32];

none := MaybeInt32[0](());
some := MaybeInt32[1](42);
```

同じ型を複数の項に置いてよい。

```mal
Choice :: [Int32, Int32];
a := Choice[0](42);
b := Choice[1](42);
```

`a` と `b` は異なる variant である。injection index は compile-time integer literal でなければならず、範囲外は compile-time error になる。

直和は二項以上でなければならない。`[]`と`[A]`はv0.5では不正であり、将来のために予約する。

n-ary sum と nested sum は異なる型である。

```mal
Flat :: [A, B, C];
Nested :: [A, [B, C]];
```

`case` は全 index を重複なく処理し、全 arm が同じ結果型を持たなければならない。

## predefined Bool

`Bool` は独立した primitive type ではなく、次の transparent alias と predefined binding である。

```mal
Bool :: [Unit, Unit];

false :: Bool := Bool[0](());
true :: Bool := Bool[1](());
```

これらは compilation unit より外側の predefined scope に存在するものとして名前解決する。compilation unit の top-level で `Bool`、`false`、`true` を再定義してはならない。local scope では通常の shadowing 規則により `false` と `true` を shadow できる。

`false` と `true` は keyword や専用 literal ではなく、型付きの immutable value である。

`Bool` は transparent なので `[Unit, Unit]` と同じ型である。比較演算は false の場合に index 0、true の場合に index 1 の値を返す。

## transparent alias

PascalCase identifier に型を対応させる。

```mal
Point :: (Float64, Float64);
Size :: (Float64, Float64);
```

`Point`、`Size`、`(Float64, Float64)` は同じ型である。alias は新しい runtime representation や nominal identity を作らない。recursive alias は認めない。

直和 injection に書かれた alias 名は、型検査時にその alias が表す直和型へ展開される。alias 自体に runtime identity は残らない。

## 関数型

```mal
Int32 -> Int32
(Int32, Int32) -> Int32
A -> B -> C
```

`->` は右結合なので、最後の例は `A -> (B -> C)` である。直和は `[]` で区切られるため、`[A, B] -> C` の domain は直和型全体になる。

複数 parameter の関数は意味上、一つの直積を受け取る。0 parameter の関数は `Unit` を受け取る。

関数型の値は lexical closure である。同じ関数型を持つ capture-free な関数と capture を持つ関数は、型として区別しない。closure の code や environment を観察する操作と、関数値の equality は存在しない。

## external opaque type

```mal
extern Mem;
extern File;
```

opaque type の内部表現を mal program から構成・分解・観察することはできない。値は通常の mal 値と同様に binding でき、複製・破棄できるものとして型検査する。

このため `File` や `Mem` のような resource の一意所有権は保証されない。resource correctness は [`extern` contract](extern.md) の責務であり、決定理由は[D015](../design/decisions.md#d015-opaque-valueはcopyable-handleとする)に記録する。
