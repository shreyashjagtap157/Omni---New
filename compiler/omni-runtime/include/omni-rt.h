#ifndef OMNI_RT_H
#define OMNI_RT_H

#ifdef __cplusplus
extern "C" {
#endif

/* ABI version identity */
#define OMNI_RT_ABI_VERSION 1u

/* Invalid allocation handle (never returned as a live handle) */
#define OMNI_RT_INVALID_ALLOCATION_ID 0ull

/* Opaque allocation handle type */
typedef unsigned long long omni_rt_handle_t;

/* Allocate memory through the Omni runtime.
 * size: allocation size in bytes
 * align: alignment (must be power of two)
 * Returns: allocation handle, or OMNI_RT_INVALID_ALLOCATION_ID on failure */
omni_rt_handle_t omni_rt_alloc(long long size, long long align);

/* Deallocate an opaque runtime allocation.
 * Returns: 0 on success, -1 on failure */
int omni_rt_dealloc(omni_rt_handle_t id);

/* Write one byte to stdout.
 * byte: byte value (0-255)
 * Returns: 0 on success, -1 on host I/O failure */
int omni_rt_write_stdout_byte(long long byte);

/* Flush stdout.
 * Returns: 0 on success, -1 on host I/O failure */
int omni_rt_flush_stdout(void);

/* Terminate with the supplied platform process exit status.
 * Does not return. */
void omni_rt_exit(int code);

/* Terminate immediately under the runtime abort policy.
 * Does not return. */
void omni_rt_abort(void);

#ifdef __cplusplus
}
#endif

#endif /* OMNI_RT_H */
