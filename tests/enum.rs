use std::mem::MaybeUninit;

#[test]
fn test_transmute_bytes_to_enum() {
    #[derive(Clone, Copy, Debug, PartialEq)]
    #[repr(C, u8)]
    enum Enum<T> {
        A { one: u32, two: u16 },
        B { one: u8, two: T },
    }

    // #[repr(C)]
    // #[derive(Clone, Copy)]
    // struct CRepr<T: Copy> {
    //     tag: u8,
    //     data: Merged<T>,
    // }

    // #[repr(C)]
    // #[derive(Clone, Copy)]
    // union Merged<T: Copy> {
    //     a: A,
    //     b: B<T>,
    // }

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
    let tag_align = 0;
    let data_align = 1.max(std::mem::align_of::<A>()).max(std::mem::align_of::<B<u16>>()); 
    let a_one_align = std::mem::offset_of!(A, one);
    let a_two_align = std::mem::offset_of!(A, two);
    let b_one_align = std::mem::offset_of!(B<u16>, one);
    let b_two_align = std::mem::offset_of!(B<u16>, two);

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
    let out = unsafe { std::mem::transmute_copy::<[MaybeUninit<u8>; SIZE], Enum<u16>>(&out) };
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
    let out = unsafe { std::mem::transmute_copy::<[MaybeUninit<u8>; SIZE], Enum<u16>>(&out) };
    assert_eq!(out, second);
}
