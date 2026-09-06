# プログラム構造

Status: Current v0.5 profile

## compilation unit

v0.5は一つのsource fileからなる単一compilation unitを扱う。package manager、import、module、dependency resolutionは
言語仕様に含めない。

## top-level item

top-level には次を置ける。

```text
type alias
external opaque type declaration
external operation declaration
value binding
```

```mal
Point :: (Float64, Float64);
extern print :: Engram -> Unit;

distance :: (Point, Point) -> Float64 :=
    \(a :: Point, b :: Point) {
        (ax, ay) := a;
        (bx, by) := b;
        extern sqrt(
            (ax - bx) * (ax - bx) + (ay - by) * (ay - by)
        );
    };
```

type alias と extern declaration は unit 全体から参照できる。value binding は source order で scope に入り、[自己再帰の例外](execution.md#再帰) を除いて前方参照できない。

top-level value の RHS は、literal、product/sum、integer conversion、lambda、およびそれらからなる作用のない closed expression に制限する。他のtop-level valueへの参照と`extern` callは認めない。`false`と`true`はclosedなpredefined constantとして参照できる。top-level lambda は外側に local scope を持たないが、その内側にある nested lambda は明示capture listを使用できる。詳細と理由は[D018](../design/decisions.md#d018-top-level-initializationは作用のないclosed-valueに限定する)に記録する。

## entry point

実行可能 program は次の binding を一つ持つ。

```mal
main :: Unit -> Int32 := \() { 0 };
```

backendは`main()`の結果をprocess exit statusへ渡す。library compilationや他のentry pointはv0.5の言語仕様外である。
