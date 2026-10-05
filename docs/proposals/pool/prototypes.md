# 試作で確かめたこと

Status: Current mal v0.7 experiments; historical C-host evidence separated

現在の再現可能な試作は二層ある。`runtime/c11/pool.c`と
[`runtime_pool.rs`](../../../crates/mal-compiler/tests/driver/runtime_pool.rs)は、source constructからまだ選択されず`mal.h`にも公開しない
内部kernelとして、初回採択に必要なrepresentationとlifecycleを検証する。`.scratch/pool-prototype/`は現行Buffer上のmal source
semantic referenceとして、Pool周辺、container、ImPool、Vector候補を検証する。後者はPoolのfrontend admissionやloweringを実装したものではない。

2026-10-05に試作をmal v0.7の前提へ更新した。用語と内部entry pointを`Header`へ揃え、bitmap occupancy、stable shared identity、
growth後のcoordinate保存、zero-stride payload、capacityとallocation sizeのoverflow trapを検査する。managed caseでは、external opaque
相当のtrivial bits、owned child、plain tagを同じspecialization後runtime carrierに置き、HeaderとslotのreadだけがShare、exchangeと
relocationがMove、最後のhandle releaseがHeaderと全Live slotをDropすることを検査する。`-O2 -flto`とAddressSanitizer/UBSanの二構成で
同じharnessをcompile、link、executeする。

このcurrent prototypeはD098のdirect carrier modelと整合するが、Poolをextern signatureへ公開する試作ではない。C harnessはcompilerと
runtimeの内部境界を直接検査する。extern側の前提は、既に採択済みのBuffer、Symbol、aggregate runtime carrierと型別lifecycle glueを
再利用できることに限る。Poolのextern admissionは初回採択から除外する。

## 現行runtime kernelが確かめること

- aliasが同じstable identityを指し、Header交換とslot交換を相互に観測する。
- physical growthがbacking allocationを移しても既存coordinate、occupancy、carrier bitsを保存し、新coordinateをVacantにする。
- runtime carrierのtrivial fieldをそのまま保存し、Owned fieldだけを型別callbackでShare、Dropする。
- Headerとelementのcallback contractを共通化し、read、exchange、relocation、終了のresponsibility回数を区別する。
- terminal trapを持つ四つのoverflow経路と、zero-sized Header / elementでも残るslot stateを検査する。

