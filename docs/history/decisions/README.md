# 設計決定履歴

Status: Historical records

この文書は同じdirectoryにある個別の設計決定記録への入口である。規則は[`spec/`](../../spec/)、判断の本文、status、後継関係は各decisionを正とする。

## 現行判断

| Area | Decisions |
|---|---|
| compiler | [D002](D002.md)、[D041](D041.md)、[D045](D045.md)、[D046](D046.md)、[D047](D047.md)、[D065](D065.md)、[D066](D066.md)、[D067](D067.md)、[D069](D069.md) |
| editor tooling | [D044](D044.md) |
| closure | [D003](D003.md)、[D007](D007.md)、[D038](D038.md) |
| application、sum、Bool、`if` | [D004](D004.md)、[D005](D005.md)、[D043](D043.md)、[D049](D049.md)、[D051](D051.md)、[D072](D072.md) |
| minimalism | [D008](D008.md)、[D033](D033.md)、[D055](D055.md)、[D057](D057.md) |
| managed ownership | [D033](D033.md)、[D035](D035.md)、[D041](D041.md)、[D055](D055.md)、[D057](D057.md)、[D058](D058.md) |
| Float | [D009](D009.md)、[D019](D019.md) |
| literalとscalar operation | [D011](D011.md)、[D013](D013.md)、[D020](D020.md)、[D021](D021.md)、[D025](D025.md)、[D035](D035.md) |
| externとopaque value | [D012](D012.md)、[D015](D015.md)、[D016](D016.md)、[D039](D039.md)、[D040](D040.md)、[D053](D053.md)、[D054](D054.md) |
| top-level initialization | [D018](D018.md) |
| source file requirement | [D032](D032.md) |
| genericsとexternal memory | [D052](D052.md) |
| EngramとExternのauthority | [D031](D031.md)、[D033](D033.md)、[D035](D035.md)、[D040](D040.md)、[D052](D052.md)、[D053](D053.md)、[D054](D054.md) |
| 実行環境 | [D041](D041.md)、[D070](D070.md)、[D071](D071.md) |

一部だけがrefine、supersedeされたdecisionも、残る部分は現行判断である。後継と適用範囲は各decisionのStatus行を参照する。
Statusが`Superseded`のdecisionは上の表に含めない。

historical record内の旧構文や旧名称は当時の判断を保存するために残す。現在のsyntaxやbehaviorとして引用せず、
必ずStatus行の後継と[`spec/`](../../spec/)を確認する。
