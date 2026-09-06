#include "program.mal.h"

#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    FILE *file;
} DatabaseHandle;

static DatabaseHandle *database_handle(MalType_Database database) {
    return (DatabaseHandle *)mal_Database_bits(database);
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

MAL_DEFINE_openDatabase(context, path) {
    uint64_t length = mal_Symbol_length(path);
    if (length > SIZE_MAX - 1) {
        mal_trap(context, "database path is too long");
    }
    char *terminated = malloc((size_t)length + 1);
    DatabaseHandle *handle = malloc(sizeof(*handle));
    if (terminated == NULL || handle == NULL) {
        free(terminated);
        free(handle);
        mal_trap(context, "database handle allocation failed");
    }
    memcpy(terminated, mal_Symbol_data(path), (size_t)length);
    terminated[length] = '\0';
    handle->file = fopen(terminated, "r+b");
    if (handle->file == NULL && errno == ENOENT) {
        handle->file = fopen(terminated, "w+b");
    }
    free(terminated);
    if (handle->file == NULL) {
        free(handle);
        mal_trap(context, "cannot open database");
    }
    return mal_Database_from_bits((uintptr_t)handle);
}

MAL_DEFINE_readDatabase(context, database, memory, capacity) {
    FILE *file = database_handle(database)->file;
    if (fseek(file, 0, SEEK_SET) != 0) {
        mal_trap(context, "cannot seek database");
    }
    if (capacity > SIZE_MAX) {
        mal_trap(context, "database capacity is too large");
    }
    size_t length = fread(mal_Ptr_address(memory), 1, (size_t)capacity, file);
    if (ferror(file)) {
        clearerr(file);
        mal_trap(context, "cannot read database");
    }
    int trailing = fgetc(file);
    if (trailing != EOF) {
        mal_trap(context, "database exceeds its fixed capacity");
    }
    clearerr(file);
    return (uint64_t)length;
}

MAL_DEFINE_writeDatabase(context, database, memory, length) {
    if (length > SIZE_MAX) {
        mal_trap(context, "database length is too large");
    }
    FILE *file = database_handle(database)->file;
    if (fseek(file, 0, SEEK_SET) != 0) {
        mal_trap(context, "cannot seek database");
    }
    size_t written = fwrite(mal_Ptr_address(memory), 1, (size_t)length, file);
    if (written != (size_t)length || fflush(file) != 0) {
        mal_trap(context, "cannot write database");
    }
}

MAL_DEFINE_closeDatabase(context, database) {
    DatabaseHandle *handle = database_handle(database);
    if (fclose(handle->file) != 0) {
        free(handle);
        mal_trap(context, "cannot close database");
    }
    free(handle);
}

MAL_DEFINE_readLine(context, memory, capacity) {
    if (capacity < 2 || capacity > INT32_MAX) {
        mal_trap(context, "invalid query buffer capacity");
    }
    uint8_t *buffer = mal_Ptr_address(memory);
    if (fgets((char *)buffer, (int)capacity, stdin) == NULL) {
        if (ferror(stdin)) {
            mal_trap(context, "cannot read stdin");
        }
        return UINT64_C(0);
    }
    size_t length = strlen((char *)buffer);
    if (length > 0 && buffer[length - 1] == '\n') {
        --length;
        if (length > 0 && buffer[length - 1] == '\r') {
            --length;
        }
    } else if (!feof(stdin)) {
        mal_trap(context, "query line exceeds its buffer");
    }
    return (uint64_t)length;
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
