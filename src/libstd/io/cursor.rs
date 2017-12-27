// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use io::prelude::*;

use core::convert::TryInto;
use cmp;
use io::{self, Initializer, SeekFrom, Error, ErrorKind};

/// A `Cursor` wraps another type and provides it with a
/// [`Seek`] implementation.
///
/// `Cursor`s are typically used with in-memory buffers to allow them to
/// implement [`Read`] and/or [`Write`], allowing these buffers to be used
/// anywhere you might use a reader or writer that does actual I/O.
///
/// The standard library implements some I/O traits on various types which
/// are commonly used as a buffer, like `Cursor<`[`Vec`]`<u8>>` and
/// `Cursor<`[`&[u8]`][bytes]`>`.
///
/// # Examples
///
/// We may want to write bytes to a [`File`] in our production
/// code, but use an in-memory buffer in our tests. We can do this with
/// `Cursor`:
///
/// [`Seek`]: trait.Seek.html
/// [`Read`]: ../../std/io/trait.Read.html
/// [`Write`]: ../../std/io/trait.Write.html
/// [`Vec`]: ../../std/vec/struct.Vec.html
/// [bytes]: ../../std/primitive.slice.html
/// [`File`]: ../fs/struct.File.html
///
/// ```no_run
/// use std::io::prelude::*;
/// use std::io::{self, SeekFrom};
/// use std::fs::File;
///
/// // a library function we've written
/// fn write_ten_bytes_at_end<W: Write + Seek>(writer: &mut W) -> io::Result<()> {
///     writer.seek(SeekFrom::End(-10))?;
///
///     for i in 0..10 {
///         writer.write(&[i])?;
///     }
///
///     // all went well
///     Ok(())
/// }
///
/// # fn foo() -> io::Result<()> {
/// // Here's some code that uses this library function.
/// //
/// // We might want to use a BufReader here for efficiency, but let's
/// // keep this example focused.
/// let mut file = File::create("foo.txt")?;
///
/// write_ten_bytes_at_end(&mut file)?;
/// # Ok(())
/// # }
///
/// // now let's write a test
/// #[test]
/// fn test_writes_bytes() {
///     // setting up a real File is much slower than an in-memory buffer,
///     // let's use a cursor instead
///     use std::io::Cursor;
///     let mut buff = Cursor::new(vec![0; 15]);
///
///     write_ten_bytes_at_end(&mut buff).unwrap();
///
///     assert_eq!(&buff.get_ref()[5..15], &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Clone, Debug)]
pub struct Cursor<T> {
    inner: T,
    pos: u64,
}

impl<T> Cursor<T> {
    /// Creates a new cursor wrapping the provided underlying I/O object.
    ///
    /// Cursor initial position is `0` even if underlying object (e.
    /// g. `Vec`) is not empty. So writing to cursor starts with
    /// overwriting `Vec` content, not with appending to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn new(inner: T) -> Cursor<T> {
        Cursor { pos: 0, inner: inner }
    }

    /// Consumes this cursor, returning the underlying value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    ///
    /// let vec = buff.into_inner();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn into_inner(self) -> T { self.inner }

    /// Gets a reference to the underlying value in this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    ///
    /// let reference = buff.get_ref();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn get_ref(&self) -> &T { &self.inner }

    /// Gets a mutable reference to the underlying value in this cursor.
    ///
    /// Care should be taken to avoid modifying the internal I/O state of the
    /// underlying value as it may corrupt this cursor's position.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let mut buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    ///
    /// let reference = buff.get_mut();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn get_mut(&mut self) -> &mut T { &mut self.inner }

    /// Returns the current position of this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use std::io::prelude::*;
    /// use std::io::SeekFrom;
    ///
    /// let mut buff = Cursor::new(vec![1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(buff.position(), 0);
    ///
    /// buff.seek(SeekFrom::Current(2)).unwrap();
    /// assert_eq!(buff.position(), 2);
    ///
    /// buff.seek(SeekFrom::Current(-1)).unwrap();
    /// assert_eq!(buff.position(), 1);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn position(&self) -> u64 { self.pos }

    /// Sets the position of this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let mut buff = Cursor::new(vec![1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(buff.position(), 0);
    ///
    /// buff.set_position(2);
    /// assert_eq!(buff.position(), 2);
    ///
    /// buff.set_position(4);
    /// assert_eq!(buff.position(), 4);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn set_position(&mut self, pos: u64) { self.pos = pos; }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> io::Seek for Cursor<T> where T: AsRef<[u8]> {
    fn seek(&mut self, style: SeekFrom) -> io::Result<u64> {
        let (base_pos, offset) = match style {
            SeekFrom::Start(n) => { self.pos = n; return Ok(n); }
            SeekFrom::End(n) => (self.inner.as_ref().len() as u64, n),
            SeekFrom::Current(n) => (self.pos, n),
        };
        let new_pos = if offset >= 0 {
            base_pos.checked_add(offset as u64)
        } else {
            base_pos.checked_sub((offset.wrapping_neg()) as u64)
        };
        match new_pos {
            Some(n) => {self.pos = n; Ok(self.pos)}
            None => Err(Error::new(ErrorKind::InvalidInput,
                           "invalid seek to a negative or overflowing position"))
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> Read for Cursor<T> where T: AsRef<[u8]> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = Read::read(&mut self.fill_buf()?, buf)?;
        self.pos += n as u64;
        Ok(n)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let n = buf.len();
        Read::read_exact(&mut self.fill_buf()?, buf)?;
        self.pos += n as u64;
        Ok(())
    }

    #[inline]
    unsafe fn initializer(&self) -> Initializer {
        Initializer::nop()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> BufRead for Cursor<T> where T: AsRef<[u8]> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let amt = cmp::min(self.pos, self.inner.as_ref().len() as u64);
        Ok(&self.inner.as_ref()[(amt as usize)..])
    }
    fn consume(&mut self, amt: usize) { self.pos += amt as u64; }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Write for Cursor<&'a mut [u8]> {
    #[inline]
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let pos = cmp::min(self.pos, self.inner.len() as u64);
        let amt = (&mut self.inner[(pos as usize)..]).write(data)?;
        self.pos += amt as u64;
        Ok(amt)
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Write for Cursor<Vec<u8>> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let pos: usize = self.position().try_into().map_err(|_| {
            Error::new(ErrorKind::InvalidInput,
                       "cursor position exceeds maximum possible vector length")
        })?;
        // Make sure the internal buffer is as least as big as where we
        // currently are
        let len = self.inner.len();
        if len < pos {
            // use `resize` so that the zero filling is as efficient as possible
            self.inner.resize(pos, 0);
        }
        // Figure out what bytes will be used to overwrite what's currently
        // there (left), and what will be appended on the end (right)
        {
            let space = self.inner.len() - pos;
            let (left, right) = buf.split_at(cmp::min(space, buf.len()));
            self.inner[pos..pos + left.len()].copy_from_slice(left);
            self.inner.extend_from_slice(right);
        }

        // Bump us forward
        self.set_position((pos + buf.len()) as u64);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

#[stable(feature = "cursor_box_slice", since = "1.5.0")]
impl Write for Cursor<Box<[u8]>> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let pos = cmp::min(self.pos, self.inner.len() as u64);
        let amt = (&mut self.inner[(pos as usize)..]).write(buf)?;
        self.pos += amt as u64;
        Ok(amt)
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}
