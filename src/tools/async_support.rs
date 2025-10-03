// src/tools/async_support.rs
//! 鎻愪緵寮傛缂栫▼鐨勮緟鍔╁姛鑳藉拰宸ュ叿

use crate::tools::error::WalletError;
use futures::future::join_all;
use std::future::Future;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::info;

/// 寮傛鎿嶄綔缁撴灉绫诲瀷
pub type AsyncResult<T> = Result<T, WalletError>;

/// 瓒呮椂閰嶇疆
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub duration: Duration,
    pub operation_name: String,
}

impl TimeoutConfig {
    /// 鍒涘缓鏂扮殑瓒呮椂閰嶇疆
    pub fn new(duration: Duration, operation_name: impl Into<String>) -> Self {
        Self { duration, operation_name: operation_name.into() }
    }

    /// 鏍囧噯瓒呮椂锛?0绉掞級
    pub fn standard(operation_name: impl Into<String>) -> Self {
        Self::new(Duration::from_secs(30), operation_name)
    }

    /// 鐭秴鏃讹紙5绉掞級
    pub fn short(operation_name: impl Into<String>) -> Self {
        Self::new(Duration::from_secs(5), operation_name)
    }

    /// 闀胯秴鏃讹紙5鍒嗛挓锛?    pub fn long(operation_name: impl Into<String>) -> Self {
        Self::new(Duration::from_secs(300), operation_name)
    }
}

/// 甯﹁秴鏃剁殑寮傛鎿嶄綔鎵ц鍣?pub struct AsyncExecutor;

impl AsyncExecutor {
    /// 鎵ц寮傛鎿嶄綔锛屽甫瓒呮椂鎺у埗
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

    /// 鎵ц寮傛鎿嶄綔锛屼笉甯﹁秴鏃?    pub async fn execute<F, T>(future: F) -> AsyncResult<T>
    where
        F: Future<Output = AsyncResult<T>>,
    {
        future.await
    }

    /// 閲嶈瘯寮傛鎿嶄綔
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

/// 寮傛浠诲姟绠＄悊鍣?pub struct TaskManager<T> {
    tasks: Vec<tokio::task::JoinHandle<AsyncResult<T>>>,
}

impl<T: Send + 'static> TaskManager<T> {
    /// 鍒涘缓鏂扮殑浠诲姟绠＄悊鍣?    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// 鍚姩寮傛浠诲姟
    pub fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = AsyncResult<T>> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        self.tasks.push(handle);
    }

    /// 绛夊緟鎵€鏈変换鍔″畬鎴?    pub async fn wait_all(&mut self) -> AsyncResult<Vec<T>> {
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

    /// 鍙栨秷鎵€鏈変换鍔?    /// 娉ㄦ剰锛氳繖浼氱珛鍗充腑姝换鍔★紝鍙兘涓嶄細杩愯瀹冧滑鐨勬竻鐞嗕唬鐮侊紙Drop锛夈€?    /// 濡傛灉浠诲姟鎸佹湁闇€瑕佷紭闆呭叧闂殑璧勬簮锛堝鏂囦欢鍙ユ焺銆佺綉缁滆繛鎺ワ級锛?    /// 鏈€濂戒娇鐢ㄥ叾浠栨満鍒讹紙濡傚彇娑堜护鐗岋級鏉ラ€氱煡瀹冧滑鍏抽棴銆?    pub fn cancel_all(&mut self) {
        for handle in &self.tasks {
            handle.abort();
        }
        self.tasks.clear();
    }

    /// 鑾峰彇娲昏穬浠诲姟鏁伴噺
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
        // 鍦―rop鏃跺彇娑堟墍鏈変换鍔?        for handle in &self.tasks {
            handle.abort();
        }
    }
}

/// 寮傛淇″彿閲?pub struct AsyncSemaphore {
    semaphore: tokio::sync::Semaphore,
}

