//! Narrow C ABI symbol-export boundary for the safe Omni runtime.
//!
//! The implementation and process-local state live in `omni-runtime`, which
//! inherits the workspace-wide `unsafe_code = "forbid"` policy. This crate
//! contains only C ABI forwarding shims. The `unsafe(no_mangle)` attribute is
//! individually lint-exempted because symbol export is an unsafe linkage
//! contract; no unsafe blocks or unsafe functions are used here.

/// Version of the exported runtime ABI.
pub const RUNTIME_ABI_VERSION: u32 = omni_runtime::RUNTIME_ABI_VERSION;

/// Zero is never returned as a live allocation identifier.
pub const INVALID_ALLOCATION_ID: i64 = omni_runtime::INVALID_ALLOCATION_ID as i64;

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn omni_rt_alloc(size: i64, align: i64) -> i64 {
    omni_runtime::omni_rt_alloc(size, align)
}

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn omni_rt_dealloc(id: i64) -> i32 {
    omni_runtime::omni_rt_dealloc(id)
}

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn omni_rt_write_stdout_byte(byte: i64) -> i32 {
    omni_runtime::omni_rt_write_stdout_byte(byte)
}

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn omni_rt_flush_stdout() -> i32 {
    omni_runtime::omni_rt_flush_stdout()
}

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn omni_rt_exit(code: i32) -> ! {
    omni_runtime::omni_rt_exit(code)
}

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn omni_rt_abort() -> ! {
    omni_runtime::omni_rt_abort()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exported_allocation_symbols_round_trip_through_runtime() {
        let id = omni_rt_alloc(16, 8);
        assert!(id > 0);
        assert_eq!(omni_rt_dealloc(id), 0);
        assert_eq!(omni_rt_dealloc(id), -1);
    }

    #[test]
    fn invalid_allocation_arguments_fail_without_panicking() {
        assert_eq!(omni_rt_alloc(-1, 8), INVALID_ALLOCATION_ID);
        assert_eq!(omni_rt_alloc(8, -1), INVALID_ALLOCATION_ID);
        assert_eq!(omni_rt_alloc(8, 3), INVALID_ALLOCATION_ID);
    }

    #[test]
    fn exported_abi_version_matches_runtime() {
        assert_eq!(RUNTIME_ABI_VERSION, 1);
        assert_eq!(INVALID_ALLOCATION_ID, 0);
    }
}
