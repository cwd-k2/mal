#include "program.mal.h"

MAL_DEFINE_memory(call) {
    static uint8_t bytes[10];
    return mal_Ptr_return(call, bytes);
}
