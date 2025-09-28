// src/security/anti_debug.rs
//! 反调试工具
//! 提供检测调试器的功能

use crate::tools::error::WalletError;

/// 调试器检测器
pub struct DebuggerDetector;

impl DebuggerDetector {
    /// 检测是否正在被调试
    /// 在测试环境中，它会检查 `DEBUG_MODE` 环境变量。
    pub fn is_being_debugged() -> Result<bool, WalletError> {
        // 在测试环境中禁用反调试检测，避免访问违规
        #[cfg(test)]
        {
            // 允许通过环境变量模拟调试器存在
            if std::env::var("DEBUG_MODE").unwrap_or_default() == "1" {
                Ok(true)
            } else {
                Ok(false)
            }
        }

        // 在非测试环境中，执行特定于平台的检查
        // The #[cfg(not(test))] block is essential to prevent real anti-debug
        // checks from interfering with the test runner itself, which can sometimes
        // be flagged as a debugger.
        #[cfg(not(test))]
        {
            // Windows 平台检测
            #[cfg(windows)]
            {
                Self::is_being_debugged_windows()
            }

            // Linux 平台检测
            #[cfg(target_os = "linux")]
            {
                Self::is_being_debugged_linux()
            }

            // macOS 平台检测
            #[cfg(target_os = "macos")]
            {
                Self::is_being_debugged_macos()
            }

            // 其他平台不支持
            #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
            {
                Err(WalletError::UnsupportedPlatform(
                    "Debugger detection not supported on this platform".to_string(),
                ))
            }
        }
    }

    /// Windows 调试器检测
    #[cfg(windows)]
    #[allow(dead_code)] // 在测试编译时不使用，但保留以防将来需要
    fn is_being_debugged_windows() -> Result<bool, WalletError> {
        use winapi::um::debugapi::IsDebuggerPresent;

        // 只使用安全的 Windows API，避免低级内存访问
        let is_debugger_present = unsafe { IsDebuggerPresent() != 0 };

        Ok(is_debugger_present)
    }

    /// Linux 调试器检测
    #[cfg(target_os = "linux")]
    #[allow(dead_code)] // 在测试编译时不使用，但保留以防将来需要
    fn is_being_debugged_linux() -> Result<bool, WalletError> {
        // 检查 /proc/self/status 中的 TracerPid
        match std::fs::read_to_string("/proc/self/status") {
            Ok(content) => {
                for line in content.lines() {
                    if line.starts_with("TracerPid:") {
                        let tracer_pid: i32 =
                            line.split(':').nth(1).unwrap_or("0").trim().parse().unwrap_or(0);
                        return Ok(tracer_pid != 0);
                    }
                }
                Ok(false)
            }
            Err(e) => Err(WalletError::IoError(e)),
        }
    }

    /// macOS 调试器检测
    #[cfg(target_os = "macos")]
    #[allow(dead_code)] // 在测试编译时不使用，但保留以防将来需要
    fn is_being_debugged_macos() -> Result<bool, WalletError> {
        // 使用 ptrace 或其他 macOS 特定方法
        // 这里简化实现
        Ok(false)
    }

    /// 执行反调试措施
    pub fn perform_anti_debug_actions() -> Result<(), WalletError> {
        if Self::is_being_debugged()? {
            #[cfg(feature = "strict_security")]
            {
                // 严格安全模式：记录警告并可能终止程序
                log::warn!("Debugger detected! This may compromise security.");
                // 可以选择终止程序或采取其他措施
                // std::process::exit(1);
            }

            #[cfg(not(feature = "strict_security"))]
            {
                // 非严格模式：只记录警告
                log::warn!("Debugger detected! This may compromise security.");
            }

            Ok(())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_detection_in_normal_test_env() {
        // 在测试环境中，is_being_debugged() 应该返回 Ok(false)
        std::env::remove_var("DEBUG_MODE"); // 确保环境变量未设置
        let result = DebuggerDetector::is_being_debugged();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    #[serial_test::serial] // 确保串行执行以避免环境变量冲突
    fn test_debugger_detection_with_debug_mode_simulation() {
        // 通过设置环境变量来模拟调试器存在
        std::env::set_var("DEBUG_MODE", "1");
        let result = DebuggerDetector::is_being_debugged();
        assert!(result.is_ok());
        assert!(result.unwrap(), "Should detect debugger when DEBUG_MODE is set");

        // 清理环境变量
        std::env::remove_var("DEBUG_MODE");
    }

    #[test]
    fn test_anti_debug_actions() {
        // 在标准测试环境中，此操作应成功且不执行任何操作
        let result = DebuggerDetector::perform_anti_debug_actions();
        assert!(result.is_ok());
    }
}
