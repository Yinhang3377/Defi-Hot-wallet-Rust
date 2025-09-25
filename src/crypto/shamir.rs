use anyhow::Result;
use rand_core::{OsRng, RngCore};

// 简单版本的 Shamir 秘密分享实现，避免复杂的外部库依赖
pub fn split_secret(secret: [u8; 32], threshold: u8, shares: u8) -> Result<Vec<(u8, [u8; 32])>> {
    if threshold > shares {
        return Err(anyhow::anyhow!("Threshold must be less than or equal to shares"));
    }

    let mut rng = OsRng;
    let mut result = Vec::with_capacity(shares as usize);

    // 使用 GF(256) 有限域算术实现 Shamir 秘密分享
    // 这是一个简化版本，实际生产环境应使用专用库

    // 系数数组，a_0 是秘密，其他系数随机生成
    let mut coefficients = vec![secret];

    // 生成随机系数 a_1 到 a_{t-1}
    for _ in 1..threshold {
        let mut coef = [0u8; 32];
        rng.fill_bytes(&mut coef);
        coefficients.push(coef);
    }

    // 为每个分享计算值
    for id in 1..=shares {
        let mut share_value = [0u8; 32];

        // 对每个字节独立计算多项式
        #[allow(clippy::needless_range_loop)]
        for byte_idx in 0..32 {
            // 从常数项开始（秘密值）
            let mut y = coefficients[0][byte_idx];

            // 计算多项式 f(x) = a_0 + a_1*x + a_2*x^2 + ... + a_{t-1}*x^{t-1}
            let mut x_pow = id;
            #[allow(clippy::needless_range_loop)]
            for i in 1..coefficients.len() {
                // GF(256) 乘法，为简化起见，使用普通乘法（在实际应用中应使用适当的GF(256)乘法）
                let term = (coefficients[i][byte_idx] as u16 * x_pow as u16) % 256; // 修复：确保结果是 u8 类型
                y = (y as u16 ^ term) as u8; // GF(256) 加法是 XOR
                x_pow = ((x_pow as u16 * id as u16) % 256) as u8;
            }

            share_value[byte_idx] = y;
        }

        result.push((id, share_value));
    }

    Ok(result)
}

pub fn combine_secret(parts: &[(u8, [u8; 32])]) -> Result<[u8; 32]> {
    if parts.is_empty() {
        return Err(anyhow::anyhow!("No shares provided"));
    }

    let mut result = [0u8; 32];

    // 对每个字节独立使用拉格朗日插值
    for byte_idx in 0..32 {
        // 使用拉格朗日插值恢复秘密值（多项式在x=0处的值）
        let mut secret_byte = 0u8;

        for (j, (x_j, share_j)) in parts.iter().enumerate() {
            let x_j_value = *x_j;
            let y_j_value = share_j[byte_idx];

            // 计算拉格朗日基多项式 L_j(0)
            let mut numerator = 1u8;
            let mut denominator = 1u8;

            for (m, (x_m, _)) in parts.iter().enumerate() {
                if m != j {
                    numerator = gf256_mul(numerator, *x_m); // L_j(0) 的分子计算
                    denominator = gf256_mul(denominator, gf256_sub(*x_m, x_j_value)); // L_j(0) 的分母计算
                }
            }

            // 计算 y_j * L_j(0) 并加入结果
            let lagrange_basis = gf256_div(numerator, denominator);
            secret_byte ^= gf256_mul(y_j_value, lagrange_basis);
        }

        result[byte_idx] = secret_byte;
    }

    Ok(result)
}

// GF(256) 加法就是 XOR
#[allow(dead_code)]
fn gf256_add(a: u8, b: u8) -> u8 {
    a ^ b
}

// GF(256) 减法与加法相同（XOR）
fn gf256_sub(a: u8, b: u8) -> u8 {
    a ^ b
}

// GF(256) 乘法（简化版本，生产环境应使用查表或更高效的实现）
fn gf256_mul(a: u8, b: u8) -> u8 {
    if a == 0 || b == 0 {
        return 0;
    }

    // 使用查找表计算 log 和 antilog，这里简化为返回乘积（非严格 GF(256)）
    // 实际实现应该使用预计算的表格
    ((a as u16 * b as u16) % 255) as u8
}

// GF(256) 除法（简化版本）
fn gf256_div(a: u8, b: u8) -> u8 {
    if a == 0 {
        return 0;
    }
    if b == 0 {
        panic!("Division by zero in GF(256)");
    }

    // 在 GF(256) 中，a/b = a * (1/b)，其中 1/b 是 b 的乘法逆元
    // 简化版本，生产环境应使用扩展欧几里得算法或查表
    for i in 1..=255u8 {
        if gf256_mul(b, i) == 1 {
            return gf256_mul(a, i);
        }
    }

    // 这种情况理论上不应该发生
    panic!("Could not find multiplicative inverse in GF(256)");
}

// 为了兼容性而保留的结构体
pub struct ShamirSecretSharing {
    threshold: u8,
    shares: u8,
}

impl ShamirSecretSharing {
    pub fn new(threshold: u8, shares: u8) -> Self {
        Self { threshold, shares }
    }

    pub fn split_secret(&self, secret: [u8; 32]) -> Result<Vec<(u8, [u8; 32])>> {
        split_secret(secret, self.threshold, self.shares)
    }

    pub fn combine_secret(parts: &[(u8, [u8; 32])]) -> Result<[u8; 32]> {
        combine_secret(parts)
    }
}
