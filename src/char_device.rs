#[cfg(unix)]
use std::os::unix::{
    fs::FileTypeExt,
    io::{AsRawFd, IntoRawFd, RawFd},
};
#[cfg(target_os = "wasi")]
use std::os::wasi::{
    fs::FileTypeExt,
    io::{AsRawFd, IntoRawFd, RawFd},
};
use std::{
    fmt::Arguments,
    fs::{File, OpenOptions},
    io::{self, IoSlice, IoSliceMut, Read, Write},
    path::Path,
};
use unsafe_io::{FromUnsafeFile, IntoUnsafeFile};
#[cfg(windows)]
use {
    std::os::windows::io::{AsRawHandle, IntoRawHandle, RawHandle},
    unsafe_io::{AsRawHandleOrSocket, IntoRawHandleOrSocket, RawHandleOrSocket},
};

/// An unbuffered character device.
///
/// This is a wrapper around `std::fs::File` which is intended for use with
/// character device "files" such as "/dev/tty".
#[derive(Debug)]
pub struct CharDevice(std::fs::File);

impl CharDevice {
    /// Construct a new `CharDevice`. Fail if the given handle isn't a valid
    /// handle for a character device, or it can't be determined.
    #[inline]
    pub fn new<Filelike: IntoUnsafeFile + Read + Write>(filelike: Filelike) -> io::Result<Self> {
        Self::_new(File::from_filelike(filelike))
    }

    fn _new(file: File) -> io::Result<Self> {
        #[cfg(not(windows))]
        {
            let file_type = file.metadata()?.file_type();
            if !file_type.is_char_device() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "raw fd is not a char device",
                ));
            }
        }

        #[cfg(windows)]
        {
            let file_type = winapi_util::file::typ(&file)?;
            if !file_type.is_char() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "raw handle is not a char device",
                ));
            }
        }

        Ok(Self(file))
    }

    /// Construct a new `CharDevice` from the given filename. Fail if the given
    /// handle isn't a valid handle for a character device, or it can't be
    /// determined.
    #[inline]
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::new(OpenOptions::new().read(true).write(true).open(path)?)
    }

    /// Construct a new `CharDevice`.
    ///
    /// # Safety
    ///
    /// Doesn't check that the handle is valid or a character device.
    #[inline]
    pub unsafe fn new_unchecked<Filelike: IntoUnsafeFile>(filelike: Filelike) -> Self {
        Self(File::from_filelike(filelike))
    }

    /// Construct a new `CharDevice` which discards writes and reads nothing.
    ///
    /// This is "/dev/null" on Posix-ish platforms and "nul" on Windows.
    #[inline]
    pub fn null() -> io::Result<Self> {
        #[cfg(unix)]
        {
            Self::open("/dev/null")
        }

        #[cfg(windows)]
        {
            Self::open("nul")
        }
    }

    /// Creates a new independently owned handle to the underlying device.
    #[inline]
    pub fn try_clone(&self) -> io::Result<Self> {
        self.0.try_clone().map(Self)
    }

    /// Return the number of bytes which are ready to be read immediately.
    #[inline]
    pub fn num_ready_bytes(&self) -> io::Result<u64> {
        #[cfg(not(windows))]
        {
            posish::io::fionread(self)
        }

        #[cfg(windows)]
        {
            // Return the conservatively correct result.
            Ok(0)
        }
    }
}

impl Read for CharDevice {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    #[cfg(can_vector)]
    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.0.read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.0.read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.0.read_exact(buf)
    }
}

impl Write for CharDevice {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    #[cfg(can_vector)]
    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.0.write_all(buf)
    }

    #[cfg(write_all_vectored)]
    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice]) -> io::Result<()> {
        self.0.write_all_vectored(bufs)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: Arguments) -> io::Result<()> {
        self.0.write_fmt(fmt)
    }
}

#[cfg(not(windows))]
impl AsRawFd for CharDevice {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

#[cfg(windows)]
impl AsRawHandleOrSocket for CharDevice {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.0.as_raw_handle_or_socket()
    }
}

#[cfg(windows)]
impl AsRawHandle for CharDevice {
    #[inline]
    fn as_raw_handle(&self) -> RawHandle {
        self.0.as_raw_handle()
    }
}

#[cfg(not(windows))]
impl IntoRawFd for CharDevice {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

#[cfg(windows)]
impl IntoRawHandle for CharDevice {
    #[inline]
    fn into_raw_handle(self) -> RawHandle {
        self.0.into_raw_handle()
    }
}

#[cfg(windows)]
impl IntoRawHandleOrSocket for CharDevice {
    #[inline]
    fn into_raw_handle_or_socket(self) -> RawHandleOrSocket {
        self.0.into_raw_handle_or_socket()
    }
}
