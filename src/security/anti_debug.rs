<<<<<<< HEAD
/// Anti-debug / detection stub
pub fn detect_debugger() -> bool {
    false
=======
//! 防调试、防内存转储、防反序列化攻击

pub fn anti_debug_check() -> bool {
    // TODO: 检查是否被调试或内存转储
    true
>>>>>>> be35db3d094cb6edd3c63585f33fdcb299a57158
}
