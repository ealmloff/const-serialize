A rust serialization library that works in const with complex(ish) types like enums, nested structs and arrays. Const rust does not have an allocator, so this library cannot work in a cross architecture environment with Vecs, slices or strings.

```rust
#[derive(Clone, Copy, Debug, PartialEq, SerializeConst)]
struct Struct {
    a: u32,
    b: u8,
    c: u32,
    d: Enum,
}

#[derive(Clone, Copy, Debug, PartialEq, SerializeConst)]
#[repr(C, u8)]
enum Enum {
    A { one: u32, two: u16 },
    B { one: u8, two: u16 } = 15,
}


const {
    let data = [Struct {
        a: 0x11111111,
        b: 0x22,
        c: 0x33333333,
        d: Enum::A {
            one: 0x44444444,
            two: 0x5555,
        },
    }; 3];
    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&data, buf);
    let buf = buf.read();
    let [first, second, third] = match deserialize_const!([Struct; 3], buf) {
        Some(data) => data,
        None => panic!("data mismatch"),
    };
    if !(first.equal(&data[0]) && second.equal(&data[1]) && third.equal(&data[2])) {
        panic!("data mismatch");
    }
}

impl Struct {
    const fn equal(&self, other: &Struct) -> bool {
        self.a == other.a && self.b == other.b && self.c == other.c && self.d.equal(&other.d)
    }
}

impl Enum {
    const fn equal(&self, other: &Enum) -> bool {
        match (self, other) {
            (
                Enum::A { one, two },
                Enum::A {
                    one: other_one,
                    two: other_two,
                },
            ) => *one == *other_one && *two == *other_two,
            (
                Enum::B { one, two },
                Enum::B {
                    one: other_one,
                    two: other_two,
                },
            ) => *one == *other_one && *two == *other_two,
            _ => false,
        }
    }
}
```
