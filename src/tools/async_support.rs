// src/tools/async_support.rs
//! 异步支持工具
//! 提供异步编程的辅助功能和工具

use crate::tools::error::WalletError;
use futures::future::join_all;
use std::future::Future;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::info;

/// 异步操作结果类型
pub type AsyncResult<T> = Result<T, WalletError>;

/// 超时配置
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub duration: Duration,
    pub operation_name: String,
}

impl TimeoutConfig {
    /// 创建新的超时配置
    pub fn new(duration: Duration, operation_name: impl Into<String>) -> Self {
        Self { duration, operation_name: operation_name.into() }
    }

    /// 标准超时（30秒）
    pub fn standard(operation_name: impl Into<String>) -> Self {
        Self::new(Duration::from_secs(30), operation_name)
    }

    /// 短超时（5秒）
    pub fn short(operation_name: impl Into<String>) -> Self {
        Self::new(Duration::from_secs(5), operation_name)
    }

    /// 长超时（5分钟）
    pub fn long(operation_name: impl Into<String>) -> Self {
        Self::new(Duration::from_secs(300), operation_name)
    }
}

/// 带超时的异步操作执行器
pub struct AsyncExecutor;

impl AsyncExecutor {
    /// 执行异步操作，带超时控制
    pub async fn execute_with_timeout<F, T>(future: F, config: TimeoutConfig) -> AsyncResult<T>
    where
        F: Future<Output = AsyncResult<T>>,
    {
        match timeout(config.duration, future).await {
            Ok(result) => result,
            Err(_) => Err(WalletError::TimeoutError(format!(
                "Operation '{}' timed out after {:?}",
                config.operation_name, config.duration
            ))),
        }
    }

    /// 执行异步操作，不带超时
    pub async fn execute<F, T>(future: F) -> AsyncResult<T>
    where
        F: Future<Output = AsyncResult<T>>,
    {
        future.await
    }

    /// 重试异步操作
    pub async fn retry<F, Fut, T>(
        mut operation: F,
        max_attempts: usize,
        delay: Duration,
    ) -> AsyncResult<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = AsyncResult<T>>,
    {
        let mut current_delay = delay;
        let mut last_error: Option<WalletError> = None;

        for attempt in 1..=max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Only retry on retryable errors
                    if !e.is_retryable() {
                        return Err(e);
                    }
                    last_error = Some(e);
                    if attempt < max_attempts {
                        info!(
                            "Operation failed (attempt {}/{}). Retrying in {:?}...",
                            attempt, max_attempts, current_delay
                        );
                        tokio::time::sleep(current_delay).await;
                        // Exponential backoff: double the delay for the next attempt
                        current_delay *= 2;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| WalletError::GenericError("Retry operation failed".to_string())))
    }
}

/// 异步任务管理器
pub struct TaskManager<T> {
    tasks: Vec<tokio::task::JoinHandle<AsyncResult<T>>>,
}

impl<T: Send + 'static> TaskManager<T> {
    /// 创建新的任务管理器
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// 启动异步任务
    pub fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = AsyncResult<T>> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        self.tasks.push(handle);
    }

    /// 等待所有任务完成
    pub async fn wait_all(&mut self) -> AsyncResult<Vec<T>> {
        let mut successful_results = Vec::new();

        for handle in self.tasks.drain(..) {
            match handle.await {
                // Task completed successfully
                Ok(Ok(value)) => successful_results.push(value),
                // Task returned an error
                Ok(Err(e)) => return Err(e),
                // Task panicked
                Err(e) => {
                    return Err(WalletError::AsyncError(format!("Task panicked: {}", e)));
                }
            }
        }

        Ok(successful_results)
    }

    /// 取消所有任务
    /// 注意：这会立即中止任务，可能不会运行它们的清理代码（Drop）。
    /// 如果任务持有需要优雅关闭的资源（如文件句柄、网络连接），
    /// 最好使用其他机制（如取消令牌）来通知它们关闭。
    pub fn cancel_all(&mut self) {
        for handle in &self.tasks {
            handle.abort();
        }
        self.tasks.clear();
    }

    /// 获取活跃任务数量
    pub fn active_count(&self) -> usize {
        self.tasks.len()
    }
}

impl<T: Send + 'static> Default for TaskManager<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for TaskManager<T> {
    fn drop(&mut self) {
        // 在Drop时取消所有任务
        for handle in &self.tasks {
            handle.abort();
        }
    }
}

/// 异步信号量
pub struct AsyncSemaphore {
    semaphore: tokio::sync::Semaphore,
}

impl AsyncSemaphore {
    /// 创建新的信号量
    pub fn new(permits: usize) -> Self {
        Self { semaphore: tokio::sync::Semaphore::new(permits) }
    }

    /// 获取许可
    pub async fn acquire(&self) -> Result<SemaphorePermit<'_>, WalletError> {
        match self.semaphore.acquire().await {
            Ok(permit) => Ok(SemaphorePermit { _permit: permit }),
            Err(_) => Err(WalletError::AsyncError("Failed to acquire semaphore".to_string())),
        }
    }

    /// 尝试获取许可（非阻塞）
    pub fn try_acquire(&self) -> Option<SemaphorePermit<'_>> {
        self.semaphore.try_acquire().ok().map(|permit| SemaphorePermit { _permit: permit })
    }

    /// 获取可用许可数量
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// 信号量许可
pub struct SemaphorePermit<'a> {
    _permit: tokio::sync::SemaphorePermit<'a>,
}

