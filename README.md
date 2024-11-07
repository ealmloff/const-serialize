A rust serialization library that works in const with complex(ish) types like nested structs and arrays.
```rust
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

const _ASSERT: () = {
    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&DATA, buf);
    let buf = buf.read();
    const SIZE: usize = std::mem::size_of::<[OtherStruct; 3]>();
    let [first, second, third] = unsafe { deserialize_const::<SIZE, [OtherStruct; 3]>(buf) };
    if !(first.equal(&DATA[0]) && second.equal(&DATA[1]) && third.equal(&DATA[2])) {
        panic!("data mismatch");
    }
};
```