impl AsyncSemaphore {
    /// 鍒涘缓鏂扮殑淇″彿閲?    pub fn new(permits: usize) -> Self {
        Self { semaphore: tokio::sync::Semaphore::new(permits) }
    }

    /// 鑾峰彇璁稿彲
    pub async fn acquire(&self) -> Result<SemaphorePermit<'_>, WalletError> {
        match self.semaphore.acquire().await {
            Ok(permit) => Ok(SemaphorePermit { _permit: permit }),
            Err(_) => Err(WalletError::AsyncError("Failed to acquire semaphore".to_string())),
        }
    }

    /// 灏濊瘯鑾峰彇璁稿彲锛堥潪闃诲锛?    pub fn try_acquire(&self) -> Option<SemaphorePermit<'_>> {
        self.semaphore.try_acquire().ok().map(|permit| SemaphorePermit { _permit: permit })
    }

    /// 鑾峰彇鍙敤璁稿彲鏁伴噺
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// 淇″彿閲忚鍙?pub struct SemaphorePermit<'a> {
    _permit: tokio::sync::SemaphorePermit<'a>,
}

/// 寮傛浜嬩欢鎬荤嚎
pub struct AsyncEventBus<T> {
    sender: tokio::sync::broadcast::Sender<T>,
}

impl<T> AsyncEventBus<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// 鍒涘缓鏂扮殑浜嬩欢鎬荤嚎
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(capacity);
        Self { sender }
    }

    /// 鍙戝竷浜嬩欢
    pub fn publish(&self, event: T) -> Result<(), WalletError> {
        match self.sender.send(event) {
            Ok(_) => Ok(()),
            Err(_) => Err(WalletError::AsyncError("Failed to publish event".to_string())),
        }
    }

    /// 璁㈤槄浜嬩欢
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<T> {
        self.sender.subscribe()
    }
}

/// 寮傛寤惰繜鎵ц鍣?pub struct AsyncDelayExecutor {
    delay: Duration,
}

impl AsyncDelayExecutor {
    /// 鍒涘缓鏂扮殑寤惰繜鎵ц鍣?    pub fn new(delay: Duration) -> Self {
        Self { delay }
    }

    /// 寤惰繜鎵ц寮傛鎿嶄綔
    pub async fn execute_after<F, T>(&self, operation: F) -> AsyncResult<T>
    where
        F: Future<Output = AsyncResult<T>>,
    {
        tokio::time::sleep(self.delay).await;
        operation.await
    }
}

/// 寮傛鎬ц兘鐩戞帶
pub struct AsyncPerformanceMonitor {
    start_time: Instant,
    operation_name: String,
}

impl AsyncPerformanceMonitor {
    /// 寮€濮嬬洃鎺?    pub fn start(operation_name: impl Into<String>) -> Self {
        Self { start_time: Instant::now(), operation_name: operation_name.into() }
    }

    /// 缁撴潫鐩戞帶骞惰褰曟€ц兘
    pub fn finish(self) {
        let duration = self.start_time.elapsed();
        info!(operation = %self.operation_name, ?duration, "Async operation completed");
    }

    /// 缁撴潫鐩戞帶骞惰繑鍥炴寔缁椂闂?    pub fn finish_with_duration(self) -> Duration {
        let duration = self.start_time.elapsed();
        info!(operation = %self.operation_name, ?duration, "Async operation completed");
        duration
    }
}

/// 寮傛宸ュ叿鍑芥暟
///
/// 骞跺彂鎵ц澶氫釜寮傛鎿嶄綔銆?pub async fn concurrent_execute<F, T>(futures: Vec<F>) -> Vec<AsyncResult<T>>
where
    F: Future<Output = AsyncResult<T>> + Send,
    T: Send,
{
    join_all(futures).await
}

/// 椤哄簭鎵ц寮傛鎿嶄綔锛岀洿鍒扮涓€涓垚鍔?pub async fn execute_until_success<F, Fut, T>(operations: Vec<F>) -> AsyncResult<T>
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

        // 鎴愬姛鐨勬搷浣?        let result = AsyncExecutor::execute_with_timeout(async { Ok(42) }, config.clone()).await;
        assert_eq!(result.unwrap(), 42);

        // 瓒呮椂鐨勬搷浣?        let result = AsyncExecutor::execute_with_timeout(
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

        // 绗笁涓幏鍙栧簲璇ョ瓑寰咃紝浣嗘垜浠繖閲屾祴璇曞彲鐢ㄦ暟閲?        assert_eq!(semaphore.available_permits(), 0);

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
