// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Filesystem manipulation operations.
//!
//! This module contains basic methods to manipulate the contents of the local
//! filesystem. All methods in this module represent cross-platform filesystem
//! operations. Extra platform-specific functionality can be found in the
//! extension traits of `std::os::$platform`.

#![stable(feature = "rust1", since = "1.0.0")]

use fmt;
use ffi::OsString;
use io::{self, SeekFrom, Seek, Read, Initializer, Write};
use path::{Path, PathBuf};
use sys::fs as fs_imp;
use sys_common::{AsInnerMut, FromInner, AsInner, IntoInner};
use time::SystemTime;

/// A reference to an open file on the filesystem.
///
/// An instance of a `File` can be read and/or written depending on what options
/// it was opened with. Files also implement [`Seek`] to alter the logical cursor
/// that the file contains internally.
///
/// Files are automatically closed when they go out of scope.
///
/// # Examples
///
/// Create a new file and write bytes to it:
///
/// ```no_run
/// use std::fs::File;
/// use std::io::prelude::*;
///
/// # fn foo() -> std::io::Result<()> {
/// let mut file = File::create("foo.txt")?;
/// file.write_all(b"Hello, world!")?;
/// # Ok(())
/// # }
/// ```
///
/// Read the contents of a file into a [`String`]:
///
/// ```no_run
/// use std::fs::File;
/// use std::io::prelude::*;
///
/// # fn foo() -> std::io::Result<()> {
/// let mut file = File::open("foo.txt")?;
/// let mut contents = String::new();
/// file.read_to_string(&mut contents)?;
/// assert_eq!(contents, "Hello, world!");
/// # Ok(())
/// # }
/// ```
///
/// It can be more efficient to read the contents of a file with a buffered
/// [`Read`]er. This can be accomplished with [`BufReader<R>`]:
///
/// ```no_run
/// use std::fs::File;
/// use std::io::BufReader;
/// use std::io::prelude::*;
///
/// # fn foo() -> std::io::Result<()> {
/// let file = File::open("foo.txt")?;
/// let mut buf_reader = BufReader::new(file);
/// let mut contents = String::new();
/// buf_reader.read_to_string(&mut contents)?;
/// assert_eq!(contents, "Hello, world!");
/// # Ok(())
/// # }
/// ```
///
/// [`Seek`]: ../io/trait.Seek.html
/// [`String`]: ../string/struct.String.html
/// [`Read`]: ../io/trait.Read.html
/// [`BufReader<R>`]: ../io/struct.BufReader.html
#[stable(feature = "rust1", since = "1.0.0")]
pub struct File {
    inner: fs_imp::File,
}

/// Metadata information about a file.
///
/// This structure is returned from the [`metadata`] or
/// [`symlink_metadata`] function or method and represents known
/// metadata about a file such as its permissions, size, modification
/// times, etc.
///
/// [`metadata`]: fn.metadata.html
/// [`symlink_metadata`]: fn.symlink_metadata.html
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Clone)]
pub struct Metadata(fs_imp::FileAttr);

/// Iterator over the entries in a directory.
///
/// This iterator is returned from the [`read_dir`] function of this module and
/// will yield instances of [`io::Result`]`<`[`DirEntry`]`>`. Through a [`DirEntry`]
/// information like the entry's path and possibly other metadata can be
/// learned.
///
/// # Errors
///
/// This [`io::Result`] will be an [`Err`] if there's some sort of intermittent
/// IO error during iteration.
///
/// [`read_dir`]: fn.read_dir.html
/// [`DirEntry`]: struct.DirEntry.html
/// [`io::Result`]: ../io/type.Result.html
/// [`Err`]: ../result/enum.Result.html#variant.Err
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Debug)]
pub struct ReadDir(fs_imp::ReadDir);

/// Entries returned by the [`ReadDir`] iterator.
///
/// [`ReadDir`]: struct.ReadDir.html
///
/// An instance of `DirEntry` represents an entry inside of a directory on the
/// filesystem. Each entry can be inspected via methods to learn about the full
/// path or possibly other metadata through per-platform extension traits.
#[stable(feature = "rust1", since = "1.0.0")]
pub struct DirEntry(fs_imp::DirEntry);

/// Options and flags which can be used to configure how a file is opened.
///
/// This builder exposes the ability to configure how a [`File`] is opened and
/// what operations are permitted on the open file. The [`File::open`] and
/// [`File::create`] methods are aliases for commonly used options using this
/// builder.
///
/// [`File`]: struct.File.html
/// [`File::open`]: struct.File.html#method.open
/// [`File::create`]: struct.File.html#method.create
///
/// Generally speaking, when using `OpenOptions`, you'll first call [`new`],
/// then chain calls to methods to set each option, then call [`open`],
/// passing the path of the file you're trying to open. This will give you a
/// [`io::Result`][result] with a [`File`][file] inside that you can further
/// operate on.
///
/// [`new`]: struct.OpenOptions.html#method.new
/// [`open`]: struct.OpenOptions.html#method.open
/// [result]: ../io/type.Result.html
/// [file]: struct.File.html
///
/// # Examples
///
/// Opening a file to read:
///
/// ```no_run
/// use std::fs::OpenOptions;
///
/// let file = OpenOptions::new().read(true).open("foo.txt");
/// ```
///
/// Opening a file for both reading and writing, as well as creating it if it
/// doesn't exist:
///
/// ```no_run
/// use std::fs::OpenOptions;
///
/// let file = OpenOptions::new()
///             .read(true)
///             .write(true)
///             .create(true)
///             .open("foo.txt");
/// ```
#[derive(Clone, Debug)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct OpenOptions(fs_imp::OpenOptions);

/// Representation of the various permissions on a file.
///
/// This module only currently provides one bit of information, [`readonly`],
/// which is exposed on all currently supported platforms. Unix-specific
/// functionality, such as mode bits, is available through the
/// `os::unix::PermissionsExt` trait.
///
/// [`readonly`]: struct.Permissions.html#method.readonly
#[derive(Clone, PartialEq, Eq, Debug)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Permissions(fs_imp::FilePermissions);

