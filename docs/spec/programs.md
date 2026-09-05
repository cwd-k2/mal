# プログラム構造

Status: Current v0.4 profile

## compilation unit

v0.4 は単一 compilation unit を基本とする。package manager、import、module、dependency resolution は言語仕様に含めない。

compiler が複数 source file を受け取る場合も、一つの top-level scope を構成する入力として扱う。file 間の順序規則は compiler CLI が明示しなければならない。

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
extern print :: String -> Unit;

distance :: (Point, Point) -> Float64 :=
    \(a :: Point, b :: Point) {
        (ax, ay) := a;
        (bx, by) := b;
        return extern sqrt(
            (ax - bx) * (ax - bx) + (ay - by) * (ay - by)
        );
    };
```

type alias と extern declaration は unit 全体から参照できる。value binding は source order で scope に入り、[自己再帰の例外](execution.md#再帰) を除いて前方参照できない。

top-level value の RHS は、literal、product/sum、integer conversion、lambda、およびそれらからなる作用のない closed expression に制限する。他のtop-level valueへの参照と`extern` callは認めない。`false`と`true`はclosedなpredefined constantとして参照できる。top-level lambda は外側に local scope を持たないが、その内側にある nested lambda は明示capture listを使用できる。詳細と理由は[D018](../design/decisions.md#d018-top-level-initializationは作用のないclosed-valueに限定する)に記録する。

## entry point

実行可能 program は次の binding を一つ持つ。

```mal
main :: Unit -> Int32 := \() {
    return 0;
};
```

backend は `main()` の結果を process exit status へ渡す。library compilation や他の entry point は v0.4 の言語仕様外である。
