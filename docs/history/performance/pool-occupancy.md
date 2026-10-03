# Pool占有tagの表現比較

Status: Historical measurement record

この文書は、IxPool kernelを実装する前に占有tagの初期表現を選ぶため、2026-10-03にdirect Cで行った比較を記録する。
Poolの意味と現在の実装方針は[意味論](../../proposals/pool/model/semantics.md)と
[compilerとruntimeの実装](../../proposals/pool/runtime/implementation.md)を正とする。

## 比較条件

対象revisionは`febd1031`、host compilerはClang 21.1.8、CPUはIntel Core Ultra 7 258Vである。
`.scratch/pool-prototype/pool-tag-benchmark.c`を`-O2 -flto -fuse-ld=lld`でbuildし、65,536 slotに対する
100,000,000回のVacant/Live反転を10回交互に実行したmedianを測った。payloadは`UInt8`相当の1 byteと`UInt64`相当の8 byte、
coordinateはxorshiftで散らすrandomと先頭から巡回するsequentialの二種類である。

比較した表現は次の三つである。

| 表現 | tag | payload | `UInt8`の総量 | `UInt64`の総量 |
|---|---|---|---:|---:|
| separate byte | slotごとの1 byte配列 | 別配列 | 128 KiB | 576 KiB |
| bitmap | 8 slotごとの1 byte | 別配列 | 72 KiB | 520 KiB |
| inline sum | payloadと同じstruct | 同じ配列 | 128 KiB | 1,024 KiB |

inline sumのstrideはcanonicalな`[Unit, V]`と同じく`UInt8`で2 byte、`UInt64`で16 byteになる。separate表現の
allocation headerやalignment paddingは表に含めていない。

## wall-clock

| Access | Payload | separate byte | bitmap | inline sum |
|---|---|---:|---:|---:|
| random | `UInt8` | 488.1 ms | 456.9 ms | 472.7 ms |
| random | `UInt64` | 455.3 ms | 431.9 ms | 530.4 ms |
| sequential | `UInt8` | 133.5 ms | 134.9 ms | 132.7 ms |
| sequential | `UInt64` | 135.5 ms | 136.7 ms | 136.2 ms |

randomではbitmapがseparate byteより5–6%速く、inline sumより`UInt8`で約3%、`UInt64`で約19%速かった。
sequentialでは全表現が約133–137 msに収まり、bitmapのread-modify-writeによる明確な不利は見えなかった。

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