/// A structure representing a type of file with accessors for each file type.
/// It is returned by [`Metadata::file_type`] method.
///
/// [`Metadata::file_type`]: struct.Metadata.html#method.file_type
#[stable(feature = "file_type", since = "1.1.0")]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct FileType(fs_imp::FileType);

/// A builder used to create directories in various manners.
///
/// This builder also supports platform-specific options.
#[stable(feature = "dir_builder", since = "1.6.0")]
#[derive(Debug)]
pub struct DirBuilder {
    inner: fs_imp::DirBuilder,
    recursive: bool,
}

/// Read the entire contents of a file into a bytes vector.
///
/// This is a convenience function for using [`File::open`] and [`read_to_end`]
/// with fewer imports and without an intermediate variable.
///
/// [`File::open`]: struct.File.html#method.open
/// [`read_to_end`]: ../io/trait.Read.html#method.read_to_end
///
/// # Errors
///
/// This function will return an error if `path` does not already exist.
/// Other errors may also be returned according to [`OpenOptions::open`].
///
/// [`OpenOptions::open`]: struct.OpenOptions.html#method.open
///
/// It will also return an error if it encounters while reading an error
/// of a kind other than [`ErrorKind::Interrupted`].
///
/// [`ErrorKind::Interrupted`]: ../../std/io/enum.ErrorKind.html#variant.Interrupted
///
/// # Examples
///
/// ```no_run
/// #![feature(fs_read_write)]
///
/// use std::fs;
/// use std::net::SocketAddr;
///
/// # fn foo() -> Result<(), Box<std::error::Error + 'static>> {
/// let foo: SocketAddr = String::from_utf8_lossy(&fs::read("address.txt")?).parse()?;
/// # Ok(())
/// # }
/// ```
#[unstable(feature = "fs_read_write", issue = "46588")]
pub fn read<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Read the entire contents of a file into a string.
///
/// This is a convenience function for using [`File::open`] and [`read_to_string`]
/// with fewer imports and without an intermediate variable.
///
/// [`File::open`]: struct.File.html#method.open
/// [`read_to_string`]: ../io/trait.Read.html#method.read_to_string
///
/// # Errors
///
/// This function will return an error if `path` does not already exist.
/// Other errors may also be returned according to [`OpenOptions::open`].
///
/// [`OpenOptions::open`]: struct.OpenOptions.html#method.open
///
/// It will also return an error if it encounters while reading an error
/// of a kind other than [`ErrorKind::Interrupted`],
/// or if the contents of the file are not valid UTF-8.
///
/// [`ErrorKind::Interrupted`]: ../../std/io/enum.ErrorKind.html#variant.Interrupted
///
/// # Examples
///
/// ```no_run
/// #![feature(fs_read_write)]
///
/// use std::fs;
/// use std::net::SocketAddr;
///
/// # fn foo() -> Result<(), Box<std::error::Error + 'static>> {
/// let foo: SocketAddr = fs::read_string("address.txt")?.parse()?;
/// # Ok(())
/// # }
/// ```
#[unstable(feature = "fs_read_write", issue = "46588")]
pub fn read_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut string = String::new();
    File::open(path)?.read_to_string(&mut string)?;
    Ok(string)
}

/// Write a slice as the entire contents of a file.
///
/// This function will create a file if it does not exist,
/// and will entirely replace its contents if it does.
///
/// This is a convenience function for using [`File::create`] and [`write_all`]
/// with fewer imports.
///
/// [`File::create`]: struct.File.html#method.create
/// [`write_all`]: ../io/trait.Write.html#method.write_all
///
/// # Examples
///
/// ```no_run
/// #![feature(fs_read_write)]
///
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// fs::write("foo.txt", b"Lorem ipsum")?;
/// # Ok(())
/// # }
/// ```
#[unstable(feature = "fs_read_write", issue = "46588")]
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
    File::create(path)?.write_all(contents.as_ref())
}

impl File {
    /// Attempts to open a file in read-only mode.
    ///
    /// See the [`OpenOptions::open`] method for more details.
    ///
    /// # Errors
    ///
    /// This function will return an error if `path` does not already exist.
    /// Other errors may also be returned according to [`OpenOptions::open`].
    ///
    /// [`OpenOptions::open`]: struct.OpenOptions.html#method.open
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let mut f = File::open("foo.txt")?;
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<File> {
        OpenOptions::new().read(true).open(path.as_ref())
    }

    /// Opens a file in write-only mode.
    ///
    /// This function will create a file if it does not exist,
    /// and will truncate it if it does.
    ///
    /// See the [`OpenOptions::open`] function for more details.
    ///
    /// [`OpenOptions::open`]: struct.OpenOptions.html#method.open
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let mut f = File::create("foo.txt")?;
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<File> {
        OpenOptions::new().write(true).create(true).truncate(true).open(path.as_ref())
    }

    /// Attempts to sync all OS-internal metadata to disk.
    ///
    /// This function will attempt to ensure that all in-core data reaches the
    /// filesystem before returning.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::prelude::*;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let mut f = File::create("foo.txt")?;
    /// f.write_all(b"Hello, world!")?;
    ///
    /// f.sync_all()?;
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn sync_all(&self) -> io::Result<()> {
        self.inner.fsync()
    }

    /// This function is similar to [`sync_all`], except that it may not
    /// synchronize file metadata to the filesystem.
    ///
    /// This is intended for use cases that must synchronize content, but don't
    /// need the metadata on disk. The goal of this method is to reduce disk
    /// operations.
    ///
    /// Note that some platforms may simply implement this in terms of
    /// [`sync_all`].
    ///
    /// [`sync_all`]: struct.File.html#method.sync_all
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::prelude::*;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let mut f = File::create("foo.txt")?;
    /// f.write_all(b"Hello, world!")?;
    ///
    /// f.sync_data()?;
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn sync_data(&self) -> io::Result<()> {
        self.inner.datasync()
    }

