#[cfg(feature = "async-std")]
use char_device::AsyncCharDevice;
use char_device::CharDevice;

#[test]
fn null() {
    use std::io::{Read, Write};

    let mut char_device = CharDevice::null().unwrap();
    char_device.write_all(b"abcdefg").unwrap();

    let mut buf = vec![0_u8; 32];
    assert_eq!(char_device.read(&mut buf).unwrap(), 0);
}

#[cfg(feature = "async-std")]
#[async_std::test]
async fn async_null() {
    use async_std::io::prelude::{ReadExt, WriteExt};

    let mut char_device = AsyncCharDevice::null().await.unwrap();
    char_device.write_all(b"abcdefg").await.unwrap();

    let mut buf = vec![0_u8; 32];
    assert_eq!(char_device.read(&mut buf).await.unwrap(), 0);
}
