//! Minimal Stage-1 runtime support for Omni.
//!
//! The runtime owns process state, deterministic buffered host I/O, and a safe
//! allocation abstraction used by compiler/runtime integration. It intentionally
//! exposes allocation identities rather than raw host pointers so the runtime
//! model cannot forge or silently bypass provenance.

use std::collections::BTreeMap;
use std::fmt;
use std::io::{self, Write};

/// Lifecycle state of an Omni process/runtime instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeState {
    Running,
    Exited(i32),
    Panicked,
    Aborted,
}

impl RuntimeState {
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    pub fn exit_code(&self) -> Option<i32> {
        match self {
            Self::Exited(code) => Some(*code),
            _ => None,
        }
    }
}

/// Opaque identity for runtime-managed storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AllocationId(u64);

impl AllocationId {
    pub fn get(self) -> u64 {
        self.0
    }
}

/// Runtime failures are explicit and deterministic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    NotRunning { state: RuntimeState },
    InvalidAlignment(u32),
    AllocationSizeOverflow { size: usize },
    UnknownAllocation(AllocationId),
    OutOfBounds {
        allocation: AllocationId,
        offset: usize,
        size: usize,
        allocation_size: usize,
    },
    UninitializedRead {
        allocation: AllocationId,
        offset: usize,
        size: usize,
    },
    ImmutableAllocation(AllocationId),
    AlreadyTerminated(RuntimeState),
    HostIo(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRunning { state } => write!(f, "runtime is not running: {state:?}"),
            Self::InvalidAlignment(align) => {
                write!(f, "invalid allocation alignment {align}; expected a positive power of two")
            }
            Self::AllocationSizeOverflow { size } => {
                write!(f, "allocation size {size} cannot be represented safely")
            }
            Self::UnknownAllocation(id) => write!(f, "unknown allocation {}", id.0),
            Self::OutOfBounds { allocation, offset, size, allocation_size } => write!(
                f,
                "allocation {} access {}..{} exceeds size {}",
                allocation.0,
                offset,
                offset.saturating_add(*size),
                allocation_size
            ),
            Self::UninitializedRead { allocation, offset, size } => write!(
                f,
                "allocation {} read {}..{} contains uninitialized bytes",
                allocation.0,
                offset,
                offset.saturating_add(*size)
            ),
            Self::ImmutableAllocation(id) => {
                write!(f, "allocation {} is immutable", id.0)
            }
            Self::AlreadyTerminated(state) => write!(f, "runtime is already terminated: {state:?}"),
            Self::HostIo(message) => write!(f, "host I/O failure: {message}"),
        }
    }
}

impl std::error::Error for RuntimeError {}

#[derive(Debug, Clone)]
struct Allocation {
    bytes: Vec<u8>,
    initialized: Vec<bool>,
    align: u32,
    mutable: bool,
}

