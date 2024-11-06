use crate::const_vec::ConstVec;

pub struct ConstReadBuffer<'a> {
    location: usize,
    memory: &'a [u8],
}

impl<'a> ConstReadBuffer<'a> {
    pub const fn new(memory: &'a [u8]) -> Self {
        Self {
            location: 0,
            memory,
        }
    }

    pub const fn get(mut self) -> (Self, u8) {
        let value = self.memory[self.location];
        self.location += 1;
        (self, value)
    }

    pub const fn as_ref(&self) -> &[u8] {
        self.memory
    }
}

pub struct ConstWriteBuffer {
    memory: ConstVec<u8>,
}

impl ConstWriteBuffer {
    pub const fn new() -> Self {
        Self {
            memory: ConstVec::new(),
        }
    }

    pub const fn push(self, value: u8) -> Self {
        let memory = self.memory.push(value);
        Self { memory }
    }

    pub const fn as_ref(&self) -> &[u8] {
        self.memory.as_ref()
    }

    pub const fn read(&self) -> ConstReadBuffer {
        ConstReadBuffer::new(self.memory.as_ref())
    }
}

// macro_rules! impl_put_get_number {
//     ($type:ty, $put_method:ident, $put_method_slice:ident, $put_method_array: ident, $get_method:ident, $get_method_slice:ident, $get_method_array:ident) => {
//         impl ConstWriteBuffer {
//             pub const fn $put_method(&mut self, index: $type) {
//                 let buf = index.to_le_bytes();
//                 let mut i = 0;
//                 while i < buf.len() {
//                     self.push(buf[i]);
//                     i += 1;
//                 }
//             }

//             pub const fn $put_method_slice(&mut self, index: &[$type]) {
//                 let mut i = 0;
//                 self.put_usize(index.len());
//                 while i < index.len() {
//                     self.$put_method(index[i]);
//                     i += 1;
//                 }
//             }

//             pub const fn $put_method_array<const N: usize>(&mut self, index: &[$type; N]) {
//                 let mut i = 0;
//                 while i < N {
//                     self.$put_method(index[i]);
//                     i += 1;
//                 }
//             }
//         }

//         impl ConstReadBuffer<'_> {
//             pub const fn $get_method(&mut self) -> $type {
//                 let mut buf = [0u8; std::mem::size_of::<$type>()];
//                 let mut i = 0;
//                 while i < buf.len() {
//                     buf[i] = self.get();
//                     i += 1;
//                 }
//                 <$type>::from_le_bytes(buf)
//             }

//             pub const fn $get_method_slice(&mut self, slice: &mut ConstVec<$type>) {
//                 let mut i = 0;
//                 let len = self.get_usize();
//                 while i < len {
//                     slice.push(self.$get_method());
//                     i += 1;
//                 }
//             }

//             pub const fn $get_method_array<const N: usize>(&mut self) -> [$type; N] {
//                 let mut i = 0;
//                 let mut array: [MaybeUninit<$type>; N] = [MaybeUninit::uninit(); N];
//                 while i < N {
//                     array[i] = MaybeUninit::new(self.$get_method());
//                     i += 1;
//                 }
//                 unsafe { std::mem::transmute_copy(&array) }
//             }
//         }
//     };
// }

// impl ConstWriteBuffer {
//     pub const fn put_usize(&mut self, index: usize) {
//         self.put_u64(index as u64)
//     }

//     pub const fn put_usize_slice(&mut self, index: &[usize]) {
//         let mut i = 0;
//         self.put_usize(index.len());
//         while i < index.len() {
//             self.put_usize(index[i]);
//             i += 1;
//         }
//     }

//     pub const fn put_isize(&mut self, index: isize) {
//         self.put_i64(index as i64)
//     }

//     pub const fn put_isize_slice(&mut self, index: &[isize]) {
//         let mut i = 0;
//         self.put_usize(index.len());
//         while i < index.len() {
//             self.put_isize(index[i]);
//             i += 1;
//         }
//     }

//     pub const fn put_bool(&mut self, index: bool) {
//         self.put_u8(if index { 1 } else { 0 });
//     }

//     pub const fn put_bool_slice(&mut self, index: &[bool]) {
//         let mut i = 0;
//         self.put_usize(index.len());
//         while i < index.len() {
//             self.put_bool(index[i]);
//             i += 1;
//         }
//     }

//     pub const fn put_bool_array<const N: usize>(&mut self, index: &[bool; N]) {
//         let mut i = 0;
//         while i < N {
//             self.put_bool(index[i]);
//             i += 1;
//         }
//     }
// }

// impl ConstReadBuffer<'_> {
//     pub const fn get_usize(&mut self) -> usize {
//         self.get_u64() as usize
//     }

//     pub const fn put_usize_slice(&mut self, slice: &mut ConstVec<usize>) {
//         let mut i = 0;
//         let len = self.get_usize();
//         while i < len {
//             slice.push(self.get_usize());
//             i += 1;
//         }
//     }

//     pub const fn get_isize(&mut self) -> isize {
//         self.get_i64() as isize
//     }

//     pub const fn put_isize_slice(&mut self, slice: &mut ConstVec<isize>) {
//         let mut i = 0;
//         let len = self.get_usize();
//         while i < len {
//             slice.push(self.get_isize());
//             i += 1;
//         }
//     }

//     pub const fn get_bool(&mut self) -> bool {
//         self.get_u8() != 0
//     }

//     pub const fn put_bool_slice(&mut self, slice: &mut ConstVec<bool>) {
//         let mut i = 0;
//         let len = self.get_usize();
//         while i < len {
//             slice.push(self.get_bool());
//             i += 1;
//         }
//     }