    /// Truncates or extends the underlying file, updating the size of
    /// this file to become `size`.
    ///
    /// If the `size` is less than the current file's size, then the file will
    /// be shrunk. If it is greater than the current file's size, then the file
    /// will be extended to `size` and have all of the intermediate data filled
    /// in with 0s.
    ///
    /// # Errors
    ///
    /// This function will return an error if the file is not opened for writing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let mut f = File::create("foo.txt")?;
    /// f.set_len(10)?;
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn set_len(&self, size: u64) -> io::Result<()> {
        self.inner.truncate(size)
    }

    /// Queries metadata about the underlying file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let mut f = File::open("foo.txt")?;
    /// let metadata = f.metadata()?;
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn metadata(&self) -> io::Result<Metadata> {
        self.inner.file_attr().map(Metadata)
    }

    /// Creates a new independently owned handle to the underlying file.
    ///
    /// The returned `File` is a reference to the same state that this object
    /// references. Both handles will read and write with the same cursor
    /// position.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let mut f = File::open("foo.txt")?;
    /// let file_copy = f.try_clone()?;
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "file_try_clone", since = "1.9.0")]
    pub fn try_clone(&self) -> io::Result<File> {
        Ok(File {
            inner: self.inner.duplicate()?
        })
    }

    /// Changes the permissions on the underlying file.
    ///
    /// # Platform-specific behavior
    ///
    /// This function currently corresponds to the `fchmod` function on Unix and
    /// the `SetFileInformationByHandle` function on Windows. Note that, this
    /// [may change in the future][changes].
    ///
    /// [changes]: ../io/index.html#platform-specific-behavior
    ///
    /// # Errors
    ///
    /// This function will return an error if the user lacks permission change
    /// attributes on the underlying file. It may also return an error in other
    /// os-specific unspecified cases.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs::File;
    ///
    /// let file = File::open("foo.txt")?;
    /// let mut perms = file.metadata()?.permissions();
    /// perms.set_readonly(true);
    /// file.set_permissions(perms)?;
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "set_permissions_atomic", since = "1.16.0")]
    pub fn set_permissions(&self, perm: Permissions) -> io::Result<()> {
        self.inner.set_permissions(perm.0)
    }
}

impl AsInner<fs_imp::File> for File {
    fn as_inner(&self) -> &fs_imp::File { &self.inner }
}
impl FromInner<fs_imp::File> for File {
    fn from_inner(f: fs_imp::File) -> File {
        File { inner: f }
    }
}
impl IntoInner<fs_imp::File> for File {
    fn into_inner(self) -> fs_imp::File {
        self.inner
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    #[inline]
    unsafe fn initializer(&self) -> Initializer {
        Initializer::nop()
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> { self.inner.flush() }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Read for &'a File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    #[inline]
    unsafe fn initializer(&self) -> Initializer {
        Initializer::nop()
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Write for &'a File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> { self.inner.flush() }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Seek for &'a File {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl OpenOptions {
    /// Creates a blank new set of options ready for configuration.
    ///
    /// All options are initially set to `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::OpenOptions;
    ///
    /// let mut options = OpenOptions::new();
    /// let file = options.read(true).open("foo.txt");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn new() -> OpenOptions {
        OpenOptions(fs_imp::OpenOptions::new())
    }

    /// Sets the option for read access.
    ///
    /// This option, when true, will indicate that the file should be
    /// `read`-able if opened.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().read(true).open("foo.txt");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn read(&mut self, read: bool) -> &mut OpenOptions {
        self.0.read(read); self
    }

    /// Sets the option for write access.
    ///
    /// This option, when true, will indicate that the file should be
    /// `write`-able if opened.
    ///
    /// If the file already exists, any write calls on it will overwrite its
    /// contents, without truncating it.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().write(true).open("foo.txt");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn write(&mut self, write: bool) -> &mut OpenOptions {
        self.0.write(write); self
    }

    /// Sets the option for the append mode.
    ///
    /// This option, when true, means that writes will append to a file instead
    /// of overwriting previous contents.
    /// Note that setting `.write(true).append(true)` has the same effect as
    /// setting only `.append(true)`.
    ///
    /// For most filesystems, the operating system guarantees that all writes are
    /// atomic: no writes get mangled because another process writes at the same
    /// time.
    ///
    /// One maybe obvious note when using append-mode: make sure that all data
    /// that belongs together is written to the file in one operation. This
    /// can be done by concatenating strings before passing them to [`write()`],
    /// or using a buffered writer (with a buffer of adequate size),
    /// and calling [`flush()`] when the message is complete.
    ///
    /// If a file is opened with both read and append access, beware that after
    /// opening, and after every write, the position for reading may be set at the
    /// end of the file. So, before writing, save the current position (using
    /// [`seek`]`(`[`SeekFrom`]`::`[`Current`]`(0))`, and restore it before the next read.
    ///
    /// ## Note
    ///
    /// This function doesn't create the file if it doesn't exist. Use the [`create`]
    /// method to do so.
    ///
    /// [`write()`]: ../../std/fs/struct.File.html#method.write
    /// [`flush()`]: ../../std/fs/struct.File.html#method.flush
    /// [`seek`]: ../../std/fs/struct.File.html#method.seek
    /// [`SeekFrom`]: ../../std/io/enum.SeekFrom.html
    /// [`Current`]: ../../std/io/enum.SeekFrom.html#variant.Current
    /// [`create`]: #method.create
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().append(true).open("foo.txt");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn append(&mut self, append: bool) -> &mut OpenOptions {
        self.0.append(append); self
    }

    /// Sets the option for truncating a previous file.
    ///
    /// If a file is successfully opened with this option set it will truncate
    /// the file to 0 length if it already exists.
    ///
    /// The file must be opened with write access for truncate to work.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().write(true).truncate(true).open("foo.txt");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn truncate(&mut self, truncate: bool) -> &mut OpenOptions {
        self.0.truncate(truncate); self
    }

    /// Sets the option for creating a new file.
    ///
    /// This option indicates whether a new file will be created if the file
    /// does not yet already exist.
    ///
    /// In order for the file to be created, [`write`] or [`append`] access must
    /// be used.
    ///
    /// [`write`]: #method.write
    /// [`append`]: #method.append
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().write(true).create(true).open("foo.txt");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn create(&mut self, create: bool) -> &mut OpenOptions {
        self.0.create(create); self
    }

    /// Sets the option to always create a new file.
    ///
    /// This option indicates whether a new file will be created.
    /// No file is allowed to exist at the target location, also no (dangling)
    /// symlink.
    ///
    /// This option is useful because it is atomic. Otherwise between checking
    /// whether a file exists and creating a new one, the file may have been
    /// created by another process (a TOCTOU race condition / attack).
    ///
    /// If `.create_new(true)` is set, [`.create()`] and [`.truncate()`] are
    /// ignored.
    ///
    /// The file must be opened with write or append access in order to create
    /// a new file.
    ///
    /// [`.create()`]: #method.create
    /// [`.truncate()`]: #method.truncate
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().write(true)
    ///                              .create_new(true)
    ///                              .open("foo.txt");
    /// ```
    #[stable(feature = "expand_open_options2", since = "1.9.0")]
    pub fn create_new(&mut self, create_new: bool) -> &mut OpenOptions {
        self.0.create_new(create_new); self
    }

    /// Opens a file at `path` with the options specified by `self`.
    ///
    /// # Errors
    ///
    /// This function will return an error under a number of different
    /// circumstances. Some of these error conditions are listed here, together
    /// with their [`ErrorKind`]. The mapping to [`ErrorKind`]s is not part of
    /// the compatibility contract of the function, especially the `Other` kind
    /// might change to more specific kinds in the future.
    ///
    /// * [`NotFound`]: The specified file does not exist and neither `create`
    ///   or `create_new` is set.
    /// * [`NotFound`]: One of the directory components of the file path does
    ///   not exist.
    /// * [`PermissionDenied`]: The user lacks permission to get the specified
    ///   access rights for the file.
    /// * [`PermissionDenied`]: The user lacks permission to open one of the
    ///   directory components of the specified path.
    /// * [`AlreadyExists`]: `create_new` was specified and the file already
    ///   exists.
    /// * [`InvalidInput`]: Invalid combinations of open options (truncate
    ///   without write access, no access mode set, etc.).
    /// * [`Other`]: One of the directory components of the specified file path
    ///   was not, in fact, a directory.
    /// * [`Other`]: Filesystem-level errors: full disk, write permission
    ///   requested on a read-only file system, exceeded disk quota, too many
    ///   open files, too long filename, too many symbolic links in the
    ///   specified path (Unix-like systems only), etc.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().open("foo.txt");
    /// ```
    ///
    /// [`ErrorKind`]: ../io/enum.ErrorKind.html
    /// [`AlreadyExists`]: ../io/enum.ErrorKind.html#variant.AlreadyExists
    /// [`InvalidInput`]: ../io/enum.ErrorKind.html#variant.InvalidInput
    /// [`NotFound`]: ../io/enum.ErrorKind.html#variant.NotFound
    /// [`Other`]: ../io/enum.ErrorKind.html#variant.Other
    /// [`PermissionDenied`]: ../io/enum.ErrorKind.html#variant.PermissionDenied
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
        self._open(path.as_ref())
    }