runtime kernel単独ではfrontend admission、LLVM lowering、`Store` effect、mal sourceからのcontainer実装、ImPool、Vector、Pool carrierのextern
admissionを確かめない。次の実装順は[実装を再開する位置](runtime/implementation.md#実装を再開する位置)を正とする。

## 現行mal source semantic reference

`.scratch/pool-prototype/`はbackend overlayを組み立てず、各`.mal` fileの`require`だけでsource graphを解決する。用語は`Header`、lifetimeは
Engram、element admissionは現行`Storable`に揃え、明示的なPool解放を持たない。IxPoolとImPoolは現行Bufferでemulateするためcost modelには
ならないが、container algorithmとsnapshot semanticsを現行compilerでcheck、build、実行できる。

Vectorはmal内のstructural snapshot候補だけを試し、`Address`、raw-memory admission / observation、extern signatureを持たない。C runtime
extensionの試作はPoolやVectorを公開せず、採択済みのBufferとSymbolをdirect carrierとして使う。`run.nu`はABI 0x000a00のfile headerを
`malc emit header`で生成してからC実装をbuildし、全programをvalgrindで実行する。

## 廃止したC-host試作から残す証拠

2026-09-30から10-03の旧C host backendは現行prototypeに残さない。以下では、現行Buffer emulationでも再現する証拠と、Poolの核とcontainer
algorithmを選んだhistorical evidenceを区別して記録する。旧C host固有のAddress copy境界、host-mappable element制限、明示freeは
D098で廃止され、実装要件ではない。

旧二実装は周辺operationとstack、binary heap、Map、Deque、SlotMap、木を一つのsourceで共有して動かした。containerはIxPoolの核と周辺に
しか依存せず、Buffer emulationではImPoolの常時copy semanticsも検査した。この結果は[検証の段階](runtime/implementation.md#検証の段階)の
step 6から8に対するhistorical evidenceであり、現行kernelのtest結果とは分ける。

ImPoolはBuffer上のemulationだけで実装する。emulationは参照数を観測できないため、更新のたびにcopyする。意味はstorageを
再利用する場合と同じである。`freeze`と`thaw`もcopyする。productionではresponsibility planが一意な最終利用を証明した場合だけstorageを
moveできるが、その情報を観測できないsemantic referenceで共有やmoveを仮定しない。

核はpoolのconstructor `F`をkeyに持つoperation familyとして書き、IxPoolとImPoolで一つの名前を共有した。更新はどれもpoolを
返し、IxPoolは同じidentityへのhandleを、ImPoolはsuccessor snapshotを返す。周辺は`F`の上に一度だけ書け、IxPool上のcontainerとImPool上の
Vectorが同じ周辺を使った。呼び出しは`header<IxPool>(map)`のようにconstructorを明示し、これはopaqueのどの層として見るかの
指定も兼ねる。更新がpoolを返すため、更新で終わる`Unit`のblockには末尾の`()`が要った。この形を書く過程で、kind多相な型parameterを
扱うcompilerの不具合を四つ見つけて直し、本体が求めるkindをspecializationで検査する規則を
[D093](../../history/decisions/active/D093.md)として決めた。

## Bufferの参照実装

[Buffer実装](containers/buffer.md)をBufferと別名の`Buf<T>`としてIxPoolの上に書き、predefinedな`Buffer<UInt64>`と比べた。
同じ擬似乱数列で`new`、`put`、`fill`、重なる範囲の`copy`、別の値からの`copy`を1000回、aliasを通して両方へ適用し、各操作の後に
長さと全要素が一致することを二つの試作で確かめた。`fill`と`copy`は参照実装と同じくslot操作のloopで書き、`copy`はoffsetの大小で
loopの向きを選ぶ。同じBufferで前後どちらへ重なる`copy`も、範囲の一括primitiveなしに現行Bufferと一致した。overflowのtrapは
`trap` primitiveがないため比べていない。

## ImPoolとfreezeとthaw

Buffer上のemulationで、[Vector](containers/sequences.md#vector)の`Vector<T>`をImPoolの上に書いた。更新は前の値を変えず、`freeze`の
後のIxPoolへの書き込みも、`thaw`したIxPoolへの書き込みも、値と他のIxPoolから観測されないことを確かめた。`Symbol`を要素に
しても、valgrindで全allocationの解放とerror 0を確かめた。D096の採択後は`Vector<Vector<T>>`も動かし、外側の更新が内側Vectorを
変更しないことを確かめた。Buffer handleを要素にしたVectorでは、外側snapshotがhandle carrierを保存し、内側identityの変更を
共有観測することも確かめた。
要素を`takeAt`で取り出して更新し、戻す`vectorUpdate`は、外側のplaceにreferenceを残さず
responsibilityを移す更新経路として動いた。内側への別aliasまで排除するものではない。

## constructorについてgenericなcontainer

Buffer上のemulationで、binary heapを核と周辺の返すpoolを引き回す形でconstructor `F`について一度だけ書き、IxPoolとImPoolの
両方で動かした。IxPool上では返り値を捨ててaliasから同じheapを観測でき、ImPool上では途中の値が後のpushとpopで変わらなかった。
擬似乱数の300個をpushしてpopした結果は両方で減少せず、valgrindで全allocationの解放とerror 0を確かめた。handle版とsnapshot版の
containerに別のsourceは要らない（[入れ子](model/identity.md#入れ子)）。snapshot版heap自体の入れ子はまだ確かめていないが、同じ
representationを使うnested Vectorとhandle elementのstructural snapshotはD096採択後の試作で検証した。

## slot遷移とcontainer

- 要素型ごとの実装はstorageの作成、`peek`、`swap`、`header`、`swapHeader`だけで足り、周辺は全て核の上の通常のgeneric関数として
  二つの試作で同じsourceに書けた。
- 核はLiveとVacantのpreconditionを持たない。C host試作でVacantとLiveのどちらのslotで`swap`しても、trapせずに古い値の側を返した。
- `takeAt`により、tombstoneのないMap削除、同じidentityでのrehash、Dequeのring展開、heapの穴を動かすsift、木とSlotMapの
  slot再利用を、要素をcopyせずに書けた。
- C host試作の検査は、containerの`grow`忘れを範囲のpreconditionへの違反として、利用者のprecondition違反を周辺が仮定する
  slot状態への違反としてtrapへ変えた。messageはIxPoolの条件を示し、container operationを示さない。
- Buffer上のemulationで`Symbol`を要素にしたMap、Deque、heap、SlotMapを動かし、valgrindで全allocationの解放とerror 0を
  確かめた。`takeAt`と`initAt`による移動は、managed valueのresponsibilityを一つに保った。

### algorithm corpusによるminimality監査

試作のcontainer本体（Map、Deque、stack、heap、SlotMap、木、Buf）でconstructorを明示したcallを数えると、`takeAt`は12回、
`initAt`は18回、`moveAt`は3回使われた。`slot`と`dropAt`は周辺の定義以外から直接使われない。この頻度はprimitive採用の根拠では
ないが、境界の性質を確認する材料になる。

`takeAt`と`initAt`は、現在のBufferでは表しにくい「responsibilityを複製せずLiveとVacantの間で移す」経路であり、Mapの削除、
Dequeの展開、heapのhole、coordinate再利用の全てが依存する。一方、`moveAt`はその二つの合成で同じ費用になり、`slot`と`dropAt`も
`swap`から失うものがない。従って後三者をtrusted primitiveへ昇格させる根拠はない。利用頻度はlibraryの周辺operationとして名前を
持つ理由にはなるが、意味やruntime authorityを増やさない。

Typical 90の79個のcanonical programでは、現行Bufferの`fill`が148回、`get`が235回、`put`が134回現れ、途中のVacant slotを
直接扱う例はない。これはdense workspaceとsequenceのcorpusであり、Pool試作の代わりにはならない。同時に、全Bufferを
bitmap付きIxPool表現へ一律に置き換える根拠にもならない。代表7題をhandwritten Cと再測定した結果はMal/Cで0.83から1.16であり、
dense表現をas-ifで残す必要を支持する。占有表現、時間、RSSの値は
[Pool占有tagの表現比較](../../history/performance/pool-occupancy.md)に記録する。

## `peek`とsnapshot変換

C host試作の初版ではbyte IxPoolのstorageを`Symbol`相当のtextと共有し、以後の書き込みでcopy-on-writeした。この方式はsnapshotの
意味を満たすが、liveなIxPoolの全ての書き込みにstorageの一意性検査を課す。選択した実装方針では、共有中のIxPoolとsnapshotを
同時に残さず、通常はcopyし、一意な最終利用だけstorageをmoveする。このため試作も両方向の変換をcopyへ改めた。external handleを
使うC hostもBuffer上のemulationもsource responsibilityを観測できず、move可能な場合を測定しない。

`peek`はこのstorage戦略とは独立に、slotを変更しない読み出しである。読み出しを書き込みの組で派生させると、探索ごとのhost callと
lifecycle処理が増える。[区分](api/pool.md#区分)が`peek`を計算量の核に置く理由は、copy-on-writeの有無ではなく、readをslot交換から
独立させることにある。

## 現行Bufferから見た導入境界

現行実装の照合では、`Buffer<Buffer<T>>`に必要なBuffer handleの再帰的retain/releaseとmanaged element callbackが既にあった。
D096はfrontend admissionを開き、alias、上書き、成長、重なるcopy、解放を通すpositive testを追加した。試作でもnested Vectorと
Buffer handle elementが同じruntime-owned pathで動く。

external opaque carrierはlifecycle上はTrivialである。D097でBufferへのstorageを採択し、D098でBuffer elementを含む全値を
specialization後の一つのruntime carrier layoutへ統一した。現行Pool harnessはtrivial bitsとOwned childを同じaggregate carrierへ
置くことで、この分解を直接検査する。詳細は
[compilerとruntimeの実装](runtime/implementation.md#現行bufferから分離する実装境界)を正とする。

旧Vector admission / observation試作はAddressとcanonical copyを前提としていたため、D098後の根拠には使わない。現在のVector案は
mal内のstructural snapshotとしてだけ評価し、C連携は既存BufferまたはSymbol carrierを直接扱うextern operationへ任せる。

## ropeとflatな`Symbol`

同じtextの操作を、AVL木のropeと、現在の`Symbol`のflatなbyte ownerの方式で比べた。flatな方式では、consumingな`+`が一意な
ownerをその場で伸ばす。1.4 MBの行の反転、split、比較、書き出しはflatが4〜12倍速く、ropeが勝ったのは大きなtextの中央への
挿入の反復だけだった。immutableなtext内部のstorage共有はどちらでも成り立ち、Poolとのsnapshot変換とは独立である。
`Symbol`の表現はflatのままでよく、ropeはIxPool上の別containerとして持つ方が合う。

## IxPoolとBufferの非対称

IxPoolはBufferの上に、BufferはIxPoolの上に、どちらも意味の上では書ける。Buffer上のIxPoolはslotごとのsum tagと、`takeAt`ごとの
`Share`と`Drop`を払うが、IxPool上のBufferが払うのは占有tagの更新とIxPool終了時の走査だけである。IxPoolをprimitiveに
するのはこの非対称のためである。
Vacantを末尾だけに限ったdense primitiveへ`[Unit, T]`を載せる形とは意味が同じであり、残る差はこれらのcostだけである。
占有tag単体ではbitmapが最小memoryになり、randomなword payloadでbyte tagよりCとRustの双方で約5–6%速かった。ただしbyte
payloadのRustでは三方式が2%以内、sequential accessでも差は小さい。従ってbitmapはIxPool kernelの初期表現であってsource ABIではなく、
dense BufferとVectorはoccupancyをinvariantから消すas-if representationを使う。

## 試作から出た言語とcompilerの変更

- phantomな型parameterにしか現れない型argumentを推論できなかった。[D092](../../history/decisions/active/D092.md)で、
  constructorでない場合に推論する規則へ改めた。
- phantomなopaque argumentを持つgeneric codeの拒否、同じfileのopaque層をinferenceが見ない問題、diagnosticでopaque applicationの
  `<`が欠ける問題、operation implementationの欠落がkeyを示さない問題を修正した。

## containerの試作
試作では、六つの例を次の条件で動かし、全IxPoolの解放まで確認した。

- stack：2000個をpushし、逆順にpopする。
- Deque：両端へpushして折り返しを作り、両端からpopした後、折り返したまま成長させても順序が保たれることを確かめる。
- binary heap：擬似乱数の2000個をpushし、popの結果が減少しないことと個数を確かめる。
- SlotMap：2000個を挿入し、3個に1個を削除した後、同数を挿入し直す。削除した`SlotKey<T>`はmissingになり、残した`SlotKey<T>`と新しい`SlotKey<T>`は
  値を返す。
- Map：2000個を挿入し、100個を置き換え、1000個を削除した後、全keyの存在と値を確かめる。rehashを含む。
- 木：擬似乱数のkeyを2000個挿入し、半分を削除して、残りの存在、削除したkeyの不在、中順の単調性を確かめる。削除したkeyを
  入れ直してもcapacityは増えない。
