use elliptic_curve::{group::GroupEncoding, Field, Group};
use elliptic_curve_tools::serdes::*; // 确保补丁生效
use rstest::*;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)] // 保持 derive 以便测试
struct TestStruct<G: Group + GroupEncoding> {
    #[serde(with = "prime_field")]
    scalar: G::Scalar,
    #[serde(with = "group")]
    point: G,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)] // 保持 derive 以便测试
struct TestStructArray<G: Group + GroupEncoding, const N: usize> {
    #[serde(with = "prime_field_array")]
    scalar: [G::Scalar; N],
    #[serde(with = "group_array")]
    point: [G; N],
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)] // 保持 derive 以便测试
struct TestStructVec<G: Group + GroupEncoding> {
    #[serde(with = "prime_field_vec")]
    scalar: Vec<G::Scalar>,
    #[serde(with = "group_vec")]
    point: Vec<G>,
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)] // 保持 derive 以便测试
struct TestStructBoxedSlice<G: Group + GroupEncoding> {
    #[serde(with = "prime_field_boxed_slice")]
    scalar: Box<[G::Scalar]>,
    #[serde(with = "group_boxed_slice")]
    point: Box<[G]>,
}

// 参数化测试：覆盖所有曲线类型和格式
#[rstest]
#[case::k256(k256::ProjectivePoint::default())]
#[case::p256(p256::ProjectivePoint::default())]
#[case::p384(p384::ProjectivePoint::default())]
#[case::curve25519_edwards(curve25519_dalek_ml::edwards::EdwardsPoint::default())]
#[case::curve25519_ristretto(curve25519_dalek_ml::ristretto::RistrettoPoint::default())]
#[case::bls12381_g1(blsful::inner_types::G1Projective::default())]
#[case::bls12381_g2(blsful::inner_types::G2Projective::default())]
#[case::ed448_edwards(ed448_goldilocks_plus::EdwardsPoint::default())]
fn comprehensive_serialization<G: Group + GroupEncoding>(#[case] _g: G) {
    let test_struct = TestStruct { scalar: <G::Scalar as Field>::ONE, point: G::generator() };

    // 分别测试每个格式，避免类型不匹配
    // postcard
    let res = postcard::to_stdvec(&test_struct);
    assert!(res.is_ok());
    let output = res.unwrap();
    let res = postcard::from_bytes::<TestStruct<G>>(&output);
    assert!(res.is_ok());
    assert_eq!(test_struct, res.unwrap());

    // bare
    let res = serde_bare::to_vec(&test_struct);
    assert!(res.is_ok());
    let output = res.unwrap();
    let res = serde_bare::from_slice::<TestStruct<G>>(&output);
    assert!(res.is_ok());
    assert_eq!(test_struct, res.unwrap());

    // cbor
    let res = serde_cbor::to_vec(&test_struct);
    assert!(res.is_ok());
    let output = res.unwrap();
    let res = serde_cbor::from_slice::<TestStruct<G>>(&output);
    assert!(res.is_ok());
    assert_eq!(test_struct, res.unwrap());

    // json
    let res = serde_json::to_string(&test_struct);
    assert!(res.is_ok());
    let output = res.unwrap();
    let res = serde_json::from_str::<TestStruct<G>>(&output);
    assert!(res.is_ok());
    assert_eq!(test_struct, res.unwrap());

    // yaml
    let res = serde_yaml::to_string(&test_struct);
    assert!(res.is_ok());
    let output = res.unwrap();
    let res = serde_yaml::from_str::<TestStruct<G>>(&output);
    assert!(res.is_ok());
    assert_eq!(test_struct, res.unwrap());

    // toml (使用 from_str)
    let res = toml::to_string(&test_struct);
    assert!(res.is_ok());
    let output = res.unwrap();
    let res = toml::from_str::<TestStruct<G>>(&output);
    assert!(res.is_ok());
    assert_eq!(test_struct, res.unwrap());

    // ciborium 特殊处理
    let mut buffer = Vec::with_capacity(86);
    assert!(ciborium::into_writer(&test_struct, &mut buffer).is_ok());
    let res = ciborium::from_reader::<TestStruct<G>, &[u8]>(buffer.as_slice());
    assert!(res.is_ok());
    assert_eq!(test_struct, res.unwrap());

    // bincode
    let res = bincode::serialize(&test_struct);
    assert!(res.is_ok());
    let output = res.unwrap();
    let res: Result<TestStruct<G>, _> = bincode::deserialize(&output);
    assert!(res.is_ok());
    assert_eq!(test_struct, res.unwrap());
}

