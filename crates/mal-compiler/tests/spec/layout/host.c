#include "program.mal.h"
#include <string.h>

static unsigned char memory[256];

MAL_DEFINE_area(call) { return mal_Address_return(call, memory); }

/* Compare only bytes that carry data; padding is unspecified. */
static void expect(mal_call_t *call, const char *what, const unsigned char *bytes, const int *mask, size_t n) {
    for (size_t i = 0; i < n; i++) {
        if (mask[i] && memory[i] != bytes[i]) {
            mal_call_trap(call, what);
        }
    }
}

MAL_DEFINE_verifyWritten(call, which) {
    switch (which) {
    case 1: { /* (UInt8, Int32): stride 8, offsets 0 and 4 */
        unsigned char e[16] = {1,0,0,0, 4,3,2,1, 2,0,0,0, 0,0,0,0};
        int m[16] = {1,0,0,0, 1,1,1,1, 1,0,0,0, 1,1,1,1};
        expect(call, "T1 layout", e, m, 16);
        break; }
    case 2: { /* [Unit, UInt8]: tag UInt8 at 0, payload at 1, stride 2 */
        unsigned char e[4] = {1,171, 1,1};
        int m[4] = {1,1, 1,1};
        expect(call, "T2 layout", e, m, 4);
        break; }
    case 3: { /* [Int32, Int64]: tag at 0, payload aligned to 8, stride 16 */
        unsigned char e[32] = {1,0,0,0,0,0,0,0, 0x88,0x77,0x66,0x55,0x44,0x33,0x22,0x11,
                               1,0,0,0,0,0,0,0, 1,0,0,0,0,0,0,0};
        int m[32] = {1,0,0,0,0,0,0,0, 1,1,1,1,1,1,1,1, 1,0,0,0,0,0,0,0, 1,1,1,1,1,1,1,1};
        expect(call, "T3 layout", e, m, 32);
        break; }
    case 4: { /* ((UInt8, Int32), UInt8): nested product not flattened, stride 12 */
        unsigned char e[24] = {5,0,0,0, 6,0,0,0, 7,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0};
        int m[24] = {1,0,0,0, 1,1,1,1, 1,0,0,0, 1,0,0,0, 1,1,1,1, 1,0,0,0};
        expect(call, "T4 layout", e, m, 24);
        break; }
    case 5: { /* Bool: [Unit, Unit], one byte tag */
        unsigned char e[2] = {1, 0};
        int m[2] = {1, 1};
        expect(call, "Bool layout", e, m, 2);
        break; }
    case 6: { /* 257 variants: variant 256 with a UInt16 tag */
        unsigned char e[2] = {0x00, 0x01};
        int m[2] = {1, 1};
        expect(call, "Wide tag width", e, m, 2);
        break; }
    }
    return mal_Unit_return(call);
}

MAL_DEFINE_prepare(call, which) {
    memset(memory, 0, sizeof memory);
    if (which == 1) { /* two (UInt8, Int32) elements: (3, 4) and (9, -2) */
        unsigned char e[16] = {3,0,0,0, 4,0,0,0, 9,0,0,0, 0xfe,0xff,0xff,0xff};
        memcpy(memory, e, 16);
    } else { /* two [Unit, UInt8] elements: variant 0, then variant 1 with 200 */
        unsigned char e[4] = {0,0, 1,200};
        memcpy(memory, e, 4);
    }
    return mal_Unit_return(call);
}
