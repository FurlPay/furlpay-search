//! L1-L5 memory allocation and page locking tier management.

/// Memory tier classification following the storage-tier performance matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryTier {
    /// L1: CPU Cache — pinned active vectors. 20-60 ns.
    CpuCache,
    /// L2: Resident RAM — memory-locked index (mlock). 100-180 µs.
    ResidentRam,
    /// L3: Warm OS Buffer — OS page cache (mmap). 350-850 µs.
    WarmBuffer,
    /// L4: Cold NVMe Storage — cold page fault I/O. 45-120 ms.
    ColdStorage,
    /// L5: Remote Persistent — object store (S3/GCS). 250-1500 ms.
    RemotePersistent,
}

impl MemoryTier {
    /// Estimated p99 latency in microseconds for this tier.
    pub fn p99_latency_us(&self) -> u64 {
        match self {
            MemoryTier::CpuCache => 0,
            MemoryTier::ResidentRam => 180,
            MemoryTier::WarmBuffer => 850,
            MemoryTier::ColdStorage => 120_000,
            MemoryTier::RemotePersistent => 1_500_000,
        }
    }

    /// Returns true if this tier can satisfy sub-millisecond queries.
    pub fn is_sub_millisecond(&self) -> bool {
        matches!(
            self,
            MemoryTier::CpuCache | MemoryTier::ResidentRam | MemoryTier::WarmBuffer
        )
    }
}

/// Page locking status for a memory region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageLockStatus {
    Locked,
    Cached,
    Cold,
}

/// Configuration for memory tier management.
#[derive(Debug, Clone)]
pub struct MemoryTierConfig {
    pub max_locked_bytes: u64,
    pub use_hugepages: bool,
    pub prefault_on_load: bool,
}

impl Default for MemoryTierConfig {
    fn default() -> Self {
        Self {
            max_locked_bytes: 1024 * 1024 * 1024, // 1 GB
            use_hugepages: cfg!(target_os = "linux"),
            prefault_on_load: true,
        }
    }
}
