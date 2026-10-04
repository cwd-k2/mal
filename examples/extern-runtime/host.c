#include "program.mal.h"

MAL_DEFINE_sampleStorage(call) {
    mal_Samples_t samples = mal_Buffer_make(call, sizeof(mal_Sample_t), 2);
    const mal_Sample_t first = { .field_0 = 7, .field_1 = 9 };
    const mal_Sample_t second = { .field_0 = 40, .field_1 = 1 };
    mal_Buffer_new(call, samples, &first, sizeof(first));
    mal_Buffer_new(call, samples, &second, sizeof(second));
    return mal_Samples_return(call, samples);
}

MAL_DEFINE_inspectSamples(call, samples) {
    if (mal_Buffer_count(samples) != 2) {
        mal_call_trap(call, "unexpected sample count");
    }
    const mal_Sample_t *values = mal_Buffer_data(samples);
    const int valid = values[0].field_0 == 7
        && values[0].field_1 == 9
        && values[1].field_0 == 41
        && values[1].field_1 == 2;
    return mal_Int32_return(call, valid ? 0 : 1);
}