    fn _open(&self, path: &Path) -> io::Result<File> {
        let inner = fs_imp::File::open(path, &self.0)?;
        Ok(File { inner: inner })
    }
}

impl AsInnerMut<fs_imp::OpenOptions> for OpenOptions {
    fn as_inner_mut(&mut self) -> &mut fs_imp::OpenOptions { &mut self.0 }
}

impl Metadata {
    /// Returns the file type for this metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = fs::metadata("foo.txt")?;
    ///
    /// println!("{:?}", metadata.file_type());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "file_type", since = "1.1.0")]
    pub fn file_type(&self) -> FileType {
        FileType(self.0.file_type())
    }

    /// Returns whether this metadata is for a directory.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = fs::metadata("foo.txt")?;
    ///
    /// assert!(!metadata.is_dir());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn is_dir(&self) -> bool { self.file_type().is_dir() }

    /// Returns whether this metadata is for a regular file.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = fs::metadata("foo.txt")?;
    ///
    /// assert!(metadata.is_file());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn is_file(&self) -> bool { self.file_type().is_file() }

    /// Returns the size of the file, in bytes, this metadata is for.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = fs::metadata("foo.txt")?;
    ///
    /// assert_eq!(0, metadata.len());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn len(&self) -> u64 { self.0.size() }

    /// Returns the permissions of the file this metadata is for.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = fs::metadata("foo.txt")?;
    ///
    /// assert!(!metadata.permissions().readonly());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn permissions(&self) -> Permissions {
        Permissions(self.0.perm())
    }

    /// Returns the last modification time listed in this metadata.
    ///
    /// The returned value corresponds to the `mtime` field of `stat` on Unix
    /// platforms and the `ftLastWriteTime` field on Windows platforms.
    ///
    /// # Errors
    ///
    /// This field may not be available on all platforms, and will return an
    /// `Err` on platforms where it is not available.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = fs::metadata("foo.txt")?;
    ///
    /// if let Ok(time) = metadata.modified() {
    ///     println!("{:?}", time);
    /// } else {
    ///     println!("Not supported on this platform");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "fs_time", since = "1.10.0")]
    pub fn modified(&self) -> io::Result<SystemTime> {
        self.0.modified().map(FromInner::from_inner)
    }

    /// Returns the last access time of this metadata.
    ///
    /// The returned value corresponds to the `atime` field of `stat` on Unix
    /// platforms and the `ftLastAccessTime` field on Windows platforms.
    ///
    /// Note that not all platforms will keep this field update in a file's
    /// metadata, for example Windows has an option to disable updating this
    /// time when files are accessed and Linux similarly has `noatime`.
    ///
    /// # Errors
    ///
    /// This field may not be available on all platforms, and will return an
    /// `Err` on platforms where it is not available.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = fs::metadata("foo.txt")?;
    ///
    /// if let Ok(time) = metadata.accessed() {
    ///     println!("{:?}", time);
    /// } else {
    ///     println!("Not supported on this platform");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "fs_time", since = "1.10.0")]
    pub fn accessed(&self) -> io::Result<SystemTime> {
        self.0.accessed().map(FromInner::from_inner)
    }

    /// Returns the creation time listed in the this metadata.
    ///
    /// The returned value corresponds to the `birthtime` field of `stat` on
    /// Unix platforms and the `ftCreationTime` field on Windows platforms.
    ///
    /// # Errors
    ///
    /// This field may not be available on all platforms, and will return an
    /// `Err` on platforms where it is not available.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = fs::metadata("foo.txt")?;
    ///
    /// if let Ok(time) = metadata.created() {
    ///     println!("{:?}", time);
    /// } else {
    ///     println!("Not supported on this platform");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "fs_time", since = "1.10.0")]
    pub fn created(&self) -> io::Result<SystemTime> {
        self.0.created().map(FromInner::from_inner)
    }
}

