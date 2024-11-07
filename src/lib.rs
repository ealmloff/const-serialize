use std::mem::MaybeUninit;

mod const_serde;
mod const_vec;

use crate::const_serde::{ConstReadBuffer, ConstWriteBuffer};

/// Plain old data for a field. Stores the offset of the field in the struct and the encoding of the field.
#[derive(Debug, Copy, Clone)]
pub struct PlainOldData {
    offset: usize,
    encoding: &'static Encoding,
}

/// Encoding for a struct. The struct encoding is just a list of fields with offsets
#[derive(Debug, Copy, Clone)]
pub struct StructEncoding {
    size: usize,
    data: &'static [PlainOldData],
}

/// The encoding for a constant sized array. The array encoding is just a length and an item encoding.
#[derive(Debug, Copy, Clone)]
pub struct ListEncoding {
    len: usize,
    item_encoding: &'static Encoding,
}

/// The encoding for a primitive type. The bytes will be reversed if the target is big endian.
#[derive(Debug, Copy, Clone)]
pub struct PrimitiveEncoding {
    size: usize,
    reverse_bytes: bool,
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
        Encoding::Struct(encoding) => serialize_const_struct(ptr, to, &encoding),
        Encoding::List(encoding) => serialize_const_list(ptr, to, &encoding),
        Encoding::Primitive(encoding) => serialize_const_primitive(ptr, to, &encoding),
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

/// Deserialize a buffer into a type
/// # Safety
/// N must be `std::mem::size_of::<T>()`
pub const unsafe fn deserialize_const<const N: usize, T: SerializeConst>(
    from: ConstReadBuffer,
) -> T {
    // Create uninitized memory with the size of the type
    let out = [MaybeUninit::uninit(); N];
    // Fill in the bytes into the buffer for the type
    let (_, out) = deserialize_const_ptr(from, &T::ENCODING, (0, out));
    // Now that the memory is filled in, transmute it into the type
    unsafe { std::mem::transmute_copy::<[MaybeUninit<u8>; N], T>(&out) }
}

#[test]
fn test_crimes() {
    struct MyStruct {
        a: u32,
        b: u8,
        c: u32,
        d: u32,
    }
    const SIZE: usize = std::mem::size_of::<MyStruct>();
    let mut out = [MaybeUninit::uninit(); SIZE];
    let first_align = std::mem::offset_of!(MyStruct, a);
    let second_align = std::mem::offset_of!(MyStruct, b);
    let third_align = std::mem::offset_of!(MyStruct, c);
    let fourth_align = std::mem::offset_of!(MyStruct, d);
    for (i, byte) in 1234u32.to_le_bytes().iter().enumerate() {
        out[i + first_align] = MaybeUninit::new(*byte);
    }
    for (i, byte) in 12u8.to_le_bytes().iter().enumerate() {
        out[i + second_align] = MaybeUninit::new(*byte);
    }
    for (i, byte) in 13u32.to_le_bytes().iter().enumerate() {
        out[i + third_align] = MaybeUninit::new(*byte);
    }
    for (i, byte) in 14u32.to_le_bytes().iter().enumerate() {
        out[i + fourth_align] = MaybeUninit::new(*byte);
    }
    let out = unsafe { std::mem::transmute_copy::<[MaybeUninit<u8>; SIZE], MyStruct>(&out) };
    assert_eq!(out.a, 1234);
    assert_eq!(out.b, 12);
    assert_eq!(out.c, 13);
    assert_eq!(out.d, 14);
}

#[test]
fn test_serialize_const_layout_primitive() {
    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&1234u32, buf);
    assert_eq!(buf.as_ref(), 1234u32.to_le_bytes());
    let buf = buf.read();
    const SIZE_U32: usize = std::mem::size_of::<u32>();
    unsafe { assert_eq!(deserialize_const::<SIZE_U32, u32>(buf), 1234u32) };

    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&1234u64, buf);
    assert_eq!(buf.as_ref(), 1234u64.to_le_bytes());
    let buf = buf.read();
    const SIZE_U64: usize = std::mem::size_of::<u64>();
    unsafe { assert_eq!(deserialize_const::<SIZE_U64, u64>(buf), 1234u64) };

    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&1234i32, buf);
    assert_eq!(buf.as_ref(), 1234i32.to_le_bytes());
    let buf = buf.read();
    const SIZE_I32: usize = std::mem::size_of::<i32>();
    unsafe { assert_eq!(deserialize_const::<SIZE_I32, i32>(buf), 1234i32) };

    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&1234i64, buf);
    assert_eq!(buf.as_ref(), 1234i64.to_le_bytes());
    let buf = buf.read();
    const SIZE_I64: usize = std::mem::size_of::<i64>();
    unsafe { assert_eq!(deserialize_const::<SIZE_I64, i64>(buf), 1234i64) };

    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&true, buf);
    assert_eq!(buf.as_ref(), [1u8]);
    let buf = buf.read();
    const SIZE_BOOL: usize = std::mem::size_of::<bool>();
    unsafe { assert_eq!(deserialize_const::<SIZE_BOOL, bool>(buf), true) };
}

