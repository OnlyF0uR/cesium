#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "convert.h"

__attribute__((import_module("env"), import_name("h_gen_id")))
extern int64_t h_gen_id();

int32_t initialize() {
    return 0;
}

int32_t create() {
    int64_t big_ptr = h_gen_id();

    // Get the pointer and length from the combined value
    const uint8_t *ptr;
    size_t length;

    unfold_ptr(big_ptr, &ptr, &length);

    // Get the char* from the pointer
    const char *str = (const char *)ptr;

    return 0;
}
