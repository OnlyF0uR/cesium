#include "convert.h"

void unfold_ptr(int64_t combined, const uint8_t **ptr, size_t *length) {
    *length = (size_t)(combined >> 32); // Extract length from the upper 32 bits
    *ptr = (const uint8_t *)(uintptr_t)(combined & 0xFFFFFFFF); // Extract pointer from the lower 32 bits
}

void unfold_ptrs(__int128 encoded, UnfoldedPtrs *result) {
    result->ptr1 = (uint32_t)(encoded & 0xFFFFFFFF);               // Extract ptr1
    result->len1 = (uint32_t)((encoded >> 32) & 0xFFFFFFFF);       // Extract len1
    result->ptr2 = (uint32_t)((encoded >> 64) & 0xFFFFFFFF);       // Extract ptr2
    result->len2 = (uint32_t)((encoded >> 96) & 0xFFFFFFFF);       // Extract len2
}
