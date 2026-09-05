# implementation roadmap

Status: Current; no active milestone

この文書はreference compilerの現在位置と、active milestoneがある場合の順序・acceptance criteriaを所有する。
言語とhost interfaceは[`spec/`](../spec/)、stageの責務は[responsibilities](responsibilities.md)、検証方法は
[test policy](../development/testing.md)を正とする。

## 現在位置

M16まで完了している。新しい実装作業を始める前に、解決する問題、変更する責務境界、範囲外、
検証可能な完了条件をこの文書へ追加する。候補だけの機能や将来向けframeworkはmilestoneにしない。

| Gate | Outcome |
|---|---|
| M0 / S0 | 最初のnative vertical slice、checked-in example、再利用可能なnative fixture |
| M1 | 固定幅integer、短縮numeric suffix、byte literal |
| M2 | product、destructuring、opaque handle、aggregate ABI |
| M3 | immutable `String`とextern copy contract |
| M4 | annotated self recursionとdirect tail-call lowering |
| M5 / R0 | strict `Float32` / `Float64` profileとv0.4 release readiness |
| M6 | v0.5の型なし`Ptr`とnumeric scalar memory primitive |
| M7 | public buildの最適化contractとgenerated Cのcost削減 |
| M8 | compiler責務の監査とin-memory tooling boundary |
| M9 | VS Codeのlanguage registration、lexical highlighting、editing configuration |
| M10 | lossless lexical sourceとcanonical formatter |
| M11 | stdio language serverのdiagnostic、full sync、document formatting |
| M12 | semantic editor query、LSP semantic request、VS Code language client |
| M13 | reference compilerのstage別compile-time baselineとlossless lexing costの分離 |
| M14 | range付きMarkdown hover、byte literal内のbracketを隔離するTextMate scopeとeditor regression |
| M15 | 共通block result、optional terminal semicolon、canonical compact/expanded block formatting |
| M16 | canonical top-level declaration grouping、binding間の空行、comment attachment |

M0当時の境界は[M0 implementation record](m0.md)、M7の測定と採否判断は
[performance record](../development/performance.md)に残す。それ以外の完了内容は現在の仕様、test、example、Git履歴を正とし、
この文書へ実装 chronologyを重複させない。
