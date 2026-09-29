#include "program.mal.h"

static uint8_t storage[33];

MAL_DEFINE_sampleStorage(call) {
    return mal_Storage_return(
        call,
        (mal_Storage_t){
            .field_0 = storage + 1,
            .field_1 = 2,
        }
    );
}

MAL_DEFINE_incrementSample(call, value) {
    if (value.field_1 >= 2) {
        mal_call_trap(call, "sample index is out of bounds");
    }
    mal_Sample_t sample = mal_Sample_read(call, value.field_0, value.field_1);
    sample.field_0 += 1;
    sample.field_1 += 1;
    mal_Sample_write(call, value.field_0, value.field_1, sample);
    return mal_Unit_return(call);
}
