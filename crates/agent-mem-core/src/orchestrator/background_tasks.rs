use super::memory_extraction::MemoryExtractor;
use super::{ChatRequest, Result};
use agent_mem_traits::Message;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};
use uuid::Uuid;

/// 后台任务状态
#[derive(Clone, Debug)]
pub enum TaskState {
    /// 正在运行
    Running,
    /// 已完成
    Completed { duration: Duration, detail: String },
    /// 失败
    Failed { duration: Duration, error: String },
}

/// 后台任务信息
#[derive(Clone, Debug)]
pub struct TaskStatus {
    pub id: String,
    pub task_type: String,
    pub started_at: Instant,
    pub state: TaskState,
}

/// 后台任务管理器
#[derive(Debug, Default)]
pub struct BackgroundTaskManager {
    tasks: Arc<RwLock<HashMap<String, TaskStatus>>>,
}

impl BackgroundTaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 列出所有任务状态（用于测试/调试）
    pub async fn list_tasks(&self) -> Vec<TaskStatus> {
        self.tasks.read().await.values().cloned().collect()
    }

    /// 记录任务进入运行状态
    async fn record_running(&self, task_status: TaskStatus) {
        let mut tasks = self.tasks.write().await;
        tasks.insert(task_status.id.clone(), task_status);
    }

    /// 更新任务状态
    async fn update_state(&self, task_id: &str, new_state: TaskState) {
        let mut tasks = self.tasks.write().await;
        if let Some(status) = tasks.get_mut(task_id) {
            status.state = new_state;
        }
    }

    /// 启动后台任务
    fn spawn_task<F>(&self, task_type: &str, future: F) -> String
    where
        F: std::future::Future<Output = TaskResult> + Send + 'static,
    {
        let task_id = format!("{}-{}", task_type, Uuid::new_v4());
        let task_id_for_future = task_id.clone();
        let manager = self.clone();
        let task_type_string = task_type.to_string();
        let started_at = Instant::now();
        tokio::spawn(async move {
            manager
                .record_running(TaskStatus {
                    id: task_id_for_future.clone(),
                    task_type: task_type_string.clone(),
                    started_at,
                    state: TaskState::Running,
                })
                .await;

            let result = future.await;
            let duration = started_at.elapsed();

            match result {
                TaskResult::Completed { detail } => {
                    info!(
                        "✅ Background task {} completed in {:?}: {}",
                        task_id_for_future, duration, detail
                    );
                    manager
                        .update_state(
                            &task_id_for_future,
                            TaskState::Completed { duration, detail },
                        )
                        .await;
                }
                TaskResult::Failed { error } => {
                    error!(
                        "❌ Background task {} failed after {:?}: {}",
                        task_id_for_future, duration, error
                    );
                    manager
                        .update_state(&task_id_for_future, TaskState::Failed { duration, error })
                        .await;
                }
            }
        });

        task_id
    }

    /// 启动记忆提取后台任务
    pub fn spawn_memory_extraction(
        &self,
        extractor: Arc<MemoryExtractor>,
        request: ChatRequest,
        messages: Vec<Message>,
    ) -> String {
        let task_type = "memory_extraction";
        self.spawn_task(task_type, async move {
            match run_memory_extraction(extractor, request, messages).await {
                Ok(count) => TaskResult::Completed {
                    detail: format!("extracted {} memories", count),
                },
                Err(e) => TaskResult::Failed {
                    error: e.to_string(),
                },
            }
        })
    }
}

impl Clone for BackgroundTaskManager {
    fn clone(&self) -> Self {
        Self {
            tasks: self.tasks.clone(),
        }
    }
}

/// 后台任务结果
pub enum TaskResult {
    Completed { detail: String },
    Failed { error: String },
}

async fn run_memory_extraction(
    extractor: Arc<MemoryExtractor>,
    request: ChatRequest,
    messages: Vec<Message>,
) -> Result<usize> {
    let agent_id = request.agent_id.clone();
    let user_id = request.user_id.clone();

    let extracted = extractor
        .extract_from_conversation(&messages, &agent_id, &user_id)
        .await?;

    let count = extractor.save_memories(extracted).await?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};

    #[tokio::test]
    async fn test_background_task_completion() {
        let manager = BackgroundTaskManager::new();
        let task_id = manager.spawn_task("test_task", async move {
            sleep(TokioDuration::from_millis(10)).await;
            TaskResult::Completed {
                detail: "done".to_string(),
            }
        });

        sleep(TokioDuration::from_millis(20)).await;
        let tasks = manager.list_tasks().await;
        let status = tasks.iter().find(|t| t.id == task_id).cloned();
        assert!(status.is_some());
        match status.unwrap().state {
            TaskState::Completed { .. } => {}
            _ => panic!("Task should be completed"),
        }
    }

    #[tokio::test]
    async fn test_background_task_failure() {
        let manager = BackgroundTaskManager::new();
        let task_id = manager.spawn_task("fail_task", async move {
            TaskResult::Failed {
                error: "boom".to_string(),
            }
        });

        sleep(TokioDuration::from_millis(5)).await;
        let tasks = manager.list_tasks().await;
        let status = tasks.iter().find(|t| t.id == task_id).cloned();
        assert!(status.is_some());
        match status.unwrap().state {
            TaskState::Failed { .. } => {}
            _ => panic!("Task should be failed"),
        }
    }
}
