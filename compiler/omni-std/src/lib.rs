//! The Omni standard library — core primitives.
//!
//! This crate provides the runtime foundation for Omni's standard library.
//! It is built on top of the Omni runtime C ABI (`omni_rt_*` entry points)
//! exposed by `omni-runtime`. As the Omni codegen gains support for external
//! function calls and the self-hosting compiler matures, this crate will be
//! populated with Omni source modules (.omni) compiled alongside user programs.

#![forbid(unsafe_code)]

use omni_runtime::{omni_rt_exit, omni_rt_write_stdout_byte};

pub use omni_runtime::{AllocationId, RuntimeError, INVALID_ALLOCATION_ID};

/// Version of the standard library ABI surface.
pub const STD_LIB_ABI_VERSION: u32 = 1;

/// Write a single byte to stdout.
pub fn write_byte(byte: u8) -> i32 {
    omni_rt_write_stdout_byte(byte as i64)
}

/// Write a UTF-8 string to stdout.
pub fn write_string(s: &str) {
    for b in s.bytes() {
        let _ = write_byte(b);
    }
}

/// Terminate the process with the given exit status.
pub fn exit(code: i32) -> ! {
    omni_rt_exit(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn std_lib_abi_version_is_explicit() {
        assert_eq!(STD_LIB_ABI_VERSION, 1);
    }

    #[test]
    fn runtime_abi_version_is_propagated() {
        use omni_runtime::RUNTIME_ABI_VERSION;
        assert_eq!(RUNTIME_ABI_VERSION, 1);
    }

    #[test]
    fn invalid_allocation_id_is_zero() {
        assert_eq!(INVALID_ALLOCATION_ID, 0);
    }

    #[test]
    fn write_byte_is_reachable() {
        // The function must exist and be callable; actual host I/O is not
        // asserted in unit tests.
        let _ = write_byte as fn(u8) -> i32;
    }
}
