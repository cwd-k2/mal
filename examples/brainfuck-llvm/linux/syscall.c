#define _GNU_SOURCE

#include "../program.mal.h"

#include <errno.h>
#include <stdint.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/syscall.h>
#include <unistd.h>

extern long syscall(long number, ...);

static uint32_t system_error(void) {
    return (uint32_t)(errno == 0 ? EIO : errno);
}

MAL_DEFINE_systemMmap(call, request) {
    void *address = request.field_0.tag == mal_OptionalAddress_tag_0
        ? NULL
        : request.field_0.payload.variant_1;
    if (request.field_1 == 0 || request.field_1 > SIZE_MAX || request.field_5 > INT64_MAX) {
        return mal_AddressResult_return_1(call, (uint32_t)EINVAL);
    }
    errno = 0;
    void *memory = (void *)(uintptr_t)syscall(
        SYS_mmap,
        address,
        (size_t)request.field_1,
        request.field_2,
        request.field_3,
        request.field_4,
        (int64_t)request.field_5
    );
    if (memory == MAP_FAILED) {
        return mal_AddressResult_return_1(call, system_error());
    }
    return mal_AddressResult_return_0(call, memory);
}

MAL_DEFINE_systemMremap(call, request) {
    if (request.field_1 == 0
        || request.field_1 > SIZE_MAX
        || request.field_2 == 0
        || request.field_2 > SIZE_MAX) {
        return mal_AddressResult_return_1(call, (uint32_t)EINVAL);
    }
    errno = 0;
    void *memory = (void *)(uintptr_t)syscall(
        SYS_mremap,
        request.field_0,
        (size_t)request.field_1,
        (size_t)request.field_2,
        request.field_3
    );
    if (memory == MAP_FAILED) {
        return mal_AddressResult_return_1(call, system_error());
    }
    return mal_AddressResult_return_0(call, memory);
}

MAL_DEFINE_systemMunmap(call, request) {
    if (request.field_1 == 0 || request.field_1 > SIZE_MAX) {
        return mal_SyscallStatus_return_1(call, (uint32_t)EINVAL);
    }
    errno = 0;
    if (syscall(
            SYS_munmap,
            request.field_0,
            (size_t)request.field_1
        ) != 0) {
        return mal_SyscallStatus_return_1(call, system_error());
    }
    return mal_SyscallStatus_return_0(call);
}

MAL_DEFINE_systemOpenat(call, request) {
    errno = 0;
    long descriptor = syscall(
        SYS_openat,
        request.field_0,
        request.field_1,
        request.field_2,
        request.field_3
    );
    if (descriptor < 0) {
        return mal_FileDescriptorResult_return_1(call, system_error());
    }
    if (descriptor > INT32_MAX) {
        syscall(SYS_close, descriptor);
        return mal_FileDescriptorResult_return_1(call, (uint32_t)EOVERFLOW);
    }
    return mal_FileDescriptorResult_return_0(call, (int32_t)descriptor);
}

MAL_DEFINE_systemLseek(call, request) {
    errno = 0;
    long offset = syscall(SYS_lseek, request.field_0, request.field_1, request.field_2);
    if (offset < 0) {
        return mal_OffsetResult_return_1(call, system_error());
    }
    return mal_OffsetResult_return_0(call, (uint64_t)offset);
}

MAL_DEFINE_systemClose(call, descriptor) {
    errno = 0;
    if (syscall(SYS_close, descriptor) != 0) {
        return mal_SyscallStatus_return_1(call, system_error());
    }
    return mal_SyscallStatus_return_0(call);
}

MAL_DEFINE_systemRead(call, request) {
    if (request.field_2 > SIZE_MAX || request.field_3 > SIZE_MAX) {
        return mal_TransferResult_return_1(call, (uint32_t)EINVAL);
    }
    errno = 0;
    long transferred = syscall(
        SYS_read,
        request.field_0,
        (uint8_t *)request.field_1 + (size_t)request.field_2,
        (size_t)request.field_3
    );
    if (transferred < 0) {
        return mal_TransferResult_return_1(call, system_error());
    }
    return mal_TransferResult_return_0(call, (uint64_t)transferred);
}

MAL_DEFINE_systemWrite(call, request) {
    if (request.field_2 > SIZE_MAX || request.field_3 > SIZE_MAX) {
        return mal_TransferResult_return_1(call, (uint32_t)EINVAL);
    }
    errno = 0;
    long transferred = syscall(
        SYS_write,
        request.field_0,
        (const uint8_t *)request.field_1 + (size_t)request.field_2,
        (size_t)request.field_3
    );
    if (transferred < 0) {
        return mal_TransferResult_return_1(call, system_error());
    }
    return mal_TransferResult_return_0(call, (uint64_t)transferred);
}

MAL_DEFINE_storeZero(call, value) {
    if (value.field_1 > SIZE_MAX) {
        mal_call_trap(call, "invalid byte offset");
    }
    ((uint8_t *)value.field_0)[(size_t)value.field_1] = 0;
    return mal_Unit_return(call);
}

MAL_DEFINE_argumentLength(call, address) {
    return mal_USize_return(call, strlen((const char *)address));
}
