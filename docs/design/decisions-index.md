# 設計決定index

Status: Current navigation

この文書は`decisions/`にある個別の設計決定記録への入口だけを持つ。規則は[`spec/`](../spec/)、判断の本文と
statusは各decisionを正とし、ここでは現行判断とhistorical recordを分けて示す。

## 現行判断

| Area | Decisions |
|---|---|
| compiler | [D002](decisions/D002.md) |
| closure | [D003](decisions/D003.md)、[D007](decisions/D007.md) |
| sum、Bool、`if`、`case` | [D004](decisions/D004.md)、[D005](decisions/D005.md)、[D023](decisions/D023.md) |
| minimalism | [D008](decisions/D008.md)、[D033](decisions/D033.md) |
| Float | [D009](decisions/D009.md)、[D019](decisions/D019.md) |
| literalとscalar operation | [D011](decisions/D011.md)、[D013](decisions/D013.md)、[D020](decisions/D020.md)、[D021](decisions/D021.md)、[D025](decisions/D025.md)、[D035](decisions/D035.md) |
| externとopaque value | [D012](decisions/D012.md)、[D015](decisions/D015.md)、[D016](decisions/D016.md)、[D034](decisions/D034.md) |
| top-level initialization | [D018](decisions/D018.md) |
| source file requirement | [D032](decisions/D032.md) |
| `Ptr` memory primitive | [D022](decisions/D022.md)、[D024](decisions/D024.md)、[D035](decisions/D035.md) |
| EngramとExternのauthority | [D031](decisions/D031.md)、[D033](decisions/D033.md)、[D034](decisions/D034.md)、[D035](decisions/D035.md)、[D036](decisions/D036.md) |

D012はD016とD032、D022はD024、D031はD033、D033はD034でrefineされているが、元の判断を撤回していない。
D008のmemory management節はD033が、D009、D022、D028、D029、D033のtrapに関する一部はD035が置き換える。

## 後継があるhistorical record

| Record | Current successor |
|---|---|
| [D001: local capture禁止](decisions/D001.md) | [D003](decisions/D003.md) |
| [D006: `b'…'` byte literal](decisions/D006.md) | [D025](decisions/D025.md) |
| [D010: byte型としてのEngram](decisions/D010.md) | [D031](decisions/D031.md) |
| [D017: immutable byte型名](decisions/D017.md) | [D031](decisions/D031.md) |
| [D026: Engram descriptor memory operation](decisions/D026.md) | [D031](decisions/D031.md) |
| [D027: storage-size query](decisions/D027.md) | [D031](decisions/D031.md)で対象型をrefine |
| [D028: Engram operator](decisions/D028.md) | [D031](decisions/D031.md)で`Symbol` operationへrefine |
| [D029: Engram concatenation](decisions/D029.md) | [D031](decisions/D031.md)で`Symbol` operationへrefine |
| [D030: process argument descriptor](decisions/D030.md) | [D031](decisions/D031.md)でexternal descriptorへrefine |
| [D014: shift countを一律にtrap](decisions/D014.md) | [D035](decisions/D035.md) |

historical record内の旧構文や旧名称は当時の判断を保存するために残す。現在のsyntaxやbehaviorとして引用せず、
必ずsuccessorと[`spec/`](../spec/)を確認する。
