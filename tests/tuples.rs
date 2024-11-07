use const_serialize::{deserialize_const, serialize_const, ConstWriteBuffer};

#[test]
fn test_serialize_const_layout_tuple() {
    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&(1234u32, 5678u16), buf);
    let buf = buf.read();
    assert_eq!(
        deserialize_const!((u32, u16), buf),
        Some((1234u32, 5678u16))
    );

    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&(1234f64, 5678u16, 90u8), buf);
    let buf = buf.read();
    assert_eq!(
        deserialize_const!((f64, u16, u8), buf),
        Some((1234f64, 5678u16, 90u8))
    );

    let mut buf = ConstWriteBuffer::new();
    buf = serialize_const(&(1234u32, 5678u16, 90u8, 1000000f64), buf);
    let buf = buf.read();
    assert_eq!(
        deserialize_const!((u32, u16, u8, f64), buf),
        Some((1234u32, 5678u16, 90u8, 1000000f64))
    );
}
