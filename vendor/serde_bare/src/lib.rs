#![forbid(unsafe_code)]
//! # serde_bare
//!
//! An implementation of the BARE (<https://baremessages.org>) encoding format draft.
//!
//! ## Mapping from the Serde data model
//!
//! ### `bool`, `i8` through `i64`, `u8` through `u64`, `f32`, `f64`, `string`
//!
//! Serialize as the BARE types of the same name.
//!
//! ### `i128`, `u128`
//!
//! Serialize as `data<16>`.
//!
//! ### `char`
//!
//! Serializes as `u32`.
//!
//! ### `byte array`
//!
//! Serializes as `data`.
//!
//! ### `option`
//!
//! Serializes as `optional<type>`
//!
//! ### `seq`
//!
//! Serializes as `[]type`.
//! Sequences with unknown lengths are not representable in BARE.
//!
//! ### `map`
//!
//! Serializes as `map[type]type`.
//!
//! ### `unit`
//!
//! Serializes as `void`.
//!
//! ### `unit_struct`
//!
//! Serializes as `void`.
//! The container name is ignored.
//!
//! ### `unit_variant`
//!
//! Serialized as the variant index as a `uint` followed by the variant data.
//! The container name and variant name are ignored.
//!
//! ### `newtype_struct`
//!
//! Serialized the same as the contained type.
//! The container name is ignored.
//!
//! ### `newtype_variant`
//!
//! Serialized as the variant index as a `uint` followed by the variant data.
//! The container name and variant name are ignored.
//!
//! ### `tuple`
//!
//! Serialized as `struct`.
//!
//! ### `tuple_struct`
//!
//! Serialized as `struct`.
//! The container name is ignored.
//!
//! ### `tuple_variant`
//!
//! Serialized as the variant index as a `uint` followed by the variant data.
//! The container name and variant name are ignored.
//!
//! ### `struct`
//!
//! Serialized as `struct`.
//!
//! ### `struct_variant`
//!
//! Serialized as a `uint` followed by the variant data.
//! The container name and variant name are ignored.
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(test)]
extern crate std as test_std;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

#[cfg(all(not(feature = "alloc"), not(feature = "std")))]
compile_error!("compiling without one of either `std` or `alloc` features enabled is not supported yet");

pub mod de;
pub mod error;
pub mod ser;

#[doc(inline)]
#[cfg(feature = "std")]
pub use de::from_reader;
#[doc(inline)]
pub use de::from_slice;

#[doc(inline)]
pub use error::Result;

#[doc(inline)]
pub use ser::to_vec;
#[doc(inline)]
#[cfg(feature = "std")]
pub use ser::to_writer;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Uint(pub u64);

impl Default for Uint {
    fn default() -> Uint {
        Uint(0)
    }
}

impl serde::ser::Serialize for Uint {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeTuple;

        let Uint(mut x) = *self;
        let mut buf = [0u8; 10];
        let mut i = 0usize;
        while x >= 0x80 {
            buf[i] = (x as u8) | 0x80;
            x >>= 7;
            i += 1;
        }
        buf[i] = x as u8;
        i += 1;

        let mut s = serializer.serialize_tuple(usize::MAX)?;
        for b in buf.iter().take(i) {
            s.serialize_element(&b)?;
        }
        s.end()
    }
}

impl<'de> serde::de::Deserialize<'de> for Uint {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        use std::fmt;

        struct UintVisitor;
        impl<'de> serde::de::Visitor<'de> for UintVisitor {
            type Value = Uint;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a BARE encoded variable-length integer")
            }

            fn visit_seq<A>(self, mut seq: A) -> core::result::Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut x = 0u64;
                let mut s = 0usize;
                for i in 0.. {
                    let b = seq.next_element::<u8>()?;
                    if let Some(b) = b {
                        if i > 9 || i == 9 && b > 1 {
                            // No more than 10 bytes can be in a BARE uint/int
                            return Err(serde::de::Error::custom(
                                "continuation bit indicated an invalid variable-length integer",
                            ));
                        }
                        if b < 0x80 {
                            // No continuation bit is set
                            return Ok(Uint(x | (b as u64) << s));
                        }
                        x |= ((b & 0x7f) as u64) << s;
                        s += 7;
                    } else {
                        // Since we're calling next_element for u8 it's probably impossible to
                        // enter this branch without having raised an io::Error earlier, but better
                        // to handle it anyway instead of introducing a potential panic
                        return Err(serde::de::Error::custom(
                            "expected further bytes in variable-length integer",
                        ));
                    }
                }
                unreachable!()
            }
        }
        deserializer.deserialize_tuple(usize::MAX, UintVisitor)
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Int(pub i64);

impl Default for Int {
    fn default() -> Int {
        Int(0)
    }
}

impl serde::ser::Serialize for Int {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let Int(x) = *self;
        let mut ux = (x as u64) << 1;
        if x < 0 {
            ux = !ux;
        }
        Uint(ux).serialize(serializer)
    }
}

impl<'de> serde::de::Deserialize<'de> for Int {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let Uint(ux) = <Uint as serde::de::Deserialize>::deserialize(deserializer)?;
        let mut x = (ux >> 1) as i64;
        if ux & 1 != 0 {
            x = !x;
        }
        Ok(Int(x))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_int() {
        const CASES: &'static [(i64, &'static [u8])] = &[
            (0, &[0]),
            (1, &[2]),
            (i64::MIN, &[255, 255, 255, 255, 255, 255, 255, 255, 255, 1]),
            (i64::MAX, &[254, 255, 255, 255, 255, 255, 255, 255, 255, 1]),
        ];
        for &(n, bytes) in CASES {
            test_std::println!("testing {}", n);
            let int = Int(n);
            let got_bytes = to_vec(&int).unwrap();
            assert_eq!(got_bytes, bytes);
            let got_int = from_slice::<Int>(&got_bytes).unwrap();
            assert_eq!(got_int, int);
        }
    }

    #[test]
    fn test_uint() {
        const CASES: &'static [(u64, &'static [u8])] = &[
            (0, &[0]),
            (1, &[1]),
            (275, &[147, 2]),
            (u64::MAX, &[255, 255, 255, 255, 255, 255, 255, 255, 255, 1]),
        ];
        for &(n, bytes) in CASES {
            test_std::println!("testing {}", n);
            let int = Uint(n);
            let got_bytes = to_vec(&int).unwrap();
            assert_eq!(got_bytes, bytes);
            let got_int = from_slice::<Uint>(&got_bytes).unwrap();
            assert_eq!(got_int, int);
        }
    }

    #[test]
    fn test_uint_too_long() {
        // Too many bytes
        let bytes: &'static [u8] = &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1];
        let result = from_slice::<Uint>(&bytes);
        assert!(result.is_err());

        // Too many bits of precision (effectively u64::MAX + 1)
        let bytes: &'static [u8] = &[255, 255, 255, 255, 255, 255, 255, 255, 255, 2];
        let result = from_slice::<Uint>(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_uint_too_short() {
        let bytes: &'static [u8] = &[255, 255, 255];
        let result = from_slice::<Uint>(&bytes);
        assert!(result.is_err());
    }
}
