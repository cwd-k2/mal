#include "program.mal.h"

#include <stdio.h>

MAL_DEFINE__printInt32(call, value) {
    printf("%d\n", value);
    return mal_Unit_return(call);
}
