#![cfg(unix)]

#[cfg(feature = "async-std")]
use char_device::AsyncStdCharDevice;
use char_device::CharDevice;
#[cfg(feature = "tokio")]
use char_device::TokioCharDevice;

#[test]
fn tty() {
    // For now, just ensure that we can open this.
    let _tty = match CharDevice::open("/dev/tty") {
        Ok(tty) => tty,
        Err(e) => match e.raw_os_error() {
            // Headless environments sometimes lack /dev/tty.
            Some(libc::ENXIO) => return,
            _ => Err(e).unwrap(),
        },
    };
}

#[cfg(feature = "async-std")]
#[async_std::test]
async fn async_std_tty() {
    // For now, just ensure that we can open this.
    let _tty = match AsyncStdCharDevice::open("/dev/tty").await {
        Ok(tty) => tty,
        Err(e) => match e.raw_os_error() {
            // Headless environments sometimes lack /dev/tty.
            Some(libc::ENXIO) => return,
            _ => Err(e).unwrap(),
        },
    };
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn tokio_tty() {
    // For now, just ensure that we can open this.
    let _tty = match TokioCharDevice::open("/dev/tty").await {
        Ok(tty) => tty,
        Err(e) => match e.raw_os_error() {
            // Headless environments sometimes lack /dev/tty.
            Some(libc::ENXIO) => return,
            _ => Err(e).unwrap(),
        },
    };
}
