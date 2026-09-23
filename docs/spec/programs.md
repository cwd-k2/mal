# プログラム構造

Status: Accepted v0.6 profile

## program と source file

programは一つのroot `.mal` fileと、そこから`require`で到達できるsource fileからなる。各`.mal` fileは独立した
top-level scopeを持ち、先頭に0個以上のrequire declarationを置ける。

```mal
require "./geometry.mal";
require "./geometry.c";
```

pathはSymbol literalからdecodeしたUTF-8で、宣言を持つfileのdirectoryから解決する相対pathとし、空であってはならない。
file種別はpathの末尾にある`.mal`または`.c`で決める。`.mal` requirementは対象file自身が
定義したpublicなtop-level名を宣言元fileへ導入する。requireしたfileから導入された名前を自動的に再公開しない。
同じcanonical pathへ複数経路から到達しても一つのsource fileとして扱い、`.mal` requirementのcycleはcompile-time
errorとする。

`.c` requirementは名前を導入せず、reference compilerのC build inputへ推移的に追加する。同じcanonical pathのC sourceは
一度だけcompileする。`.c` requirementの意味は[C host ABI](c-host-abi.md#build-model)に定める。package名、探索path、remote
dependency、namespace、一般的なqualified name、require alias、selective importは持たない。

直接requireした複数fileのpublic名同士、または導入したpublic名と宣言元fileのtop-level名が同じnamespaceで重複すれば
compile-time errorである。private名はfile identityごとに区別し、別fileの同名private declarationとは衝突しない。

top-levelのtype identifierまたはvalue identifierは、先頭が`_`ならそのfileだけから参照できるprivate名、それ以外なら
require元へ導入できるpublic名である。このvisibilityはmal source間のname lookupだけに作用し、必要なextern declarationや
host-visible typeをgenerated headerから除去しない。private型の値は、名前を参照できないfileでもpublic operationの引数や
結果として受け渡せる。

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
extern printBytes :: (Address, USize) -> Unit;

distance :: (Point, Point) -> Float64 :=
    (a, b) -> {
        (ax, ay) := a;
        (bx, by) := b;
        sqrt(
            (ax - bx) * (ax - bx) + (ay - by) * (ay - by)
        );
    };
```

type alias と extern declaration が導入する名前は unit 全体から参照できる。value binding は source order で scope に入り、[自己再帰の例外](execution.md#再帰) を除いて前方参照できない。

top-level valueのRHSは、literal、product/sum、numeric conversion、external function、lambda、および
それらからなる作用のない closed expression に制限する。他のtop-level valueへの参照とfunction applicationは認めない。
direct blockとdirect result blockもtop-level initializerには認めない。
`false`と`true`はclosedなpredefined constantとして参照できる。top-level lambda は外側に local scope を持たないが、その内側にある
nested lambda は外側lambdaのlocalをlexically captureできる。詳細と理由は[D018](../history/decisions/D018.md)に記録する。

## entry point

実行可能 program のroot fileは次のbindingを一つ持つ。requireされるfileに`main`を宣言してはならない。

```mal
main :: Unit -> Int32 := () -> 0;
```

backendは`main()`の結果をprocess exit statusへ渡す。library compilationや他のentry pointはprofile外である。

command-line argumentを受け取る実行可能programは、代わりに次のentry pointを持てる。

```mal
Arguments :: (USize, Address);
extern argumentLength :: Address -> USize;

main :: Arguments -> Int32 := (argumentCount, arguments) ->
    if (argumentCount == 0usize)
    then 0
    else {
        pointers := from<Address>(arguments, 0usize, argumentCount);
        firstAddress := pointers.get(0usize);
        // An operation-specific extern contract supplies the byte length.
        first :: Symbol := *from<UInt8>(firstAddress, 0usize, argumentLength(firstAddress));
        0;
    };
```

productの第一要素は実行ファイル名を除くargument数である。第二要素はhostが提供するargument collection capabilityである。
そのrepresentationはmal language profileに含めない。reference C hostでは`argv[1]`以降のC pointer列を指し、
`from<Address>`で必要なpointerをBufferへcopyできる。各C stringのlengthやencoding interpretationはextern contractが提供する。
argument数が0のときも第二要素はhost contractが定める値であり、要素を読み出してはならない。

argument bytesの終端、encoding、access operationはhost profileが定める。reference C hostでは各Addressが終端NULを持つC stringを
指すが、`from<UInt8>`へ渡すlengthには終端NULを含める必要はない。pointer列と各byte regionは`main`のreturnまでread-onlyで有効である。
`count`以上のpointerをcopyしてはならない。

`Unit -> Int32`と`(USize, Address) -> Int32`以外の`main`型はcompile-time errorである。設計理由は
[D030](../history/decisions/D030.md)に記録する。