#[test]
fn test_serialize_const_layout_list() {
    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&[1u8, 2, 3] as &[u8; 3], buf);
    println!("{:?}", buf.as_ref());
    let buf = buf.read();
    const SIZE_ARRAY: usize = std::mem::size_of::<[u8; 3]>();
    unsafe { assert_eq!(deserialize_const::<SIZE_ARRAY, [u8; 3]>(buf), [1, 2, 3]) };
}

#[test]
fn test_serialize_const_layout_nested_lists() {
    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(
        &[[1u8, 2, 3], [4u8, 5, 6], [7u8, 8, 9]] as &[[u8; 3]; 3],
        buf,
    );
    println!("{:?}", buf.as_ref());
    let buf = buf.read();
    const SIZE_ARRAY: usize = std::mem::size_of::<[[u8; 3]; 3]>();
    assert_eq!(
        unsafe { deserialize_const::<SIZE_ARRAY, [[u8; 3]; 3]>(buf) },
        [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
    );
}

#[test]
fn test_serialize_const_layout_struct() {
    #[derive(Debug, PartialEq)]
    struct Struct {
        a: u32,
        b: u8,
        c: u32,
        d: u32,
    }

    unsafe impl SerializeConst for Struct {
        const ENCODING: Encoding = Encoding::Struct(StructEncoding {
            size: std::mem::size_of::<Struct>(),
            data: &[
                PlainOldData {
                    offset: std::mem::offset_of!(Struct, a),
                    encoding: &u32::ENCODING,
                },
                PlainOldData {
                    offset: std::mem::offset_of!(Struct, b),
                    encoding: &u8::ENCODING,
                },
                PlainOldData {
                    offset: std::mem::offset_of!(Struct, c),
                    encoding: &u32::ENCODING,
                },
                PlainOldData {
                    offset: std::mem::offset_of!(Struct, d),
                    encoding: &u32::ENCODING,
                },
            ],
        });
    }

    #[derive(Debug, PartialEq)]
    struct OtherStruct {
        a: u32,
        b: u8,
        c: Struct,
        d: u32,
    }

    unsafe impl SerializeConst for OtherStruct {
        const ENCODING: Encoding = Encoding::Struct(StructEncoding {
            size: std::mem::size_of::<OtherStruct>(),
            data: &[
                PlainOldData {
                    offset: std::mem::offset_of!(OtherStruct, a),
                    encoding: &u32::ENCODING,
                },
                PlainOldData {
                    offset: std::mem::offset_of!(OtherStruct, b),
                    encoding: &u8::ENCODING,
                },
                PlainOldData {
                    offset: std::mem::offset_of!(OtherStruct, c),
                    encoding: &Struct::ENCODING,
                },
                PlainOldData {
                    offset: std::mem::offset_of!(OtherStruct, d),
                    encoding: &u32::ENCODING,
                },
            ],
        });
    }

    println!("{:?}", OtherStruct::ENCODING);

    let data = Struct {
        a: 0x11111111,
        b: 0x22,
        c: 0x33333333,
        d: 0x44444444,
    };
    let data = OtherStruct {
        a: 0x11111111,
        b: 0x22,
        c: data,
        d: 0x44444444,
    };
    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&data, buf);
    println!("{:?}", buf.as_ref());
    let buf = buf.read();
    const SIZE: usize = std::mem::size_of::<OtherStruct>();
    let data2 = unsafe { deserialize_const::<SIZE, OtherStruct>(buf) };
    assert_eq!(data, data2);
}