//     pub const fn get_bool_array<const N: usize>(&mut self) -> [bool; N] {
//         let mut i = 0;
//         let mut array: [MaybeUninit<bool>; N] = [MaybeUninit::uninit(); N];
//         while i < N {
//             array[i] = MaybeUninit::new(self.get_bool());
//             i += 1;
//         }
//         unsafe { std::mem::transmute_copy(&array) }
//     }
// }

// impl_put_get_number!(
//     u8,
//     put_u8,
//     put_u8_slice,
//     put_u8_array,
//     get_u8,
//     get_u8_slice,
//     get_u8_array
// );
// impl_put_get_number!(
//     u16,
//     put_u16,
//     put_u16_slice,
//     put_u16_array,
//     get_u16,
//     get_u16_slice,
//     get_u16_array
// );
// impl_put_get_number!(
//     u32,
//     put_u32,
//     put_u32_slice,
//     put_u32_array,
//     get_u32,
//     get_u32_slice,
//     get_u32_array
// );
// impl_put_get_number!(
//     u64,
//     put_u64,
//     put_u64_slice,
//     put_u64_array,
//     get_u64,
//     get_u64_slice,
//     get_u64_array
// );
// impl_put_get_number!(
//     i8,
//     put_i8,
//     put_i8_slice,
//     put_i8_array,
//     get_i8,
//     get_i8_slice,
//     get_i8_array
// );
// impl_put_get_number!(
//     i16,
//     put_i16,
//     put_i16_slice,
//     put_i16_array,
//     get_i16,
//     get_i16_slice,
//     get_i16_array
// );
// impl_put_get_number!(
//     i32,
//     put_i32,
//     put_i32_slice,
//     put_i32_array,
//     get_i32,
//     get_i32_slice,
//     get_i32_array
// );
// impl_put_get_number!(
//     i64,
//     put_i64,
//     put_i64_slice,
//     put_i64_array,
//     get_i64,
//     get_i64_slice,
//     get_i64_array
// );

// impl ConstWriteBuffer {
//     pub const fn put_str(&mut self, string: &str) {
//         self.put_u8_slice(string.as_bytes())
//     }
// }

// impl<'a> ConstReadBuffer<'a> {
//     pub const fn get_str(&mut self) -> &'a str {
//         let size = self.get_usize();
//         let start = self.location;
//         let bytes = self.memory.split_at(start).1;
//         let bytes = bytes.split_at(size).0;
//         match std::str::from_utf8(bytes) {
//             Ok(string) => string,
//             Err(_) => panic!("Invalid UTF-8 string"),
//         }
//     }
// }

// #[test]
// fn test_serialize_const_numbers() {
//     let mut buf = ConstWriteBuffer::new();
//     buf.put_u8(123);
//     buf.put_u16(1234);
//     buf.put_u32(123456);
//     buf.put_u64(12345678901234);
//     buf.put_i8(-123);
//     buf.put_i16(-1234);
//     buf.put_i32(-123456);
//     buf.put_i64(-12345678901234);
//     let mut read = buf.read();
//     assert_eq!(read.get_u8(), 123);
//     assert_eq!(read.get_u16(), 1234);
//     assert_eq!(read.get_u32(), 123456);
//     assert_eq!(read.get_u64(), 12345678901234);
//     assert_eq!(read.get_i8(), -123);
//     assert_eq!(read.get_i16(), -1234);
//     assert_eq!(read.get_i32(), -123456);
//     assert_eq!(read.get_i64(), -12345678901234);
// }

// #[test]
// fn test_serialize_const_slice() {
//     let mut buf = ConstWriteBuffer::new();
//     buf.put_u16_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
//     let mut read = buf.read();
//     let mut out = ConstVec::new();
//     read.get_u16_slice(&mut out);
//     assert_eq!(out.as_ref(), &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
// }

// #[test]
// fn test_serialize_const_array() {
//     let mut buf = ConstWriteBuffer::new();
//     buf.put_u8_array(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
//     let mut read = buf.read();
//     let out: [u8; 10] = read.get_u8_array();
//     assert_eq!(out.as_ref(), &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
// }

// #[test]
// fn test_serialize_const_bool() {
//     let mut buf = ConstWriteBuffer::new();
//     buf.put_bool(true);
//     buf.put_bool(false);
//     let mut read = buf.read();
//     assert_eq!(read.get_bool(), true);
//     assert_eq!(read.get_bool(), false);
// }

// #[test]
// fn test_serialize_const_string() {
//     let mut buf = ConstWriteBuffer::new();
//     buf.put_str("Hello world");
//     let mut read = buf.read();
//     assert_eq!(read.get_str(), "Hello world");
// }

// #[test]
// fn test_serialize_const() {
//     #[derive(Debug)]
//     struct MyStruct<'a> {
//         value: u32,
//         name: &'a str,
//         flag: bool,
//         array: [u8; 10],
//     }

//     impl<'a> MyStruct<'a> {
//         const fn serialize(&self, buf: &mut ConstWriteBuffer) {
//             buf.put_u32(self.value);
//             buf.put_str(self.name);
//             buf.put_bool(self.flag);
//             buf.put_u8_array(&self.array);
//         }

//         const fn deserialize(buf: &mut ConstReadBuffer<'a>) -> Self {
//             Self {
//                 value: buf.get_u32(),
//                 name: buf.get_str(),
//                 flag: buf.get_bool(),
//                 array: buf.get_u8_array(),
//             }
//         }
//     }

//     let mut buf = ConstWriteBuffer::new();
//     let my_struct = MyStruct {
//         value: 1234,
//         name: "Hello world",
//         flag: true,
//         array: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
//     };
//     my_struct.serialize(&mut buf);
//     println!("{:?}", buf.as_ref());
//     MyStruct::deserialize(&mut buf.read());
// }
