//! Resource limits for plugins
//!
//! Provides fine-grained resource control for CPU, memory, and I/O operations.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Resource limits configuration
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct ResourceLimits {
    /// Memory limits
    pub memory: MemoryLimits,

    /// CPU limits
    pub cpu: CpuLimits,

    /// I/O limits
    pub io: IoLimits,
}


/// Memory limits
#[derive(Debug, Clone)]
pub struct MemoryLimits {
    /// Maximum heap memory in bytes
    pub max_heap_bytes: usize,

    /// Maximum stack memory in bytes
    pub max_stack_bytes: usize,

    /// Maximum total allocations
    pub max_total_allocations: usize,
}

impl Default for MemoryLimits {
    fn default() -> Self {
        Self {
            max_heap_bytes: 100 * 1024 * 1024, // 100 MB
            max_stack_bytes: 1024 * 1024,  // 1 MB
            max_total_allocations: 10_000,
        }
    }
}

/// CPU limits
#[derive(Debug, Clone)]
pub struct CpuLimits {
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,

    /// Maximum number of instructions
    pub max_instructions: u64,

    /// Maximum CPU time (user + system)
    pub max_cpu_time_ms: u64,
}

impl Default for CpuLimits {
    fn default() -> Self {
        Self {
            max_execution_time_ms: 5000,     // 5 seconds
            max_instructions: 1_000_000_000, // 1 billion
            max_cpu_time_ms: 4000,           // 4 seconds
        }
    }
}

/// I/O limits
#[derive(Debug, Clone)]
pub struct IoLimits {
    /// Maximum network requests
    pub max_network_requests: usize,

    /// Maximum file operations
    pub max_file_operations: usize,

    /// Maximum bytes read
    pub max_read_bytes: usize,

    /// Maximum bytes written
    pub max_write_bytes: usize,

    /// Maximum concurrent connections
    pub max_concurrent_connections: usize,
}

impl Default for IoLimits {
    fn default() -> Self {
        Self {
            max_network_requests: 100,
            max_file_operations: 1000,
            max_read_bytes: 10 * 1024 * 1024,  // 10 MB
            max_write_bytes: 10 * 1024 * 1024, // 10 MB
            max_concurrent_connections: 10,
        }
    }
}

/// Resource usage tracker
#[derive(Debug)]
pub struct ResourceUsage {
    /// Memory usage
    memory_used: AtomicUsize,
    total_allocations: AtomicUsize,

    /// CPU usage
    start_time: RwLock<Option<Instant>>,
    total_instructions: AtomicU64,
    cpu_time_ms: AtomicU64,

    /// I/O usage
    network_requests: AtomicUsize,
    file_operations: AtomicUsize,
    bytes_read: AtomicUsize,
    bytes_written: AtomicUsize,
    concurrent_connections: AtomicUsize,
}

impl ResourceUsage {
    pub fn new() -> Self {
        Self {
            memory_used: AtomicUsize::new(0),
            total_allocations: AtomicUsize::new(0),
            start_time: RwLock::new(None),
            total_instructions: AtomicU64::new(0),
            cpu_time_ms: AtomicU64::new(0),
            network_requests: AtomicUsize::new(0),
            file_operations: AtomicUsize::new(0),
            bytes_read: AtomicUsize::new(0),
            bytes_written: AtomicUsize::new(0),
            concurrent_connections: AtomicUsize::new(0),
        }
    }

    /// Record memory allocation
    pub fn record_allocation(&self, bytes: usize) {
        self.memory_used.fetch_add(bytes, Ordering::SeqCst);
        self.total_allocations.fetch_add(1, Ordering::SeqCst);
    }

    /// Record memory deallocation
    pub fn record_deallocation(&self, bytes: usize) {
        self.memory_used.fetch_sub(bytes, Ordering::SeqCst);
    }

    /// Record instruction execution
    pub fn record_instructions(&self, count: u64) {
        self.total_instructions.fetch_add(count, Ordering::SeqCst);
    }

    /// Record CPU time
    pub fn record_cpu_time(&self, ms: u64) {
        self.cpu_time_ms.fetch_add(ms, Ordering::SeqCst);
    }

    /// Record network request
    pub fn record_network_request(&self) {
        self.network_requests.fetch_add(1, Ordering::SeqCst);
    }

    /// Record file operation
    pub fn record_file_operation(&self) {
        self.file_operations.fetch_add(1, Ordering::SeqCst);
    }

    /// Record bytes read
    pub fn record_read(&self, bytes: usize) {
        self.bytes_read.fetch_add(bytes, Ordering::SeqCst);
    }