#[stable(feature = "std_debug", since = "1.16.0")]
impl fmt::Debug for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Metadata")
            .field("file_type", &self.file_type())
            .field("is_dir", &self.is_dir())
            .field("is_file", &self.is_file())
            .field("permissions", &self.permissions())
            .field("modified", &self.modified())
            .field("accessed", &self.accessed())
            .field("created", &self.created())
            .finish()
    }
}

impl AsInner<fs_imp::FileAttr> for Metadata {
    fn as_inner(&self) -> &fs_imp::FileAttr { &self.0 }
}

impl Permissions {
    /// Returns whether these permissions describe a readonly (unwritable) file.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let mut f = File::create("foo.txt")?;
    /// let metadata = f.metadata()?;
    ///
    /// assert_eq!(false, metadata.permissions().readonly());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn readonly(&self) -> bool { self.0.readonly() }

    /// Modifies the readonly flag for this set of permissions. If the
    /// `readonly` argument is `true`, using the resulting `Permission` will
    /// update file permissions to forbid writing. Conversely, if it's `false`,
    /// using the resulting `Permission` will update file permissions to allow
    /// writing.
    ///
    /// This operation does **not** modify the filesystem. To modify the
    /// filesystem use the `fs::set_permissions` function.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    ///
    /// # fn foo() -> std::io::Result<()> {
    /// let f = File::create("foo.txt")?;
    /// let metadata = f.metadata()?;
    /// let mut permissions = metadata.permissions();
    ///
    /// permissions.set_readonly(true);
    ///
    /// // filesystem doesn't change
    /// assert_eq!(false, metadata.permissions().readonly());
    ///
    /// // just this particular `permissions`.
    /// assert_eq!(true, permissions.readonly());
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn set_readonly(&mut self, readonly: bool) {
        self.0.set_readonly(readonly)
    }
}

impl FileType {
    /// Test whether this file type represents a directory.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = fs::metadata("foo.txt")?;
    /// let file_type = metadata.file_type();
    ///
    /// assert_eq!(file_type.is_dir(), false);
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "file_type", since = "1.1.0")]
    pub fn is_dir(&self) -> bool { self.0.is_dir() }

    /// Test whether this file type represents a regular file.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = fs::metadata("foo.txt")?;
    /// let file_type = metadata.file_type();
    ///
    /// assert_eq!(file_type.is_file(), true);
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "file_type", since = "1.1.0")]
    pub fn is_file(&self) -> bool { self.0.is_file() }

    /// Test whether this file type represents a symbolic link.
    ///
    /// The underlying [`Metadata`] struct needs to be retrieved
    /// with the [`fs::symlink_metadata`] function and not the
    /// [`fs::metadata`] function. The [`fs::metadata`] function
    /// follows symbolic links, so [`is_symlink`] would always
    /// return false for the target file.
    ///
    /// [`Metadata`]: struct.Metadata.html
    /// [`fs::metadata`]: fn.metadata.html
    /// [`fs::symlink_metadata`]: fn.symlink_metadata.html
    /// [`is_symlink`]: struct.FileType.html#method.is_symlink
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> std::io::Result<()> {
    /// use std::fs;
    ///
    /// let metadata = fs::symlink_metadata("foo.txt")?;
    /// let file_type = metadata.file_type();
    ///
    /// assert_eq!(file_type.is_symlink(), false);
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "file_type", since = "1.1.0")]
    pub fn is_symlink(&self) -> bool { self.0.is_symlink() }
}

impl AsInner<fs_imp::FileType> for FileType {
    fn as_inner(&self) -> &fs_imp::FileType { &self.0 }
}

impl FromInner<fs_imp::FilePermissions> for Permissions {
    fn from_inner(f: fs_imp::FilePermissions) -> Permissions {
        Permissions(f)
    }
}

impl AsInner<fs_imp::FilePermissions> for Permissions {
    fn as_inner(&self) -> &fs_imp::FilePermissions { &self.0 }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        self.0.next().map(|entry| entry.map(DirEntry))
    }
}

impl DirEntry {
    /// Returns the full path to the file that this entry represents.
    ///
    /// The full path is created by joining the original path to `read_dir`
    /// with the filename of this entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    /// # fn foo() -> std::io::Result<()> {
    /// for entry in fs::read_dir(".")? {
    ///     let dir = entry?;
    ///     println!("{:?}", dir.path());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// This prints output like:
    ///
    /// ```text
    /// "./whatever.txt"
    /// "./foo.html"
    /// "./hello_world.rs"
    /// ```
    ///
    /// The exact text, of course, depends on what files you have in `.`.
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn path(&self) -> PathBuf { self.0.path() }

    /// Return the metadata for the file that this entry points at.
    ///
    /// This function will not traverse symlinks if this entry points at a
    /// symlink.
    ///
    /// # Platform-specific behavior
    ///
    /// On Windows this function is cheap to call (no extra system calls
    /// needed), but on Unix platforms this function is the equivalent of
    /// calling `symlink_metadata` on the path.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    ///
    /// if let Ok(entries) = fs::read_dir(".") {
    ///     for entry in entries {
    ///         if let Ok(entry) = entry {
    ///             // Here, `entry` is a `DirEntry`.
    ///             if let Ok(metadata) = entry.metadata() {
    ///                 // Now let's show our entry's permissions!
    ///                 println!("{:?}: {:?}", entry.path(), metadata.permissions());
    ///             } else {
    ///                 println!("Couldn't get metadata for {:?}", entry.path());
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[stable(feature = "dir_entry_ext", since = "1.1.0")]
    pub fn metadata(&self) -> io::Result<Metadata> {
        self.0.metadata().map(Metadata)
    }

