use io_lifetimes::{FromFilelike, IntoFilelike};
use std::io::IoSlice;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::fs::{File, OpenOptions};
use tokio::io::{self, AsyncRead, AsyncWrite, ReadBuf};
#[cfg(not(windows))]
use {
    io_extras::os::rustix::{AsRawFd, AsRawReadWriteFd, AsReadWriteFd, RawFd},
    io_lifetimes::{AsFd, BorrowedFd},
    rustix::fs::FileTypeExt,
};
#[cfg(windows)]
use {
    io_extras::os::windows::{
        AsHandleOrSocket, AsRawHandleOrSocket, AsRawReadWriteHandleOrSocket,
        AsReadWriteHandleOrSocket, BorrowedHandleOrSocket, RawHandleOrSocket,
    },
    io_lifetimes::{AsFilelike, AsHandle, BorrowedHandle},
    std::os::windows::io::{AsRawHandle, RawHandle},
};

/// An unbuffered character device.
///
/// This is a wrapper around [`tokio::fs::File`] which is intended for use
/// with character device "files" such as "/dev/tty".
///
/// TODO: "Unbuffered" here isn't entirely accurate, given how tokio deals
/// with the underlying OS APIs being effectively synchronous. Figure out what
/// to say here.
#[derive(Debug)]
#[repr(transparent)]
pub struct TokioCharDevice(tokio::fs::File);

impl TokioCharDevice {
    /// Construct a new `CharDevice`. Fail if the given handle isn't a valid
    /// handle for a character device, or it can't be determined.
    #[inline]
    pub async fn new<Filelike: IntoFilelike + AsyncRead + AsyncWrite>(
        filelike: Filelike,
    ) -> io::Result<Self> {
        Self::_new(File::from_std(filelike)).await
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
            let file_type =
                winx::winapi_util::file::typ(&*file.as_filelike_view::<std::fs::File>())?;
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
        Self::_new(OpenOptions::new().read(true).write(true).open(path).await?).await
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

impl AsyncRead for TokioCharDevice {
    #[inline]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for TokioCharDevice {
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
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

#[cfg(not(windows))]
impl AsRawFd for TokioCharDevice {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

#[cfg(windows)]
impl AsRawHandle for TokioCharDevice {
    #[inline]
    fn as_raw_handle(&self) -> RawHandle {
        self.0.as_raw_handle()
    }
}

#[cfg(windows)]
impl AsRawHandleOrSocket for TokioCharDevice {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.0.as_raw_handle_or_socket()
    }
}

#[cfg(not(windows))]
impl AsFd for TokioCharDevice {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

#[cfg(windows)]
impl AsHandle for TokioCharDevice {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        self.0.as_handle()
    }
}

#[cfg(windows)]
impl AsHandleOrSocket for TokioCharDevice {
    #[inline]
    fn as_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        BorrowedHandleOrSocket::from_handle(self.0.as_handle())
    }
}

#[cfg(not(windows))]
impl AsRawReadWriteFd for TokioCharDevice {
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
impl AsReadWriteFd for TokioCharDevice {
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
impl AsRawReadWriteHandleOrSocket for TokioCharDevice {
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
impl AsReadWriteHandleOrSocket for TokioCharDevice {
    #[inline]
    fn as_read_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.as_handle_or_socket()
    }

    #[inline]
    fn as_write_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.as_handle_or_socket()
    }
}
