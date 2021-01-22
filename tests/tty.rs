#![cfg(unix)]

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
