#include "program.mal.h"

MAL_DEFINE_memory(context) {
    static uint8_t bytes[10];
    return mal_Ptr_from_address(bytes);
}
