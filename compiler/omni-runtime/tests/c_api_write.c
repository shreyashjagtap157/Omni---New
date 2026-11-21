#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include "omni-rt.h"

int main(void) {
    printf("=== PLAT-001 C API Test 2: Write/flush/ABI ===\n");

    /* Write bytes to stdout */
    const char *msg = "Hello from C via Omni runtime!\n";
    for (int i = 0; msg[i] != '\0'; i++) {
        int rc = omni_rt_write_stdout_byte((long long)(unsigned char)msg[i]);
        assert(rc == 0);
    }

    /* Flush stdout */
    int rc = omni_rt_flush_stdout();
    assert(rc == 0);

    /* Allocate and verify handle is non-zero */
    omni_rt_handle_t id1 = omni_rt_alloc(100, 16);
    assert(id1 != OMNI_RT_INVALID_ALLOCATION_ID);

    omni_rt_handle_t id2 = omni_rt_alloc(200, 16);
    assert(id2 != OMNI_RT_INVALID_ALLOCATION_ID);
    assert(id2 != id1); /* Different handles for different allocations */

    /* Deallocate both */
    assert(omni_rt_dealloc(id1) == 0);
    assert(omni_rt_dealloc(id2) == 0);

    /* Deallocate invalid handle */
    assert(omni_rt_dealloc(999999) == -1);

    printf("=== PLAT-001 C API Test 2: PASSED ===\n");
    return 0;
}