/// Minimal deterministic runtime state.
///
/// Host output is buffered first so abstract-machine and test callers can
/// observe exactly what the runtime would emit without depending on host
/// scheduling or terminal buffering.
#[derive(Debug, Clone)]
pub struct Runtime {
    state: RuntimeState,
    allocations: BTreeMap<AllocationId, Allocation>,
    next_allocation: u64,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    /// Construct a fresh running runtime.
    pub fn new() -> Self {
        Self {
            state: RuntimeState::Running,
            allocations: BTreeMap::new(),
            next_allocation: 0,
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    /// Reinitialize process-scope runtime state.
    pub fn startup(&mut self) {
        self.state = RuntimeState::Running;
        self.allocations.clear();
        self.stdout.clear();
        self.stderr.clear();
    }

    pub fn state(&self) -> &RuntimeState {
        &self.state
    }

    pub fn is_running(&self) -> bool {
        self.state.is_running()
    }

    /// Request orderly process termination.
    pub fn exit(&mut self, code: i32) -> Result<(), RuntimeError> {
        if !self.state.is_running() {
            return Err(RuntimeError::AlreadyTerminated(self.state.clone()));
        }
        self.state = RuntimeState::Exited(code);
        self.allocations.clear();
        Ok(())
    }

    /// Enter the runtime panic state.
    pub fn panic(&mut self) -> Result<(), RuntimeError> {
        if !self.state.is_running() {
            return Err(RuntimeError::AlreadyTerminated(self.state.clone()));
        }
        self.state = RuntimeState::Panicked;
        self.allocations.clear();
        Ok(())
    }

    /// Enter the process-abort state.
    pub fn abort(&mut self) -> Result<(), RuntimeError> {
        if !self.state.is_running() {
            return Err(RuntimeError::AlreadyTerminated(self.state.clone()));
        }
        self.state = RuntimeState::Aborted;
        self.allocations.clear();
        Ok(())
    }

    /// Buffer stdout bytes.
    pub fn write_stdout(&mut self, bytes: &[u8]) -> Result<(), RuntimeError> {
        self.ensure_running()?;
        self.stdout.extend_from_slice(bytes);
        Ok(())
    }

    /// Buffer stderr bytes.
    pub fn write_stderr(&mut self, bytes: &[u8]) -> Result<(), RuntimeError> {
        self.ensure_running()?;
        self.stderr.extend_from_slice(bytes);
        Ok(())
    }

    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }

    pub fn take_stdout(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.stdout)
    }

