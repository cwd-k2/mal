# Buffer生成物とownership cost

Status: Historical measurement record

この文書は、2026-10-03のLLVM backendにおける`Buffer<UInt64>`と`Buffer<Buffer<UInt64>>`の生成物を、
同じalgorithmを書いたdirect Cと比較した記録である。Bufferの言語仕様は
[AddressとBuffer](../../spec/memory.md)、Pool導入時のstorage分類とloweringは
[compilerとruntimeの実装](../../proposals/pool/runtime/implementation.md)を正とする。

## 比較条件

対象revisionは`b27627a5`、compilerは`malc 0.6.0-dev`、host compilerはClang 21.1.8、CPUは
Intel Core Ultra 7 258Vである。Malは`--optimization production`、Cは`-O2 -flto -fuse-ld=lld`でbuildした。
入力とbuild方法は`.scratch/pool-prototype/buffer/benchmark-*`と`.scratch/pool-prototype/benchmark.nu`に置いた。

swap workloadは1024個の異なる要素に対し、102,400,000回の隣接swapを行う。最終要素を検査するためloopは
除去できない。raw pairは`UInt64`、owned pairはMalの入れ子Buffer handleと、同じretain、replace、releaseを
行うC objectを比べる。wall-clockは`hyperfine --warmup 3 --runs 20`で各programを測定した。

| Carrier | Direct C | Mal Buffer | Mal / C |
|---|---:|---:|---:|
| raw `UInt64` | 61.8 ± 1.0 ms | 91.6 ± 1.9 ms | 1.48x |
| reference-counted handle | 161.5 ± 6.7 ms | 233.6 ± 7.2 ms | 1.45x |

絶対時間はCPU frequencyと実行環境に依存する。ここでは同じbuildと入力の下での比率と、以下の
instruction、memory、IRの対応を判断材料とする。

## 動的instructionとmemory access

Valgrind Cachegrindでiteration数を1,024,000へ縮小した同一workloadを測定した。表の値はprocess全体であり、
97%以上が`main`に帰属した。

| Carrier | Instructions | Data reads | Data writes | Conditional branches |
|---|---:|---:|---:|---:|
| C raw | 6,842,604 | 1,573,927 | 1,557,401 | 551,532 |
| Mal canonical | 7,376,637 | 1,576,959 | 2,074,582 | 556,217 |
| C owned | 27,124,148 | 9,308,133 | 3,110,386 | 5,200,445 |
| Mal runtime-owned | 60,351,655 | 13,484,260 | 2,148,423 | 12,440,834 |

raw pairではMalの動的instructionが約8%、data referenceが約17%多い。特にwriteは約33%多い。最適化後IRと
machine codeでは、Cが4回のsource swapをまとめて中間値を再利用するのに対し、Malは2回ずつで、
byte offsetの計算と余分なstoreが残った。Buffer data取得call自体はinlineされ、hot loopの前へhoistされて
いた。したがってraw差はruntime callが残ったことではなく、loop fusionとaddressingの差である。

runtime-owned pairではMalのinstructionとconditional branchがCの約2.2倍になった。Malのhot loopは各handleの
null/tag test、retain、release後のzero test、destructor/free slow pathを持つ。branch mispredictionは少なく、予測失敗ではなく
branch数とmemory operation数が主なcostである。

Mal runtime-ownedのL1 data read missは1,091,536、C ownedは1,311だった。LLC missはどちらも約1100なので、
Malの増加分はL2で解決している。Cの内側objectは16 byteの1 allocationだが、Malの内側Bufferは約56 byteの
managed environmentと約56 byteのbyte ownerを別々に持つ。1024個の内側Bufferのheaderとstrideによりhot working setが
Cachegrindの48 KiB L1 data cacheに収まらない。

## allocationと生成物の大きさ

Memcheckの全allocationは正常にfreeされ、errorとleakは0だった。Massifのpeak heapは次のとおりである。

| Carrier | Allocations | Requested bytes | Peak heap |
|---|---:|---:|---:|
| C raw | 1 | 8,192 | 8,192 B |
| Mal canonical | 2 | 8,296 | 8,296 B |
| C owned | 1,025 | 24,576 | 24,576 B |
| Mal runtime-owned | 2,050 | 123,000 | 123,000 B |

Mal runtime-ownedの2,050 allocationは、1024個の内側Bufferそれぞれにmanaged environmentとbacking ownerの2回、
外側Bufferに2回である。この二重allocationと大きいheaderは現行representationのcostであり、「managed handleを
要素に持てる」こと自体の意味論的な下限ではない。一方、外側storageに内側payloadを埋め込んで一つの
allocationに平坦化すると、複数slotから共有される内側identityを壊す。compactなstable objectやallocationの統合は
可能だが、identityは独立に残す必要がある。

