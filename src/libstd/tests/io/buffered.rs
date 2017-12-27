// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter, LineWriter, SeekFrom};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use test;

/// A dummy reader intended at testing short-reads propagation.
pub struct ShortReader {
    lengths: Vec<usize>,
}

impl Read for ShortReader {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        if self.lengths.is_empty() {
            Ok(0)
        } else {
            Ok(self.lengths.remove(0))
        }
    }
}

#[test]
fn test_buffered_reader() {
    let inner: &[u8] = &[5, 6, 7, 0, 1, 2, 3, 4];
    let mut reader = BufReader::with_capacity(2, inner);

    let mut buf = [0, 0, 0];
    let nread = reader.read(&mut buf);
    assert_eq!(nread.unwrap(), 3);
    let b: &[_] = &[5, 6, 7];
    assert_eq!(buf, b);

    let mut buf = [0, 0];
    let nread = reader.read(&mut buf);
    assert_eq!(nread.unwrap(), 2);
    let b: &[_] = &[0, 1];
    assert_eq!(buf, b);

    let mut buf = [0];
    let nread = reader.read(&mut buf);
    assert_eq!(nread.unwrap(), 1);
    let b: &[_] = &[2];
    assert_eq!(buf, b);

    let mut buf = [0, 0, 0];
    let nread = reader.read(&mut buf);
    assert_eq!(nread.unwrap(), 1);
    let b: &[_] = &[3, 0, 0];
    assert_eq!(buf, b);

    let nread = reader.read(&mut buf);
    assert_eq!(nread.unwrap(), 1);
    let b: &[_] = &[4, 0, 0];
    assert_eq!(buf, b);

    assert_eq!(reader.read(&mut buf).unwrap(), 0);
}

#[test]
fn test_buffered_reader_seek() {
    let inner: &[u8] = &[5, 6, 7, 0, 1, 2, 3, 4];
    let mut reader = BufReader::with_capacity(2, io::Cursor::new(inner));

    assert_eq!(reader.seek(SeekFrom::Start(3)).ok(), Some(3));
    assert_eq!(reader.fill_buf().ok(), Some(&[0, 1][..]));
    assert_eq!(reader.seek(SeekFrom::Current(0)).ok(), Some(3));
    assert_eq!(reader.fill_buf().ok(), Some(&[0, 1][..]));
    assert_eq!(reader.seek(SeekFrom::Current(1)).ok(), Some(4));
    assert_eq!(reader.fill_buf().ok(), Some(&[1, 2][..]));
    reader.consume(1);
    assert_eq!(reader.seek(SeekFrom::Current(-2)).ok(), Some(3));
}

#[test]
fn test_buffered_reader_seek_underflow() {
    // gimmick reader that yields its position modulo 256 for each byte
    struct PositionReader {
        pos: u64
    }
    impl Read for PositionReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let len = buf.len();
            for x in buf {
                *x = self.pos as u8;
                self.pos = self.pos.wrapping_add(1);
            }
            Ok(len)
        }
    }
    impl Seek for PositionReader {
        fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
            match pos {
                SeekFrom::Start(n) => {
                    self.pos = n;
                }
                SeekFrom::Current(n) => {
                    self.pos = self.pos.wrapping_add(n as u64);
                }
                SeekFrom::End(n) => {
                    self.pos = u64::max_value().wrapping_add(n as u64);
                }
            }
            Ok(self.pos)
        }
    }

    let mut reader = BufReader::with_capacity(5, PositionReader { pos: 0 });
    assert_eq!(reader.fill_buf().ok(), Some(&[0, 1, 2, 3, 4][..]));
    assert_eq!(reader.seek(SeekFrom::End(-5)).ok(), Some(u64::max_value()-5));
    assert_eq!(reader.fill_buf().ok().map(|s| s.len()), Some(5));
    // the following seek will require two underlying seeks
    let expected = 9223372036854775802;
    assert_eq!(reader.seek(SeekFrom::Current(i64::min_value())).ok(), Some(expected));
    assert_eq!(reader.fill_buf().ok().map(|s| s.len()), Some(5));
    // seeking to 0 should empty the buffer.
    assert_eq!(reader.seek(SeekFrom::Current(0)).ok(), Some(expected));
    assert_eq!(reader.get_ref().pos, expected);
}