#[test]
fn test_serialize_const_layout_struct_list() {
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Struct {
        a: u32,
        b: u8,
        c: u32,
        d: u32,
    }

    impl Struct {
        const fn equal(&self, other: &Struct) -> bool {
            self.a == other.a && self.b == other.b && self.c == other.c && self.d == other.d
        }
    }

    unsafe impl SerializeConst for Struct {
        const ENCODING: Encoding = Encoding::Struct(StructEncoding {
            size: std::mem::size_of::<Struct>(),
            data: &[
                PlainOldData {
                    offset: std::mem::offset_of!(Struct, a),
                    encoding: &u32::ENCODING,
                },
                PlainOldData {
                    offset: std::mem::offset_of!(Struct, b),
                    encoding: &u8::ENCODING,
                },
                PlainOldData {
                    offset: std::mem::offset_of!(Struct, c),
                    encoding: &u32::ENCODING,
                },
                PlainOldData {
                    offset: std::mem::offset_of!(Struct, d),
                    encoding: &u32::ENCODING,
                },
            ],
        });
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct OtherStruct {
        a: u32,
        b: u8,
        c: Struct,
        d: u32,
    }

    impl OtherStruct {
        const fn equal(&self, other: &OtherStruct) -> bool {
            self.a == other.a && self.b == other.b && self.c.equal(&other.c) && self.d == other.d
        }
    }

    unsafe impl SerializeConst for OtherStruct {
        const ENCODING: Encoding = Encoding::Struct(StructEncoding {
            size: std::mem::size_of::<OtherStruct>(),
            data: &[
                PlainOldData {
                    offset: std::mem::offset_of!(OtherStruct, a),
                    encoding: &u32::ENCODING,
                },
                PlainOldData {
                    offset: std::mem::offset_of!(OtherStruct, b),
                    encoding: &u8::ENCODING,
                },
                PlainOldData {
                    offset: std::mem::offset_of!(OtherStruct, c),
                    encoding: &Struct::ENCODING,
                },
                PlainOldData {
                    offset: std::mem::offset_of!(OtherStruct, d),
                    encoding: &u32::ENCODING,
                },
            ],
        });
    }

    const INNER_DATA: Struct = Struct {
        a: 0x11111111,
        b: 0x22,
        c: 0x33333333,
        d: 0x44444444,
    };
    const DATA: [OtherStruct; 3] = [
        OtherStruct {
            a: 0x11111111,
            b: 0x22,
            c: INNER_DATA,
            d: 0x44444444,
        },
        OtherStruct {
            a: 0x111111,
            b: 0x23,
            c: INNER_DATA,
            d: 0x44444444,
        },
        OtherStruct {
            a: 0x11111111,
            b: 0x11,
            c: INNER_DATA,
            d: 0x44441144,
        },
    ];

    const SIZE: usize = std::mem::size_of::<[OtherStruct; 3]>();
    const _ASSERT: () = {
        let mut buf = ConstWriteBuffer::new();
        buf = serialize_const(&DATA, buf);
        let buf = buf.read();
        let [first, second, third] = unsafe { deserialize_const::<SIZE, [OtherStruct; 3]>(buf) };
        if !(first.equal(&DATA[0]) && second.equal(&DATA[1]) && third.equal(&DATA[2])) {
            panic!("data mismatch");
        }
    };
    const _ASSERT_2: () = {
        let mut buf = ConstWriteBuffer::new();
        const DATA_AGAIN: [[OtherStruct; 3]; 3] = [DATA, DATA, DATA];
        const ARR_SIZE: usize = std::mem::size_of::<[[OtherStruct; 3]; 3]>();
        buf = serialize_const(&DATA_AGAIN, buf);
        let buf = buf.read();
        let [first, second, third] =
            unsafe { deserialize_const::<ARR_SIZE, [[OtherStruct; 3]; 3]>(buf) };
        if !(first[0].equal(&DATA[0]) && first[1].equal(&DATA[1]) && first[2].equal(&DATA[2])) {
            panic!("data mismatch");
        }
        if !(second[0].equal(&DATA[0]) && second[1].equal(&DATA[1]) && second[2].equal(&DATA[2])) {
            panic!("data mismatch");
        }
        if !(third[0].equal(&DATA[0]) && third[1].equal(&DATA[1]) && third[2].equal(&DATA[2])) {
            panic!("data mismatch");
        }
    };

    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&DATA, buf);
    println!("{:?}", buf.as_ref());
    let buf = buf.read();
    let data2 = unsafe { deserialize_const::<SIZE, [OtherStruct; 3]>(buf) };
    assert_eq!(DATA, data2);
}