ELF section sizeはC benchmarkが3,461 byte、Mal canonicalが3,882 byte、Mal nestedが5,107 byteだった。
`main`のmachine code sizeはMal canonical、Mal nestedの順に502 byteと1,365 byteである。nestedにはこれに加えて
element retain、release、destructorが生成される。resident setはloaderとpage粒度に隠れ、C raw 1436 KiB、
Mal canonical 1440 KiB、C owned 1564 KiB、Mal nested 1568 KiBだった。この規模ではRSSではなくheap profileを
representation比較に使う。
なおC benchmarkは三つのmodeを一つのexecutableに含むため、ELF全体のsizeは厳密な同一program比較ではない。
`main`の増分と生成glueの存在を確認するdiagnosticとして扱う。

## 消去できるownership operation

別のidentity workloadでは、slotからhandleを`get`し、他の操作を挟まず同じslotへ`put`することを
102,400,000回繰り返した。CはLTO後にallocation、loop、retain/releaseをすべて消去した。Malはloopを残し、
93.6 ± 1.3 msを要した。縮小版のMalは約26.8M instructionを実行し、Cはstartupを含む176kだった。

LLVMはMal側の2回のretainを`refcount += 2`へまとめたが、次の順序は残した。

1. outer Bufferのslotからinner handleをloadする。
2. local resultと`put`用にinner handleを2回retainする。
3. slotの旧handleをreleaseし、zeroならdestructorとfreeを呼ぶ。
4. handleをslotへstoreする。
5. local resultをreleaseし、zeroならdestructorとfreeを呼ぶ。

このsource patternに観測可能な差はなく、除去は言語意味上安全である。しかし現行IRだけからは、outerのelement storageと
innerのrefcount headerが別allocationであること、releaseがouter Bufferのdata pointerを書き換えないこと、retain後に
slotを読み直しても同じhandleであることを証明できない。releaseは間接destructor呼び出しとfreeを持つため、
LLVMが独力でloadとreleaseを並べ替えるのは正しくない。これはBufferの意味ではなく、typed ownershipと
allocation provenanceがraw pointer IRへ消えた結果である。

## 原理的に残るものと改善可能なもの

`get` resultは独立なowned responsibilityである。後続の`put`がslotの古いresponsibilityをdropできるよう、
一般の`get`にはshareが要る。現行`put`はoperandをborrowしてstorage用のresponsibilityをshareし、その後に
旧値をdropする。この順序はself-assignmentでreferentを先にfreeしないために必要である。任意のaliasと
任意の中間操作を認める限り、このreference count operationを一律に除去できない。

一方、次はcostの下限ではない。

- 同じBufferの同じindexに対する、介在effectのない`get`と`put`はtyped compilerで消去できる。
- `Store`とlast-use planにより、再利用されないoperandのresponsibilityを`put`やPool operationへmoveし、
  runtime内のretainとcallerのreleaseを相殺できる。
- Poolの`swap`はslot carrierとoperand carrierをmoveで交換するため、`get`のshareと`put`のretainの組で書くより
  ownership trafficを構造的に避けられる。
- provenanceとlocal alias scopeを保ったLLVM intrinsicまたはmetadataがあれば、data pointerの再loadと一部の
  retain/releaseをbackendで除去できる。任意のmanaged handleをouter storageに対してglobal `noalias`とするのは、
  同じidentityが複数slotに保存され得るため不正である。
- Buffer objectとbacking ownerの二重allocation、header size、growthしない小さなobjectのrepresentationは改善できる。

逆に、すべてのmanaged `put`をmove扱いすること、別の生存responsibilityを証明せずreleaseのzero branchを
消すこと、nested Bufferをdeep-copyやflat payloadへ置き換えることはできない。それぞれsource valueの再利用、
referent lifetime、共有されるinner identityを壊す。growthまたはalias経由のcallbackを跨いで古いslot pointerを
再利用することも、backing relocationとLLVM allocation lifetimeの両方に反する。

## Pool案への含意

この測定は`Buffer<Buffer<T>>`を禁止する理由にはならない。正しくdropでき、原理的なidentityとauthorityを
保ったまま実行できている。見えた差は、次の三層に分けて扱う。

1. source semanticsが要求する独立responsibilityとshared identity。
2. `get` / `put`のAPIが作るshareとdrop。Poolのmove-based `swap`はここを減らす。
3. 現行Buffer representationのallocationと、typed factを失ったLLVM IRの最適化限界。

導入順は、これらを「managedは遅い」と一つにまとめず、まず現行Bufferの`Runtime(_, Owned)`で
representationとlifecycle glueを分け、次に`Store`と`swap`でresponsibility transferを表し、その後に
profileに基づいてcompact representationやbackendのalias factを追加する。
