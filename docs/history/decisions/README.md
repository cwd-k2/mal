# 設計決定履歴

Status: Historical records

この文書は同じdirectoryにある個別の設計決定記録への入口だけを持つ。規則は[`spec/`](../../spec/)、判断の本文と
statusは各decisionを正とし、ここでは現行判断とhistorical recordを分けて示す。

## 現行判断

| Area | Decisions |
|---|---|
| compiler | [D002](D002.md)、[D041](D041.md) |
| closure | [D003](D003.md)、[D007](D007.md)、[D038](D038.md) |
| sum、Bool、`if`、`case` | [D004](D004.md)、[D005](D005.md)、[D023](D023.md) |
| minimalism | [D008](D008.md)、[D033](D033.md) |
| Float | [D009](D009.md)、[D019](D019.md) |
| literalとscalar operation | [D011](D011.md)、[D013](D013.md)、[D020](D020.md)、[D021](D021.md)、[D025](D025.md)、[D035](D035.md) |
| externとopaque value | [D012](D012.md)、[D015](D015.md)、[D016](D016.md)、[D039](D039.md)、[D040](D040.md) |
| top-level initialization | [D018](D018.md) |
| source file requirement | [D032](D032.md) |
| `Ptr` memory primitive | [D022](D022.md)、[D024](D024.md)、[D035](D035.md)、[D037](D037.md) |
| EngramとExternのauthority | [D031](D031.md)、[D033](D033.md)、[D035](D035.md)、[D040](D040.md) |

D012はD016とD032、D022はD024、D031はD033、D033はD034でrefineされているが、元の判断を撤回していない。
D008のmemory management節はD033が、D009、D022、D028、D029、D033のtrapに関する一部はD035が置き換える。
D022、D024、D027、D031のsource spellingはD037が置き換える。
D007のlambda parameter source spellingはD038が置き換える。
D001とD018のcall siteに`extern`を置くsource spellingはD039が置き換える。
D041はD002のRust compilerと最初のC backendを維持し、次のexecution backend境界を追加する。

## 後継があるhistorical record

| Record | Current successor |
|---|---|
| [D001: local capture禁止](D001.md) | [D003](D003.md) |
| [D006: `b'…'` byte literal](D006.md) | [D025](D025.md) |
| [D010: byte型としてのEngram](D010.md) | [D031](D031.md) |
| [D017: immutable byte型名](D017.md) | [D031](D031.md) |
| [D026: Engram descriptor memory operation](D026.md) | [D031](D031.md) |
| [D027: storage-size query](D027.md) | [D031](D031.md)で対象型をrefine |
| [D028: Engram operator](D028.md) | [D031](D031.md)で`Symbol` operationへrefine |
| [D029: Engram concatenation](D029.md) | [D031](D031.md)で`Symbol` operationへrefine |
| [D030: process argument descriptor](D030.md) | [D031](D031.md)でexternal descriptorへrefine |
| [D034: C adapter ownership helper](D034.md) | [D040](D040.md) |
| [D036: Symbol admission builder](D036.md) | [D040](D040.md) |
| [D014: shift countを一律にtrap](D014.md) | [D035](D035.md) |

historical record内の旧構文や旧名称は当時の判断を保存するために残す。現在のsyntaxやbehaviorとして引用せず、
必ずsuccessorと[`spec/`](../../spec/)を確認する。