    /// Return the file type for the file that this entry points at.
    ///
    /// This function will not traverse symlinks if this entry points at a
    /// symlink.
    ///
    /// # Platform-specific behavior
    ///
    /// On Windows and most Unix platforms this function is free (no extra
    /// system calls needed), but some Unix platforms may require the equivalent
    /// call to `symlink_metadata` to learn about the target file type.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    ///
    /// if let Ok(entries) = fs::read_dir(".") {
    ///     for entry in entries {
    ///         if let Ok(entry) = entry {
    ///             // Here, `entry` is a `DirEntry`.
    ///             if let Ok(file_type) = entry.file_type() {
    ///                 // Now let's show our entry's file type!
    ///                 println!("{:?}: {:?}", entry.path(), file_type);
    ///             } else {
    ///                 println!("Couldn't get file type for {:?}", entry.path());
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[stable(feature = "dir_entry_ext", since = "1.1.0")]
    pub fn file_type(&self) -> io::Result<FileType> {
        self.0.file_type().map(FileType)
    }

    /// Returns the bare file name of this directory entry without any other
    /// leading path component.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    ///
    /// if let Ok(entries) = fs::read_dir(".") {
    ///     for entry in entries {
    ///         if let Ok(entry) = entry {
    ///             // Here, `entry` is a `DirEntry`.
    ///             println!("{:?}", entry.file_name());
    ///         }
    ///     }
    /// }
    /// ```
    #[stable(feature = "dir_entry_ext", since = "1.1.0")]
    pub fn file_name(&self) -> OsString {
        self.0.file_name()
    }
}

#[stable(feature = "dir_entry_debug", since = "1.13.0")]
impl fmt::Debug for DirEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("DirEntry")
            .field(&self.path())
            .finish()
    }
}

impl AsInner<fs_imp::DirEntry> for DirEntry {
    fn as_inner(&self) -> &fs_imp::DirEntry { &self.0 }
}

/// Removes a file from the filesystem.
///
/// Note that there is no
/// guarantee that the file is immediately deleted (e.g. depending on
/// platform, other open file descriptors may prevent immediate removal).
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `unlink` function on Unix
/// and the `DeleteFile` function on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `path` points to a directory.
/// * The user lacks permissions to remove the file.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// fs::remove_file("a.txt")?;
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn remove_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
    fs_imp::unlink(path.as_ref())
}

/// Given a path, query the file system to get information about a file,
/// directory, etc.
///
/// This function will traverse symbolic links to query information about the
/// destination file.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `stat` function on Unix
/// and the `GetFileAttributesEx` function on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The user lacks permissions to perform `metadata` call on `path`.
/// * `path` does not exist.
///
/// # Examples
///
/// ```rust
/// # fn foo() -> std::io::Result<()> {
/// use std::fs;
///
/// let attr = fs::metadata("/some/file/path.txt")?;
/// // inspect attr ...
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn metadata<P: AsRef<Path>>(path: P) -> io::Result<Metadata> {
    fs_imp::stat(path.as_ref()).map(Metadata)
}

/// Query the metadata about a file without following symlinks.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `lstat` function on Unix
/// and the `GetFileAttributesEx` function on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The user lacks permissions to perform `metadata` call on `path`.
/// * `path` does not exist.
///
/// # Examples
///
/// ```rust
/// # fn foo() -> std::io::Result<()> {
/// use std::fs;
///
/// let attr = fs::symlink_metadata("/some/file/path.txt")?;
/// // inspect attr ...
/// # Ok(())
/// # }
/// ```
#[stable(feature = "symlink_metadata", since = "1.1.0")]
pub fn symlink_metadata<P: AsRef<Path>>(path: P) -> io::Result<Metadata> {
    fs_imp::lstat(path.as_ref()).map(Metadata)
}

/// Rename a file or directory to a new name, replacing the original file if
/// `to` already exists.
///
/// This will not work if the new name is on a different mount point.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `rename` function on Unix
/// and the `MoveFileEx` function with the `MOVEFILE_REPLACE_EXISTING` flag on Windows.
///
/// Because of this, the behavior when both `from` and `to` exist differs. On
/// Unix, if `from` is a directory, `to` must also be an (empty) directory. If
/// `from` is not a directory, `to` must also be not a directory. In contrast,
/// on Windows, `from` can be anything, but `to` must *not* be a directory.
///
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `from` does not exist.
/// * The user lacks permissions to view contents.
/// * `from` and `to` are on separate filesystems.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// fs::rename("a.txt", "b.txt")?; // Rename a.txt to b.txt
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<()> {
    fs_imp::rename(from.as_ref(), to.as_ref())
}

/// Copies the contents of one file to another. This function will also
/// copy the permission bits of the original file to the destination file.
///
/// This function will **overwrite** the contents of `to`.
///
/// Note that if `from` and `to` both point to the same file, then the file
/// will likely get truncated by this operation.
///
/// On success, the total number of bytes copied is returned and it is equal to
/// the length of the `to` file as reported by `metadata`.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `open` function in Unix
/// with `O_RDONLY` for `from` and `O_WRONLY`, `O_CREAT`, and `O_TRUNC` for `to`.
/// `O_CLOEXEC` is set for returned file descriptors.
/// On Windows, this function currently corresponds to `CopyFileEx`. Alternate
/// NTFS streams are copied but only the size of the main stream is returned by
/// this function.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The `from` path is not a file.
/// * The `from` file does not exist.
/// * The current process does not have the permission rights to access
///   `from` or write `to`.
///
/// # Examples
///
/// ```no_run
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// fs::copy("foo.txt", "bar.txt")?;  // Copy foo.txt to bar.txt
/// # Ok(()) }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
    fs_imp::copy(from.as_ref(), to.as_ref())
}

