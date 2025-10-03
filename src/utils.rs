use hex::{FromHex, ToHex};

/// 宸ュ叿鐩稿叧鐨勯敊璇被鍨?#[derive(Debug, thiserror::Error)]
pub enum UtilsError {
    #[error("Invalid hex string: {0}")]
    InvalidHexString(String),
}

/// 灏嗗崄鍏繘鍒跺瓧绗︿覆杞崲涓哄瓧鑺傚悜閲忋€?///
/// # Arguments
/// * `hex_string` - 瑕佽浆鎹㈢殑鍗佸叚杩涘埗瀛楃涓层€?///
/// # Returns
/// 杞崲鍚庣殑瀛楄妭鍚戦噺锛屽鏋滃け璐ュ垯杩斿洖 `UtilsError`銆?pub fn hex_to_bytes(hex_string: &str) -> Result<Vec<u8>, UtilsError> {
    if hex_string.is_empty() {
        return Err(UtilsError::InvalidHexString("Hex string cannot be empty".to_string()));
    }

    Vec::from_hex(hex_string).map_err(|e| UtilsError::InvalidHexString(e.to_string()))
}

/// 灏嗗瓧鑺傚悜閲忚浆鎹负鍗佸叚杩涘埗瀛楃涓层€?///
/// # Arguments
/// * `bytes` - 瑕佽浆鎹㈢殑瀛楄妭鍒囩墖銆?///
/// # Returns
/// 杞崲鍚庣殑鍗佸叚杩涘埗瀛楃涓层€?pub fn bytes_to_hex(bytes: &[u8]) -> String {
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