    pub fn take_stderr(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.stderr)
    }

    /// Flush buffered stdout to the host process.
    pub fn flush_stdout(&mut self) -> Result<(), RuntimeError> {
        self.ensure_running()?;
        io::stdout()
            .write_all(&self.stdout)
            .and_then(|_| io::stdout().flush())
            .map_err(|e| RuntimeError::HostIo(e.to_string()))?;
        self.stdout.clear();
        Ok(())
    }

    /// Flush buffered stderr to the host process.
    pub fn flush_stderr(&mut self) -> Result<(), RuntimeError> {
        self.ensure_running()?;
        io::stderr()
            .write_all(&self.stderr)
            .and_then(|_| io::stderr().flush())
            .map_err(|e| RuntimeError::HostIo(e.to_string()))?;
        self.stderr.clear();
        Ok(())
    }

    /// Allocate runtime-managed storage with explicit alignment.
    pub fn allocate(
        &mut self,
        size: usize,
        align: u32,
        mutable: bool,
    ) -> Result<AllocationId, RuntimeError> {
        self.ensure_running()?;
        if align == 0 || !align.is_power_of_two() {
            return Err(RuntimeError::InvalidAlignment(align));
        }

        let initialized = vec![false; size];
        let bytes = vec![0u8; size];
        let id = AllocationId(self.next_allocation);
        self.next_allocation = self.next_allocation.checked_add(1).ok_or(
            RuntimeError::AllocationSizeOverflow { size },
        )?;
        self.allocations.insert(id, Allocation { bytes, initialized, align, mutable });
        Ok(id)
    }

    pub fn deallocate(&mut self, id: AllocationId) -> Result<(), RuntimeError> {
        self.ensure_running()?;
        if self.allocations.remove(&id).is_none() {
            return Err(RuntimeError::UnknownAllocation(id));
        }
        Ok(())
    }

    pub fn allocation_is_live(&self, id: AllocationId) -> bool {
        self.allocations.contains_key(&id)
    }

    pub fn allocation_info(&self, id: AllocationId) -> Result<(usize, u32, bool), RuntimeError> {
        let allocation = self
            .allocations
            .get(&id)
            .ok_or(RuntimeError::UnknownAllocation(id))?;
        Ok((allocation.bytes.len(), allocation.align, allocation.mutable))
    }

    pub fn read(&self, id: AllocationId, offset: usize, size: usize) -> Result<Vec<u8>, RuntimeError> {
        let allocation = self
            .allocations
            .get(&id)
            .ok_or(RuntimeError::UnknownAllocation(id))?;
        Self::checked_range(id, allocation.bytes.len(), offset, size)?;
        if allocation.initialized[offset..offset + size].iter().any(|initialized| !initialized) {
            return Err(RuntimeError::UninitializedRead { allocation: id, offset, size });
        }
        Ok(allocation.bytes[offset..offset + size].to_vec())
    }

    pub fn write(
        &mut self,
        id: AllocationId,
        offset: usize,
        data: &[u8],
    ) -> Result<(), RuntimeError> {
        self.ensure_running()?;
        let allocation = self
            .allocations
            .get_mut(&id)
            .ok_or(RuntimeError::UnknownAllocation(id))?;
        if !allocation.mutable {
            return Err(RuntimeError::ImmutableAllocation(id));
        }
        Self::checked_range(id, allocation.bytes.len(), offset, data.len())?;
        allocation.bytes[offset..offset + data.len()].copy_from_slice(data);
        allocation.initialized[offset..offset + data.len()].fill(true);
        Ok(())
    }

    pub fn is_initialized(
        &self,
        id: AllocationId,
        offset: usize,
    ) -> Result<bool, RuntimeError> {
        let allocation = self
            .allocations
            .get(&id)
            .ok_or(RuntimeError::UnknownAllocation(id))?;
        if offset >= allocation.bytes.len() {
            return Err(RuntimeError::OutOfBounds {
                allocation: id,
                offset,
                size: 1,
                allocation_size: allocation.bytes.len(),
            });
        }
        Ok(allocation.initialized[offset])
    }

    fn ensure_running(&self) -> Result<(), RuntimeError> {
        if self.state.is_running() {
            Ok(())
        } else {
            Err(RuntimeError::NotRunning { state: self.state.clone() })
        }
    }

    fn checked_range(
        id: AllocationId,
        allocation_size: usize,
        offset: usize,
        size: usize,
    ) -> Result<(), RuntimeError> {
        let end = offset.checked_add(size).ok_or(RuntimeError::OutOfBounds {
            allocation: id,
            offset,
            size,
            allocation_size,
        })?;
        if end > allocation_size {
            return Err(RuntimeError::OutOfBounds {
                allocation: id,
                offset,
                size,
                allocation_size,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_is_explicit() {
        let mut runtime = Runtime::new();
        assert!(runtime.is_running());
        runtime.exit(7).unwrap();
        assert_eq!(runtime.state(), &RuntimeState::Exited(7));
        assert_eq!(runtime.exit(8), Err(RuntimeError::AlreadyTerminated(RuntimeState::Exited(7))));
    }

    #[test]
    fn allocation_tracks_initialization_and_mutability() {
        let mut runtime = Runtime::new();
        let id = runtime.allocate(4, 8, true).unwrap();
        assert_eq!(runtime.allocation_info(id).unwrap(), (4, 8, true));
        assert!(matches!(
            runtime.read(id, 0, 1),
            Err(RuntimeError::UninitializedRead { .. })
        ));
        runtime.write(id, 1, &[10, 20]).unwrap();
        assert_eq!(runtime.read(id, 1, 2).unwrap(), vec![10, 20]);
        assert!(!runtime.is_initialized(id, 0).unwrap());
        assert!(runtime.is_initialized(id, 1).unwrap());
    }

    #[test]
    fn immutable_and_bounds_fail_closed() {
        let mut runtime = Runtime::new();
        let id = runtime.allocate(2, 1, false).unwrap();
        assert!(matches!(
            runtime.write(id, 0, &[1]),
            Err(RuntimeError::ImmutableAllocation(_))
        ));
        assert!(matches!(
            runtime.read(id, 1, 2),
            Err(RuntimeError::OutOfBounds { .. })
        ));
    }

    #[test]
    fn buffered_io_is_observable_and_deterministic() {
        let mut runtime = Runtime::new();
        runtime.write_stdout(b"hello").unwrap();
        runtime.write_stderr(b"oops").unwrap();
        assert_eq!(runtime.stdout(), b"hello");
        assert_eq!(runtime.stderr(), b"oops");
        assert_eq!(runtime.take_stdout(), b"hello");
        assert_eq!(runtime.take_stderr(), b"oops");
        assert!(runtime.stdout().is_empty());
        assert!(runtime.stderr().is_empty());
    }
}
