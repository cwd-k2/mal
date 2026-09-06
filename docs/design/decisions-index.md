# 設計決定index

Status: Current navigation

この文書は[設計決定記録](decisions.md)への入口だけを持つ。規則は[`spec/`](../spec/)、判断の本文とstatusは
各decisionを正とし、ここでは現行判断とhistorical recordを分けて示す。

## 現行判断

| Area | Decisions |
|---|---|
| compiler | [D002](decisions.md#decision-d002) |
| closure | [D003](decisions.md#decision-d003)、[D007](decisions.md#decision-d007) |
| sum、Bool、`if`、`case` | [D004](decisions.md#decision-d004)、[D005](decisions.md#decision-d005)、[D023](decisions.md#decision-d023) |
| minimalism | [D008](decisions.md#decision-d008) |
| Float | [D009](decisions.md#decision-d009)、[D019](decisions.md#decision-d019) |
| literalとscalar operation | [D011](decisions.md#decision-d011)、[D013](decisions.md#decision-d013)、[D014](decisions.md#decision-d014)、[D020](decisions.md#decision-d020)、[D021](decisions.md#decision-d021)、[D025](decisions.md#decision-d025) |
| externとopaque value | [D012](decisions.md#decision-d012)、[D015](decisions.md#decision-d015)、[D016](decisions.md#decision-d016) |
| top-level initialization | [D018](decisions.md#decision-d018) |
| `Ptr` memory primitive | [D022](decisions.md#decision-d022)、[D024](decisions.md#decision-d024) |
| EngramとExternのauthority | [D031](decisions.md#decision-d031) |

D012はD016、D022はD024でrefineされているが、元の判断を撤回していない。

## 後継があるhistorical record

| Record | Current successor |
|---|---|
| [D001: local capture禁止](decisions.md#decision-d001) | [D003](decisions.md#decision-d003) |
| [D006: `b'…'` byte literal](decisions.md#decision-d006) | [D025](decisions.md#decision-d025) |
| [D010: byte型としてのEngram](decisions.md#decision-d010) | [D031](decisions.md#decision-d031) |
| [D017: immutable byte型名](decisions.md#decision-d017) | [D031](decisions.md#decision-d031) |
| [D026: Engram descriptor memory operation](decisions.md#decision-d026) | [D031](decisions.md#decision-d031) |
| [D027: storage-size query](decisions.md#decision-d027) | [D031](decisions.md#decision-d031)で対象型をrefine |
| [D028: Engram operator](decisions.md#decision-d028) | [D031](decisions.md#decision-d031)で`Symbol` operationへrefine |
| [D029: Engram concatenation](decisions.md#decision-d029) | [D031](decisions.md#decision-d031)で`Symbol` operationへrefine |
| [D030: process argument descriptor](decisions.md#decision-d030) | [D031](decisions.md#decision-d031)でexternal descriptorへrefine |

historical record内の旧構文や旧名称は当時の判断を保存するために残す。現在のsyntaxやbehaviorとして引用せず、
必ずsuccessorと[`spec/`](../spec/)を確認する。
