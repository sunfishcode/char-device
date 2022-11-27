use async_std::fs::{File, OpenOptions};
use async_std::io::{self, IoSlice, IoSliceMut, Read, Write};
use async_std::path::Path;
use io_lifetimes::{FromFilelike, IntoFilelike};
use std::pin::Pin;
use std::task::{Context, Poll};
#[cfg(windows)]
use {
    ::async_std::os::windows::io::{AsRawHandle, IntoRawHandle, RawHandle},
    io_extras::os::windows::{
        AsHandleOrSocket, AsRawHandleOrSocket, AsRawReadWriteHandleOrSocket,
        AsReadWriteHandleOrSocket, BorrowedHandleOrSocket,
        IntoRawHandleOrSocket, OwnedHandleOrSocket, RawHandleOrSocket,
    },
    io_lifetimes::{AsFilelike, AsHandle, BorrowedHandle, OwnedHandle},
};
#[cfg(not(windows))]
use {
    io_extras::os::rustix::{AsRawFd, AsRawReadWriteFd, AsReadWriteFd, IntoRawFd, RawFd},
    io_lifetimes::{AsFd, BorrowedFd, OwnedFd},
    rustix::fs::FileTypeExt,
};

/// An unbuffered character device.
///
/// This is a wrapper around [`async_std::fs::File`] which is intended for use
/// with character device "files" such as "/dev/tty".
///
/// TODO: "Unbuffered" here isn't entirely accurate, given how async-std deals
/// with the underlying OS APIs being effectively synchronous. Figure out what
/// to say here.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct AsyncStdCharDevice(async_std::fs::File);

impl AsyncStdCharDevice {
    /// Construct a new `CharDevice`. Fail if the given handle isn't a valid
    /// handle for a character device, or it can't be determined.
    #[inline]
    pub async fn new<Filelike: IntoFilelike + Read + Write>(
        filelike: Filelike,
    ) -> io::Result<Self> {
        Self::_new(File::from_into_filelike(filelike)).await
    }

    async fn _new(file: File) -> io::Result<Self> {
        #[cfg(not(windows))]
        {
            let file_type = file.metadata().await?.file_type();
            if !file_type.is_char_device() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "raw fd is not a char device",
                ));
            }
        }

        #[cfg(windows)]
        {
            let file_type = winapi_util::file::typ(&*file.as_filelike_view::<std::fs::File>())?;
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
    pub async fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::new(OpenOptions::new().read(true).write(true).open(path).await?).await
    }

    /// Construct a new `CharDevice`.
    ///
    /// # Safety
    ///
    /// Doesn't check that the handle is valid or a character device.
    #[inline]
    pub unsafe fn new_unchecked<Filelike: IntoFilelike>(filelike: Filelike) -> Self {
        Self(File::from_into_filelike(filelike))
    }

    /// Construct a new `CharDevice` which discards writes and reads nothing.
    ///
    /// This is "/dev/null" on Posix-ish platforms and "nul" on Windows.
    #[inline]
    pub async fn null() -> io::Result<Self> {
        #[cfg(unix)]
        {
            Self::open("/dev/null").await
        }

        #[cfg(windows)]
        {
            Self::open("nul").await
        }
    }

    /// Return the number of bytes which are ready to be read immediately.
    #[inline]
    pub fn num_ready_bytes(&self) -> io::Result<u64> {
        #[cfg(not(windows))]
        {
            Ok(rustix::io::ioctl_fionread(self)?)
        }

        #[cfg(windows)]
        {
            // Return the conservatively correct result.
            Ok(0)
        }
    }
}

impl Read for AsyncStdCharDevice {
    #[inline]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }

    #[inline]
    fn poll_read_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_read_vectored(cx, bufs)
    }
}

impl Write for AsyncStdCharDevice {
    #[inline]
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write_vectored(cx, bufs)
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    #[inline]
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_close(cx)
    }
}

#[cfg(not(windows))]
impl AsRawFd for AsyncStdCharDevice {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

#[cfg(windows)]
impl AsRawHandle for AsyncStdCharDevice {
    #[inline]
    fn as_raw_handle(&self) -> RawHandle {
        self.0.as_raw_handle()
    }
}

#[cfg(windows)]
impl AsRawHandleOrSocket for AsyncStdCharDevice {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.0.as_raw_handle_or_socket()
    }
}

#[cfg(not(windows))]
impl AsFd for AsyncStdCharDevice {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

#[cfg(windows)]
impl AsHandle for AsyncStdCharDevice {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        self.0.as_handle()
    }
}

#[cfg(windows)]
impl AsHandleOrSocket for AsyncStdCharDevice {
    #[inline]
    fn as_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.0.as_handle_or_socket()
    }
}

#[cfg(not(windows))]
impl IntoRawFd for AsyncStdCharDevice {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

#[cfg(not(windows))]
impl From<AsyncStdCharDevice> for OwnedFd {
    #[inline]
    fn from(device: AsyncStdCharDevice) -> OwnedFd {
        device.0.into()
    }
}

#[cfg(windows)]
impl IntoRawHandle for AsyncStdCharDevice {
    #[inline]
    fn into_raw_handle(self) -> RawHandle {
        self.0.into_raw_handle()
    }
}

#[cfg(windows)]
impl From<AsyncStdCharDevice> for OwnedHandle {
    #[inline]
    fn from(device: AsyncStdCharDevice) -> OwnedHandle {
        device.0.into()
    }
}

#[cfg(windows)]
impl IntoRawHandleOrSocket for AsyncStdCharDevice {
    #[inline]
    fn into_raw_handle_or_socket(self) -> RawHandleOrSocket {
        self.0.into_raw_handle_or_socket()
    }
}

#[cfg(windows)]
impl From<AsyncStdCharDevice> for OwnedHandleOrSocket  {
    #[inline]
    fn from(char_device: AsyncStdCharDevice) -> Self {
        char_device.0.into()
    }
}

#[cfg(not(windows))]
impl AsRawReadWriteFd for AsyncStdCharDevice {
    #[inline]
    fn as_raw_read_fd(&self) -> RawFd {
        self.as_raw_fd()
    }

    #[inline]
    fn as_raw_write_fd(&self) -> RawFd {
        self.as_raw_fd()
    }
}

#[cfg(not(windows))]
impl AsReadWriteFd for AsyncStdCharDevice {
    #[inline]
    fn as_read_fd(&self) -> BorrowedFd<'_> {
        self.as_fd()
    }

    #[inline]
    fn as_write_fd(&self) -> BorrowedFd<'_> {
        self.as_fd()
    }
}

#[cfg(windows)]
impl AsRawReadWriteHandleOrSocket for AsyncStdCharDevice {
    #[inline]
    fn as_raw_read_handle_or_socket(&self) -> RawHandleOrSocket {
        self.as_raw_handle_or_socket()
    }

    #[inline]
    fn as_raw_write_handle_or_socket(&self) -> RawHandleOrSocket {
        self.as_raw_handle_or_socket()
    }
}

#[cfg(windows)]
impl AsReadWriteHandleOrSocket for AsyncStdCharDevice {
    #[inline]
    fn as_read_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.as_handle_or_socket()
    }

    #[inline]
    fn as_write_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.as_handle_or_socket()
    }
}
