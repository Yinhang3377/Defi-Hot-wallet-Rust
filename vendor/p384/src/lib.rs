#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/p384/0.9.0"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, unused_qualifications)]

#[cfg(feature = "ecdsa")]
#[cfg_attr(docsrs, doc(cfg(feature = "ecdsa")))]
pub mod ecdsa;

pub use elliptic_curve;

#[cfg(feature = "pkcs8")]
pub use elliptic_curve::pkcs8;

use elliptic_curve::bigint::U384;

/// NIST P-384 elliptic curve.
///
/// This curve is also known as secp384r1 (SECG) and is specified in
/// FIPS 186-4: Digital Signature Standard (DSS):
///
/// <https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.186-4.pdf>
///
/// It's included in the US National Security Agency's "Suite B" and is widely
/// used in protocols like TLS and the associated X.509 PKI.
///
/// Its equation is `y² = x³ - 3x + b` over a ~384-bit prime field where `b` is
/// the "verifiably random"† constant:
///
/// ```text
/// b = 2758019355995970587784901184038904809305690585636156852142
///     8707301988689241309860865136260764883745107765439761230575
/// ```
///
/// † *NOTE: the specific origins of this constant have never been fully disclosed
///   (it is the SHA-1 digest of an inexplicable NSA-selected constant)*
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct NistP384;

impl elliptic_curve::Curve for NistP384 {
    /// 384-bit field modulus
    type UInt = U384;

    /// Curve order
    const ORDER: U384 =
        U384::from_be_hex("ffffffffffffffffffffffffffffffffffffffffffffffffc7634d81f4372ddf581a0db248b0a77aecec196accc52973");
}

impl elliptic_curve::PrimeCurve for NistP384 {}

impl elliptic_curve::PointCompression for NistP384 {
    const COMPRESS_POINTS: bool = false;
}

#[cfg(feature = "jwk")]
#[cfg_attr(docsrs, doc(cfg(feature = "jwk")))]
impl elliptic_curve::JwkParameters for NistP384 {
    const CRV: &'static str = "P-384";
}

#[cfg(feature = "pkcs8")]
impl elliptic_curve::AlgorithmParameters for NistP384 {
    const OID: pkcs8::ObjectIdentifier = pkcs8::ObjectIdentifier::new("1.3.132.0.34");
}

/// NIST P-384 field element serialized as bytes.
///
/// Byte array containing a serialized field element value (base field or scalar).
pub type FieldBytes = elliptic_curve::FieldBytes<NistP384>;

/// NIST P-384 SEC1 encoded point.
pub type EncodedPoint = elliptic_curve::sec1::EncodedPoint<NistP384>;

/// NIST P-384 secret key.
pub type SecretKey = elliptic_curve::SecretKey<NistP384>;

impl elliptic_curve::sec1::ValidatePublicKey for NistP384 {}