/// Creates a new hard link on the filesystem.
///
/// The `dst` path will be a link pointing to the `src` path. Note that systems
/// often require these two paths to both be located on the same filesystem.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `link` function on Unix
/// and the `CreateHardLink` function on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The `src` path is not a file or doesn't exist.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// fs::hard_link("a.txt", "b.txt")?; // Hard link a.txt to b.txt
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    fs_imp::link(src.as_ref(), dst.as_ref())
}

/// Creates a new symbolic link on the filesystem.
///
/// The `dst` path will be a symbolic link pointing to the `src` path.
/// On Windows, this will be a file symlink, not a directory symlink;
/// for this reason, the platform-specific `std::os::unix::fs::symlink`
/// and `std::os::windows::fs::{symlink_file, symlink_dir}` should be
/// used instead to make the intent explicit.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// fs::soft_link("a.txt", "b.txt")?;
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_deprecated(since = "1.1.0",
             reason = "replaced with std::os::unix::fs::symlink and \
                       std::os::windows::fs::{symlink_file, symlink_dir}")]
pub fn soft_link<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    fs_imp::symlink(src.as_ref(), dst.as_ref())
}

/// Reads a symbolic link, returning the file that the link points to.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `readlink` function on Unix
/// and the `CreateFile` function with `FILE_FLAG_OPEN_REPARSE_POINT` and
/// `FILE_FLAG_BACKUP_SEMANTICS` flags on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `path` is not a symbolic link.
/// * `path` does not exist.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// let path = fs::read_link("a.txt")?;
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn read_link<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    fs_imp::readlink(path.as_ref())
}

/// Returns the canonical form of a path with all intermediate components
/// normalized and symbolic links resolved.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `realpath` function on Unix
/// and the `CreateFile` and `GetFinalPathNameByHandle` functions on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `path` does not exist.
/// * A component in path is not a directory.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// let path = fs::canonicalize("../a/../foo.txt")?;
/// # Ok(())
/// # }
/// ```
#[stable(feature = "fs_canonicalize", since = "1.5.0")]
pub fn canonicalize<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    fs_imp::canonicalize(path.as_ref())
}

/// Creates a new, empty directory at the provided path
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `mkdir` function on Unix
/// and the `CreateDirectory` function on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * User lacks permissions to create directory at `path`.
/// * `path` already exists.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// fs::create_dir("/some/dir")?;
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn create_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    DirBuilder::new().create(path.as_ref())
}

/// Recursively create a directory and all of its parent components if they
/// are missing.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `mkdir` function on Unix
/// and the `CreateDirectory` function on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * If any directory in the path specified by `path`
/// does not already exist and it could not be created otherwise. The specific
/// error conditions for when a directory is being created (after it is
/// determined to not exist) are outlined by `fs::create_dir`.
///
/// Notable exception is made for situations where any of the directories
/// specified in the `path` could not be created as it was being created concurrently.
/// Such cases are considered to be successful. That is, calling `create_dir_all`
/// concurrently from multiple threads or processes is guaranteed not to fail
/// due to a race condition with itself.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// fs::create_dir_all("/some/dir")?;
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    DirBuilder::new().recursive(true).create(path.as_ref())
}

/// Removes an existing, empty directory.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `rmdir` function on Unix
/// and the `RemoveDirectory` function on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The user lacks permissions to remove the directory at the provided `path`.
/// * The directory isn't empty.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// fs::remove_dir("/some/dir")?;
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn remove_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    fs_imp::rmdir(path.as_ref())
}

/// Removes a directory at this path, after removing all its contents. Use
/// carefully!
///
/// This function does **not** follow symbolic links and it will simply remove the
/// symbolic link itself.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to `opendir`, `lstat`, `rm` and `rmdir` functions on Unix
/// and the `FindFirstFile`, `GetFileAttributesEx`, `DeleteFile`, and `RemoveDirectory` functions
/// on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// See `file::remove_file` and `fs::remove_dir`.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// # fn foo() -> std::io::Result<()> {
/// fs::remove_dir_all("/some/dir")?;
/// # Ok(())
/// # }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    fs_imp::remove_dir_all(path.as_ref())
}

/// Returns an iterator over the entries within a directory.
///
/// The iterator will yield instances of [`io::Result`]`<`[`DirEntry`]`>`.
/// New errors may be encountered after an iterator is initially constructed.
///
/// [`io::Result`]: ../io/type.Result.html
/// [`DirEntry`]: struct.DirEntry.html
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `opendir` function on Unix
/// and the `FindFirstFile` function on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The provided `path` doesn't exist.
/// * The process lacks permissions to view the contents.
/// * The `path` points at a non-directory file.
///
/// # Examples
///
/// ```
/// use std::io;
/// use std::fs::{self, DirEntry};
/// use std::path::Path;
///
/// // one possible implementation of walking a directory only visiting files
/// fn visit_dirs(dir: &Path, cb: &Fn(&DirEntry)) -> io::Result<()> {
///     if dir.is_dir() {
///         for entry in fs::read_dir(dir)? {
///             let entry = entry?;
///             let path = entry.path();
///             if path.is_dir() {
///                 visit_dirs(&path, cb)?;
///             } else {
///                 cb(&entry);
///             }
///         }
///     }
///     Ok(())
/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn read_dir<P: AsRef<Path>>(path: P) -> io::Result<ReadDir> {
    fs_imp::readdir(path.as_ref()).map(ReadDir)
}

/// Changes the permissions found on a file or a directory.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `chmod` function on Unix
/// and the `SetFileAttributes` function on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `path` does not exist.
/// * The user lacks the permission to change attributes of the file.
///
/// # Examples
///
/// ```
/// # fn foo() -> std::io::Result<()> {
/// use std::fs;
///
/// let mut perms = fs::metadata("foo.txt")?.permissions();
/// perms.set_readonly(true);
/// fs::set_permissions("foo.txt", perms)?;
/// # Ok(())
/// # }
/// ```
#[stable(feature = "set_permissions", since = "1.1.0")]
pub fn set_permissions<P: AsRef<Path>>(path: P, perm: Permissions)
                                       -> io::Result<()> {
    fs_imp::set_perm(path.as_ref(), perm.0)
}

