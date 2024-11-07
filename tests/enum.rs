use const_serialize::{deserialize_const, serialize_const, ConstWriteBuffer};
use std::mem::MaybeUninit;

#[test]
fn test_transmute_bytes_to_enum() {
    #[derive(Clone, Copy, Debug, PartialEq)]
    #[repr(C, u8)]
    enum Enum<T> {
        A { one: u32, two: u16 },
        B { one: u8, two: T } = 15,
    }

    #[repr(C)]
    #[derive(Debug, PartialEq)]
    struct A {
        one: u32,
        two: u16,
    }

    #[repr(C)]
    #[derive(Debug, PartialEq)]
    struct B<T> {
        one: u8,
        two: T,
    }

    const SIZE: usize = std::mem::size_of::<Enum<u16>>();
    let mut out = [MaybeUninit::uninit(); SIZE];
    let discriminate_size = std::mem::size_of::<u8>();
    let tag_align = 0;
    let union_alignment = std::mem::align_of::<A>().max(std::mem::align_of::<B<u16>>());
    let data_align = (discriminate_size / union_alignment) + union_alignment;
    let a_one_align = std::mem::offset_of!(A, one);
    let a_two_align = std::mem::offset_of!(A, two);
    let b_one_align = std::mem::offset_of!(B<u16>, one);
    let b_two_align = std::mem::offset_of!(B<u16>, two);

    let one = 1234u32;
    let two = 5678u16;
    let first = Enum::A { one, two };
    for (i, byte) in one.to_le_bytes().iter().enumerate() {
        out[data_align + i + a_one_align] = MaybeUninit::new(*byte);
    }
    for (i, byte) in two.to_le_bytes().iter().enumerate() {
        out[data_align + i + a_two_align] = MaybeUninit::new(*byte);
    }
    out[tag_align] = MaybeUninit::new(0);
    let out = unsafe { std::mem::transmute_copy::<[MaybeUninit<u8>; SIZE], Enum<u16>>(&out) };
    assert_eq!(out, first);

    let mut out = [MaybeUninit::uninit(); SIZE];
    let one = 123u8;
    let two = 58u16;
    let second = Enum::B { one, two };
    for (i, byte) in one.to_le_bytes().iter().enumerate() {
        out[data_align + i + b_one_align] = MaybeUninit::new(*byte);
    }
    for (i, byte) in two.to_le_bytes().iter().enumerate() {
        out[data_align + i + b_two_align] = MaybeUninit::new(*byte);
    }
    out[tag_align] = MaybeUninit::new(15);
    let out = unsafe { std::mem::transmute_copy::<[MaybeUninit<u8>; SIZE], Enum<u16>>(&out) };
    assert_eq!(out, second);
}

#[test]
fn test_serialize_enum() {
    #[derive(Clone, Copy, Debug, PartialEq)]
    #[repr(C, u8)]
    enum Enum {
        A { one: u32, two: u16 },
        B { one: u8, two: u16 } = 15,
    }

    #[repr(C)]
    struct A {
        one: u32,
        two: u16,
    }

    #[repr(C)]
    struct B {
        one: u8,
        two: u16,
    }

    unsafe impl const_serialize::SerializeConst for Enum {
        const ENCODING: const_serialize::Encoding =
            const_serialize::Encoding::Enum(const_serialize::EnumEncoding::new(
                std::mem::size_of::<Self>(),
                std::mem::size_of::<u8>(),
                cfg!(target_endian = "big"),
                {
                    let union_alignment = {
                        let first = std::mem::align_of::<A>();
                        let second = std::mem::align_of::<B>();
                        if first > second {
                            first
                        } else {
                            second
                        }
                    };
                    (std::mem::size_of::<u8>() / union_alignment) + union_alignment
                },
                {
                    const DATA: &'static [const_serialize::EnumVariant] = &[
                        const_serialize::EnumVariant::new(
                            0,
                            const_serialize::StructEncoding::new(
                                std::mem::size_of::<A>(),
                                &[
                                    const_serialize::PlainOldData::new(
                                        std::mem::offset_of!(A, one),
                                        u32::ENCODING,
                                    ),
                                    const_serialize::PlainOldData::new(
                                        std::mem::offset_of!(A, two),
                                        u16::ENCODING,
                                    ),
                                ],
                            ),
                        ),
                        const_serialize::EnumVariant::new(
                            15,
                            const_serialize::StructEncoding::new(
                                std::mem::size_of::<B>(),
                                &[
                                    const_serialize::PlainOldData::new(
                                        std::mem::offset_of!(B, one),
                                        u8::ENCODING,
                                    ),
                                    const_serialize::PlainOldData::new(
                                        std::mem::offset_of!(B, two),
                                        u16::ENCODING,
                                    ),
                                ],
                            ),
                        ),
                    ];
                    DATA
                },
            ));
    }

    let data = Enum::A {
        one: 0x11111111,
        two: 0x22,
    };
    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&data, buf);
    println!("{:?}", buf.as_ref());
    let buf = buf.read();
    assert_eq!(deserialize_const!(Enum, buf), Some(data));
}
