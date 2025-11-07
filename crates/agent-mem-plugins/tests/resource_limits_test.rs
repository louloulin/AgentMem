//! Resource limits integration tests

use agent_mem_plugins::security::{
    CpuLimits, IoLimits, MemoryLimits, ResourceLimitError, ResourceLimits, ResourceMonitor,
    ResourceUsage,
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_comprehensive_memory_tracking() {
    let usage = ResourceUsage::new();

    // Simulate memory allocations
    usage.record_allocation(1024); // 1 KB
    usage.record_allocation(2048); // 2 KB
    usage.record_allocation(4096); // 4 KB

    assert_eq!(usage.memory_used(), 7168); // 7 KB total
    assert_eq!(usage.total_allocations(), 3);

    // Simulate deallocations
    usage.record_deallocation(1024);
    assert_eq!(usage.memory_used(), 6144); // 6 KB remaining

    // Allocations count doesn't decrease on deallocation
    assert_eq!(usage.total_allocations(), 3);
}

#[test]
fn test_cpu_instruction_tracking() {
    let usage = ResourceUsage::new();

    // Simulate instruction execution
    usage.record_instructions(1000);
    usage.record_instructions(2000);
    usage.record_instructions(3000);

    assert_eq!(usage.total_instructions(), 6000);

    // Simulate CPU time
    usage.record_cpu_time(100); // 100 ms
    usage.record_cpu_time(200); // 200 ms

    assert_eq!(usage.cpu_time_ms(), 300);
}

#[test]
fn test_io_operations_tracking() {
    let usage = ResourceUsage::new();

    // Network requests
    for _ in 0..5 {
        usage.record_network_request();
    }
    assert_eq!(usage.network_requests(), 5);

    // File operations
    for _ in 0..3 {
        usage.record_file_operation();
    }
    assert_eq!(usage.file_operations(), 3);

    // Bytes read/written
    usage.record_read(1024);
    usage.record_read(2048);
    assert_eq!(usage.bytes_read(), 3072);

    usage.record_write(512);
    usage.record_write(1024);
    assert_eq!(usage.bytes_written(), 1536);
}

#[test]
fn test_connection_management() {
    let usage = ResourceUsage::new();

    // Open connections
    usage.increment_connections();
    usage.increment_connections();
    usage.increment_connections();
    assert_eq!(usage.concurrent_connections(), 3);

    // Close connections
    usage.decrement_connections();
    assert_eq!(usage.concurrent_connections(), 2);

    usage.decrement_connections();
    usage.decrement_connections();
    assert_eq!(usage.concurrent_connections(), 0);
}

#[test]
fn test_resource_monitor_strict_memory_limit() {
    let mut limits = ResourceLimits::default();
    limits.memory.max_heap_bytes = 5000; // 5 KB limit

    let monitor = ResourceMonitor::new(limits);
    let usage = monitor.usage();

    // Test gradual memory consumption
    usage.record_allocation(1000);
    assert!(monitor.check_limits().is_ok());

    usage.record_allocation(2000);
    assert!(monitor.check_limits().is_ok());

    usage.record_allocation(1500);
    assert!(monitor.check_limits().is_ok());

    // This should exceed the limit
    usage.record_allocation(1000);
    let result = monitor.check_limits();
    assert!(result.is_err());

    match result.unwrap_err() {
        ResourceLimitError::MemoryLimitExceeded { used, limit } => {
            assert_eq!(used, 5500);
            assert_eq!(limit, 5000);
        }
        _ => panic!("Expected MemoryLimitExceeded error"),
    }
}

#[test]
fn test_resource_monitor_allocation_count_limit() {
    let mut limits = ResourceLimits::default();
    limits.memory.max_total_allocations = 5;

    let monitor = ResourceMonitor::new(limits);
    let usage = monitor.usage();

    // Make 5 allocations (within limit)
    for _ in 0..5 {
        usage.record_allocation(100);
    }
    assert!(monitor.check_limits().is_ok());

    // 6th allocation should fail
    usage.record_allocation(100);
    let result = monitor.check_limits();
    assert!(result.is_err());

    match result.unwrap_err() {
        ResourceLimitError::AllocationLimitExceeded { count, limit } => {
            assert_eq!(count, 6);
            assert_eq!(limit, 5);
        }
        _ => panic!("Expected AllocationLimitExceeded error"),
    }
}

#[test]
fn test_resource_monitor_instruction_limit() {
    let mut limits = ResourceLimits::default();
    limits.cpu.max_instructions = 10000;

    let monitor = ResourceMonitor::new(limits);
    let usage = monitor.usage();

    // Within limit
    usage.record_instructions(5000);
    assert!(monitor.check_limits().is_ok());

    // Exceed limit
    usage.record_instructions(6000);
    let result = monitor.check_limits();
    assert!(result.is_err());

    match result.unwrap_err() {
        ResourceLimitError::InstructionLimitExceeded { count, limit } => {
            assert_eq!(count, 11000);
            assert_eq!(limit, 10000);
        }
        _ => panic!("Expected InstructionLimitExceeded error"),
    }
}

#[test]
fn test_resource_monitor_cpu_time_limit() {
    let mut limits = ResourceLimits::default();
    limits.cpu.max_cpu_time_ms = 200;

    let monitor = ResourceMonitor::new(limits);
    let usage = monitor.usage();

    // Within limit
    usage.record_cpu_time(100);
    assert!(monitor.check_limits().is_ok());

    // Exceed limit
    usage.record_cpu_time(150);
    let result = monitor.check_limits();
    assert!(result.is_err());

    match result.unwrap_err() {
        ResourceLimitError::CpuTimeLimitExceeded { used_ms, limit_ms } => {
            assert_eq!(used_ms, 250);
            assert_eq!(limit_ms, 200);
        }
        _ => panic!("Expected CpuTimeLimitExceeded error"),
    }
}

#[test]
fn test_resource_monitor_io_limits() {
    let mut limits = ResourceLimits::default();
    limits.io.max_network_requests = 3;
    limits.io.max_file_operations = 5;
    limits.io.max_read_bytes = 1024;
    limits.io.max_write_bytes = 2048;

    let monitor = ResourceMonitor::new(limits);
    let usage = monitor.usage();

    // Test network request limit
    usage.record_network_request();
    usage.record_network_request();
    usage.record_network_request();
    assert!(monitor.check_limits().is_ok());

    usage.record_network_request();
    assert!(matches!(
        monitor.check_limits().unwrap_err(),
        ResourceLimitError::NetworkRequestLimitExceeded { .. }
    ));

    // Reset and test file operation limit
    monitor.reset();

    for _ in 0..5 {
        usage.record_file_operation();
    }
    assert!(monitor.check_limits().is_ok());

    usage.record_file_operation();
    assert!(matches!(
        monitor.check_limits().unwrap_err(),
        ResourceLimitError::FileOperationLimitExceeded { .. }
    ));

    // Reset and test read limit
    monitor.reset();

    usage.record_read(1024);
    assert!(monitor.check_limits().is_ok());

    usage.record_read(1);
    assert!(matches!(
        monitor.check_limits().unwrap_err(),
        ResourceLimitError::ReadLimitExceeded { .. }
    ));

    // Reset and test write limit
    monitor.reset();

    usage.record_write(2048);
    assert!(monitor.check_limits().is_ok());

    usage.record_write(1);
    assert!(matches!(
        monitor.check_limits().unwrap_err(),
        ResourceLimitError::WriteLimitExceeded { .. }
    ));
}

#[test]
fn test_resource_monitor_connection_limit() {
    let mut limits = ResourceLimits::default();
    limits.io.max_concurrent_connections = 3;

    let monitor = ResourceMonitor::new(limits);
    let usage = monitor.usage();

    // Within limit
    usage.increment_connections();
    usage.increment_connections();
    usage.increment_connections();
    assert!(monitor.check_limits().is_ok());

    // Exceed limit
    usage.increment_connections();
    let result = monitor.check_limits();
    assert!(result.is_err());

    match result.unwrap_err() {
        ResourceLimitError::ConnectionLimitExceeded { count, limit } => {
            assert_eq!(count, 4);
            assert_eq!(limit, 3);
        }
        _ => panic!("Expected ConnectionLimitExceeded error"),
    }

    // Decrease connection should bring us back within limit
    usage.decrement_connections();
    assert!(monitor.check_limits().is_ok());
}

#[test]
fn test_execution_time_tracking() {
    let usage = ResourceUsage::new();

    usage.start_timing();
    thread::sleep(Duration::from_millis(50));

    let elapsed = usage.elapsed_time().unwrap();
    assert!(elapsed.as_millis() >= 50);
    assert!(elapsed.as_millis() < 100); // Should be less than 100ms
}

#[test]
fn test_concurrent_resource_usage() {
    let usage = Arc::new(ResourceUsage::new());
    let mut handles = vec![];

    // Spawn 10 threads, each recording allocations
    for i in 0..10 {
        let usage_clone = usage.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                usage_clone.record_allocation(i * 10);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Total allocations should be 10 threads * 100 allocations = 1000
    assert_eq!(usage.total_allocations(), 1000);

    // Total memory = sum of (i * 10 * 100) for i from 0 to 9
    // = (0 + 1 + 2 + ... + 9) * 10 * 100
    // = 45 * 1000 = 45000
    assert_eq!(usage.memory_used(), 45000);
}

#[test]
fn test_resource_limits_display() {
    let error = ResourceLimitError::MemoryLimitExceeded {
        used: 1024,
        limit: 512,
    };
    let message = format!("{}", error);
    assert!(message.contains("Memory limit exceeded"));
    assert!(message.contains("1024"));
    assert!(message.contains("512"));
}

#[test]
fn test_default_resource_limits_reasonable() {
    let limits = ResourceLimits::default();

    // Memory limits should be reasonable
    assert_eq!(limits.memory.max_heap_bytes, 100 * 1024 * 1024); // 100 MB
    assert_eq!(limits.memory.max_stack_bytes, 1 * 1024 * 1024); // 1 MB
    assert_eq!(limits.memory.max_total_allocations, 10_000);

    // CPU limits should be reasonable
    assert_eq!(limits.cpu.max_execution_time_ms, 5000); // 5 seconds
    assert_eq!(limits.cpu.max_instructions, 1_000_000_000);
    assert_eq!(limits.cpu.max_cpu_time_ms, 4000);

    // I/O limits should be reasonable
    assert_eq!(limits.io.max_network_requests, 100);
    assert_eq!(limits.io.max_file_operations, 1000);
    assert_eq!(limits.io.max_read_bytes, 10 * 1024 * 1024); // 10 MB
    assert_eq!(limits.io.max_write_bytes, 10 * 1024 * 1024); // 10 MB
    assert_eq!(limits.io.max_concurrent_connections, 10);
}

#[test]
fn test_custom_resource_limits() {
    let limits = ResourceLimits {
        memory: MemoryLimits {
            max_heap_bytes: 1024,
            max_stack_bytes: 512,
            max_total_allocations: 10,
        },
        cpu: CpuLimits {
            max_execution_time_ms: 100,
            max_instructions: 1000,
            max_cpu_time_ms: 80,
        },
        io: IoLimits {
            max_network_requests: 5,
            max_file_operations: 10,
            max_read_bytes: 512,
            max_write_bytes: 256,
            max_concurrent_connections: 2,
        },
    };

    let monitor = ResourceMonitor::new(limits);

    // Verify custom limits are enforced
    let usage = monitor.usage();

    usage.record_allocation(2000);
    assert!(matches!(
        monitor.check_limits().unwrap_err(),
        ResourceLimitError::MemoryLimitExceeded { .. }
    ));
}
