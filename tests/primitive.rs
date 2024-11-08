use const_serialize::{deserialize_const, serialize_const, ConstWriteBuffer};

#[test]
fn test_serialize_const_layout_primitive() {
    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&1234u32, buf);
    if cfg!(feature = "test-big-endian") {
        assert_eq!(buf.as_ref(), 1234u32.to_be_bytes());
    } else {
        assert_eq!(buf.as_ref(), 1234u32.to_le_bytes());
    }
    let buf = buf.read();
    assert_eq!(deserialize_const!(u32, buf), Some(1234u32));

    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&1234u64, buf);
    if cfg!(feature = "test-big-endian") {
        assert_eq!(buf.as_ref(), 1234u64.to_be_bytes());
    } else {
        assert_eq!(buf.as_ref(), 1234u64.to_le_bytes());
    }
    let buf = buf.read();
    assert_eq!(deserialize_const!(u64, buf), Some(1234u64));

    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&1234i32, buf);
    if cfg!(feature = "test-big-endian") {
        assert_eq!(buf.as_ref(), 1234i32.to_be_bytes());
    } else {
        assert_eq!(buf.as_ref(), 1234i32.to_le_bytes());
    }
    let buf = buf.read();
    assert_eq!(deserialize_const!(i32, buf), Some(1234i32));

    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&1234i64, buf);
    if cfg!(feature = "test-big-endian") {
        assert_eq!(buf.as_ref(), 1234i64.to_be_bytes());
    } else {
        assert_eq!(buf.as_ref(), 1234i64.to_le_bytes());
    }
    let buf = buf.read();
    assert_eq!(deserialize_const!(i64, buf), Some(1234i64));

    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&true, buf);
    assert_eq!(buf.as_ref(), [1u8]);
    let buf = buf.read();
    assert_eq!(deserialize_const!(bool, buf), Some(true));

    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&0.631f32, buf);
    if cfg!(feature = "test-big-endian") {
        assert_eq!(buf.as_ref(), 0.631f32.to_be_bytes());
    } else {
        assert_eq!(buf.as_ref(), 0.631f32.to_le_bytes());
    }
    let buf = buf.read();
    assert_eq!(deserialize_const!(f32, buf), Some(0.631));
}

#[test]

fn test_serialize_primitive_too_little_data() {
    let mut buf = ConstWriteBuffer::new();
    buf = buf.push(1);
    buf = buf.push(1);
    buf = buf.push(1);
    let buf = buf.read();
    assert_eq!(deserialize_const!(u64, buf), None);
}