#[test]
fn test_buffered_writer() {
    let inner = Vec::new();
    let mut writer = BufWriter::with_capacity(2, inner);

    writer.write(&[0, 1]).unwrap();
    assert_eq!(*writer.get_ref(), [0, 1]);

    writer.write(&[2]).unwrap();
    assert_eq!(*writer.get_ref(), [0, 1]);

    writer.write(&[3]).unwrap();
    assert_eq!(*writer.get_ref(), [0, 1]);

    writer.flush().unwrap();
    assert_eq!(*writer.get_ref(), [0, 1, 2, 3]);

    writer.write(&[4]).unwrap();
    writer.write(&[5]).unwrap();
    assert_eq!(*writer.get_ref(), [0, 1, 2, 3]);

    writer.write(&[6]).unwrap();
    assert_eq!(*writer.get_ref(), [0, 1, 2, 3, 4, 5]);

    writer.write(&[7, 8]).unwrap();
    assert_eq!(*writer.get_ref(), [0, 1, 2, 3, 4, 5, 6, 7, 8]);

    writer.write(&[9, 10, 11]).unwrap();
    assert_eq!(*writer.get_ref(), [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

    writer.flush().unwrap();
    assert_eq!(*writer.get_ref(), [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
}

#[test]
fn test_buffered_writer_inner_flushes() {
    let mut w = BufWriter::with_capacity(3, Vec::new());
    w.write(&[0, 1]).unwrap();
    assert_eq!(*w.get_ref(), []);
    let w = w.into_inner().unwrap();
    assert_eq!(w, [0, 1]);
}

#[test]
fn test_buffered_writer_seek() {
    let mut w = BufWriter::with_capacity(3, io::Cursor::new(Vec::new()));
    w.write_all(&[0, 1, 2, 3, 4, 5]).unwrap();
    w.write_all(&[6, 7]).unwrap();
    assert_eq!(w.seek(SeekFrom::Current(0)).ok(), Some(8));
    assert_eq!(&w.get_ref().get_ref()[..], &[0, 1, 2, 3, 4, 5, 6, 7][..]);
    assert_eq!(w.seek(SeekFrom::Start(2)).ok(), Some(2));
    w.write_all(&[8, 9]).unwrap();
    assert_eq!(&w.into_inner().unwrap().into_inner()[..], &[0, 1, 8, 9, 4, 5, 6, 7]);
}

#[test]
fn test_read_until() {
    let inner: &[u8] = &[0, 1, 2, 1, 0];
    let mut reader = BufReader::with_capacity(2, inner);
    let mut v = Vec::new();
    reader.read_until(0, &mut v).unwrap();
    assert_eq!(v, [0]);
    v.truncate(0);
    reader.read_until(2, &mut v).unwrap();
    assert_eq!(v, [1, 2]);
    v.truncate(0);
    reader.read_until(1, &mut v).unwrap();
    assert_eq!(v, [1]);
    v.truncate(0);
    reader.read_until(8, &mut v).unwrap();
    assert_eq!(v, [0]);
    v.truncate(0);
    reader.read_until(9, &mut v).unwrap();
    assert_eq!(v, []);
}

#[test]
fn test_line_buffer_fail_flush() {
    // Issue #32085
    struct FailFlushWriter<'a>(&'a mut Vec<u8>);

    impl<'a> Write for FailFlushWriter<'a> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::Other, "flush failed"))
        }
    }

    let mut buf = Vec::new();
    {
        let mut writer = LineWriter::new(FailFlushWriter(&mut buf));
        let to_write = b"abc\ndef";
        if let Ok(written) = writer.write(to_write) {
            assert!(written < to_write.len(), "didn't flush on new line");
            // PASS
            return;
        }
    }
    assert!(buf.is_empty(), "write returned an error but wrote data");
}

#[test]
fn test_line_buffer() {
    let mut writer = LineWriter::new(Vec::new());
    writer.write(&[0]).unwrap();
    assert_eq!(*writer.get_ref(), []);
    writer.write(&[1]).unwrap();
    assert_eq!(*writer.get_ref(), []);
    writer.flush().unwrap();
    assert_eq!(*writer.get_ref(), [0, 1]);
    writer.write(&[0, b'\n', 1, b'\n', 2]).unwrap();
    assert_eq!(*writer.get_ref(), [0, 1, 0, b'\n', 1, b'\n']);
    writer.flush().unwrap();
    assert_eq!(*writer.get_ref(), [0, 1, 0, b'\n', 1, b'\n', 2]);
    writer.write(&[3, b'\n']).unwrap();
    assert_eq!(*writer.get_ref(), [0, 1, 0, b'\n', 1, b'\n', 2, 3, b'\n']);
}

