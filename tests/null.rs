/*
#[cfg(feature = "async-std")]
use char_device::AsyncStdCharDevice;
*/
use char_device::CharDevice;
/*
#[cfg(feature = "tokio")]
use char_device::TokioCharDevice;
*/

#[test]
fn null() {
    use std::io::{Read, Write};

    let mut char_device = CharDevice::null().unwrap();
    char_device.write_all(b"abcdefg").unwrap();

    let mut buf = vec![0_u8; 32];
    assert_eq!(char_device.read(&mut buf).unwrap(), 0);
}

/*
#[cfg(feature = "async-std")]
#[async_std::test]
async fn async_std_null() {
    use async_std::io::prelude::{ReadExt, WriteExt};

    let mut char_device = AsyncStdCharDevice::null().await.unwrap();
    char_device.write_all(b"abcdefg").await.unwrap();

    let mut buf = vec![0_u8; 32];
    assert_eq!(char_device.read(&mut buf).await.unwrap(), 0);
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn tokio_null() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut char_device = TokioCharDevice::null().await.unwrap();
    char_device.write_all(b"abcdefg").await.unwrap();

    let mut buf = vec![0_u8; 32];
    assert_eq!(char_device.read(&mut buf).await.unwrap(), 0);
}
*/
