#include "program.mal.h"

MAL_DEFINE_sampleStorage(call) {
    mal_type(Samples) samples = mal_buffer(call, mal_storageof(mal_type(Sample)), 2);
    const mal_product(mal_type(Int64), mal_type(UInt8)) first = {
        .field_0 = 7,
        .field_1 = 9,
    };
    const mal_type(Sample) second = { .field_0 = 40, .field_1 = 1 };
    mal_push(call, samples, first);
    mal_push(call, samples, second);
    return mal_move(samples);
}

MAL_DEFINE_inspectSamples(call, samples) {
    if (mal_count(call, samples) != 2) {
        mal_trap(call, "unexpected sample count");
    }
    const mal_type(Sample) *values = mal_data(call, samples);
    const int valid = values[0].field_0 == 7
        && values[0].field_1 == 9
        && values[1].field_0 == 41
        && values[1].field_1 == 2;
    return valid ? 0 : 1;
}
