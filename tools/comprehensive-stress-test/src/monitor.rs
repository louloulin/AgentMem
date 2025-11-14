//! 系统监控模块

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::{System, Pid};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::debug;

pub struct SystemMonitor {
    system: Arc<RwLock<System>>,
    monitoring: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone)]
pub struct SystemStats {
    pub cpu_usage: f64,
    pub memory_used_mb: f64,
    pub memory_total_mb: f64,
    pub memory_usage_percent: f64,
    pub process_memory_mb: f64,
    pub process_cpu_usage: f64,
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
            monitoring: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start_monitoring<F>(&self, interval_ms: u64, callback: F)
    where
        F: Fn(SystemStats) + Send + Sync + 'static,
    {
        *self.monitoring.write().await = true;

        let system = self.system.clone();
        let monitoring = self.monitoring.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(interval_ms));

            while *monitoring.read().await {
                ticker.tick().await;

                let mut sys = system.write().await;
                sys.refresh_all();

                let stats = Self::collect_stats(&sys);
                callback(stats);

                debug!("System stats collected");
            }
        });
    }

    pub async fn stop_monitoring(&self) {
        *self.monitoring.write().await = false;
    }

    pub async fn get_current_stats(&self) -> SystemStats {
        let mut sys = self.system.write().await;
        sys.refresh_all();
        Self::collect_stats(&sys)
    }

    fn collect_stats(system: &System) -> SystemStats {
        // CPU 使用率
        let cpu_usage = system.global_cpu_usage() as f64;

        // 内存使用
        let memory_used = system.used_memory() as f64 / 1024.0 / 1024.0; // MB
        let memory_total = system.total_memory() as f64 / 1024.0 / 1024.0; // MB
        let memory_usage_percent = if memory_total > 0.0 {
            (memory_used / memory_total) * 100.0
        } else {
            0.0
        };

        // 当前进程的资源使用
        let pid = std::process::id();
        let (process_memory_mb, process_cpu_usage) = if let Some(process) = system.process(Pid::from_u32(pid)) {
            let mem = process.memory() as f64 / 1024.0 / 1024.0; // MB
            let cpu = process.cpu_usage() as f64;
            (mem, cpu)
        } else {
            (0.0, 0.0)
        };

        SystemStats {
            cpu_usage,
            memory_used_mb: memory_used,
            memory_total_mb: memory_total,
            memory_usage_percent,
            process_memory_mb,
            process_cpu_usage,
        }
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_monitor() {
        let monitor = SystemMonitor::new();
        let stats = monitor.get_current_stats().await;

        assert!(stats.cpu_usage >= 0.0);
        assert!(stats.memory_total_mb > 0.0);
        assert!(stats.memory_usage_percent >= 0.0 && stats.memory_usage_percent <= 100.0);
    }
}

