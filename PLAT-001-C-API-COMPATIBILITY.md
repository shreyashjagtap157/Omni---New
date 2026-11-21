# PLAT-001: C API Compatibility Evaluation

## Status: Evaluated (locally, not platform-qualified)

## C ABI Surface (from `compiler/omni-runtime/src/lib.rs`)

| Symbol | Signature | Description |
|--------|-----------|-------------|
| `omni_rt_alloc` | `(size: i64, align: i64) -> i64` | Allocate memory, returns handle (0 = failure) |
| `omni_rt_dealloc` | `(id: i64) -> i32` | Deallocate, 0 = success, -1 = failure |
| `omni_rt_write_stdout_byte` | `(byte: i64) -> i32` | Write byte to stdout, 0 = success |
| `omni_rt_flush_stdout` | `() -> i32` | Flush stdout, 0 = success |
| `omni_rt_exit` | `(code: i32) -> !` | Terminate process |
| `omni_rt_abort` | `() -> !` | Abort process immediately |

| Constant | Value | Description |
|----------|-------|-------------|
| `RUNTIME_ABI_VERSION` | `1` | Initial runtime ABI surface version |
| `INVALID_ALLOCATION_ID` | `0` | Never returned as live handle |

## Evaluation

### Calling Convention
- C-compatible `extern "C"` functions
- `i64`/`i32` integer parameters and return values
- No variadic functions
- No struct/aggregate parameters or returns (would need target layout metadata)

### Memory Management
- Allocation handles are opaque `u64` identifiers
- Handle 0 reserved for failure
- Handles stable until deallocation
- Deallocation frees the handle for reuse

### Error Boundaries
- Allocation failures return `INVALID_ALLOCATION_ID` (0) without consuming an ID
- Deallocation of unknown ID returns -1
- Write/flush return -1 on host I/O failure
- `exit`/`abort` are noreturn, no error return possible

### Missing for Full C API Compatibility

| Item | Status | Required For |
|------|--------|--------------|
| C header file (`omni-rt.h`) | Missing | Installer SDK directory already creates placeholder |
| Symbol exports documented | Missing | Linker integration guidance |
| Calling convention formally specified | Missing | Cross-compiler compatibility |
| Aggregate/struct ABI layout | Missing | Target-specific, not yet defined |
| Reference/pointer ABI | Missing | Pointer/lifetime representation not defined |
| Enum tagged layout | Missing | Downstream work |
| Error code enumeration | Missing | E-code diagnostics future work |
| Version compatibility matrix | Missing | Single version (1) only |

### Conclusion

The C ABI surface is minimal and intentionally limited. All functions use C-compatible calling convention with integer parameters/returns. No external dependencies are needed for the core C API. The remaining items (aggregate ABI, reference/pointer representation, enum layout) are downstream target-specific decisions, not blockers for the current scope.

**Platform qualification**: Not yet performed (requires target platform testing).
**Local evaluation**: Complete.