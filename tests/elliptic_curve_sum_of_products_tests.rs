// 注意：需要根据实际导入路径调整
use defi_hot_wallet::tools::sum_of_products::sum_of_products;
use k256::{ProjectivePoint, Scalar};

#[test]
fn test_sum_of_products_basic() {
    // 基本测试：P*s + Q*t
    let points = vec![
        ProjectivePoint::GENERATOR, // P = G
        ProjectivePoint::GENERATOR.double(), // Q = 2G
    ];

    let scalars = vec![
        Scalar::from(3u64), // s = 3
        Scalar::from(2u64), // t = 2
    ];

    // 计算 3*G + 2*(2*G) = 3*G + 4*G = 7*G
    let result = sum_of_products(&scalars, &points).unwrap();
    let expected = ProjectivePoint::GENERATOR * Scalar::from(7u64);

    assert_eq!(result, expected);
}

#[test]
fn test_sum_of_products_empty() {
    // 空输入应返回单位元
    let points: Vec<ProjectivePoint> = vec![];
    let scalars: Vec<Scalar> = vec![];

    let result = sum_of_products(&scalars, &points).unwrap();
    assert_eq!(result, ProjectivePoint::IDENTITY);
}

#[test]
fn test_sum_of_products_single() {
    // 单个点和标量
    let points = vec![ProjectivePoint::GENERATOR];
    let scalars = vec![Scalar::from(42u64)];

    let result = sum_of_products(&scalars, &points).unwrap();
    let expected = ProjectivePoint::GENERATOR * Scalar::from(42u64);

    assert_eq!(result, expected);
}

#[test]
fn test_sum_of_products_with_identity() {
    // 包含单位元
    let points = vec![
        ProjectivePoint::GENERATOR,
        ProjectivePoint::IDENTITY,
    ];

    let scalars = vec![
        Scalar::from(5u64),
        Scalar::from(10u64), // 不影响结果
    ];

    let result = sum_of_products(&scalars, &points).unwrap();
    let expected = ProjectivePoint::GENERATOR * Scalar::from(5u64);

    assert_eq!(result, expected);
}

#[test]
fn test_sum_of_products_with_zero_scalar() {
    // 包含零标量
    let points = vec![
        ProjectivePoint::GENERATOR,
        ProjectivePoint::GENERATOR.double(),
    ];

    let scalars = vec![
        Scalar::from(3u64),
        Scalar::ZERO, // 零标量
    ];

    let result = sum_of_products(&scalars, &points).unwrap();
    let expected = ProjectivePoint::GENERATOR * Scalar::from(3u64);

    assert_eq!(result, expected);
}

#[test]
fn test_sum_of_products_mismatched_lengths() {
    // 点和标量数量不匹配
    let points = vec![ProjectivePoint::GENERATOR];
    let scalars = vec![Scalar::ONE, Scalar::ONE];

    // 应该会返回错误
    let result = sum_of_products(&scalars, &points);
    assert!(result.is_err());
}