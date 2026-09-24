# first-class primitive `trap`の導入計画

Status: Exploratory

この文書は、programを明示的に異常終了させるpredefined primitive `trap`を導入する案と、判断前に解決する論点を管理する。
現在の言語規則は[`spec/`](../spec/)、trapの現行規則は[実行意味論](../spec/execution.md#trap)を正とする。この提案は採択前のため、
現在のprogramが`trap`の存在へ依存してはならない。

## 目的と範囲

mal sourceだけでは、programが到達してはならない状態を表明できない。範囲外の`Buffer` accessなど未検査preconditionの違反は
結果を保証せず、利用者が検査付きのoperationを`mal`で書いても、失敗を確定した終了として表せない。`trap`はこの終了だけを
与える。allocation失敗など既存のtrap条件、未検査preconditionの扱い、host固有のfailure policyは変更しない。

## 現状

`(Address, USize) -> []`のようなexternをhostが実装すれば、同じ効果を表現できる。空直和`[]`を返すexternは宣言でき、
`fail()[]`のようにempty eliminationで受けるとその後は到達不能になる。ただしこの方法にはhost C sourceが必要であり、hostを持たない
programでは使えない。

`[]`を返すexternの正常完了は表現できない。C host ABIは`[]`のterminal return helperを公開せず、bodyが従える規則は
`mal_call_trap`で終了するか、戻らないことだけである。公開されないcarrier型でtagを偽造してreturnするのはcontract違反である。
現在の実装は、`[]`に有効なtagは存在しないため、この値を境界のtag検査でtrapさせる。未定義動作にはならない。

## 提案するcontract

```mal
trap :: Symbol -> [];
```

`trap(message)`は引数を通常のapplication規則で評価した後、programを直ちに異常終了する。mal codeから捕捉も回復もできない。
それまでに完了した`extern`の作用は巻き戻さない。結果型が`[]`なので、`trap(message)[]`は既存のempty eliminationと同じく
`Abrupt`になり、任意の型の位置に置け、後続のcodeは到達不能として拒否される。

`message`は診断のための入力であり、出力先、書式、process exit statusは現行のtrapと同じく規定しない。predefined
bindingであり、通常のfunction valueとして参照、shadow、引数渡しができる。generic bindingではない。

## 既存規則との関係

- [実行意味論のtrap](../spec/execution.md#trap)の条件に「明示的な`trap`」を加える。
- [言語の範囲](../spec/scope.md)は外部世界への作用を`extern`と明示的なmemory accessに限る。`trap`のstderr出力と異常終了は
  この規則の例外になり、書式を規定しない診断としてその旨を明記する必要がある。
- externでtrapを定義する従来の方法は変えない。hostが持つfailure policyは引き続きextern contractに置く。
- 未検査preconditionは検査へ変更しない。利用者は`trap`で検査付きoperationを書ける。

## Baseline実行形

`Symbol`のbyte列を`mal_call_trap`と同じ終了経路へ渡すnoreturnのruntime helperを一つ加え、LLVM backendはそのcallの直後を
`unreachable`にする。message Symbolのownerは終了までに解放する必要がない。

## 導入順

1. 仕様案で型、評価順、作用の記述、診断の扱いを固定する。
2. resolver、checker、editorへpredefined valueの型とdocumentationを追加する。
3. core以降でprimitive callとして保持し、`Abrupt`をcontrol lowering上のterminatorへ写す。
4. LLVM backendとC runtimeにnoreturn helperを実装する。
5. 適合testに`trap`のcaseを加え、baselineとproductionの両方で確認する。

## 検証境界

focused testは少なくとも次を固定する。

- `trap(message)[]`が任意の型の位置でcheckを通り、その後のexpressionがunreachableとして拒否される。
- trapより前に完了したexternの作用が保持され、messageの評価中に起きた作用も順序どおりである。
- messageが空、NULを含む、UTF-8でない場合も終了する。
- `trap`をshadowした場合は通常のlexical scopeに従い、function valueとして参照、引数渡しできる。
- baselineとproductionで同じobservable behaviorを得る。

## 採択前の未決事項

- messageをSymbolで受けるか、引数を持たない形にするか。
- 診断の出力先と書式を仕様に含めるか。含めない場合の扱いを利用者へどう示すか。
- 名前を`trap`、`abort`、`unreachable`のどれにするか。
- `[]`を返す形では`trap(message)[]`と書く必要がある。値の型を選べる`trap<A>`を併設するか、
  それとも`Abrupt`を保つため`[]`だけにするか。
- portableなprocess exit statusを定めるか。現在は定めていない。
- primitive導入が、externとして定義する場合よりsystem全体のcontractと実装を実際に小さくするか。