// 测试数组和向量结构
#[cfg(any(feature = "alloc", feature = "std"))]
#[test]
fn array_and_vec_structs() {
    let array_struct = TestStructArray::<k256::ProjectivePoint, 2> {
        scalar: [<k256::Scalar as Field>::ONE; 2],
        point: [k256::ProjectivePoint::GENERATOR; 2],
    };
    let vec_struct = TestStructVec {
        scalar: vec![<k256::Scalar as Field>::ONE; 2],
        point: vec![k256::ProjectivePoint::GENERATOR; 2],
    };
    let boxed_struct = TestStructBoxedSlice {
        scalar: vec![<k256::Scalar as Field>::ONE; 2].into_boxed_slice(),
        point: vec![k256::ProjectivePoint::GENERATOR; 2].into_boxed_slice(),
    };

    // 分别测试每个结构
    // array
    let res = serde_json::to_string(&array_struct);
    assert!(res.is_ok());
    let output = res.unwrap();
    let res = serde_json::from_str::<TestStructArray<k256::ProjectivePoint, 2>>(&output);
    assert!(res.is_ok());
    assert_eq!(array_struct, res.unwrap());

    // vec
    let res = serde_json::to_string(&vec_struct);
    assert!(res.is_ok());
    let output = res.unwrap();
    let res = serde_json::from_str::<TestStructVec<k256::ProjectivePoint>>(&output);
    assert!(res.is_ok());
    assert_eq!(vec_struct, res.unwrap());

    // boxed
    let res = serde_json::to_string(&boxed_struct);
    assert!(res.is_ok());
    let output = res.unwrap();
    let res = serde_json::from_str::<TestStructBoxedSlice<k256::ProjectivePoint>>(&output);
    assert!(res.is_ok());
    assert_eq!(boxed_struct, res.unwrap());
}

// 边界和错误测试
#[rstest]
#[case::zero(k256::Scalar::ZERO)]
#[case::one(<k256::Scalar as Field>::ONE)]
#[case::large(k256::Scalar::from(u64::MAX))]
fn boundary_scalars(#[case] scalar: k256::Scalar) {
    let test_struct = TestStruct { scalar, point: k256::ProjectivePoint::GENERATOR };
    let res = serde_json::to_string(&test_struct);
    assert!(res.is_ok());
    let output = res.unwrap();
    let res = serde_json::from_str(&output);
    assert!(res.is_ok());
    assert_eq!(test_struct, res.unwrap());
}

#[test]
fn identity_and_random_points() {
    let identity = TestStruct { scalar: <k256::Scalar as Field>::ONE, point: k256::ProjectivePoint::IDENTITY };
    let random = TestStruct { scalar: <k256::Scalar as Field>::ONE, point: k256::ProjectivePoint::GENERATOR * k256::Scalar::from(42u64) };

    for (name, test_struct) in vec![("identity", identity), ("random", random)] {
        let res = serde_json::to_string(&test_struct);
        assert!(res.is_ok(), "Serialization failed for {}", name);
        let output = res.unwrap();
        let res = serde_json::from_str(&output);
        assert!(res.is_ok(), "Deserialization failed for {}", name);
        assert_eq!(test_struct, res.unwrap());
    }
}

#[test]
fn empty_and_large_structs() {
    let empty_array = TestStructArray::<k256::ProjectivePoint, 0> { scalar: [], point: [] };
    let res = serde_json::to_string(&empty_array);
    assert!(res.is_ok());
    let output = res.unwrap();
    let res = serde_json::from_str(&output);
    assert!(res.is_ok());
    assert_eq!(empty_array, res.unwrap());

    #[cfg(any(feature = "alloc", feature = "std"))]
    {
        let large_vec = TestStructVec {
            scalar: vec![<k256::Scalar as Field>::ONE; 1000],
            point: vec![k256::ProjectivePoint::GENERATOR; 1000],
        };
        let res = bincode::serialize(&large_vec);
        assert!(res.is_ok());
        let output = res.unwrap();
        let res: Result<TestStructVec<k256::ProjectivePoint>, _> = bincode::deserialize(&output);
        assert!(res.is_ok());
        assert_eq!(large_vec, res.unwrap());
    }
}

#[test]
fn error_cases() {
    let invalid_hex = r#""gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg""#;
    let res: Result<k256::Scalar, _> = serde_json::from_str(invalid_hex);
    assert!(res.is_err());

    let invalid_group = r#"{"point": "invalid"}"#;
    let res: Result<TestStruct<k256::ProjectivePoint>, _> = serde_json::from_str(invalid_group);
    assert!(res.is_err());

    let invalid_bincode = vec![0xFF; 10];
    let res: Result<TestStruct<k256::ProjectivePoint>, _> = bincode::deserialize(&invalid_bincode);
    assert!(res.is_err());
}

