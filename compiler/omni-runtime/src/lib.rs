//! Minimal runtime support for Omni's initial native execution vertical slice.
//!
//! The runtime keeps allocation handles opaque at the native boundary. This
//! deliberately avoids exposing host pointers from the safe Rust runtime while
//! providing real process-local storage, initialization tracking, deallocation,
//! minimal stdout, exit, and abort primitives. A later target ABI may replace
//! the handle representation without changing the higher-level Runtime API.

use std::collections::BTreeMap;
use std::fmt;
use std::io::{self, Write};
use std::sync::{Mutex, OnceLock};

/// Version of the initial runtime ABI surface.
pub const RUNTIME_ABI_VERSION: u32 = 1;

/// Zero is never returned as a live allocation identifier.
pub const INVALID_ALLOCATION_ID: AllocationId = 0;

pub type AllocationId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    InvalidSize(usize),
    InvalidAlignment(u32),
    AllocationIdOverflow,
    UnknownAllocation(AllocationId),
    ImmutableAllocation(AllocationId),
    OutOfBounds {
        id: AllocationId,
        offset: usize,
        size: usize,
        allocation_size: usize,
    },
    UninitializedRead {
        id: AllocationId,
        offset: usize,
        size: usize,
    },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSize(size) => write!(f, "invalid allocation size {size}"),
            Self::InvalidAlignment(align) => write!(f, "invalid allocation alignment {align}"),
            Self::AllocationIdOverflow => write!(f, "allocation identifier space exhausted"),
            Self::UnknownAllocation(id) => write!(f, "unknown allocation {id}"),
            Self::ImmutableAllocation(id) => write!(f, "allocation {id} is immutable"),
            Self::OutOfBounds { id, offset, size, allocation_size } => write!(
                f,
                "allocation {id} access [{offset}, {}) exceeds allocation size {allocation_size}",
                offset.saturating_add(*size)
            ),
            Self::UninitializedRead { id, offset, size } => write!(
                f,
                "allocation {id} read [{offset}, {}) touches uninitialized bytes",
                offset.saturating_add(*size)
            ),
        }
    }
}

impl std::error::Error for RuntimeError {}

#[derive(Debug)]
struct Allocation {
    bytes: Vec<u8>,
    initialized: Vec<bool>,
    align: u32,
    mutable: bool,
}

/// Process-local runtime state.
///
/// This is intentionally safe Rust state rather than a direct host-pointer
/// wrapper. Allocation identities are opaque and stable until deallocation.
#[derive(Debug)]
pub struct Runtime {
    next_id: AllocationId,
    allocations: BTreeMap<AllocationId, Allocation>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self { next_id: 1, allocations: BTreeMap::new() }
    }

    pub fn allocate(
        &mut self,
        size: usize,
        align: u32,
        mutable: bool,
    ) -> Result<AllocationId, RuntimeError> {
        validate_alignment(align)?;
        let id = self.next_id;
        self.next_id = id.checked_add(1).ok_or(RuntimeError::AllocationIdOverflow)?;
        self.allocations.insert(
            id,
            Allocation {
                bytes: vec![0; size],
                initialized: vec![false; size],
                align,
                mutable,
            },
        );
        Ok(id)
    }

    pub fn deallocate(&mut self, id: AllocationId) -> Result<(), RuntimeError> {
        self.allocations
            .remove(&id)
            .map(|_| ())
            .ok_or(RuntimeError::UnknownAllocation(id))
    }

    pub fn read(
        &self,
        id: AllocationId,
        offset: usize,
        size: usize,
    ) -> Result<Vec<u8>, RuntimeError> {
        let allocation = self.allocations.get(&id).ok_or(RuntimeError::UnknownAllocation(id))?;
        check_range(id, offset, size, allocation.bytes.len())?;
        if !allocation.initialized[offset..offset + size].iter().all(|initialized| *initialized) {
            return Err(RuntimeError::UninitializedRead { id, offset, size });
        }
        Ok(allocation.bytes[offset..offset + size].to_vec())
    }

    pub fn write(
        &mut self,
        id: AllocationId,
        offset: usize,
        data: &[u8],
    ) -> Result<(), RuntimeError> {
        let allocation =
            self.allocations.get_mut(&id).ok_or(RuntimeError::UnknownAllocation(id))?;
        if !allocation.mutable {
            return Err(RuntimeError::ImmutableAllocation(id));
        }
        check_range(id, offset, data.len(), allocation.bytes.len())?;
        allocation.bytes[offset..offset + data.len()].copy_from_slice(data);
        allocation.initialized[offset..offset + data.len()].fill(true);
        Ok(())
    }

    pub fn is_live(&self, id: AllocationId) -> bool {
        self.allocations.contains_key(&id)
    }

    pub fn allocation_info(&self, id: AllocationId) -> Option<(usize, u32, bool)> {
        self.allocations
            .get(&id)
            .map(|allocation| (allocation.bytes.len(), allocation.align, allocation.mutable))
    }
}

fn validate_alignment(align: u32) -> Result<(), RuntimeError> {
    if align == 0 || !align.is_power_of_two() {
        return Err(RuntimeError::InvalidAlignment(align));
    }
    Ok(())
}

