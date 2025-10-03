// 娉ㄦ剰锛氶渶瑕佹牴鎹疄闄呭鍏ヨ矾寰勮皟鏁?use defi_hot_wallet::tools::sum_of_products::sum_of_products;
use k256::{ProjectivePoint, Scalar}; // 纭繚瀵煎叆 Scalar 鍜?ProjectivePoint

#[test]
fn test_sum_of_products_basic() {
    // 鍩烘湰娴嬭瘯锛歅*s + Q*t
    let points = vec![
        ProjectivePoint::GENERATOR,          // P = G
        ProjectivePoint::GENERATOR.double(), // Q = 2G
    ];

    let scalars = vec![
        Scalar::from(3u64), // s = 3
        Scalar::from(2u64), // t = 2
    ];

    // 璁＄畻 3*G + 2*(2*G) = 3*G + 4*G = 7*G // 纭繚瀵煎叆 ProjectivePoint
    let result = sum_of_products(&scalars, &points).unwrap();
    let expected = ProjectivePoint::GENERATOR * Scalar::from(7u64);

    assert_eq!(result, expected);
}

#[test]
fn test_sum_of_products_empty() {
    // 绌鸿緭鍏ュ簲杩斿洖鍗曚綅鍏?    let points: Vec<ProjectivePoint> = vec![];
    let scalars: Vec<Scalar> = vec![];

    let result = sum_of_products(&scalars, &points).unwrap(); // 纭繚瀵煎叆 ProjectivePoint
    assert_eq!(result, ProjectivePoint::IDENTITY);
}

#[test]
fn test_sum_of_products_single() {
    // 鍗曚釜鐐瑰拰鏍囬噺
    let points = vec![ProjectivePoint::GENERATOR];
    let scalars = vec![Scalar::from(42u64)]; // 纭繚瀵煎叆 Scalar

    let result = sum_of_products(&scalars, &points).unwrap();
    let expected = ProjectivePoint::GENERATOR * Scalar::from(42u64);

    assert_eq!(result, expected);
}

#[test]
fn test_sum_of_products_with_identity() {
    // 鍖呭惈鍗曚綅鍏?    let points = vec![ProjectivePoint::GENERATOR, ProjectivePoint::IDENTITY];

    let scalars = vec![
        Scalar::from(5u64),
        Scalar::from(10u64), // 涓嶅奖鍝嶇粨鏋?    ];

    let result = sum_of_products(&scalars, &points).unwrap();
    let expected = ProjectivePoint::GENERATOR * Scalar::from(5u64);

    assert_eq!(result, expected);
}

#[test]
fn test_sum_of_products_with_zero_scalar() {
    // 鍖呭惈闆舵爣閲?    let points = vec![ProjectivePoint::GENERATOR, ProjectivePoint::GENERATOR.double()];

    let scalars = vec![
        Scalar::from(3u64),
        Scalar::ZERO, // 闆舵爣閲?    ];

    let result = sum_of_products(&scalars, &points).unwrap();
    let expected = ProjectivePoint::GENERATOR * Scalar::from(3u64);

    assert_eq!(result, expected);
}

#[test]
fn test_sum_of_products_mismatched_lengths() {
    // 鐐瑰拰鏍囬噺鏁伴噺涓嶅尮閰?    let points = vec![ProjectivePoint::GENERATOR];
    let scalars = vec![Scalar::ONE, Scalar::ONE]; // 纭繚瀵煎叆 Scalar

    // 搴旇浼氳繑鍥為敊璇?    let result = sum_of_products(&scalars, &points);
    assert!(result.is_err());
}
