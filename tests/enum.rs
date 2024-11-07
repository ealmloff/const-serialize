use std::mem::MaybeUninit;

#[test]
fn test_transmute_bytes_to_enum() {
    #[derive(Clone, Copy, Debug, PartialEq)]
    #[repr(C, u8)]
    enum Enum {
        A { one: u32, two: u16 },
        B { one: u8, two: u16 },
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CRepr {
        tag: u8,
        data: Merged,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    union Merged {
        a: A,
        b: B,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct A {
        one: u32,
        two: u16,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct B {
        one: u8,
        two: u16,
    }

    const SIZE: usize = std::mem::size_of::<Enum>();
    let mut out = [MaybeUninit::uninit(); SIZE];
    let tag_align = std::mem::offset_of!(CRepr, tag);
    let data_align = std::mem::offset_of!(CRepr, data);
    let a_one_align = std::mem::offset_of!(A, one);
    let a_two_align = std::mem::offset_of!(A, two);
    let b_one_align = std::mem::offset_of!(B, one);
    let b_two_align = std::mem::offset_of!(B, two);

    let one = 1234u32;
    let two = 5678u16;
    let first = Enum::A {
        one,
        two,
    };
    for (i, byte) in one.to_le_bytes().iter().enumerate() {
        out[data_align + i + a_one_align] = MaybeUninit::new(*byte);
    }
    for (i, byte) in two.to_le_bytes().iter().enumerate() {
        out[data_align + i + a_two_align] = MaybeUninit::new(*byte);
    }
    out[tag_align] = MaybeUninit::new(0);
    let out = unsafe { std::mem::transmute_copy::<[MaybeUninit<u8>; SIZE], Enum>(&out) };
    assert_eq!(out, first);

    let mut out = [MaybeUninit::uninit(); SIZE];
    let one = 123u8;
    let two = 58u16;
    let second = Enum::B {
        one,
        two,
    };
    for (i, byte) in one.to_le_bytes().iter().enumerate() {
        out[data_align + i + b_one_align] = MaybeUninit::new(*byte);
    }
    for (i, byte) in two.to_le_bytes().iter().enumerate() {
        out[data_align + i + b_two_align] = MaybeUninit::new(*byte);
    }
    out[tag_align] = MaybeUninit::new(1);
    let out = unsafe { std::mem::transmute_copy::<[MaybeUninit<u8>; SIZE], Enum>(&out) };
    assert_eq!(out, second);
}
