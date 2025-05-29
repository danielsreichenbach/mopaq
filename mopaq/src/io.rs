//! I/O abstractions for MPQ archives

use crate::Result;
use std::io::{Read, Seek, SeekFrom};

/// Trait for reading from MPQ archives
pub trait MpqRead: Read + Seek {
    /// Read exact number of bytes at the given offset
    fn read_at(&mut self, offset: u64, buf: &mut [u8]) -> Result<()>;
}

/// Buffered reader for MPQ archives
#[derive(Debug)]
pub struct BufferedMpqReader<R> {
    inner: R,
}

impl<R: Read + Seek> BufferedMpqReader<R> {
    /// Create a new buffered reader
    pub fn new(inner: R) -> Self {
        Self { inner }
    }
}

impl<R: Read + Seek> MpqRead for BufferedMpqReader<R> {
    fn read_at(&mut self, offset: u64, buf: &mut [u8]) -> Result<()> {
        self.inner.seek(SeekFrom::Start(offset))?;
        self.inner.read_exact(buf)?;
        Ok(())
    }
}

impl<R: Read> Read for BufferedMpqReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: Seek> Seek for BufferedMpqReader<R> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(pos)
    }
}
