#include "program.mal.h"

#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static FILE *file_handle(MalType_File file) {
    return (FILE *)mal_File_bits(file);
}

MAL_DEFINE_allocate(context, size) {
    if (size == 0 || size > SIZE_MAX) {
        mal_trap(context, "invalid allocation size");
    }
    uint8_t *memory = malloc((size_t)size);
    if (memory == NULL) {
        mal_trap(context, "allocation failed");
    }
    return mal_Ptr_from_address(memory);
}

MAL_DEFINE_release(context, memory) {
    free(mal_Ptr_address(memory));
}

MAL_DEFINE_openReadWriteCreate(context, path) {
    uint64_t length = mal_Symbol_length(path);
    if (length > SIZE_MAX - 1) {
        mal_trap(context, "file path is too long");
    }
    if (length > 0 && memchr(mal_Symbol_data(path), '\0', (size_t)length) != NULL) {
        mal_trap(context, "file path contains a null byte");
    }
    char *terminated = malloc((size_t)length + 1);
    if (terminated == NULL) {
        mal_trap(context, "file path allocation failed");
    }
    if (length > 0) {
        memcpy(terminated, mal_Symbol_data(path), (size_t)length);
    }
    terminated[length] = '\0';
    FILE *file = fopen(terminated, "r+b");
    if (file == NULL && errno == ENOENT) {
        file = fopen(terminated, "w+b");
    }
    free(terminated);
    if (file == NULL) {
        mal_trap(context, "cannot open file");
    }
    return mal_File_from_bits((uintptr_t)file);
}

MAL_DEFINE_standardInput(context) {
    return mal_File_from_bits((uintptr_t)stdin);
}

MAL_DEFINE_readFile(context, file, memory, capacity) {
    if (capacity > SIZE_MAX) {
        mal_trap(context, "file read capacity is too large");
    }
    FILE *handle = file_handle(file);
    size_t length = fread(mal_Ptr_address(memory), 1, (size_t)capacity, handle);
    if (ferror(handle)) {
        mal_trap(context, "cannot read file");
    }
    return (uint64_t)length;
}

MAL_DEFINE_writeFile(context, file, memory, length) {
    if (length > SIZE_MAX) {
        mal_trap(context, "file write length is too large");
    }
    FILE *handle = file_handle(file);
    size_t written = fwrite(mal_Ptr_address(memory), 1, (size_t)length, handle);
    if (ferror(handle)) {
        mal_trap(context, "cannot write file");
    }
    return (uint64_t)written;
}

MAL_DEFINE_rewindFile(context, file) {
    if (fseek(file_handle(file), 0, SEEK_SET) != 0) {
        mal_trap(context, "cannot rewind file");
    }
}

MAL_DEFINE_flushFile(context, file) {
    if (fflush(file_handle(file)) != 0) {
        mal_trap(context, "cannot flush file");
    }
}

MAL_DEFINE_closeFile(context, file) {
    if (fclose(file_handle(file)) != 0) {
        mal_trap(context, "cannot close file");
    }
}

MAL_DEFINE_writeSymbol(context, value) {
    uint64_t length = mal_Symbol_length(value);
    if (length > SIZE_MAX
        || fwrite(mal_Symbol_data(value), 1, (size_t)length, stdout) != (size_t)length) {
        mal_trap(context, "cannot write stdout");
    }
}

MAL_DEFINE_writeMemory(context, memory, length) {
    if (length > SIZE_MAX
        || fwrite(mal_Ptr_address(memory), 1, (size_t)length, stdout) != (size_t)length) {
        mal_trap(context, "cannot write stdout");
    }
}

MAL_DEFINE_fail(context, message) {
    if (mal_Symbol_length(message) > 0) {
        fwrite(mal_Symbol_data(message), 1, (size_t)mal_Symbol_length(message), stderr);
        fputc('\n', stderr);
    }
    mal_trap(context, "host rejected the database");
}
