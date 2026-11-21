#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include "omni-rt.h"

int main(void) {
    printf("=== PLAT-001 C API Test 1: Basic alloc/dealloc ===\n");

    /* Check ABI version */
    printf("ABI version: %u (expected 1)\n", OMNI_RT_ABI_VERSION);
    assert(OMNI_RT_ABI_VERSION == 1u);

    /* Check invalid handle constant */
    printf("Invalid handle: %llu (expected 0)\n", (unsigned long long)OMNI_RT_INVALID_ALLOCATION_ID);
    assert(OMNI_RT_INVALID_ALLOCATION_ID == 0ull);

    /* Allocate memory */
    omni_rt_handle_t id = omni_rt_alloc(64, 8);
    printf("Allocated handle: %llu\n", (unsigned long long)id);
    assert(id != OMNI_RT_INVALID_ALLOCATION_ID);

    /* Deallocate */
    int rc = omni_rt_dealloc(id);
    printf("Dealloc result: %d (expected 0)\n", rc);
    assert(rc == 0);

    /* Deallocate again should fail */
    rc = omni_rt_dealloc(id);
    printf("Double-dealloc result: %d (expected -1)\n", rc);
    assert(rc == -1);

    /* Allocate invalid size - zero size is valid (returns handle) */
    id = omni_rt_alloc(0, 8);
    printf("Zero-size alloc handle: %llu (valid - zero-size alloc succeeds)\n", (unsigned long long)id);
    assert(id != OMNI_RT_INVALID_ALLOCATION_ID);
    omni_rt_dealloc(id);

    /* Allocate invalid alignment */
    id = omni_rt_alloc(64, 3);
    printf("Bad-align alloc handle: %llu (expected 0)\n", (unsigned long long)id);
    assert(id == OMNI_RT_INVALID_ALLOCATION_ID);

    printf("=== PLAT-001 C API Test 1: PASSED ===\n");
    return 0;
}
