use std::{
    io::IoSlice,
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    fs::{File, OpenOptions},
    io::{self, AsyncRead, AsyncWrite, ReadBuf},
};
use unsafe_io::{FromUnsafeFile, IntoUnsafeFile, OwnsRaw};
#[cfg(not(windows))]
use {
    posish::fs::FileTypeExt,
    unsafe_io::os::posish::{AsRawFd, RawFd},
};
#[cfg(windows)]
use {
    std::os::windows::io::{AsRawHandle, RawHandle},
    unsafe_io::{
        os::windows::{AsRawHandleOrSocket, RawHandleOrSocket},
        AsUnsafeFile,
    },
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
    pub async fn new<Filelike: IntoUnsafeFile + AsyncRead + AsyncWrite>(
        filelike: Filelike,
    ) -> io::Result<Self> {
        Self::_new(File::from_filelike(filelike)).await
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
            let file_type = winapi_util::file::typ(&*file.as_file_view())?;
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
    pub unsafe fn new_unchecked<Filelike: IntoUnsafeFile>(filelike: Filelike) -> Self {
        Self(File::from_filelike(filelike))
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
            posish::io::fionread(self)
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

// Safety: `CharDevice` wraps a `File` which owns its handle.
unsafe impl OwnsRaw for TokioCharDevice {}