use elliptic_curve::Group;

/// 椭圆曲线上的点-标量乘积求和
///
/// 计算：∑(scalar_i * point_i)，比单独计算和累加更高效
///
/// # 参数
///
/// * `scalars` - 标量数组
/// * `points` - 曲线点数组，长度必须与 scalars 相同
///
/// # 返回
///
/// * `Ok(G)` - 计算结果点
/// * `Err(String)` - 错误信息
pub fn sum_of_products<G: Group>(scalars: &[G::Scalar], points: &[G]) -> Result<G, String> {
    if scalars.len() != points.len() {
        return Err("Mismatched scalar and point lengths".to_string());
    }

    if scalars.is_empty() {
        return Ok(G::identity());
    }

    let mut result = G::identity();

    for (scalar, point) in scalars.iter().zip(points.iter()) {
        // 计算 scalar * point 并累加到结果中
        result += *point * *scalar;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::{ProjectivePoint, Scalar};

    #[test]
    fn test_basic_sum() {
        let scalars = vec![Scalar::ONE, Scalar::from(2u64)];
        let points = vec![ProjectivePoint::GENERATOR, ProjectivePoint::GENERATOR * Scalar::from(2u64)];

        let result = sum_of_products(&scalars, &points).unwrap();
        let expected = ProjectivePoint::GENERATOR * Scalar::from(5u64);
        assert_eq!(result, expected);
    }
}