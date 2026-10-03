# Pool占有tagの表現比較

Status: Historical measurement record

この文書は、IxPool kernelを実装する前に占有tagの初期表現を選ぶため、2026-10-03にdirect Cで行った比較を記録する。
Poolの意味と現在の実装方針は[意味論](../../proposals/pool/model/semantics.md)と
[compilerとruntimeの実装](../../proposals/pool/runtime/implementation.md)を正とする。

## 比較条件

対象revisionは`febd1031`から`024d82be`、host compilerはClang 21.1.8とRust 1.97.1、CPUは
Intel Core Ultra 7 258Vである。`.scratch/pool-prototype/pool-tag-benchmark.c`と同じ処理をsafe Rustでも書き、Cは
`-O2 -flto -fuse-ld=lld`、Rustは`-C opt-level=3 -C lto=fat -C codegen-units=1 -C panic=abort`でbuildした。
65,536 slotに対する100,000,000回のVacant/Live反転を、各言語と表現を交互に10回実行したmedianを測った。payloadは
`UInt8`相当の1 byteと`UInt64`相当の8 byte、coordinateはxorshiftで散らすrandomと先頭から巡回するsequentialの二種類である。

比較した表現は次の三つである。

| 表現 | tag | payload | `UInt8`の総量 | `UInt64`の総量 |
|---|---|---|---:|---:|
| separate byte | slotごとの1 byte配列 | 別配列 | 128 KiB | 576 KiB |
| bitmap | 8 slotごとの1 byte | 別配列 | 72 KiB | 520 KiB |
| inline sum | payloadと同じstruct | 同じ配列 | 128 KiB | 1,024 KiB |

inline sumのstrideはcanonicalな`[Unit, V]`と同じく`UInt8`で2 byte、`UInt64`で16 byteになる。separate表現の
allocation headerやalignment paddingは表に含めていない。

## Cのwall-clock

| Access | Payload | separate byte | bitmap | inline sum |
|---|---|---:|---:|---:|
| random | `UInt8` | 489.7 ms | 454.5 ms | 469.1 ms |
| random | `UInt64` | 456.9 ms | 434.5 ms | 534.5 ms |
| sequential | `UInt8` | 134.0 ms | 136.1 ms | 132.7 ms |
| sequential | `UInt64` | 133.8 ms | 136.1 ms | 134.2 ms |

randomではbitmapがseparate byteより5–6%速く、inline sumより`UInt8`で約3%、`UInt64`で約19%速かった。
sequentialでは全表現が約133–137 msに収まり、bitmapのread-modify-writeによる明確な不利は見えなかった。

## Rustでの再現

Rust版はCと同じ配列の分け方と`#[repr(C)]`のinline structを使い、同じ状態遷移とchecksumをsafe indexingで実行する。
`Vec`はpayloadもzero-initializeする点がCの`malloc`と異なるが、allocationは各processで一度だけであり、次の値は100,000,000回の
steady-state loopを支配する結果である。

| Access | Payload | separate byte | bitmap | inline sum |
|---|---|---:|---:|---:|
| random | `UInt8` | 475.4 ms | 478.5 ms | 469.7 ms |
| random | `UInt64` | 452.3 ms | 425.8 ms | 526.0 ms |
| sequential | `UInt8` | 133.0 ms | 133.7 ms | 133.9 ms |
| sequential | `UInt64` | 135.6 ms | 136.9 ms | 135.8 ms |

word payloadではbitmapがseparate byteより約6%、inline sumより約19%速く、Cの順位を再現した。byte payloadでは三方式が
2%以内に集まり、bitmapは最速ではなかった。従って「bitmapなら常に最速」とは言えない。一方、C/Rust間で順位が変わっても
大差にはならず、wordのinline sumがstride 16 byteで不利になることと、sequential accessでは表現差がほぼ消えることは共通する。
表現選択をC固有のoptimizer behaviorだけで説明する結果ではない。

## instructionとcache

