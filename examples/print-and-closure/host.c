#include "program.mal.h"

#include <stdio.h>

MAL_DEFINE_printInt32(context, value) {
    printf("%d\n", value);
}
