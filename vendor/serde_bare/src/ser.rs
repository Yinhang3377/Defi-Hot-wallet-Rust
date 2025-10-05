use crate::{error::Error, Uint};
use serde::{ser, Serialize};
use std::vec::Vec;

pub struct Serializer<W> {
    writer: W,
}

impl<W> Serializer<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<W> Serializer<IoWrite<W>> {
    pub fn from_writer(writer: W) -> Self {
        Self {
            writer: IoWrite::new(writer),
        }
    }
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    /// BARE type: bool
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.serialize_u8(v as u8)
    }

    /// BARE type: i8
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(&v.to_le_bytes())
    }

    /// BARE type: i16
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(&v.to_le_bytes())
    }

    /// BARE type: i32
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(&v.to_le_bytes())
    }

    /// BARE type: i64
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(&v.to_le_bytes())
    }

    serde::serde_if_integer128! {
        /// BARE type: data\<16\>
        fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
            self.writer.write_all(&v.to_le_bytes())
        }
    }

    /// BARE type: u8
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(&v.to_le_bytes())
    }

    /// BARE type: u16
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(&v.to_le_bytes())
    }

    /// BARE type: u32
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(&v.to_le_bytes())
    }

    /// BARE type: u64
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(&v.to_le_bytes())
    }

    serde::serde_if_integer128! {
        /// BARE type: data\<16\>
        fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
            self.writer.write_all(&v.to_le_bytes())
        }
    }

    /// BARE type: f32
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(&v.to_le_bytes())
    }

    /// BARE type: f64
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(&v.to_le_bytes())
    }

    /// BARE type: u32
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_u32(v as u32)
    }

    /// BARE type: string
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Uint(v.len() as u64).serialize(&mut *self)?;
        self.writer.write_all(v.as_bytes())
    }

    /// BARE type: data
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Uint(v.len() as u64).serialize(&mut *self)?;
        self.writer.write_all(v)
    }

    /// BARE type: optional\<type\>
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_u8(0)
    }

    /// BARE type: optional\<type\>
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.serialize_u8(1)?;
        value.serialize(self)
    }

    /// BARE type: void
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    /// BARE type: void
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    /// BARE type: uint or (... | void | ...)
    /// `name` and `variant` are ignored.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Uint(variant_index.into()).serialize(self)
    }

    /// BARE type: T
    /// `name` is ignored.
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    /// BARE type: { uint, T } or (... | T | ...)
    /// `name` and `variant` are ignored.
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Uint(variant_index.into()).serialize(&mut *self)?;
        value.serialize(self)
    }

    /// BARE type if len is Some: []type
    /// BARE type \[len\]type is never used for variable-length sequences
    /// Error::SequenceLengthRequired if len is None
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Uint(len.ok_or(Error::sequence_length_required())? as u64).serialize(&mut *self)?;
        Ok(self)
    }

    /// BARE type: \[len\]type
    /// `len` is ignored.
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    /// BARE type: struct
    /// `name` and `len` are ignored.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    /// BARE type: { uint, T } or (... | T | ...)
    /// `name`, `variant`, and `len` are ignored.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Uint(variant_index.into()).serialize(&mut *self)?;
        Ok(self)
    }

    /// BARE type: map\[T\]U
    /// Error::MapLengthRequired if len is None
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Uint(len.ok_or(Error::map_length_required())? as u64).serialize(&mut *self)?;
        Ok(self)
    }

    /// BARE type: struct
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    /// BARE type: { uint, T } or (... | T | ...)
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Uint(variant_index.into()).serialize(&mut *self)?;
        Ok(self)
    }

    /// Returns false.
    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'a, W> ser::SerializeSeq for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTuple for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleStruct for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleVariant for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeMap for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeStruct for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeStructVariant for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

pub fn to_vec<T: ?Sized>(value: &T) -> Result<Vec<u8>, Error>
where
    T: Serialize,
{
    let mut vec = Vec::new();
    let mut serializer = Serializer {
        writer: VecWrite::new(&mut vec),
    };
    value.serialize(&mut serializer)?;
    Ok(vec)
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub fn to_writer<W, T: ?Sized>(writer: W, value: &T) -> Result<(), Error>
where
    W: std::io::Write,
    T: Serialize,
{
    let mut serializer = Serializer {
        writer: IoWrite::new(writer),
    };
    value.serialize(&mut serializer)?;
    Ok(())
}

pub trait Write: private::Sealed {
    fn write_all(&mut self, data: &[u8]) -> Result<(), Error>;
}

mod private {
    pub trait Sealed {}
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub struct IoWrite<W> {
    writer: W,
}

#[cfg(feature = "std")]
impl<W> IoWrite<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

#[cfg(feature = "std")]
impl<W: std::io::Write> Write for IoWrite<W> {
    fn write_all(&mut self, data: &[u8]) -> Result<(), Error> {
        self.writer.write_all(data).map_err(Error::io)
    }
}

#[cfg(feature = "std")]
impl<W> private::Sealed for IoWrite<W> {}

pub struct VecWrite<'a> {
    inner: &'a mut Vec<u8>,
}

impl<'a> VecWrite<'a> {
    pub fn new(inner: &'a mut Vec<u8>) -> Self {
        Self { inner }
    }
}

impl<'a> Write for VecWrite<'a> {
    fn write_all(&mut self, data: &[u8]) -> Result<(), Error> {
        self.inner.extend_from_slice(data);
        Ok(())
    }
}

impl<'a> private::Sealed for VecWrite<'a> {}

#[cfg(test)]
mod test {
    #[test]
    fn test_unbounded_sequence() {
        use serde::Serializer;
        use std::vec::Vec;

        let seq = [1, 2, 3];
        let mut vec = Vec::<u8>::new();
        let mut serializer = super::Serializer::new(super::VecWrite::new(&mut vec));
        assert!(serializer
            .collect_seq(seq.iter().filter_map(|x| {
                if x % 2 == 0 {
                    Some(x)
                } else {
                    None
                }
            }))
            .is_err());
    }
}