/// 异步事件总线
pub struct AsyncEventBus<T> {
    sender: tokio::sync::broadcast::Sender<T>,
}

impl<T> AsyncEventBus<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// 创建新的事件总线
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(capacity);
        Self { sender }
    }

    /// 发布事件
    pub fn publish(&self, event: T) -> Result<(), WalletError> {
        match self.sender.send(event) {
            Ok(_) => Ok(()),
            Err(_) => Err(WalletError::AsyncError("Failed to publish event".to_string())),
        }
    }

    /// 订阅事件
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<T> {
        self.sender.subscribe()
    }
}

/// 异步延迟执行器
pub struct AsyncDelayExecutor {
    delay: Duration,
}

impl AsyncDelayExecutor {
    /// 创建新的延迟执行器
    pub fn new(delay: Duration) -> Self {
        Self { delay }
    }

    /// 延迟执行异步操作
    pub async fn execute_after<F, T>(&self, operation: F) -> AsyncResult<T>
    where
        F: Future<Output = AsyncResult<T>>,
    {
        tokio::time::sleep(self.delay).await;
        operation.await
    }
}

/// 异步性能监控
pub struct AsyncPerformanceMonitor {
    start_time: Instant,
    operation_name: String,
}

impl AsyncPerformanceMonitor {
    /// 开始监控
    pub fn start(operation_name: impl Into<String>) -> Self {
        Self { start_time: Instant::now(), operation_name: operation_name.into() }
    }

    /// 结束监控并记录性能
    pub fn finish(self) {
        let duration = self.start_time.elapsed();
        info!(operation = %self.operation_name, ?duration, "Async operation completed");
    }

    /// 结束监控并返回持续时间
    pub fn finish_with_duration(self) -> Duration {
        let duration = self.start_time.elapsed();
        info!(operation = %self.operation_name, ?duration, "Async operation completed");
        duration
    }
}

/// 异步工具函数

/// 并发执行多个异步操作
pub async fn concurrent_execute<F, T>(futures: Vec<F>) -> Vec<AsyncResult<T>>
where
    F: Future<Output = AsyncResult<T>> + Send,
    T: Send,
{
    join_all(futures).await
}

/// 顺序执行异步操作，直到第一个成功
pub async fn execute_until_success<F, Fut, T>(operations: Vec<F>) -> AsyncResult<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = AsyncResult<T>>,
{
    let mut last_error = None;

    for operation in operations {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => last_error = Some(e),
        }
    }

    Err(last_error
        .unwrap_or_else(|| WalletError::GenericError("All operations failed".to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_timeout_execution() {
        let config = TimeoutConfig::short("test_operation");

        // 成功的操作
        let result = AsyncExecutor::execute_with_timeout(async { Ok(42) }, config.clone()).await;
        assert_eq!(result.unwrap(), 42);

        // 超时的操作
        let result = AsyncExecutor::execute_with_timeout(
            async {
                tokio::time::sleep(Duration::from_secs(10)).await;
                Ok(42)
            },
            config,
        )
        .await;
        assert!(matches!(result, Err(WalletError::TimeoutError(_))));
    }

    #[tokio::test]
    async fn test_retry() {
        let attempts = Arc::new(Mutex::new(0));

        let operation = {
            let attempts = Arc::clone(&attempts);
            move || {
                let attempts = Arc::clone(&attempts);
                async move {
                    let mut attempts_guard = attempts.lock().await;
                    *attempts_guard += 1;
                    if *attempts_guard < 3 {
                        Err(WalletError::NetworkError("Temporary failure".to_string()))
                    } else {
                        Ok("success")
                    }
                }
            }
        };

        let result = AsyncExecutor::retry(operation, 3, Duration::from_millis(10)).await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(*attempts.lock().await, 3);
    }

    #[tokio::test]
    async fn test_task_manager() {
        let mut manager: TaskManager<u32> = TaskManager::new();

        manager.spawn(async { Ok(1) });
        manager.spawn(async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(2)
        });

        let result = manager.wait_all().await;
        assert!(result.is_ok());
        let mut values = result.unwrap();
        values.sort(); // The order of completion is not guaranteed
        assert_eq!(values, vec![1, 2]);
        assert_eq!(manager.active_count(), 0);
    }

    #[tokio::test]
    async fn test_async_semaphore() {
        let semaphore = AsyncSemaphore::new(2);

        let permit1 = semaphore.acquire().await.unwrap();
        let permit2 = semaphore.acquire().await.unwrap();

        // 第三个获取应该等待，但我们这里测试可用数量
        assert_eq!(semaphore.available_permits(), 0);

        drop(permit1);
        assert_eq!(semaphore.available_permits(), 1);

        drop(permit2);
        assert_eq!(semaphore.available_permits(), 2);
    }

    #[tokio::test]
    async fn test_performance_monitor() {
        let monitor = AsyncPerformanceMonitor::start("test_operation");

        tokio::time::sleep(Duration::from_millis(10)).await;

        let duration = monitor.finish_with_duration();
        assert!(duration >= Duration::from_millis(10));
    }
}
