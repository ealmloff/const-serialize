use std::mem::MaybeUninit;

mod const_serde;
mod const_vec;

pub use const_serde::{ConstReadBuffer, ConstWriteBuffer};
pub use derive_const_serialize::SerializeConst;

/// Plain old data for a field. Stores the offset of the field in the struct and the encoding of the field.
#[derive(Debug, Copy, Clone)]
pub struct PlainOldData {
    offset: usize,
    encoding: &'static Encoding,
}

impl PlainOldData {
    pub const fn new(offset: usize, encoding: &'static Encoding) -> Self {
        Self { offset, encoding }
    }
}

/// Encoding for a struct. The struct encoding is just a list of fields with offsets
#[derive(Debug, Copy, Clone)]
pub struct StructEncoding {
    size: usize,
    data: &'static [PlainOldData],
}

impl StructEncoding {
    pub const fn new(size: usize, data: &'static [PlainOldData]) -> Self {
        Self { size, data }
    }
}

/// The encoding for a constant sized array. The array encoding is just a length and an item encoding.
#[derive(Debug, Copy, Clone)]
pub struct ListEncoding {
    len: usize,
    item_encoding: &'static Encoding,
}

impl ListEncoding {
    pub const fn new(len: usize, item_encoding: &'static Encoding) -> Self {
        Self { len, item_encoding }
    }
}

/// The encoding for a primitive type. The bytes will be reversed if the target is big endian.
#[derive(Debug, Copy, Clone)]
pub struct PrimitiveEncoding {
    size: usize,
    reverse_bytes: bool,
}

impl PrimitiveEncoding {
    pub const fn new(size: usize, reverse_bytes: bool) -> Self {
        Self {
            size,
            reverse_bytes,
        }
    }
}

/// The encoding for a type. This encoding defines a sequence of locations and reversed or not bytes. These bytes will be copied from during serialization and copied into during deserialization.
#[derive(Debug, Copy, Clone)]
pub enum Encoding {
    Struct(StructEncoding),
    List(ListEncoding),
    Primitive(PrimitiveEncoding),
}

impl Encoding {
    /// The size of the type in bytes.
    const fn size(&self) -> usize {
        match self {
            Encoding::Struct(encoding) => encoding.size,
            Encoding::List(encoding) => encoding.len * encoding.item_encoding.size(),
            Encoding::Primitive(encoding) => encoding.size,
        }
    }
}

/// A trait for types that can be serialized and deserialized in const.
///
/// # Safety
/// The encoding must accurately describe the memory layout of the type
pub unsafe trait SerializeConst: Sized {
    /// The memory layout of the type. This type must have plain old data; no pointers or references.
    const ENCODING: Encoding;
    const _ASSERT: () = assert!(Self::ENCODING.size() == std::mem::size_of::<Self>());
}

macro_rules! impl_serialize_const {
    ($type:ty) => {
        unsafe impl SerializeConst for $type {
            const ENCODING: Encoding = Encoding::Primitive(PrimitiveEncoding {
                size: std::mem::size_of::<$type>(),
                reverse_bytes: cfg!(target_endian = "big"),
            });
        }
    };
}

impl_serialize_const!(u8);
impl_serialize_const!(u16);
impl_serialize_const!(u32);
impl_serialize_const!(u64);
impl_serialize_const!(i8);
impl_serialize_const!(i16);
impl_serialize_const!(i32);
impl_serialize_const!(i64);
impl_serialize_const!(bool);
impl_serialize_const!(f32);
impl_serialize_const!(f64);

unsafe impl<const N: usize, T: SerializeConst> SerializeConst for [T; N] {
    const ENCODING: Encoding = Encoding::List(ListEncoding {
        len: N,
        item_encoding: &T::ENCODING,
    });
}

/// Serialize a struct that is stored at the pointer passed in
const fn serialize_const_struct(
    ptr: *const (),
    mut to: ConstWriteBuffer,
    encoding: &StructEncoding,
) -> ConstWriteBuffer {
    let mut i = 0;
    while i < encoding.data.len() {
        // Serialize the field at the offset pointer in the struct
        let PlainOldData { offset, encoding } = encoding.data[i];
        let field = unsafe { ptr.byte_add(offset) };
        to = serialize_const_ptr(field, to, encoding);
        i += 1;
    }
    to
}

/// Serialize a primitive type that is stored at the pointer passed in
const fn serialize_const_primitive(
    ptr: *const (),
    mut to: ConstWriteBuffer,
    encoding: &PrimitiveEncoding,
) -> ConstWriteBuffer {
    let ptr = ptr as *const u8;
    let mut offset = 0;
    while offset < encoding.size {
        // If the bytes are reversed, walk backwards from the end of the number when pushing bytes
        if encoding.reverse_bytes {
            to = to.push(unsafe { ptr.byte_add(encoding.size - offset - 1).read() });
        } else {
            to = to.push(unsafe { ptr.byte_add(offset).read() });
        }
        offset += 1;
    }
    to
}

