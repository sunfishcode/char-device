#![cfg(unix)]

#[cfg(feature = "async-std")]
use char_device::AsyncCharDevice;
use char_device::CharDevice;

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
async fn async_tty() {
    // For now, just ensure that we can open this.
    let _tty = match AsyncCharDevice::open("/dev/tty").await {
        Ok(tty) => tty,
        Err(e) => match e.raw_os_error() {
            // Headless environments sometimes lack /dev/tty.
            Some(libc::ENXIO) => return,
            _ => Err(e).unwrap(),
        },
    };
}
