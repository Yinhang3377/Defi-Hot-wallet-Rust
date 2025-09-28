use hex::{FromHex, ToHex};

/// 工具相关的错误类型
#[derive(Debug, thiserror::Error)]
pub enum UtilsError {
    #[error("Invalid hex string: {0}")]
    InvalidHexString(String),
}

/// 将十六进制字符串转换为字节向量。
///
/// # Arguments
/// * `hex_string` - 要转换的十六进制字符串。
///
/// # Returns
/// 转换后的字节向量，如果失败则返回 `UtilsError`。
pub fn hex_to_bytes(hex_string: &str) -> Result<Vec<u8>, UtilsError> {
    if hex_string.is_empty() {
        return Err(UtilsError::InvalidHexString("Hex string cannot be empty".to_string()));
    }

    Vec::from_hex(hex_string).map_err(|e| UtilsError::InvalidHexString(e.to_string()))
}

/// 将字节向量转换为十六进制字符串。
///
/// # Arguments
/// * `bytes` - 要转换的字节切片。
///
/// # Returns
/// 转换后的十六进制字符串。
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.encode_hex()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_bytes() {
        let bytes = hex_to_bytes("48656c6c6f").unwrap();
        assert_eq!(bytes, b"Hello");
    }

    #[test]
    fn test_hex_to_bytes_invalid() {
        assert!(hex_to_bytes("invalid").is_err());
    }

    #[test]
    fn test_bytes_to_hex() {
        let hex = bytes_to_hex(b"Hello");
        assert_eq!(hex, "48656c6c6f");
    }
}
