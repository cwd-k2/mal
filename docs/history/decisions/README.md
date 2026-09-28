# 設計決定履歴

Status: Historical records

この文書は設計決定記録への入口である。規則は[`spec/`](../../spec/)、判断の本文、status、後継関係は各decisionを正とする。

`active/`には判断の少なくとも一部が現在も有効なrecordを置く。一部だけがrefine、supersedeされた場合は`active/`に残し、
適用範囲をStatus行に記録する。`superseded/`には判断全体が失効したrecordを置く。directoryは大まかな分類であり、
現在の正確なruleを確認するときは各decisionのStatus行と[`spec/`](../../spec/)を確認する。

## Active decisions

| Area | Decisions |
|---|---|
| compiler | [D002](active/D002.md)、[D041](active/D041.md)、[D045](active/D045.md)、[D046](active/D046.md)、[D047](active/D047.md)、[D065](active/D065.md)、[D066](active/D066.md)、[D067](active/D067.md)、[D069](active/D069.md)、[D080](active/D080.md)、[D081](active/D081.md) |
| editor tooling | [D044](active/D044.md)、[D081](active/D081.md) |
| closure | [D003](active/D003.md)、[D007](active/D007.md)、[D038](active/D038.md) |
| application、sum、Bool、`if` | [D004](active/D004.md)、[D005](active/D005.md)、[D043](active/D043.md)、[D049](active/D049.md)、[D051](active/D051.md)、[D072](active/D072.md) |
| minimalism | [D008](active/D008.md)、[D033](active/D033.md)、[D055](active/D055.md)、[D057](active/D057.md) |
| managed ownership | [D033](active/D033.md)、[D035](active/D035.md)、[D041](active/D041.md)、[D055](active/D055.md)、[D057](active/D057.md)、[D058](active/D058.md)、[D075](active/D075.md)、[D080](active/D080.md)、[D083](active/D083.md) |
| Float | [D009](active/D009.md)、[D019](active/D019.md) |
| literalとscalar operation | [D011](active/D011.md)、[D013](active/D013.md)、[D020](active/D020.md)、[D021](active/D021.md)、[D025](active/D025.md)、[D035](active/D035.md) |
| externとopaque value | [D012](active/D012.md)、[D015](active/D015.md)、[D016](active/D016.md)、[D039](active/D039.md)、[D040](active/D040.md)、[D053](active/D053.md)、[D054](active/D054.md)、[D074](active/D074.md)、[D078](active/D078.md) |
| top-level initialization | [D018](active/D018.md) |
| predefined名 | [D077](active/D077.md) |
| source file requirement | [D032](active/D032.md) |
| generics、Buffer、operation family | [D052](active/D052.md)、[D075](active/D075.md)、[D081](active/D081.md) |
| EngramとExternのauthority | [D028](active/D028.md)、[D029](active/D029.md)、[D031](active/D031.md)、[D033](active/D033.md)、[D035](active/D035.md)、[D040](active/D040.md)、[D052](active/D052.md)、[D053](active/D053.md)、[D054](active/D054.md)、[D074](active/D074.md)、[D082](active/D082.md) |
| 実行環境 | [D030](active/D030.md)、[D041](active/D041.md)、[D070](active/D070.md)、[D071](active/D071.md)、[D076](active/D076.md) |

## Superseded decisions

| Decision | 退役した判断 | 直接の後継または現在のauthority |
|---|---|---|
| [D001](superseded/D001.md) | local captureの禁止 | [D003](active/D003.md) |
| [D006](superseded/D006.md) | `b'…'` byte literal | [D025](active/D025.md) |
| [D010](superseded/D010.md) | program-lifetime `Engram` | [D031](active/D031.md) |
| [D014](superseded/D014.md) | shift countの型 | [D035](active/D035.md) |
| [D017](superseded/D017.md) | immutable byte値としての`Engram` | [D031](active/D031.md) |
| [D022](superseded/D022.md) | 型なし`Ptr` baseline | [D052](active/D052.md) |
| [D023](superseded/D023.md) | 括弧付き`case` syntax | [D042](superseded/D042.md) |
| [D024](superseded/D024.md) | `Ptr`のload/store | [D052](active/D052.md) |
| [D026](superseded/D026.md) | Engram descriptorのload/store | [D031](active/D031.md) |
| [D027](superseded/D027.md) | `@T` storage width | [D052](active/D052.md) |
| [D034](superseded/D034.md) | C adapterのmanaged carrier規約 | [D040](active/D040.md) |
| [D036](superseded/D036.md) | runtime-owned `Symbol` builder | [D040](active/D040.md) |
| [D037](superseded/D037.md) | 型修飾memory primitive | [D052](active/D052.md) |
| [D042](superseded/D042.md) | valueとcontinuationの双方向表記 | [D048](superseded/D048.md) |
| [D048](superseded/D048.md) | sum return binder | [D049](active/D049.md) |
| [D050](superseded/D050.md) | bodyとbinder boundaryの統一 | [D051](active/D051.md) |
| [D056](superseded/D056.md) | Cursor loadとRegion index | [memory specification](../../spec/memory.md) |
| [D059](superseded/D059.md) | Packed builderのstable data access | [D061](superseded/D061.md)、[D063](superseded/D063.md) |
| [D060](superseded/D060.md) | lazy Packed preparation | [D062](superseded/D062.md) |
| [D061](superseded/D061.md) | `Buffer<A>`構築authority | [memory specification](../../spec/memory.md) |
| [D062](superseded/D062.md) | callback前のedit storage preparation | [memory specification](../../spec/memory.md) |
| [D063](superseded/D063.md) | non-growing Buffer helper | [memory specification](../../spec/memory.md) |
| [D064](superseded/D064.md) | Buffer helperの`noalias` | [memory specification](../../spec/memory.md) |
| [D068](superseded/D068.md) | scoped Region API | [memory specification](../../spec/memory.md) |
| [D073](superseded/D073.md) | `(Address, USize)`のprocess argument | [D076](active/D076.md) |
| [D079](superseded/D079.md) | predefined phantom `Index<T>` | [memory specification](../../spec/memory.md) |

historical record内の旧構文や旧名称は当時の判断を保存するために残す。現在のsyntaxやbehaviorとして引用せず、
必ずStatus行の後継と[`spec/`](../../spec/)を確認する。
