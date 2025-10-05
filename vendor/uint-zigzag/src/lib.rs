//! Uint is a convenience wrapper for zig-zag encoding integers to byte sequences.
//!
//! This allows better compression since the majority of numbers are quite small resulting
//! in 1 or 2 bytes in the most common case vs 4 for 32-bit numbers or 8 for 64-bit numbers.
//!
//! This also permits the user to not have to think about which value is the most efficient
//! to compress.
#![no_std]
#![deny(
    warnings,
    missing_docs,
    unused_import_braces,
    unused_qualifications,
    trivial_casts,
    trivial_numeric_casts
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use core::{
    fmt::{self, Display, Formatter},
    iter::{Product, Sum},
    ops::*,
};
use core2::io::{Error, ErrorKind, Read, Write};

#[cfg(feature = "serde")]
use serde::{
    de::{Error as DError, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{boxed::Box, vec::Vec};
#[cfg(feature = "std")]
use std::{boxed::Box, vec::Vec};

/// Uint implements zig-zag encoding to represent integers as binary sequences
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Uint(pub u128);

impl Display for Uint {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

macro_rules! impl_from {
    ($($tt:ty),+) => {
        $(
        impl From<$tt> for Uint {
            fn from(v: $tt) -> Self {
                Uint(v as u128)
            }
        }

        impl From<Uint> for $tt {
            fn from(v: Uint) -> $tt {
                v.0 as $tt
            }
        }
        )+
    };
}

impl From<u128> for Uint {
    fn from(v: u128) -> Self {
        Self(v)
    }
}

impl From<Uint> for u128 {
    fn from(v: Uint) -> Self {
        v.0
    }
}

impl_from!(u8, u16, u32, u64, usize, i8, i16, i32, i64, i128, isize);

impl TryFrom<&[u8]> for Uint {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut x = 0u128;
        let mut s = 0;
        let mut i = 0;

        while i < Self::MAX_BYTES {
            if i >= value.len() {
                return Err("invalid byte sequence");
            }

            if value[i] < 0x80 {
                let u = x | (value[i] as u128) << s;
                return Ok(Self(u));
            }
            x |= ((value[i] & 0x7f) as u128) << s;
            s += 7;
            i += 1;
        }
        Err("invalid byte sequence")
    }
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
#[cfg(any(feature = "alloc", feature = "std"))]
impl TryFrom<&Vec<u8>> for Uint {
    type Error = &'static str;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
#[cfg(any(feature = "alloc", feature = "std"))]
impl TryFrom<&Box<Vec<u8>>> for Uint {
    type Error = &'static str;

    fn try_from(value: &Box<Vec<u8>>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

macro_rules! ops_impl {
    ($tr:ident, $fn:ident, $op:tt, $($tt:ty),+) => {
        impl<'a, 'b> $tr<&'b Uint> for &'a Uint {
            type Output = Uint;

            fn $fn(self, rhs: &'b Uint) -> Self::Output {
                Uint(self.0 $op rhs.0)
            }
        }

        impl<'a> $tr<&'a Uint> for Uint {
            type Output = Uint;

            fn $fn(self, rhs: &'a Uint) -> Self::Output {
                Uint(self.0 $op rhs.0)
            }
        }

        impl<'b> $tr<Uint> for &'b Uint {
            type Output = Uint;

            fn $fn(self, rhs: Uint) -> Self::Output {
                Uint(self.0 $op rhs.0)
            }
        }

        impl $tr<Uint> for Uint {
            type Output = Uint;

            fn $fn(self, rhs: Uint) -> Self::Output {
                Uint(self.0 $op rhs.0)
            }
        }

        impl<'a> $tr<u128> for &'a Uint {
            type Output = Uint;

            fn $fn(self, rhs: u128) -> Self::Output {
                Uint(self.0 $op rhs)
            }
        }

        impl $tr<u128> for Uint {
            type Output = Uint;

            fn $fn(self, rhs: u128) -> Self::Output {
                Uint(self.0 $op rhs)
            }
        }

        impl<'a> $tr<&'a Uint> for u128 {
            type Output = u128;

            fn $fn(self, rhs: &'a Uint) -> Self::Output {
                self $op rhs.0
            }
        }

        impl $tr<Uint> for u128 {
            type Output = u128;

            fn $fn(self, rhs: Uint) -> Self::Output {
                self $op rhs.0
            }
        }

        $(
        impl<'a> $tr<$tt> for &'a Uint {
            type Output = Uint;

            fn $fn(self, rhs: $tt) -> Self::Output {
                Uint(self.0 $op rhs as u128)
            }
        }

        impl $tr<$tt> for Uint {
            type Output = Uint;

            fn $fn(self, rhs: $tt) -> Self::Output {
                Uint(self.0 $op rhs as u128)
            }
        }

        impl<'a> $tr<&'a Uint> for $tt {
            type Output = $tt;

            fn $fn(self, rhs: &'a Uint) -> Self::Output {
                self $op rhs.0 as $tt
            }
        }

        impl $tr<Uint> for $tt {
            type Output = $tt;

            fn $fn(self, rhs: Uint) -> Self::Output {
                self $op rhs.0 as $tt
            }
        }
        )+
    };
}

macro_rules! assign_ops_impl {
    ($tr:ident, $fn:ident, $op:tt, $($tt:ty),+) => {
        impl $tr<Uint> for Uint {
            fn $fn(&mut self, rhs: Uint) {
                self.0 $op rhs.0;
            }
        }

        impl $tr<u128> for Uint {
            fn $fn(&mut self, rhs: u128) {
                self.0 $op rhs;
            }
        }

        $(
        impl $tr<$tt> for Uint {
            fn $fn(&mut self, rhs: $tt) {
                self.0 $op rhs as u128;
            }
        }
        )+
    };
}

ops_impl!(Add, add, +, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);
ops_impl!(Sub, sub, -, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);
ops_impl!(Mul, mul, *, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);
ops_impl!(Div, div, /, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);
ops_impl!(Rem, rem, %, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);
ops_impl!(BitAnd, bitand, &, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);
ops_impl!(BitOr, bitor, |, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);
ops_impl!(BitXor, bitxor, ^, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);
ops_impl!(Shl, shl, <<, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);
ops_impl!(Shr, shr, >>, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);

assign_ops_impl!(AddAssign, add_assign, +=, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);
assign_ops_impl!(SubAssign, sub_assign, -=, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);
assign_ops_impl!(MulAssign, mul_assign, *=, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);
assign_ops_impl!(DivAssign, div_assign, /=, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);
assign_ops_impl!(RemAssign, rem_assign, %=, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);
assign_ops_impl!(BitAndAssign, bitand_assign, &=, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);
assign_ops_impl!(BitOrAssign, bitor_assign, |=, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);
assign_ops_impl!(BitXorAssign, bitxor_assign, ^=, u64, u32, u16, u8, usize, i128, i64, i32, i16, i8, isize);
assign_ops_impl!(ShlAssign, shl_assign, <<=, u64, u32, u16, u8, usize, i64, i128, i32, i16, i8, isize);
assign_ops_impl!(ShrAssign, shr_assign, >>=, u64, u32, u16, u8, usize, i64, i128, i32, i16, i8, isize);

macro_rules! iter_impl {
    ($($tt:ty),+) => {
        impl Sum for Uint {
            fn sum<I: Iterator<Item=Self>>(mut iter: I) -> Self {
                let mut i = 0u128;
                while let Some(v) = iter.next() {
                    i += v.0;
                }
                Self(i)
            }
        }

        impl Sum<u128> for Uint {
            fn sum<I: Iterator<Item=u128>>(mut iter: I) -> Self {
                let mut i = 0u128;
                while let Some(v) = iter.next() {
                    i += v;
                }
                Self(i)
            }
        }

        impl Product for Uint {
            fn product<I: Iterator<Item=Self>>(mut iter: I) -> Self {
                let mut i = 0u128;
                while let Some(v) = iter.next() {
                    i *= v.0;
                }
                Self(i)
            }
        }

        impl Product<u128> for Uint {
            fn product<I: Iterator<Item=u128>>(mut iter: I) -> Self {
                let mut i = 0u128;
                while let Some(v) = iter.next() {
                    i *= v;
                }
                Self(i)
            }
        }

        $(
        impl Sum<$tt> for Uint {
            fn sum<I: Iterator<Item=$tt>>(mut iter: I) -> Self {
                let mut i = 0u128;
                while let Some(v) = iter.next() {
                    i += v as u128;
                }
                Self(i)
            }
        }

        impl Product<$tt> for Uint {
            fn product<I: Iterator<Item=$tt>>(mut iter: I) -> Self {
                let mut i = 0u128;
                while let Some(v) = iter.next() {
                    i *= v as u128;
                }
                Self(i)
            }
        }
        )+
    };
}

iter_impl!(u64, u32, u16, u8, usize, i64, i32, i16, i8, isize);

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
#[cfg(feature = "serde")]
impl Serialize for Uint {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut buffer = [0u8; Self::MAX_BYTES];
        let length = self.to_bytes_with_length(&mut buffer);
        serializer.serialize_bytes(&buffer[..length])
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Uint {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UintVisitor;

        impl<'de> Visitor<'de> for UintVisitor {
            type Value = Uint;

            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "a byte sequence")
            }

            fn visit_bytes<E: DError>(self, v: &[u8]) -> Result<Self::Value, E> {
                match Uint::try_from(v) {
                    Err(_) => Err(DError::invalid_length(v.len(), &self)),
                    Ok(u) => Ok(u),
                }
            }
        }

        deserializer.deserialize_bytes(UintVisitor)
    }
}

impl Uint {
    /// The maximum number of bytes a uint will consume
    pub const MAX_BYTES: usize = 19;

    /// Peek returns the number of bytes that would be read
    /// or None if no Uint cannot be read
    ///
    /// ```
    /// use uint_zigzag::Uint;
    ///
    /// let buffer = [0x34u8];
    ///
    /// let out = Uint::peek(&buffer);
    ///
    /// assert!(out.is_some());
    ///
    /// let out = Uint::peek(&[]);
    ///
    /// assert!(out.is_none());
    /// ```
    pub fn peek(value: &[u8]) -> Option<usize> {
        let mut i = 0;

        while i < Self::MAX_BYTES {
            if i >= value.len() {
                return None;
            }
            if value[i] < 0x80 {
                return Some(i + 1);
            }

            i += 1;
        }
        None
    }

    /// Zig-zag encoding, any length from 1 to MAX_BYTES into buffer
    /// buffer must be big enough to hold the result
    ///
    /// ```
    ///  use uint_zigzag::Uint;
    ///
    /// let mut buffer = [0u8, 0u8];
    /// let mut u = Uint::from(127);
    /// u.to_bytes(&mut buffer);
    ///
    /// assert_eq!(buffer, [0x7Fu8, 0u8]);
    ///
    /// u = Uint::from(128);
    /// u.to_bytes(&mut buffer);
    /// assert_eq!(buffer, [0x80u8, 1u8]);
    /// ```
    pub fn to_bytes<M: AsMut<[u8]>>(&self, mut buffer: M) {
        self.to_bytes_with_length(buffer.as_mut());
    }

    /// Same as `to_bytes` except it returns how many bytes were actually used
    pub fn to_bytes_with_length(self, buffer: &mut [u8]) -> usize {
        let mut i = 0;
        let mut x = self.0;

        while x >= 0x80 {
            buffer[i] = (x as u8) | 0x80;
            x >>= 7;
            i += 1;
        }

        buffer[i] = x as u8;
        i += 1;
        i
    }

    /// Zig-zag encoding, any length from 1 to MAX_BYTES
    ///
    /// ```
    /// use uint_zigzag::Uint;
    ///
    /// let u = Uint(345678);
    /// let out = u.to_vec();
    ///
    /// assert_eq!(out.as_slice(), &[206, 140, 21]);
    ///
    /// let u = Uint(2048);
    /// let out = u.to_vec();
    /// assert_eq!(out.as_slice(), &[128, 16]);
    /// ```
    #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn to_vec(&self) -> Vec<u8> {
        let mut output = [0u8; Self::MAX_BYTES];
        let i = self.to_bytes_with_length(&mut output);
        output[..i].to_vec()
    }

    /// Write bytes to a stream
    pub fn to_writer<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        let mut output = [0u8; Self::MAX_BYTES];
        let length = self.to_bytes_with_length(&mut output);
        writer.write(&output[..length])
    }

    /// Read bytes from a stream
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut output = [0u8; Self::MAX_BYTES];
        let mut i = 0;
        while i < Self::MAX_BYTES {
            reader.read_exact(&mut output[i..i + 1])?;
            if Self::peek(&output[..i]).is_some() {
                break;
            }
            i += 1;
        }
        if i == Self::MAX_BYTES {
            Err(Error::new(ErrorKind::InvalidData, "invalid byte sequence"))
        } else {
            Self::try_from(&output[..i]).map_err(|m| Error::new(ErrorKind::InvalidData, m))
        }
    }
}

#[cfg(feature = "std")]
#[test]
fn max_bytes() {
    let u = Uint(u128::MAX);
    let bytes = u.to_vec();
    assert_eq!(bytes.len(), Uint::MAX_BYTES);
}
