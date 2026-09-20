#include "program.mal.h"

MAL_DEFINE_unalignedStorage(call) {
    static uint8_t bytes[33];
    return mal_Address_return(call, bytes);
}

MAL_DEFINE_incrementSample(call, address) {
    mal_SampleRecord_t sample = mal_SampleRecord_read(call, address, 0);
    sample.field_0 += 1;
    sample.field_1 += 1;
    mal_SampleRecord_write(call, address, 0, sample);
    return mal_Unit_return(call);
}
