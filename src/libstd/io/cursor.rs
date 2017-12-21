// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use std::io::{Cursor, SeekFrom};

    #[test]
    fn test_vec_writer() {
        let mut writer = Vec::new();
        assert_eq!(writer.write(&[0]).unwrap(), 1);
        assert_eq!(writer.write(&[1, 2, 3]).unwrap(), 3);
        assert_eq!(writer.write(&[4, 5, 6, 7]).unwrap(), 4);
        let b: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7];
        assert_eq!(writer, b);
    }

    #[test]
    fn test_mem_writer() {
        let mut writer = Cursor::new(Vec::new());
        assert_eq!(writer.write(&[0]).unwrap(), 1);
        assert_eq!(writer.write(&[1, 2, 3]).unwrap(), 3);
        assert_eq!(writer.write(&[4, 5, 6, 7]).unwrap(), 4);
        let b: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7];
        assert_eq!(&writer.get_ref()[..], b);
    }

    #[test]
    fn test_box_slice_writer() {
        let mut writer = Cursor::new(vec![0u8; 9].into_boxed_slice());
        assert_eq!(writer.position(), 0);
        assert_eq!(writer.write(&[0]).unwrap(), 1);
        assert_eq!(writer.position(), 1);
        assert_eq!(writer.write(&[1, 2, 3]).unwrap(), 3);
        assert_eq!(writer.write(&[4, 5, 6, 7]).unwrap(), 4);
        assert_eq!(writer.position(), 8);
        assert_eq!(writer.write(&[]).unwrap(), 0);
        assert_eq!(writer.position(), 8);

        assert_eq!(writer.write(&[8, 9]).unwrap(), 1);
        assert_eq!(writer.write(&[10]).unwrap(), 0);
        let b: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(&**writer.get_ref(), b);
    }

    #[test]
    fn test_buf_writer() {
        let mut buf = [0 as u8; 9];
        {
            let mut writer = Cursor::new(&mut buf[..]);
            assert_eq!(writer.position(), 0);
            assert_eq!(writer.write(&[0]).unwrap(), 1);
            assert_eq!(writer.position(), 1);
            assert_eq!(writer.write(&[1, 2, 3]).unwrap(), 3);
            assert_eq!(writer.write(&[4, 5, 6, 7]).unwrap(), 4);
            assert_eq!(writer.position(), 8);
            assert_eq!(writer.write(&[]).unwrap(), 0);
            assert_eq!(writer.position(), 8);

            assert_eq!(writer.write(&[8, 9]).unwrap(), 1);
            assert_eq!(writer.write(&[10]).unwrap(), 0);
        }
        let b: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(buf, b);
    }

    #[test]
    fn test_buf_writer_seek() {
        let mut buf = [0 as u8; 8];
        {
            let mut writer = Cursor::new(&mut buf[..]);
            assert_eq!(writer.position(), 0);
            assert_eq!(writer.write(&[1]).unwrap(), 1);
            assert_eq!(writer.position(), 1);

            assert_eq!(writer.seek(SeekFrom::Start(2)).unwrap(), 2);
            assert_eq!(writer.position(), 2);
            assert_eq!(writer.write(&[2]).unwrap(), 1);
            assert_eq!(writer.position(), 3);

            assert_eq!(writer.seek(SeekFrom::Current(-2)).unwrap(), 1);
            assert_eq!(writer.position(), 1);
            assert_eq!(writer.write(&[3]).unwrap(), 1);
            assert_eq!(writer.position(), 2);

            assert_eq!(writer.seek(SeekFrom::End(-1)).unwrap(), 7);
            assert_eq!(writer.position(), 7);
            assert_eq!(writer.write(&[4]).unwrap(), 1);
            assert_eq!(writer.position(), 8);

        }
        let b: &[_] = &[1, 3, 2, 0, 0, 0, 0, 4];
        assert_eq!(buf, b);
    }

    #[test]
    fn test_buf_writer_error() {
        let mut buf = [0 as u8; 2];
        let mut writer = Cursor::new(&mut buf[..]);
        assert_eq!(writer.write(&[0]).unwrap(), 1);
        assert_eq!(writer.write(&[0, 0]).unwrap(), 1);
        assert_eq!(writer.write(&[0, 0]).unwrap(), 0);
    }

    #[test]
    fn test_mem_reader() {
        let mut reader = Cursor::new(vec![0, 1, 2, 3, 4, 5, 6, 7]);
        let mut buf = [];
        assert_eq!(reader.read(&mut buf).unwrap(), 0);
        assert_eq!(reader.position(), 0);
        let mut buf = [0];
        assert_eq!(reader.read(&mut buf).unwrap(), 1);
        assert_eq!(reader.position(), 1);
        let b: &[_] = &[0];
        assert_eq!(buf, b);
        let mut buf = [0; 4];
        assert_eq!(reader.read(&mut buf).unwrap(), 4);
        assert_eq!(reader.position(), 5);
        let b: &[_] = &[1, 2, 3, 4];
        assert_eq!(buf, b);
        assert_eq!(reader.read(&mut buf).unwrap(), 3);
        let b: &[_] = &[5, 6, 7];
        assert_eq!(&buf[..3], b);
        assert_eq!(reader.read(&mut buf).unwrap(), 0);
    }

    #[test]
    fn test_boxed_slice_reader() {
        let mut reader = Cursor::new(vec![0, 1, 2, 3, 4, 5, 6, 7].into_boxed_slice());
        let mut buf = [];
        assert_eq!(reader.read(&mut buf).unwrap(), 0);
        assert_eq!(reader.position(), 0);
        let mut buf = [0];
        assert_eq!(reader.read(&mut buf).unwrap(), 1);
        assert_eq!(reader.position(), 1);
        let b: &[_] = &[0];
        assert_eq!(buf, b);
        let mut buf = [0; 4];
        assert_eq!(reader.read(&mut buf).unwrap(), 4);
        assert_eq!(reader.position(), 5);
        let b: &[_] = &[1, 2, 3, 4];
        assert_eq!(buf, b);
        assert_eq!(reader.read(&mut buf).unwrap(), 3);
        let b: &[_] = &[5, 6, 7];
        assert_eq!(&buf[..3], b);
        assert_eq!(reader.read(&mut buf).unwrap(), 0);
    }

    #[test]
    fn read_to_end() {
        let mut reader = Cursor::new(vec![0, 1, 2, 3, 4, 5, 6, 7]);
        let mut v = Vec::new();
        reader.read_to_end(&mut v).unwrap();
        assert_eq!(v, [0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_slice_reader() {
        let in_buf = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let reader = &mut &in_buf[..];
        let mut buf = [];
        assert_eq!(reader.read(&mut buf).unwrap(), 0);
        let mut buf = [0];
        assert_eq!(reader.read(&mut buf).unwrap(), 1);
        assert_eq!(reader.len(), 7);
        let b: &[_] = &[0];
        assert_eq!(&buf[..], b);
        let mut buf = [0; 4];
        assert_eq!(reader.read(&mut buf).unwrap(), 4);
        assert_eq!(reader.len(), 3);
        let b: &[_] = &[1, 2, 3, 4];
        assert_eq!(&buf[..], b);
        assert_eq!(reader.read(&mut buf).unwrap(), 3);
        let b: &[_] = &[5, 6, 7];
        assert_eq!(&buf[..3], b);
        assert_eq!(reader.read(&mut buf).unwrap(), 0);
    }

    #[test]
    fn test_read_exact() {
        let in_buf = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let reader = &mut &in_buf[..];
        let mut buf = [];
        assert!(reader.read_exact(&mut buf).is_ok());
        let mut buf = [8];
        assert!(reader.read_exact(&mut buf).is_ok());
        assert_eq!(buf[0], 0);
        assert_eq!(reader.len(), 7);
        let mut buf = [0, 0, 0, 0, 0, 0, 0];
        assert!(reader.read_exact(&mut buf).is_ok());
        assert_eq!(buf, [1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(reader.len(), 0);
        let mut buf = [0];
        assert!(reader.read_exact(&mut buf).is_err());
    }

    #[test]
    fn test_buf_reader() {
        let in_buf = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let mut reader = Cursor::new(&in_buf[..]);
        let mut buf = [];
        assert_eq!(reader.read(&mut buf).unwrap(), 0);
        assert_eq!(reader.position(), 0);
        let mut buf = [0];
        assert_eq!(reader.read(&mut buf).unwrap(), 1);
        assert_eq!(reader.position(), 1);
        let b: &[_] = &[0];
        assert_eq!(buf, b);
        let mut buf = [0; 4];
        assert_eq!(reader.read(&mut buf).unwrap(), 4);
        assert_eq!(reader.position(), 5);
        let b: &[_] = &[1, 2, 3, 4];
        assert_eq!(buf, b);
        assert_eq!(reader.read(&mut buf).unwrap(), 3);
        let b: &[_] = &[5, 6, 7];
        assert_eq!(&buf[..3], b);
        assert_eq!(reader.read(&mut buf).unwrap(), 0);
    }

    #[test]
    fn test_read_char() {
        let b = &b"Vi\xE1\xBB\x87t"[..];
        let mut c = Cursor::new(b).chars();
        assert_eq!(c.next().unwrap().unwrap(), 'V');
        assert_eq!(c.next().unwrap().unwrap(), 'i');
        assert_eq!(c.next().unwrap().unwrap(), 'ệ');
        assert_eq!(c.next().unwrap().unwrap(), 't');
        assert!(c.next().is_none());
    }

    #[test]
    fn test_read_bad_char() {
        let b = &b"\x80"[..];
        let mut c = Cursor::new(b).chars();
        assert!(c.next().unwrap().is_err());
    }

    #[test]
    fn seek_past_end() {
        let buf = [0xff];
        let mut r = Cursor::new(&buf[..]);
        assert_eq!(r.seek(SeekFrom::Start(10)).unwrap(), 10);
        assert_eq!(r.read(&mut [0]).unwrap(), 0);

        let mut r = Cursor::new(vec![10]);
        assert_eq!(r.seek(SeekFrom::Start(10)).unwrap(), 10);
        assert_eq!(r.read(&mut [0]).unwrap(), 0);

        let mut buf = [0];
        let mut r = Cursor::new(&mut buf[..]);
        assert_eq!(r.seek(SeekFrom::Start(10)).unwrap(), 10);
        assert_eq!(r.write(&[3]).unwrap(), 0);

        let mut r = Cursor::new(vec![10].into_boxed_slice());
        assert_eq!(r.seek(SeekFrom::Start(10)).unwrap(), 10);
        assert_eq!(r.write(&[3]).unwrap(), 0);
    }

    #[test]
    fn seek_past_i64() {
        let buf = [0xff];
        let mut r = Cursor::new(&buf[..]);
        assert_eq!(r.seek(SeekFrom::Start(6)).unwrap(), 6);
        assert_eq!(r.seek(SeekFrom::Current(0x7ffffffffffffff0)).unwrap(), 0x7ffffffffffffff6);
        assert_eq!(r.seek(SeekFrom::Current(0x10)).unwrap(), 0x8000000000000006);
        assert_eq!(r.seek(SeekFrom::Current(0)).unwrap(), 0x8000000000000006);
        assert!(r.seek(SeekFrom::Current(0x7ffffffffffffffd)).is_err());
        assert_eq!(r.seek(SeekFrom::Current(-0x8000000000000000)).unwrap(), 6);

        let mut r = Cursor::new(vec![10]);
        assert_eq!(r.seek(SeekFrom::Start(6)).unwrap(), 6);
        assert_eq!(r.seek(SeekFrom::Current(0x7ffffffffffffff0)).unwrap(), 0x7ffffffffffffff6);
        assert_eq!(r.seek(SeekFrom::Current(0x10)).unwrap(), 0x8000000000000006);
        assert_eq!(r.seek(SeekFrom::Current(0)).unwrap(), 0x8000000000000006);
        assert!(r.seek(SeekFrom::Current(0x7ffffffffffffffd)).is_err());
        assert_eq!(r.seek(SeekFrom::Current(-0x8000000000000000)).unwrap(), 6);

        let mut buf = [0];
        let mut r = Cursor::new(&mut buf[..]);
        assert_eq!(r.seek(SeekFrom::Start(6)).unwrap(), 6);
        assert_eq!(r.seek(SeekFrom::Current(0x7ffffffffffffff0)).unwrap(), 0x7ffffffffffffff6);
        assert_eq!(r.seek(SeekFrom::Current(0x10)).unwrap(), 0x8000000000000006);
        assert_eq!(r.seek(SeekFrom::Current(0)).unwrap(), 0x8000000000000006);
        assert!(r.seek(SeekFrom::Current(0x7ffffffffffffffd)).is_err());
        assert_eq!(r.seek(SeekFrom::Current(-0x8000000000000000)).unwrap(), 6);

        let mut r = Cursor::new(vec![10].into_boxed_slice());
        assert_eq!(r.seek(SeekFrom::Start(6)).unwrap(), 6);
        assert_eq!(r.seek(SeekFrom::Current(0x7ffffffffffffff0)).unwrap(), 0x7ffffffffffffff6);
        assert_eq!(r.seek(SeekFrom::Current(0x10)).unwrap(), 0x8000000000000006);
        assert_eq!(r.seek(SeekFrom::Current(0)).unwrap(), 0x8000000000000006);
        assert!(r.seek(SeekFrom::Current(0x7ffffffffffffffd)).is_err());
        assert_eq!(r.seek(SeekFrom::Current(-0x8000000000000000)).unwrap(), 6);
    }

    #[test]
    fn seek_before_0() {
        let buf = [0xff];
        let mut r = Cursor::new(&buf[..]);
        assert!(r.seek(SeekFrom::End(-2)).is_err());

        let mut r = Cursor::new(vec![10]);
        assert!(r.seek(SeekFrom::End(-2)).is_err());

        let mut buf = [0];
        let mut r = Cursor::new(&mut buf[..]);
        assert!(r.seek(SeekFrom::End(-2)).is_err());

        let mut r = Cursor::new(vec![10].into_boxed_slice());
        assert!(r.seek(SeekFrom::End(-2)).is_err());
    }

    #[test]
    fn test_seekable_mem_writer() {
        let mut writer = Cursor::new(Vec::<u8>::new());
        assert_eq!(writer.position(), 0);
        assert_eq!(writer.write(&[0]).unwrap(), 1);
        assert_eq!(writer.position(), 1);
        assert_eq!(writer.write(&[1, 2, 3]).unwrap(), 3);
        assert_eq!(writer.write(&[4, 5, 6, 7]).unwrap(), 4);
        assert_eq!(writer.position(), 8);
        let b: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7];
        assert_eq!(&writer.get_ref()[..], b);

        assert_eq!(writer.seek(SeekFrom::Start(0)).unwrap(), 0);
        assert_eq!(writer.position(), 0);
        assert_eq!(writer.write(&[3, 4]).unwrap(), 2);
        let b: &[_] = &[3, 4, 2, 3, 4, 5, 6, 7];
        assert_eq!(&writer.get_ref()[..], b);

        assert_eq!(writer.seek(SeekFrom::Current(1)).unwrap(), 3);
        assert_eq!(writer.write(&[0, 1]).unwrap(), 2);
        let b: &[_] = &[3, 4, 2, 0, 1, 5, 6, 7];
        assert_eq!(&writer.get_ref()[..], b);

        assert_eq!(writer.seek(SeekFrom::End(-1)).unwrap(), 7);
        assert_eq!(writer.write(&[1, 2]).unwrap(), 2);
        let b: &[_] = &[3, 4, 2, 0, 1, 5, 6, 1, 2];
        assert_eq!(&writer.get_ref()[..], b);

        assert_eq!(writer.seek(SeekFrom::End(1)).unwrap(), 10);
        assert_eq!(writer.write(&[1]).unwrap(), 1);
        let b: &[_] = &[3, 4, 2, 0, 1, 5, 6, 1, 2, 0, 1];
        assert_eq!(&writer.get_ref()[..], b);
    }

    #[test]
    fn vec_seek_past_end() {
        let mut r = Cursor::new(Vec::new());
        assert_eq!(r.seek(SeekFrom::Start(10)).unwrap(), 10);
        assert_eq!(r.write(&[3]).unwrap(), 1);
    }

    #[test]
    fn vec_seek_before_0() {
        let mut r = Cursor::new(Vec::new());
        assert!(r.seek(SeekFrom::End(-2)).is_err());
    }

    #[test]
    #[cfg(target_pointer_width = "32")]
    fn vec_seek_and_write_past_usize_max() {
        let mut c = Cursor::new(Vec::new());
        c.set_position(<usize>::max_value() as u64 + 1);
        assert!(c.write_all(&[1, 2, 3]).is_err());
    }
}