    /// Record bytes written
    pub fn record_write(&self, bytes: usize) {
        self.bytes_written.fetch_add(bytes, Ordering::SeqCst);
    }

    /// Increment concurrent connections
    pub fn increment_connections(&self) {
        self.concurrent_connections.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrement concurrent connections
    pub fn decrement_connections(&self) {
        self.concurrent_connections.fetch_sub(1, Ordering::SeqCst);
    }

    /// Start timing
    pub fn start_timing(&self) {
        let mut start_time = self.start_time.write().unwrap();
        *start_time = Some(Instant::now());
    }

    /// Get elapsed time
    pub fn elapsed_time(&self) -> Option<Duration> {
        let start_time = self.start_time.read().unwrap();
        start_time.map(|t| t.elapsed())
    }

    /// Get current memory usage
    pub fn memory_used(&self) -> usize {
        self.memory_used.load(Ordering::SeqCst)
    }

    /// Get total allocations
    pub fn total_allocations(&self) -> usize {
        self.total_allocations.load(Ordering::SeqCst)
    }

    /// Get total instructions
    pub fn total_instructions(&self) -> u64 {
        self.total_instructions.load(Ordering::SeqCst)
    }

    /// Get CPU time
    pub fn cpu_time_ms(&self) -> u64 {
        self.cpu_time_ms.load(Ordering::SeqCst)
    }

    /// Get network requests
    pub fn network_requests(&self) -> usize {
        self.network_requests.load(Ordering::SeqCst)
    }

    /// Get file operations
    pub fn file_operations(&self) -> usize {
        self.file_operations.load(Ordering::SeqCst)
    }

    /// Get bytes read
    pub fn bytes_read(&self) -> usize {
        self.bytes_read.load(Ordering::SeqCst)
    }

    /// Get bytes written
    pub fn bytes_written(&self) -> usize {
        self.bytes_written.load(Ordering::SeqCst)
    }

    /// Get concurrent connections
    pub fn concurrent_connections(&self) -> usize {
        self.concurrent_connections.load(Ordering::SeqCst)
    }

    /// Reset all counters
    pub fn reset(&self) {
        self.memory_used.store(0, Ordering::SeqCst);
        self.total_allocations.store(0, Ordering::SeqCst);
        self.total_instructions.store(0, Ordering::SeqCst);
        self.cpu_time_ms.store(0, Ordering::SeqCst);
        self.network_requests.store(0, Ordering::SeqCst);
        self.file_operations.store(0, Ordering::SeqCst);
        self.bytes_read.store(0, Ordering::SeqCst);
        self.bytes_written.store(0, Ordering::SeqCst);
        self.concurrent_connections.store(0, Ordering::SeqCst);
        let mut start_time = self.start_time.write().unwrap();
        *start_time = None;
    }
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource monitor with enforcement
pub struct ResourceMonitor {
    limits: ResourceLimits,
    usage: Arc<ResourceUsage>,
}

impl ResourceMonitor {
    pub fn new(limits: ResourceLimits) -> Self {
        Self {
            limits,
            usage: Arc::new(ResourceUsage::new()),
        }
    }

    /// Get usage tracker
    pub fn usage(&self) -> Arc<ResourceUsage> {
        self.usage.clone()
    }

    /// Check if within limits
    pub fn check_limits(&self) -> Result<(), ResourceLimitError> {
        // Check memory limits
        if self.usage.memory_used() > self.limits.memory.max_heap_bytes {
            return Err(ResourceLimitError::MemoryLimitExceeded {
                used: self.usage.memory_used(),
                limit: self.limits.memory.max_heap_bytes,
            });
        }

        if self.usage.total_allocations() > self.limits.memory.max_total_allocations {
            return Err(ResourceLimitError::AllocationLimitExceeded {
                count: self.usage.total_allocations(),
                limit: self.limits.memory.max_total_allocations,
            });
        }

        // Check CPU limits
        if let Some(elapsed) = self.usage.elapsed_time() {
            if elapsed.as_millis() as u64 > self.limits.cpu.max_execution_time_ms {
                return Err(ResourceLimitError::ExecutionTimeExceeded {
                    elapsed_ms: elapsed.as_millis() as u64,
                    limit_ms: self.limits.cpu.max_execution_time_ms,
                });
            }
        }

        if self.usage.total_instructions() > self.limits.cpu.max_instructions {
            return Err(ResourceLimitError::InstructionLimitExceeded {
                count: self.usage.total_instructions(),
                limit: self.limits.cpu.max_instructions,
            });
        }

        if self.usage.cpu_time_ms() > self.limits.cpu.max_cpu_time_ms {
            return Err(ResourceLimitError::CpuTimeLimitExceeded {
                used_ms: self.usage.cpu_time_ms(),
                limit_ms: self.limits.cpu.max_cpu_time_ms,
            });
        }

        // Check I/O limits
        if self.usage.network_requests() > self.limits.io.max_network_requests {
            return Err(ResourceLimitError::NetworkRequestLimitExceeded {
                count: self.usage.network_requests(),
                limit: self.limits.io.max_network_requests,
            });
        }

        if self.usage.file_operations() > self.limits.io.max_file_operations {
            return Err(ResourceLimitError::FileOperationLimitExceeded {
                count: self.usage.file_operations(),
                limit: self.limits.io.max_file_operations,
            });
        }

        if self.usage.bytes_read() > self.limits.io.max_read_bytes {
            return Err(ResourceLimitError::ReadLimitExceeded {
                bytes: self.usage.bytes_read(),
                limit: self.limits.io.max_read_bytes,
            });
        }

        if self.usage.bytes_written() > self.limits.io.max_write_bytes {
            return Err(ResourceLimitError::WriteLimitExceeded {
                bytes: self.usage.bytes_written(),
                limit: self.limits.io.max_write_bytes,
            });
        }

        if self.usage.concurrent_connections() > self.limits.io.max_concurrent_connections {
            return Err(ResourceLimitError::ConnectionLimitExceeded {
                count: self.usage.concurrent_connections(),
                limit: self.limits.io.max_concurrent_connections,
            });
        }

        Ok(())
    }

    /// Reset usage counters
    pub fn reset(&self) {
        self.usage.reset();
    }
}

/// Resource limit errors
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceLimitError {
    MemoryLimitExceeded { used: usize, limit: usize },
    AllocationLimitExceeded { count: usize, limit: usize },
    ExecutionTimeExceeded { elapsed_ms: u64, limit_ms: u64 },
    InstructionLimitExceeded { count: u64, limit: u64 },
    CpuTimeLimitExceeded { used_ms: u64, limit_ms: u64 },
    NetworkRequestLimitExceeded { count: usize, limit: usize },
    FileOperationLimitExceeded { count: usize, limit: usize },
    ReadLimitExceeded { bytes: usize, limit: usize },
    WriteLimitExceeded { bytes: usize, limit: usize },
    ConnectionLimitExceeded { count: usize, limit: usize },
}

impl std::fmt::Display for ResourceLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceLimitError::MemoryLimitExceeded { used, limit } => {
                write!(
                    f,
                    "Memory limit exceeded: used {used} bytes, limit {limit} bytes"
                )
            }
            ResourceLimitError::AllocationLimitExceeded { count, limit } => {
                write!(
                    f,
                    "Allocation limit exceeded: {count} allocations, limit {limit}"
                )
            }
            ResourceLimitError::ExecutionTimeExceeded {
                elapsed_ms,
                limit_ms,
            } => {
                write!(
                    f,
                    "Execution time exceeded: {elapsed_ms} ms, limit {limit_ms} ms"
                )
            }
            ResourceLimitError::InstructionLimitExceeded { count, limit } => {
                write!(
                    f,
                    "Instruction limit exceeded: {count} instructions, limit {limit}"
                )
            }
            ResourceLimitError::CpuTimeLimitExceeded { used_ms, limit_ms } => {
                write!(
                    f,
                    "CPU time limit exceeded: {used_ms} ms, limit {limit_ms} ms"
                )
            }
            ResourceLimitError::NetworkRequestLimitExceeded { count, limit } => {
                write!(
                    f,
                    "Network request limit exceeded: {count} requests, limit {limit}"
                )
            }
            ResourceLimitError::FileOperationLimitExceeded { count, limit } => {
                write!(
                    f,
                    "File operation limit exceeded: {count} operations, limit {limit}"
                )
            }
            ResourceLimitError::ReadLimitExceeded { bytes, limit } => {
                write!(
                    f,
                    "Read limit exceeded: {bytes} bytes, limit {limit} bytes"
                )
            }
            ResourceLimitError::WriteLimitExceeded { bytes, limit } => {
                write!(
                    f,
                    "Write limit exceeded: {bytes} bytes, limit {limit} bytes"
                )
            }
            ResourceLimitError::ConnectionLimitExceeded { count, limit } => {
                write!(
                    f,
                    "Connection limit exceeded: {count} connections, limit {limit}"
                )
            }
        }
    }
}

