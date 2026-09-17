# `extern` 境界

Status: Accepted v0.6 profile

## 目的

I/O、allocation、deallocation、filesystem、network、clock、randomness、process、thread、およびhost固有の
resource operationはmalの意味論へ個別に取り込まず、program固有のexternal operationに置く。external storageへの
capabilityは`Address`またはexternal opaque typeで運び、canonical memory representationとの固定された変換には
[external memory](memory.md)と[`Region`と`Packed`](packed.md)を使う。

```mal
extern Mem;
extern alloc :: ByteSize -> Mem;
extern print :: Symbol -> Unit;

output :: Symbol -> Unit := print;
```

external operationは宣言によって通常のtop-level function valueとしてscopeへ入る。呼び出しには通常のapplicationを使い、
値としてbindingしたり引数やresultとして受け渡したりできる。

```mal
main :: Unit -> Unit := () -> {
    mem := alloc(128bytes);
    output("hello");
};
```

function valueの参照、binding、受け渡しだけではhost operationを実行せず、境界transportも起きない。そのfunction valueを
applicationしたときに宣言されたhost operationを一度呼び出す。local bindingが同名のexternal operationをshadowした場合も、
通常のlexical scopeに従う。

external operationはExternに関わるoperationのすべてを表す分類ではなく、program固有のnamed host operationである。
memory operationもExtern-owned storageを観測または変更するが、その表現と評価規則は言語が定め、
host symbolを呼ばない。両者を配置する規則は[authority policy](../design/authority.md#policyとmechanismを分ける)に
定める。

## Host-mappable type

extern declarationのparameter型とresult型は[`HostMappable`](packed.md#hostmappable)を満たさなければならない。aliasはconcreteな
type argumentを代入して完全に展開した後に判定する。このjudgmentはmemory safetyやresource safetyを意味しない。各leafが
admission、observation、capability transferのどれになるかは[EngramとExtern](engrams.md#境界のoperation)に従う。

```text
HostMappable((Address, Count)) = true
HostMappable(Region<UInt8>)    = false
HostMappable(Packed<UInt8>)    = false
```

```mal
extern print :: Symbol -> Unit;
extern choose :: [Int32, Symbol] -> Int32;
```

上の二つはvalidである。次はfunction型を含むためinvalidである。

```mal
extern register :: (Int32 -> Unit) -> Unit;
extern wrapped :: [Unit, Int32 -> Int32] -> Unit;
extern makeCallback :: Unit -> (Int32 -> Int32);
```

この制約はmal内のfirst-class closureを制限しない。callback ABIとhostによるclosure保持はprofile外である。決定理由は[D016](../history/decisions/D016.md)に記録する。

## source-level semantics

external declarationはmal側の型だけを宣言する。applicationでは引数を通常の式と同じく左から右へ評価し、
continuationであるcalleeをその後に評価する。external function valueの評価自体にhostから観測できる作用はない。
host operationが返り、resultのEngram部分のadmissionとExtern capabilityのtransferが完了した後、宣言された型の
mal valueを得たものとして評価を続ける。

mal は effect system を持たず、通常の関数型は pure/impure を区別しない。

```mal
printValue :: Int32 -> Unit := (x) -> {
    printInt32(x);
    ();
};
```

この型は単に `Int32 -> Unit` である。

## host contract

型の宣言だけでは ABI、ownership、lifetime、failure を定義できない。各 backend または embedding は少なくとも次を別途定義しなければならない。

- symbol の名前解決と calling convention
- scalar、product、sum、`Symbol` の表現
- opaque value の size、alignment、copy/drop の意味
- host 側の一時byte bufferの取得方法とcopy後の解放
- host failure を trap、process termination、戻り値のどれへ写像するか
- host が保持してよい引数と、mal が保持してよい戻り値
- result capabilityをmalへtransferするcommit pointと、それ以前にhostが取得した一時resourceのcleanup

## boundary transport

admission、observation、capability transferと各leafのlifetime authorityは
[Engram仕様](engrams.md#境界のoperation)を正とする。この文書はexternの評価と型shapeだけを所有し、backend固有の
carrier、borrow、terminal return、連続表現の準備は[C host ABI](c-host-abi.md)が定める。

unboundedなstreaming inputでは、program固有のexternがAddressとcapacityを受け取ってinitialized prefixのCountを返す。
mal側はRegionを分割し、保持するprefixだけをPackedまたはSymbolへadmitする。

opaque value は copyable/droppable な handle bit pattern として振る舞い、resource の close/free 多重実行を言語は防がない。
決定理由は[D015](../history/decisions/D015.md)に記録する。

host operationがresult capabilityを正常returnする前にtrapするか、capabilityを含まないfailure resultを返す場合、
そのoperationだけが取得し、callerにもresultにも属さない一時resourceはadapterが解放する。正常resultへ含めた
capabilityのtransferはreturn時にcommitする。この規則はargumentとして受け取ったresource、以前のcallでtransfer済みの
resource、またはtrap時の一般的なstack unwindingをcleanupしない。trapし得るruntime helperを呼ぶadapterは、helperより前に
取得した一時resourceを残さない構成にするか、operation固有のcleanup手段を用意する。

Addressを返すoperationは、指すlive region、permission、lifetimeをhost contractに定める。Addressの複製はstorageを複製せず、
lifetimeを延長しない。partial I/Oのpostconditionは[`Region`と`Packed`](packed.md#partial-io)に定める。

## ABI と adapter

`extern` 宣言を任意の C function declaration と同一視しない。特に product、sum、`Symbol` は target ABI によって引数・戻り値の渡し方が異なる。

reference compilerはmal用の一貫したpublic C representationを生成し、必要に応じて手書きまたは生成した小さなC adapterを介して
host APIを呼ぶ。generated wrapperとhost bodyはtrusted computing baseに含まれるが、raw host resourceそのものではない。
runtime contextを一時的に借りてadmissionを依頼できても、Engramのownershipやlifetime authorityは得ない。
C header parserやC type systemはmalに導入しない。

reference C ABIのhost valueとterminal return規約は[C host ABI](c-host-abi.md#host-operation)だけが定める。決定理由は
[D040](../history/decisions/D040.md)に記録する。

reference compilerはprogram固有のC headerを生成する。利用者はそのheaderに対するC sourceを`.mal` fileからrequireする。
symbolはlink時に解決し、runtime `dlopen`やplugin discoveryは行わない。正確なmappingは
[C host ABI](c-host-abi.md)に定める。

## trusted boundary

generated adapterはHostMappable valueのtag、Address、spanなど、C carrierからEngramまたはcapabilityをadmitするために
[C host ABI](c-host-abi.md)が要求するrepresentation validationを行う。region、permission、lifetime、resource identity、
operation固有のpostconditionはhost implementationのcontractが保証する。

host implementationとadapterはこのcontractのtrusted computing baseに含まれる。型検査済みmal programは、contractに反して
早く失効するbuffer、範囲外のAddress、二重解放可能なhandleから保護される保証を持たない。
