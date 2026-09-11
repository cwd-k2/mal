# LLVM backend生成物例

Status: Illustrative

この文書は[実行backendの責務境界](../design/execution-backend.md)を具体的な生成物へ写した例を示す。identifier、helper signature、
file名はpublic contractではない。現在の責務は[compilerの責務境界](../implementation/responsibilities.md)を正とする。

## 入力例

non-tail recursionではcallee resultを使うまでcallerの`n`を保持する。

```mal
sum :: UInt64 -> UInt64 := (n) {
    if n == 0 {
        0
    } else {
        rest := sum(n - 1);
        n + rest
    }
};
```

backendは概ね次のartifactを構成する。

```text
program.ll          program固有の実行計画
program-shim.c      process entryとextern marshalling
program.mal.h       host向けpublic C interface
runtime C objects   allocation、control storage、Symbol
```

## LLVM module

一つのframe constructorだけを持つregionは固定幅slotを使える。次は意味を示すためerror pathと一部のattributeを省略したIRである。

```llvm
target triple = "..."
target datalayout = "..."

declare ptr @mal_control_reserve_frame(ptr, i64, i64)
declare ptr @mal_control_storage(ptr)

define internal void @mal_sum(ptr %context, i64 %initial_n, ptr %result_out) {
entry:
  br label %descend

descend:
  %n = phi i64 [ %initial_n, %entry ], [ %next_n, %push ]
  %top = phi i64 [ 0, %entry ], [ %next_top, %push ]
  %is_zero = icmp eq i64 %n, 0
  br i1 %is_zero, label %unwind, label %push

push:
  %next_top = add i64 %top, 8
  %storage = call ptr @mal_control_reserve_frame(ptr %context, i64 %top, i64 8)
  %slot = getelementptr i8, ptr %storage, i64 %top
  store i64 %n, ptr %slot, align 8
  %next_n = sub i64 %n, 1
  br label %descend

unwind:
  %result = phi i64 [ 0, %descend ], [ %next_result, %resume ]
  %resume_top = phi i64 [ %top, %descend ], [ %previous_top, %resume ]
  %finished = icmp eq i64 %resume_top, 0
  br i1 %finished, label %done, label %resume

resume:
  %previous_top = sub i64 %resume_top, 8
  %current_storage = call ptr @mal_control_storage(ptr %context)
  %frame = getelementptr i8, ptr %current_storage, i64 %previous_top
  %saved_n = load i64, ptr %frame, align 8
  %next_result = add i64 %result, %saved_n
  br label %unwind

done:
  store i64 %result, ptr %result_out, align 8
  ret void
}
```

`n`、`top`、`result`はSSA valueであり、contextのcontrol storageだけがunboundedなcontinuation storageである。既知のstate遷移は直接`br`し、
first-class calleeなどruntime選択が必要なsiteだけ`switch`する。integerのwrapを保存するoperationへ根拠なく`nsw`または`nuw`を
付けない。

複数constructorを持つregionではsiteごとのLLVM typeを生成する。

```llvm
%frame.site12 = type { i32, i64, i64, %MalSymbol }
%frame.site19 = type { i32, i64, ptr }
```

resume tag、previous offset、fieldの型とowner moveはexecution planが定める。C runtimeはこれらを解釈せず、aligned byte storageの
capacityとgrowthだけを扱う。

## C runtime

control runtimeはframeの意味を知らない汎用C11 implementationにする。

```c
void *mal_control_reserve_frame(
    MalContext *context,
    uint64_t current_bytes,
    uint64_t frame_size
);

void *mal_control_storage(MalContext *context);
```

overflow、allocation、alignment、growth failureはruntimeが検査する。LLVM側はgrowthを伴い得るcall後に古いstorage pointerを
保持せず、返されたbaseからframe addressを再取得する。

## C shim

LLVM moduleはhost-visible aggregateを直接C ABIで渡さない。program rootとextern bridgeはpointerを基本にする。

```c
extern void mal_program_entry(
    void *context,
    const void *argument,
    void *result
);

int main(void) {
    MalContext context = {0};
    int32_t result;
    mal_program_entry(&context, NULL, &result);
    mal_control_destroy(&context);
    return result;
}
```

`(UInt64, Ptr) -> Int32` entryではshimが`argv[1]`以降をpointerとbyte lengthのdescriptor列へ変換し、target data layoutに
従って構成したargument storageを第二parameterへ渡す。いずれのentry形でも第三parameterは`Int32` resultの格納先である。

extern callもLLVMからgenerated C bridgeを呼び、bridgeがpublic host valueへの変換とterminal returnを実行する。host implementationは
`program.mal.h`の`MAL_DEFINE_<name>`だけを使い、LLVM module、bridge signature、runtime carrierを参照しない。