#[test]
fn pushable_and_heapless() {
    let mut heapless_vec = heapless::Vec::<u8, 4>::new();
    heapless_vec.push(1).unwrap();
    heapless_vec.push(2).unwrap();
    heapless_vec.push(3).unwrap();
    heapless_vec.push(4).unwrap();
    assert_eq!(heapless_vec.len(), 4);

    let res = heapless_vec.push(5); // 超出容量 4
    assert!(res.is_err());  // 现在应该失败

    #[cfg(any(feature = "alloc", feature = "std"))]
    {
        let mut vec = Vec::<u8>::new();
        vec.push(1);
        vec.push(2);
        assert_eq!(vec.len(), 2);
    }
}

#[cfg(all(feature = "std", not(miri)))]
#[test]
fn concurrent_operations() {
    use std::thread;

    let json = serde_json::to_string(&TestStruct {
        scalar: <k256::Scalar as Field>::ONE,
        point: k256::ProjectivePoint::GENERATOR,
    }).unwrap();

    let handles: Vec<_> = (0..4).map(|_| {
        let json = json.clone();
        thread::spawn(move || {
            let res: Result<TestStruct<k256::ProjectivePoint>, _> = serde_json::from_str(&json);
            assert!(res.is_ok());
        })
    }).collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn performance_and_buffer_sizes() {
    let test_struct = TestStruct {
        scalar: <k256::Scalar as Field>::ONE,
        point: k256::ProjectivePoint::GENERATOR,
    };

    // postcard 缓冲区测试
    let res = postcard::to_vec::<_, 128>(&test_struct);
    assert!(res.is_ok());

    // ciborium 大缓冲区
    #[cfg(any(feature = "alloc", feature = "std"))]
    {
        let test_vec = TestStructVec {
            scalar: vec![<k256::Scalar as Field>::ONE; 100],
            point: vec![k256::ProjectivePoint::GENERATOR; 100],
        };
        let mut buffer = Vec::with_capacity(10000);
        assert!(ciborium::into_writer(&test_vec, &mut buffer).is_ok());
        let res = ciborium::from_reader::<TestStructVec<k256::ProjectivePoint>, &[u8]>(buffer.as_slice());
        assert!(res.is_ok());
    }
}

#[test]
fn cross_format_consistency() {
    let test_struct = TestStruct {
        scalar: <k256::Scalar as Field>::ONE,
        point: k256::ProjectivePoint::GENERATOR,
    };

    let json = serde_json::to_string(&test_struct).unwrap();
    let bincode = bincode::serialize(&test_struct).unwrap();
    let yaml = serde_yaml::to_string(&test_struct).unwrap();

    let from_json: TestStruct<k256::ProjectivePoint> = serde_json::from_str(&json).unwrap();
    let from_bincode: TestStruct<k256::ProjectivePoint> = bincode::deserialize(&bincode).unwrap();
    let from_yaml: TestStruct<k256::ProjectivePoint> = serde_yaml::from_str(&yaml).unwrap();

    assert_eq!(test_struct, from_json);
    assert_eq!(test_struct, from_bincode);
    assert_eq!(test_struct, from_yaml);
}


// 新增：no_std 兼容测试（使用 heapless）
#[cfg(not(any(feature = "alloc", feature = "std")))]
#[test]
fn no_std_serialization() {
    let test_struct = TestStruct {
        scalar: <k256::Scalar as Field>::ONE,
        point: k256::ProjectivePoint::GENERATOR,
    };

    // 使用 postcard（no_std 友好）
    let mut buffer = heapless::Vec::<u8, 128>::new();
    let res = postcard::to_slice(&test_struct, &mut buffer);
    assert!(res.is_ok());
    let output = res.unwrap();
    let res = postcard::from_bytes::<TestStruct<k256::ProjectivePoint>>(output);
    assert!(res.is_ok());
    assert_eq!(test_struct, res.unwrap());
}

// 新增：序列化失败测试
#[test]
fn serialization_failure() {
    let test_struct = TestStruct {
        scalar: <k256::Scalar as Field>::ONE,
        point: k256::ProjectivePoint::GENERATOR,
    };

    // postcard 小缓冲区导致失败
    let res = postcard::to_vec::<_, 10>(&test_struct); // 缓冲区太小
    assert!(res.is_err()); // 期望失败
}

// 新增：无穷大点测试（如果曲线支持）
#[test]
fn infinity_point() {
    let infinity_point = k256::ProjectivePoint::IDENTITY; // k256 的无穷远点是 IDENTITY
    let test_struct = TestStruct {
        scalar: <k256::Scalar as Field>::ONE,
        point: infinity_point,
    };

    let res = serde_json::to_string(&test_struct);
    assert!(res.is_ok());
    let output = res.unwrap();
    let res = serde_json::from_str(&output);
    assert!(res.is_ok());
    assert_eq!(test_struct, res.unwrap());
}

// 新增：格式特定错误恢复
#[test]
fn format_specific_errors() {
    let invalid_json = r#"{"scalar": "invalid", "point": "generator"}"#;
    let res: Result<TestStruct<k256::ProjectivePoint>, _> = serde_json::from_str(invalid_json);
    assert!(res.is_err());

    let invalid_yaml = "scalar: invalid\npoint: generator";
    let res: Result<TestStruct<k256::ProjectivePoint>, _> = serde_yaml::from_str(invalid_yaml);
    assert!(res.is_err());

    let invalid_toml = "scalar = 'invalid'\npoint = 'generator'";
    let res: Result<TestStruct<k256::ProjectivePoint>, _> = toml::from_str(invalid_toml);
    assert!(res.is_err());

    let invalid_bincode = vec![0x00; 5]; // 太短
    let res: Result<TestStruct<k256::ProjectivePoint>, _> = bincode::deserialize(&invalid_bincode);
    assert!(res.is_err());
}

// 新增：堆积 Vec 容量扩展测试
#[cfg(any(feature = "alloc", feature = "std"))]
#[test]
fn vec_capacity_extension() {
    let mut vec = Vec::<u8>::with_capacity(2);
    vec.push(1);
    vec.push(2);
    assert_eq!(vec.len(), 2);
    assert_eq!(vec.capacity(), 2);

    vec.push(3); // 自动扩展容量
    assert_eq!(vec.len(), 3);
    assert!(vec.capacity() >= 3);
}

// 新增：更多并发场景
#[cfg(all(feature = "std", not(miri)))]
#[test]
fn concurrent_serialization() {
    use std::thread;

    let test_struct = TestStruct {
        scalar: <k256::Scalar as Field>::ONE,
        point: k256::ProjectivePoint::GENERATOR,
    };

    let handles: Vec<_> = (0..4).map(|_| {
        let ts = test_struct.clone();
        thread::spawn(move || {
            let res = serde_json::to_string(&ts);
            assert!(res.is_ok());
        })
    }).collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

// 新增：曲线特定边界（BLS12-381 示例）
#[test]
fn curve_specific_boundaries() {
    // BLS12-381 G1
    let bls_g1_struct = TestStruct {
        scalar: <blsful::inner_types::Scalar as Field>::ONE,
        point: blsful::inner_types::G1Projective::GENERATOR,
    };
    let res = serde_json::to_string(&bls_g1_struct);
    assert!(res.is_ok());
    let output = res.unwrap();
    let res = serde_json::from_str(&output);
    assert!(res.is_ok());
    assert_eq!(bls_g1_struct, res.unwrap());

    // BLS12-381 G2
    let bls_g2_struct = TestStruct {
        scalar: <blsful::inner_types::Scalar as Field>::ONE,
        point: blsful::inner_types::G2Projective::GENERATOR,
    };
    let res = serde_json::to_string(&bls_g2_struct);
    assert!(res.is_ok());
    let output = res.unwrap();
    let res = serde_json::from_str(&output);
    assert!(res.is_ok());
    assert_eq!(bls_g2_struct, res.unwrap());
}

// 修复：移除严格的时间断言，因为性能在不同环境（调试/发布、硬件）下不同
#[test]
fn performance_boundaries() {
    let large_struct = TestStructVec {
        scalar: vec![<k256::Scalar as Field>::ONE; 10000],
        point: vec![k256::ProjectivePoint::GENERATOR; 10000],
    };
 
    // 测试序列化成功（不检查时间，因为调试模式下可能慢）
    let start = std::time::Instant::now();
    let res = bincode::serialize(&large_struct);
    let duration = start.elapsed();
    assert!(res.is_ok());
    println!("Serialization took: {:?}", duration); // 可选：打印时间用于调试

    let output = res.unwrap();
    let res: Result<TestStructVec<k256::ProjectivePoint>, _> = bincode::deserialize(&output);
    assert!(res.is_ok());
}