#[test]
fn test_read_line() {
    let in_buf: &[u8] = b"a\nb\nc";
    let mut reader = BufReader::with_capacity(2, in_buf);
    let mut s = String::new();
    reader.read_line(&mut s).unwrap();
    assert_eq!(s, "a\n");
    s.truncate(0);
    reader.read_line(&mut s).unwrap();
    assert_eq!(s, "b\n");
    s.truncate(0);
    reader.read_line(&mut s).unwrap();
    assert_eq!(s, "c");
    s.truncate(0);
    reader.read_line(&mut s).unwrap();
    assert_eq!(s, "");
}

#[test]
fn test_lines() {
    let in_buf: &[u8] = b"a\nb\nc";
    let reader = BufReader::with_capacity(2, in_buf);
    let mut it = reader.lines();
    assert_eq!(it.next().unwrap().unwrap(), "a".to_string());
    assert_eq!(it.next().unwrap().unwrap(), "b".to_string());
    assert_eq!(it.next().unwrap().unwrap(), "c".to_string());
    assert!(it.next().is_none());
}

#[test]
fn test_short_reads() {
    let inner = ShortReader{lengths: vec![0, 1, 2, 0, 1, 0]};
    let mut reader = BufReader::new(inner);
    let mut buf = [0, 0];
    assert_eq!(reader.read(&mut buf).unwrap(), 0);
    assert_eq!(reader.read(&mut buf).unwrap(), 1);
    assert_eq!(reader.read(&mut buf).unwrap(), 2);
    assert_eq!(reader.read(&mut buf).unwrap(), 0);
    assert_eq!(reader.read(&mut buf).unwrap(), 1);
    assert_eq!(reader.read(&mut buf).unwrap(), 0);
    assert_eq!(reader.read(&mut buf).unwrap(), 0);
}

#[test]
fn read_char_buffered() {
    let buf = [195, 159];
    let reader = BufReader::with_capacity(1, &buf[..]);
    assert_eq!(reader.chars().next().unwrap().unwrap(), 'ß');
}

#[test]
fn test_chars() {
    let buf = [195, 159, b'a'];
    let reader = BufReader::with_capacity(1, &buf[..]);
    let mut it = reader.chars();
    assert_eq!(it.next().unwrap().unwrap(), 'ß');
    assert_eq!(it.next().unwrap().unwrap(), 'a');
    assert!(it.next().is_none());
}

#[test]
#[should_panic]
fn dont_panic_in_drop_on_panicked_flush() {
    struct FailFlushWriter;

    impl Write for FailFlushWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> { Ok(buf.len()) }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::last_os_error())
        }
    }

    let writer = FailFlushWriter;
    let _writer = BufWriter::new(writer);

    // If writer panics *again* due to the flush error then the process will
    // abort.
    panic!();
}

#[test]
#[cfg_attr(target_os = "emscripten", ignore)]
fn panic_in_write_doesnt_flush_in_drop() {
    static WRITES: AtomicUsize = AtomicUsize::new(0);

    struct PanicWriter;

    impl Write for PanicWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            WRITES.fetch_add(1, Ordering::SeqCst);
            panic!();
        }
        fn flush(&mut self) -> io::Result<()> { Ok(()) }
    }

    thread::spawn(|| {
        let mut writer = BufWriter::new(PanicWriter);
        let _ = writer.write(b"hello world");
        let _ = writer.flush();
    }).join().unwrap_err();

    assert_eq!(WRITES.load(Ordering::SeqCst), 1);
}

#[bench]
fn bench_buffered_reader(b: &mut test::Bencher) {
    b.iter(|| {
        BufReader::new(io::empty())
    });
}

#[bench]
fn bench_buffered_writer(b: &mut test::Bencher) {
    b.iter(|| {
        BufWriter::new(io::sink())
    });
}

struct AcceptOneThenFail {
    written: bool,
    flushed: bool,
}

impl Write for AcceptOneThenFail {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if !self.written {
            assert_eq!(data, b"a\nb\n");
            self.written = true;
            Ok(data.len())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "test"))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        assert!(self.written);
        assert!(!self.flushed);
        self.flushed = true;
        Err(io::Error::new(io::ErrorKind::Other, "test"))
    }
}

#[test]
fn erroneous_flush_retried() {
    let a = AcceptOneThenFail {
        written: false,
        flushed: false,
    };

    let mut l = LineWriter::new(a);
    assert_eq!(l.write(b"a\nb\na").unwrap(), 4);
    assert!(l.get_ref().written);
    assert!(l.get_ref().flushed);
    l.get_mut().flushed = false;

    assert_eq!(l.write(b"a").unwrap_err().kind(), io::ErrorKind::Other)
}
