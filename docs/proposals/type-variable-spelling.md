# 型変数を字面で型名と区別する案

Status: Exploratory

この文書は、型変数と型名を字面で区別する案と、判断前の論点を管理する。現行の規則は[operation family](../spec/operation-families.md)と
[parametric polymorphism](../spec/generics.md)、implementation keyを型patternとして読む理由は[D094](../history/decisions/active/D094.md)を
正とする。

## 問題

型変数と型名はどちらも`TYPE_IDENT`で綴る。implementation keyでは、見える型名に解決しない名前がpattern変数になるため、
型名の書き損じは型変数として受理される。`first<Buffer, Int23>`は`Int32`の書き損じでも全ての要素型へのimplementationになり、
誤りはkeyではなく、そのimplementationを使う箇所や本体の型検査に現れる。

現在のcompilerは、keyのbinderが見える型名と一編集の距離にあり、そのimplementationの検査がerrorになった場合に、
意図した型名の候補をnoteに添える。errorにならない書き損じと、使う箇所でのerrorには候補を添えない。

## 候補

| 案 | 例 | 書き損じ | 他への影響 |
|---|---|---|---|
| 小文字の型変数 | `pure<Either<e>, a>` | 大文字の未知の名前は型名のerrorになる | 型の位置で`VALUE_IDENT`を許すと、`f(a < b, c > d)`を型引数と読み得る。名前に密着した`<`だけを型引数とする規則が要る |
| 記号付きの型変数 | `pure<Either<$e>, $a>` | 記号のない未知の名前は型名のerrorになる | 字句が一つ増える。`$`は値の変数を連想させる |
| 現行 | `pure<Either<E>, A>` | 型変数として受理される | 構文と字句を増やさない |

小文字の案はHaskellとElm、名前に密着した`<`だけを型適用とする規則はF#、記号の案はOCamlとSMLが採る。型parameterを必ず
宣言させる案は、keyごとに宣言の構文を要するため候補にしない。

## 判断の材料

- 書き損じが実際にどれだけ起き、どれだけ遠い箇所でerrorになるか。
- 小文字の案を採る場合、`<`の密着規則をformatterの規則から構文へ移す影響。
- どの案でも、`generics.md`の「型parameterの名前は見える型名と同じであってはならない」を残すか。
