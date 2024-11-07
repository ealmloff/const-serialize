use const_serialize::{deserialize_const, serialize_const, ConstWriteBuffer};

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