impl DirBuilder {
    /// Creates a new set of options with default mode/security settings for all
    /// platforms and also non-recursive.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::DirBuilder;
    ///
    /// let builder = DirBuilder::new();
    /// ```
    #[stable(feature = "dir_builder", since = "1.6.0")]
    pub fn new() -> DirBuilder {
        DirBuilder {
            inner: fs_imp::DirBuilder::new(),
            recursive: false,
        }
    }

    /// Indicates that directories should be created recursively, creating all
    /// parent directories. Parents that do not exist are created with the same
    /// security and permissions settings.
    ///
    /// This option defaults to `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::DirBuilder;
    ///
    /// let mut builder = DirBuilder::new();
    /// builder.recursive(true);
    /// ```
    #[stable(feature = "dir_builder", since = "1.6.0")]
    pub fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.recursive = recursive;
        self
    }

    /// Create the specified directory with the options configured in this
    /// builder.
    ///
    /// It is considered an error if the directory already exists unless
    /// recursive mode is enabled.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::{self, DirBuilder};
    ///
    /// let path = "/tmp/foo/bar/baz";
    /// DirBuilder::new()
    ///     .recursive(true)
    ///     .create(path).unwrap();
    ///
    /// assert!(fs::metadata(path).unwrap().is_dir());
    /// ```
    #[stable(feature = "dir_builder", since = "1.6.0")]
    pub fn create<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self._create(path.as_ref())
    }

    fn _create(&self, path: &Path) -> io::Result<()> {
        if self.recursive {
            self.create_dir_all(path)
        } else {
            self.inner.mkdir(path)
        }
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        if path == Path::new("") {
            return Ok(())
        }

        match self.inner.mkdir(path) {
            Ok(()) => return Ok(()),
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(_) if path.is_dir() => return Ok(()),
            Err(e) => return Err(e),
        }
        match path.parent() {
            Some(p) => try!(self.create_dir_all(p)),
            None => return Err(io::Error::new(io::ErrorKind::Other, "failed to create whole tree")),
        }
        match self.inner.mkdir(path) {
            Ok(()) => Ok(()),
            Err(_) if path.is_dir() => Ok(()),
            Err(e) => Err(e),
        }
    }
}

impl AsInnerMut<fs_imp::DirBuilder> for DirBuilder {
    fn as_inner_mut(&mut self) -> &mut fs_imp::DirBuilder {
        &mut self.inner
    }
}

#[cfg(all(test, not(target_os = "emscripten")))]
mod tests {
    use io::prelude::*;

    use fs::{self, File, OpenOptions};
    use io::{ErrorKind, SeekFrom};
    use path::Path;
    use rand::{StdRng, Rng};
    use str;
    use sys_common::io::test::{TempDir, tmpdir};
    use thread;

    #[cfg(windows)]
    use os::windows::fs::{symlink_dir, symlink_file};
    #[cfg(windows)]
    use sys::fs::symlink_junction;
    #[cfg(unix)]
    use os::unix::fs::symlink as symlink_dir;
    #[cfg(unix)]
    use os::unix::fs::symlink as symlink_file;
    #[cfg(unix)]
    use os::unix::fs::symlink as symlink_junction;

    macro_rules! check { ($e:expr) => (
        match $e {
            Ok(t) => t,
            Err(e) => panic!("{} failed with: {}", stringify!($e), e),
        }
    ) }

    #[test]
    fn recursive_rmdir() {
        let tmpdir = tmpdir();
        let d1 = tmpdir.join("d1");
        let dt = d1.join("t");
        let dtt = dt.join("t");
        let d2 = tmpdir.join("d2");
        let canary = d2.join("do_not_delete");
        check!(fs::create_dir_all(&dtt));
        check!(fs::create_dir_all(&d2));
        check!(check!(File::create(&canary)).write(b"foo"));
        check!(symlink_junction(&d2, &dt.join("d2")));
        let _ = symlink_file(&canary, &d1.join("canary"));
        check!(fs::remove_dir_all(&d1));

        assert!(!d1.is_dir());
        assert!(canary.exists());
    }

    #[test]
    fn recursive_rmdir_of_symlink() {
        // test we do not recursively delete a symlink but only dirs.
        let tmpdir = tmpdir();
        let link = tmpdir.join("d1");
        let dir = tmpdir.join("d2");
        let canary = dir.join("do_not_delete");
        check!(fs::create_dir_all(&dir));
        check!(check!(File::create(&canary)).write(b"foo"));
        check!(symlink_junction(&dir, &link));
        check!(fs::remove_dir_all(&link));

        assert!(!link.is_dir());
        assert!(canary.exists());
    }

    #[test]
    fn dir_entry_debug() {
        let tmpdir = tmpdir();
        File::create(&tmpdir.join("b")).unwrap();
        let mut read_dir = tmpdir.path().read_dir().unwrap();
        let dir_entry = read_dir.next().unwrap().unwrap();
        let actual = format!("{:?}", dir_entry);
        let expected = format!("DirEntry({:?})", dir_entry.0.path());
        assert_eq!(actual, expected);
    }

    #[test]
    fn create_dir_all_with_junctions() {
        let tmpdir = tmpdir();
        let target = tmpdir.join("target");

        let junction = tmpdir.join("junction");
        let b = junction.join("a/b");

        let link = tmpdir.join("link");
        let d = link.join("c/d");

        fs::create_dir(&target).unwrap();

        check!(symlink_junction(&target, &junction));
        check!(fs::create_dir_all(&b));
        // the junction itself is not a directory, but `is_dir()` on a Path
        // follows links
        assert!(junction.is_dir());
        assert!(b.exists());

        if !got_symlink_permission(&tmpdir) { return };
        check!(symlink_dir(&target, &link));
        check!(fs::create_dir_all(&d));
        assert!(link.is_dir());
        assert!(d.exists());
    }
}
