// convert.h
#ifndef CONVERT_H
#define CONVERT_H

#include <stdint.h>
#include <stddef.h>

// Struct to hold the unpacked values for `unfold_ptrs`
typedef struct {
    uint32_t ptr1;
    uint32_t len1;
    uint32_t ptr2;
    uint32_t len2;
} UnfoldedPtrs;

// Function to unpack a 64-bit integer into a pointer and length
void unfold_ptr(int64_t combined, const uint8_t **ptr, size_t *length);

// Function to unpack a 128-bit integer into four 32-bit values
void unfold_ptrs(__int128 encoded, UnfoldedPtrs *result);

#endif // CONVERT_H