fn check_range(
    id: AllocationId,
    offset: usize,
    size: usize,
    allocation_size: usize,
) -> Result<(), RuntimeError> {
    let end = offset.checked_add(size).ok_or(RuntimeError::OutOfBounds {
        id,
        offset,
        size,
        allocation_size,
    })?;
    if end > allocation_size {
        return Err(RuntimeError::OutOfBounds { id, offset, size, allocation_size });
    }
    Ok(())
}

fn global_runtime() -> &'static Mutex<Runtime> {
    static RUNTIME: OnceLock<Mutex<Runtime>> = OnceLock::new();
    RUNTIME.get_or_init(|| Mutex::new(Runtime::new()))
}

/// Opaque allocation entry point.
///
/// The handle 0 is reserved for failure. Successful handles are positive.
/// This is an internal runtime ABI, not the final published Omni pointer ABI.
#[no_mangle]
pub extern "C" fn omni_rt_alloc(size: i64, align: i64) -> i64 {
    let Ok(size) = usize::try_from(size) else {
        return INVALID_ALLOCATION_ID as i64;
    };
    let Ok(align) = u32::try_from(align) else {
        return INVALID_ALLOCATION_ID as i64;
    };
    let Ok(mut runtime) = global_runtime().lock() else {
        return INVALID_ALLOCATION_ID as i64;
    };
    runtime
        .allocate(size, align, true)
        .ok()
        .and_then(|id| i64::try_from(id).ok())
        .unwrap_or(INVALID_ALLOCATION_ID as i64)
}

/// Deallocate an opaque runtime allocation. Returns zero on success and -1 on failure.
#[no_mangle]
pub extern "C" fn omni_rt_dealloc(id: i64) -> i32 {
    let Ok(id) = u64::try_from(id) else {
        return -1;
    };
    let Ok(mut runtime) = global_runtime().lock() else {
        return -1;
    };
    match runtime.deallocate(id) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Write one byte to stdout. Returns zero on success and -1 on host I/O failure.
#[no_mangle]
pub extern "C" fn omni_rt_write_stdout_byte(byte: i64) -> i32 {
    let Ok(byte) = u8::try_from(byte) else {
        return -1;
    };
    let mut stdout = io::stdout().lock();
    match stdout.write_all(&[byte]).and_then(|_| stdout.flush()) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Flush stdout. Returns zero on success and -1 on host I/O failure.
#[no_mangle]
pub extern "C" fn omni_rt_flush_stdout() -> i32 {
    match io::stdout().lock().flush() {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Terminate with the supplied platform process exit status.
#[no_mangle]
pub extern "C" fn omni_rt_exit(code: i32) -> ! {
    std::process::exit(code)
}

/// Terminate immediately under the runtime abort policy.
#[no_mangle]
pub extern "C" fn omni_rt_abort() -> ! {
    std::process::abort()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_write_read_and_deallocate_are_real() {
        let mut runtime = Runtime::new();
        let id = runtime.allocate(8, 8, true).expect("allocation");
        assert!(runtime.is_live(id));
        assert_eq!(runtime.allocation_info(id), Some((8, 8, true)));
        assert_eq!(
            runtime.read(id, 0, 1),
            Err(RuntimeError::UninitializedRead { id, offset: 0, size: 1 })
        );

        runtime.write(id, 2, &[0xAA, 0x55]).expect("write");
        assert_eq!(runtime.read(id, 2, 2).expect("read"), vec![0xAA, 0x55]);

        runtime.deallocate(id).expect("deallocate");
        assert!(!runtime.is_live(id));
    }

    #[test]
    fn invalid_alignment_is_rejected() {
        let mut runtime = Runtime::new();
        assert_eq!(runtime.allocate(1, 0, true), Err(RuntimeError::InvalidAlignment(0)));
        assert_eq!(runtime.allocate(1, 3, true), Err(RuntimeError::InvalidAlignment(3)));
    }

    #[test]
    fn bounds_and_mutability_are_enforced() {
        let mut runtime = Runtime::new();
        let id = runtime.allocate(4, 4, false).expect("allocation");
        assert_eq!(
            runtime.write(id, 0, &[1]),
            Err(RuntimeError::ImmutableAllocation(id))
        );
        assert!(matches!(
            runtime.read(id, 3, 2),
            Err(RuntimeError::OutOfBounds { id: observed, .. }) if observed == id
        ));
        assert_eq!(runtime.deallocate(id), Ok(()));
        assert_eq!(runtime.deallocate(id), Err(RuntimeError::UnknownAllocation(id)));
    }

    #[test]
    fn zero_sized_allocations_remain_live_and_distinct() {
        let mut runtime = Runtime::new();
        let first = runtime.allocate(0, 1, true).expect("first allocation");
        let second = runtime.allocate(0, 1, true).expect("second allocation");
        assert_ne!(first, second);
        assert!(runtime.is_live(first));
        assert!(runtime.is_live(second));
        assert_eq!(runtime.write(first, 0, &[]), Ok(()));
    }

    #[test]
    fn c_abi_allocation_round_trip_is_real() {
        let id = omni_rt_alloc(16, 8);
        assert!(id > 0);
        assert_eq!(omni_rt_dealloc(id), 0);
        assert_eq!(omni_rt_dealloc(id), -1);
    }

    #[test]
    fn runtime_abi_version_is_explicit() {
        assert_eq!(RUNTIME_ABI_VERSION, 1);
        assert_eq!(INVALID_ALLOCATION_ID, 0);
    }
}