同じrandom workloadを1,000,000回へ縮め、Cachegrindの48 KiB L1 data cacheで測った。値はprocess全体である。

| Payload | 表現 | Instructions | L1 data read misses | L1 data write misses |
|---|---|---:|---:|---:|
| `UInt8` | separate byte | 18,591,368 | 929,136 | 333,011 |
| `UInt8` | bitmap | 28,288,026 | 186,352 | 195,299 |
| `UInt8` | inline sum | 19,479,343 | 629,265 | 413 |
| `UInt64` | separate byte | 18,107,755 | 1,126,015 | 491,919 |
| `UInt64` | bitmap | 26,804,413 | 465,582 | 476,874 |
| `UInt64` | inline sum | 20,040,975 | 971,047 | 412 |

bitmapはbit抽出とread-modify-writeによりinstructionを約48–52%増やす。一方、8 KiBのtag全体がL1へ収まり、random accessの
read missをseparate byteの20–41%まで減らした。inline sumのwriteは直前のtag readと同じcache lineに載るためwrite missが少ないが、
特にword payloadでは16 byte strideによりworking setが大きく、read missとwall-clockが増えた。

## 判断範囲

初期IxPool kernelは占有tagをpayloadから分離したbitmapとして実装する。これはsource semanticsではなく変更可能なruntime表現である。
bitmapは三方式で最小のmemoryを使い、このrandom workloadでは最速、sequential workloadでも差が小さかったためである。

この測定はmanaged payloadのshare/drop、growth時の二配列の移動、IxPool終了時のcapacity走査、MapやDeque全体のinstructionを含まない。
したがってbitmapを恒久的なABIにはせず、実際のPool loweringでこれらを含むcontainer benchmarkを再測定する。denseなBufferとVectorは
occupancyをcontainer invariantから導くas-if実装を選べるため、このtag costを払う必要はない。

## algorithm corpusとの照合

occupancy kernelだけを良くしても、既存のdense algorithmを悪化させてはならない。Typical 90 corpusから、heap/Dijkstra、明示的stack、
monotonic deque、密なheap/workspace、queue、union-find、hash-tableに相当する7題を選び、最大入力で現行Malと同じalgorithmの
handwritten Cを`-O2 -flto`で10回交互に測った。RSSはGNU `time`を3回実行したmedianであり、process全体を含む。

| workload | Mal | C | Mal/C | Mal RSS | C RSS |
|---|---:|---:|---:|---:|---:|
| heap / Dijkstra | 7.91 ms | 7.92 ms | 1.00 | 6,308 KiB | 7,964 KiB |
| explicit stack / graph | 6.55 ms | 6.34 ms | 1.03 | 6,436 KiB | 6,892 KiB |
| monotonic deque | 6.00 ms | 7.19 ms | 0.83 | 1,952 KiB | 1,952 KiB |
| dense heap / workspace | 246.16 ms | 233.86 ms | 1.05 | 81,572 KiB | 80,672 KiB |
| queue / graph | 6.96 ms | 6.46 ms | 1.08 | 4,864 KiB | 5,664 KiB |
| union-find | 5.53 ms | 5.38 ms | 1.03 | 2,468 KiB | 2,464 KiB |
| hash-table-like matching | 3.13 ms | 2.70 ms | 1.16 | 3,496 KiB | 3,876 KiB |

比は0.83から1.16で、現行Bufferのdense representationがCと同程度の時間で動き、RSSに大きなregressionがないことを確認できる。
RSSはallocatorの要求bytesと実行fileのmappingを分離しないため、representationの正確なmemory量には上のpayload/tag bytesを使う。このcorpusは途中に
Vacantなslotを必要としないためIxPoolの正しさを検証しない。反対にPool試作のMap、Deque、heap、SlotMap、木は`takeAt`による穴の
移動と再利用を検証するが、現時点ではproduction loweringの性能を測れない。両者を分けて保持することで、IxPool導入時には
疎なcontainerの利得とdense algorithmのregressionを同時に判定する。