impl std::error::Error for ResourceLimitError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_resource_usage_memory() {
        let usage = ResourceUsage::new();

        usage.record_allocation(1024);
        assert_eq!(usage.memory_used(), 1024);
        assert_eq!(usage.total_allocations(), 1);

        usage.record_allocation(2048);
        assert_eq!(usage.memory_used(), 3072);
        assert_eq!(usage.total_allocations(), 2);

        usage.record_deallocation(1024);
        assert_eq!(usage.memory_used(), 2048);
    }

    #[test]
    fn test_resource_usage_cpu() {
        let usage = ResourceUsage::new();

        usage.record_instructions(1000);
        assert_eq!(usage.total_instructions(), 1000);

        usage.record_cpu_time(100);
        assert_eq!(usage.cpu_time_ms(), 100);
    }

    #[test]
    fn test_resource_usage_io() {
        let usage = ResourceUsage::new();

        usage.record_network_request();
        usage.record_network_request();
        assert_eq!(usage.network_requests(), 2);

        usage.record_file_operation();
        assert_eq!(usage.file_operations(), 1);

        usage.record_read(1024);
        assert_eq!(usage.bytes_read(), 1024);

        usage.record_write(2048);
        assert_eq!(usage.bytes_written(), 2048);
    }

    #[test]
    fn test_resource_usage_connections() {
        let usage = ResourceUsage::new();

        usage.increment_connections();
        usage.increment_connections();
        assert_eq!(usage.concurrent_connections(), 2);

        usage.decrement_connections();
        assert_eq!(usage.concurrent_connections(), 1);
    }

    #[test]
    fn test_resource_usage_timing() {
        let usage = ResourceUsage::new();

        usage.start_timing();
        thread::sleep(Duration::from_millis(10));

        let elapsed = usage.elapsed_time().unwrap();
        assert!(elapsed.as_millis() >= 10);
    }

    #[test]
    fn test_resource_usage_reset() {
        let usage = ResourceUsage::new();

        usage.record_allocation(1024);
        usage.record_instructions(1000);
        usage.record_network_request();

        usage.reset();

        assert_eq!(usage.memory_used(), 0);
        assert_eq!(usage.total_instructions(), 0);
        assert_eq!(usage.network_requests(), 0);
    }

    #[test]
    fn test_resource_monitor_memory_limit() {
        let mut limits = ResourceLimits::default();
        limits.memory.max_heap_bytes = 1024;

        let monitor = ResourceMonitor::new(limits);
        let usage = monitor.usage();

        // Within limit
        usage.record_allocation(512);
        assert!(monitor.check_limits().is_ok());

        // Exceed limit
        usage.record_allocation(1024);
        let result = monitor.check_limits();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResourceLimitError::MemoryLimitExceeded { .. }
        ));
    }

    #[test]
    fn test_resource_monitor_network_limit() {
        let mut limits = ResourceLimits::default();
        limits.io.max_network_requests = 3;

        let monitor = ResourceMonitor::new(limits);
        let usage = monitor.usage();

        // Within limit
        usage.record_network_request();
        usage.record_network_request();
        assert!(monitor.check_limits().is_ok());

        // Exceed limit
        usage.record_network_request();
        usage.record_network_request();
        let result = monitor.check_limits();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResourceLimitError::NetworkRequestLimitExceeded { .. }
        ));
    }

    #[test]
    fn test_resource_monitor_execution_time_limit() {
        let mut limits = ResourceLimits::default();
        limits.cpu.max_execution_time_ms = 50;

        let monitor = ResourceMonitor::new(limits);
        let usage = monitor.usage();

        usage.start_timing();
        thread::sleep(Duration::from_millis(60));

        let result = monitor.check_limits();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResourceLimitError::ExecutionTimeExceeded { .. }
        ));
    }

    #[test]
    fn test_resource_monitor_reset() {
        let monitor = ResourceMonitor::new(ResourceLimits::default());
        let usage = monitor.usage();

        usage.record_allocation(1024);
        usage.record_network_request();

        monitor.reset();

        assert_eq!(usage.memory_used(), 0);
        assert_eq!(usage.network_requests(), 0);
    }

    #[test]
    fn test_default_limits() {
        let limits = ResourceLimits::default();

        assert_eq!(limits.memory.max_heap_bytes, 100 * 1024 * 1024);
        assert_eq!(limits.cpu.max_execution_time_ms, 5000);
        assert_eq!(limits.io.max_network_requests, 100);
    }
}
