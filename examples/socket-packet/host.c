#include "program.mal.h"

#include <errno.h>
#include <inttypes.h>
#include <limits.h>
#include <stdint.h>
#include <stdio.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>

enum {
    FRAME_HEADER_SIZE = 16,
    MAX_PAYLOAD_SIZE = 256,
};

static uint32_t current_error(void) {
    return errno == 0 ? (uint32_t)EIO : (uint32_t)errno;
}

static void encode_uint64(uint8_t *bytes, uint64_t value) {
    for (size_t index = 0; index < 8; ++index) {
        bytes[index] = (uint8_t)(value >> (56 - index * 8));
    }
}

static uint64_t decode_uint64(const uint8_t *bytes) {
    uint64_t value = UINT64_C(0);
    for (size_t index = 0; index < 8; ++index) {
        value = (value << 8) | bytes[index];
    }
    return value;
}

static uint32_t write_all(int socket, const uint8_t *bytes, size_t length) {
    size_t written = 0;
    while (written < length) {
        ssize_t result = write(socket, bytes + written, length - written);
        if (result < 0 && errno == EINTR) {
            continue;
        }
        if (result < 0) {
            return current_error();
        }
        if (result == 0) {
            return (uint32_t)EPIPE;
        }
        written += (size_t)result;
    }
    return UINT32_C(0);
}

static uint32_t read_all(int socket, uint8_t *bytes, size_t length) {
    size_t received = 0;
    while (received < length) {
        ssize_t result = read(socket, bytes + received, length - received);
        if (result < 0 && errno == EINTR) {
            continue;
        }
        if (result < 0) {
            return current_error();
        }
        if (result == 0) {
            return (uint32_t)EPIPE;
        }
        received += (size_t)result;
    }
    return UINT32_C(0);
}

static int socket_fd(mal_Socket_t socket) {
    uintptr_t bits = mal_Socket_to_bits(socket);
    return bits <= (uintptr_t)INT_MAX ? (int)bits : -1;
}

MAL_DEFINE_createSocketPair(call) {
    int sockets[2];
    if (socketpair(AF_UNIX, SOCK_STREAM, 0, sockets) != 0) {
        mal_call_trap(call, "cannot create socket pair");
    }
    return mal_SocketPair_return(
        call,
        (mal_SocketPair_t){
            .field_0 = mal_Socket_from_bits((uintptr_t)sockets[0]),
            .field_1 = mal_Socket_from_bits((uintptr_t)sockets[1]),
        }
    );
}

MAL_DEFINE_sendPacket(call, value) {
    int descriptor = socket_fd(value.field_0);
    if (descriptor < 0) {
        return mal_Status_return_1(call, (uint32_t)EBADF);
    }

    uint64_t sequence = value.field_1.field_0;
    mal_span_t payload = mal_Symbol_to_bytes(call, value.field_1.field_1);
    uint64_t length = payload.length;
    if (length > MAX_PAYLOAD_SIZE) {
        return mal_Status_return_1(call, (uint32_t)EMSGSIZE);
    }

    uint8_t header[FRAME_HEADER_SIZE];
    encode_uint64(header, sequence);
    encode_uint64(header + 8, length);
    uint32_t error = write_all(descriptor, header, sizeof(header));
    if (error == 0 && length > 0) {
        error = write_all(
            descriptor,
            payload.data,
            (size_t)length
        );
    }
    return error == 0
        ? mal_Status_return_0(call)
        : mal_Status_return_1(call, error);
}

MAL_DEFINE_receivePacket(call, socket) {
    int descriptor = socket_fd(socket);
    if (descriptor < 0) {
        return mal_ReceiveResult_return_1(call, (uint32_t)EBADF);
    }

    uint8_t header[FRAME_HEADER_SIZE];
    uint32_t error = read_all(descriptor, header, sizeof(header));
    if (error != 0) {
        return mal_ReceiveResult_return_1(call, error);
    }

    uint64_t sequence = decode_uint64(header);
    uint64_t length = decode_uint64(header + 8);
    if (length > MAX_PAYLOAD_SIZE) {
        return mal_ReceiveResult_return_1(call, (uint32_t)EMSGSIZE);
    }

    uint8_t payload[MAX_PAYLOAD_SIZE];
    if (length > 0) {
        error = read_all(descriptor, payload, (size_t)length);
        if (error != 0) {
            return mal_ReceiveResult_return_1(call, error);
        }
    }

    return mal_ReceiveResult_return_0(
        call,
        (mal_Packet_t){
            .field_0 = sequence,
            .field_1 = mal_Symbol_from_bytes(
                (mal_span_t){ .data = payload, .length = length }
            ),
        }
    );
}

MAL_DEFINE_closeSocket(call, socket) {
    int descriptor = socket_fd(socket);
    if (descriptor < 0) {
        return mal_Status_return_1(call, (uint32_t)EBADF);
    }
    if (close(descriptor) != 0) {
        return mal_Status_return_1(call, current_error());
    }
    return mal_Status_return_0(call);
}

MAL_DEFINE_writeError(call, error) {
    if (fprintf(stderr, "socket error: %" PRIu32 "\n", error) < 0) {
        mal_call_trap(call, "cannot write socket error");
    }
    return mal_Unit_return(call);
}