/// Serialize a constant sized array that is stored at the pointer passed in
const fn serialize_const_list(
    ptr: *const (),
    mut to: ConstWriteBuffer,
    encoding: &ListEncoding,
) -> ConstWriteBuffer {
    let len = encoding.len;
    let mut i = 0;
    while i < len {
        let field = unsafe { ptr.byte_add(i * encoding.item_encoding.size()) };
        to = serialize_const_ptr(field, to, encoding.item_encoding);
        i += 1;
    }
    to
}

/// Serialize a pointer to a type that is stored at the pointer passed in
const fn serialize_const_ptr(
    ptr: *const (),
    to: ConstWriteBuffer,
    encoding: &Encoding,
) -> ConstWriteBuffer {
    match encoding {
        Encoding::Struct(encoding) => serialize_const_struct(ptr, to, encoding),
        Encoding::List(encoding) => serialize_const_list(ptr, to, encoding),
        Encoding::Primitive(encoding) => serialize_const_primitive(ptr, to, encoding),
    }
}

/// Serialize a type into a buffer
pub const fn serialize_const<T: SerializeConst>(
    data: &T,
    to: ConstWriteBuffer,
) -> ConstWriteBuffer {
    let ptr = data as *const T as *const ();
    serialize_const_ptr(ptr, to, &T::ENCODING)
}

/// Deserialize a primitive type into the out buffer at the offset passed in. Returns a new version of the buffer with the data added.
const fn deserialize_const_primitive<'a, const N: usize>(
    mut from: ConstReadBuffer<'a>,
    encoding: &PrimitiveEncoding,
    out: (usize, [MaybeUninit<u8>; N]),
) -> (ConstReadBuffer<'a>, [MaybeUninit<u8>; N]) {
    let (start, mut out) = out;
    let mut offset = 0;
    while offset < encoding.size {
        // If the bytes are reversed, walk backwards from the end of the number when filling in bytes
        if encoding.reverse_bytes {
            let (from_new, value) = from.get();
            from = from_new;
            out[start + encoding.size - offset - 1] = MaybeUninit::new(value);
        } else {
            let (from_new, value) = from.get();
            from = from_new;
            out[start + offset] = MaybeUninit::new(value);
        }
        offset += 1;
    }
    (from, out)
}

/// Deserialize a struct type into the out buffer at the offset passed in. Returns a new version of the buffer with the data added.
const fn deserialize_const_struct<'a, const N: usize>(
    mut from: ConstReadBuffer<'a>,
    encoding: &StructEncoding,
    out: (usize, [MaybeUninit<u8>; N]),
) -> (ConstReadBuffer<'a>, [MaybeUninit<u8>; N]) {
    let (start, mut out) = out;
    let mut i = 0;
    while i < encoding.data.len() {
        // Deserialize the field at the offset pointer in the struct
        let PlainOldData { offset, encoding } = encoding.data[i];
        let (new_from, new_out) = deserialize_const_ptr(from, encoding, (start + offset, out));
        from = new_from;
        out = new_out;
        i += 1;
    }
    (from, out)
}

/// Deserialize a list type into the out buffer at the offset passed in. Returns a new version of the buffer with the data added.
const fn deserialize_const_list<'a, const N: usize>(
    mut from: ConstReadBuffer<'a>,
    encoding: &ListEncoding,
    out: (usize, [MaybeUninit<u8>; N]),
) -> (ConstReadBuffer<'a>, [MaybeUninit<u8>; N]) {
    let (start, mut out) = out;
    let len = encoding.len;
    let item_encoding = encoding.item_encoding;
    let mut i = 0;
    while i < len {
        let (new_from, new_out) =
            deserialize_const_ptr(from, item_encoding, (start + i * item_encoding.size(), out));
        from = new_from;
        out = new_out;
        i += 1;
    }
    (from, out)
}

/// Deserialize a type into the out buffer at the offset passed in. Returns a new version of the buffer with the data added.
const fn deserialize_const_ptr<'a, const N: usize>(
    from: ConstReadBuffer<'a>,
    encoding: &Encoding,
    out: (usize, [MaybeUninit<u8>; N]),
) -> (ConstReadBuffer<'a>, [MaybeUninit<u8>; N]) {
    match encoding {
        Encoding::Struct(encoding) => deserialize_const_struct(from, encoding, out),
        Encoding::List(encoding) => deserialize_const_list(from, encoding, out),
        Encoding::Primitive(encoding) => deserialize_const_primitive(from, encoding, out),
    }
}

#[macro_export]
macro_rules! deserialize_const {
    ($type:ty, $buffer:expr) => {{
        const __SIZE: usize = std::mem::size_of::<$type>();
        $crate::deserialize_const_raw::<__SIZE, $type>($buffer)
    }};
}

/// Deserialize a buffer into a type
/// # Safety
/// N must be `std::mem::size_of::<T>()`
pub const unsafe fn deserialize_const_raw<const N: usize, T: SerializeConst>(
    from: ConstReadBuffer,
) -> T {
    // Create uninitized memory with the size of the type
    let out = [MaybeUninit::uninit(); N];
    // Fill in the bytes into the buffer for the type
    let (_, out) = deserialize_const_ptr(from, &T::ENCODING, (0, out));
    // Now that the memory is filled in, transmute it into the type
    unsafe { std::mem::transmute_copy::<[MaybeUninit<u8>; N], T>(&out) }
}
