use k256::{ProjectivePoint as KPoint, Scalar as KScalar};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct FieldHolder {
    #[serde(with = "elliptic_curve_tools::serdes::prime_field")]
    v: KScalar,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct GroupHolder {
    #[serde(with = "elliptic_curve_tools::serdes::group")]
    g: KPoint,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct FieldArrayHolder {
    #[serde(with = "elliptic_curve_tools::serdes::prime_field_array")]
    a: [KScalar; 2],
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct GroupVecHolder {
    #[serde(with = "elliptic_curve_tools::serdes::group_vec")]
    gs: Vec<KPoint>,
}

#[test]
fn roundtrip_prime_field() {
    let v = KScalar::from(42u64);
    let holder = FieldHolder { v };
    let s = serde_json::to_string(&holder).expect("serialize field");
    let out: FieldHolder = serde_json::from_str(&s).expect("deserialize field");
    assert_eq!(holder, out);
}

#[test]
fn roundtrip_group() {
    let g = KPoint::GENERATOR;
    let holder = GroupHolder { g };
    let s = serde_json::to_string(&holder).expect("serialize group");
    let out: GroupHolder = serde_json::from_str(&s).expect("deserialize group");
    assert_eq!(holder, out);
}

#[test]
fn roundtrip_prime_field_array() {
    let a = [KScalar::from(1u64), KScalar::from(2u64)];
    let holder = FieldArrayHolder { a };
    let s = serde_json::to_string(&holder).expect("serialize array");
    let out: FieldArrayHolder = serde_json::from_str(&s).expect("deserialize array");
    assert_eq!(holder, out);
}

#[test]
fn roundtrip_group_vec() {
    let g = KPoint::GENERATOR;
    let holder = GroupVecHolder { gs: vec![g, g] };
    let s = serde_json::to_string(&holder).expect("serialize vec");
    let out: GroupVecHolder = serde_json::from_str(&s).expect("deserialize vec");
    assert_eq!(holder, out);
}

#[test]
fn invalid_hex_for_field_returns_error() {
    let bad = r#"{"v":"00"}"#;
    let parsed: Result<FieldHolder, _> = serde_json::from_str(bad);
    assert!(parsed.is_err());
}

#[test]
fn invalid_hex_for_group_returns_error() {
    let bad = r#"{"g":"abcd"}"#;
    let parsed: Result<GroupHolder, _> = serde_json::from_str(bad);
    assert!(parsed.is_err());
}
