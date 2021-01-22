use char_device::CharDevice;
use std::io::{Read, Write};

#[test]
fn null() {
    let mut char_device = CharDevice::null().unwrap();
    char_device.write_all(b"abcdefg").unwrap();

    let mut buf = vec![0_u8; 32];
    assert_eq!(char_device.read(&mut buf).unwrap(), 0);
}
