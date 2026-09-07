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
        ssize_t result = send(socket, bytes + written, length - written, MSG_NOSIGNAL);
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
        ssize_t result = recv(socket, bytes + received, length - received, 0);
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

static int socket_fd(MAL_TYPE(Socket) socket) {
    uintptr_t bits = MAL_OPERATION(Socket, bits)(socket);
    return bits <= (uintptr_t)INT_MAX ? (int)bits : -1;
}

MAL_DEFINE_createSocketPair(context) {
    int sockets[2];
    if (socketpair(AF_UNIX, SOCK_STREAM, 0, sockets) != 0) {
        mal_trap(context, "cannot create socket pair");
    }
    return MAL_OPERATION(SocketPair, make)(
        MAL_OPERATION(Socket, from_bits)((uintptr_t)sockets[0]),
        MAL_OPERATION(Socket, from_bits)((uintptr_t)sockets[1])
    );
}

MAL_DEFINE_sendPacket(context, socket, packet) {
    int descriptor = socket_fd(socket);
    if (descriptor < 0) {
        return MAL_OPERATION(Status, make_1)((uint32_t)EBADF);
    }

    MAL_TYPE(UInt64) sequence = MAL_OPERATION(Packet, get_0)(packet);
    MAL_TYPE(Symbol) payload = MAL_OPERATION(Packet, get_1)(packet);
    uint64_t length = MAL_OPERATION(Symbol, length)(payload);
    if (length > MAX_PAYLOAD_SIZE) {
        return MAL_OPERATION(Status, make_1)((uint32_t)EMSGSIZE);
    }

    uint8_t header[FRAME_HEADER_SIZE];
    encode_uint64(header, sequence);
    encode_uint64(header + 8, length);
    uint32_t error = write_all(descriptor, header, sizeof(header));
    if (error == 0 && length > 0) {
        error = write_all(
            descriptor,
            MAL_OPERATION(Symbol, data)(payload),
            (size_t)length
        );
    }
    return error == 0
        ? MAL_OPERATION(Status, make_0)()
        : MAL_OPERATION(Status, make_1)(error);
}

MAL_DEFINE_receivePacket(context, socket) {
    int descriptor = socket_fd(socket);
    if (descriptor < 0) {
        return MAL_OPERATION(ReceiveResult, make_1)((uint32_t)EBADF);
    }

    uint8_t header[FRAME_HEADER_SIZE];
    uint32_t error = read_all(descriptor, header, sizeof(header));
    if (error != 0) {
        return MAL_OPERATION(ReceiveResult, make_1)(error);
    }

    uint64_t sequence = decode_uint64(header);
    uint64_t length = decode_uint64(header + 8);
    if (length > MAX_PAYLOAD_SIZE) {
        return MAL_OPERATION(ReceiveResult, make_1)((uint32_t)EMSGSIZE);
    }

    uint8_t payloadBytes[MAX_PAYLOAD_SIZE];
    if (length > 0) {
        error = read_all(descriptor, payloadBytes, (size_t)length);
        if (error != 0) {
            return MAL_OPERATION(ReceiveResult, make_1)(error);
        }
    }

    MAL_TYPE(Symbol) payload = MAL_OPERATION(Symbol, copy_from_bytes)(
        context,
        payloadBytes,
        length
    );
    return MAL_OPERATION(ReceiveResult, make_0)(
        sequence,
        MAL_MOVE(Symbol)(&payload)
    );
}

MAL_DEFINE_closeSocket(context, socket) {
    int descriptor = socket_fd(socket);
    if (descriptor < 0) {
        return MAL_OPERATION(Status, make_1)((uint32_t)EBADF);
    }
    if (close(descriptor) != 0) {
        return MAL_OPERATION(Status, make_1)(current_error());
    }
    return MAL_OPERATION(Status, make_0)();
}

MAL_DEFINE_writeError(context, error) {
    if (fprintf(stderr, "socket error: %" PRIu32 "\n", error) < 0) {
        mal_trap(context, "cannot write socket error");
    }
}
