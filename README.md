A rust serialization library that works in const with complex(ish) types like nested structs and arrays. Enums are not supported

```rust
#[derive(Clone, Copy, Debug, PartialEq, SerializeConst)]
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

#[derive(Clone, Copy, Debug, PartialEq, SerializeConst)]
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
    let [first, second, third] = deserialize_const!([OtherStruct; 3], buf).unwrap();
    if !(first.equal(&DATA[0]) && second.equal(&DATA[1]) && third.equal(&DATA[2])) {
        panic!("data mismatch");
    }
};
```
