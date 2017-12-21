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
    use std::io::{self, Cursor, repeat};
    use test;

    #[test]
    #[cfg_attr(target_os = "emscripten", ignore)]
    fn read_until() {
        let mut buf = Cursor::new(&b"12"[..]);
        let mut v = Vec::new();
        assert_eq!(buf.read_until(b'3', &mut v).unwrap(), 2);
        assert_eq!(v, b"12");

        let mut buf = Cursor::new(&b"1233"[..]);
        let mut v = Vec::new();
        assert_eq!(buf.read_until(b'3', &mut v).unwrap(), 3);
        assert_eq!(v, b"123");
        v.truncate(0);
        assert_eq!(buf.read_until(b'3', &mut v).unwrap(), 1);
        assert_eq!(v, b"3");
        v.truncate(0);
        assert_eq!(buf.read_until(b'3', &mut v).unwrap(), 0);
        assert_eq!(v, []);
    }

    #[test]
    fn split() {
        let buf = Cursor::new(&b"12"[..]);
        let mut s = buf.split(b'3');
        assert_eq!(s.next().unwrap().unwrap(), vec![b'1', b'2']);
        assert!(s.next().is_none());

        let buf = Cursor::new(&b"1233"[..]);
        let mut s = buf.split(b'3');
        assert_eq!(s.next().unwrap().unwrap(), vec![b'1', b'2']);
        assert_eq!(s.next().unwrap().unwrap(), vec![]);
        assert!(s.next().is_none());
    }

    #[test]
    fn read_line() {
        let mut buf = Cursor::new(&b"12"[..]);
        let mut v = String::new();
        assert_eq!(buf.read_line(&mut v).unwrap(), 2);
        assert_eq!(v, "12");

        let mut buf = Cursor::new(&b"12\n\n"[..]);
        let mut v = String::new();
        assert_eq!(buf.read_line(&mut v).unwrap(), 3);
        assert_eq!(v, "12\n");
        v.truncate(0);
        assert_eq!(buf.read_line(&mut v).unwrap(), 1);
        assert_eq!(v, "\n");
        v.truncate(0);
        assert_eq!(buf.read_line(&mut v).unwrap(), 0);
        assert_eq!(v, "");
    }

    #[test]
    fn lines() {
        let buf = Cursor::new(&b"12\r"[..]);
        let mut s = buf.lines();
        assert_eq!(s.next().unwrap().unwrap(), "12\r".to_string());
        assert!(s.next().is_none());

        let buf = Cursor::new(&b"12\r\n\n"[..]);
        let mut s = buf.lines();
        assert_eq!(s.next().unwrap().unwrap(), "12".to_string());
        assert_eq!(s.next().unwrap().unwrap(), "".to_string());
        assert!(s.next().is_none());
    }

    #[test]
    fn read_to_end() {
        let mut c = Cursor::new(&b""[..]);
        let mut v = Vec::new();
        assert_eq!(c.read_to_end(&mut v).unwrap(), 0);
        assert_eq!(v, []);

        let mut c = Cursor::new(&b"1"[..]);
        let mut v = Vec::new();
        assert_eq!(c.read_to_end(&mut v).unwrap(), 1);
        assert_eq!(v, b"1");

        let cap = 1024 * 1024;
        let data = (0..cap).map(|i| (i / 3) as u8).collect::<Vec<_>>();
        let mut v = Vec::new();
        let (a, b) = data.split_at(data.len() / 2);
        assert_eq!(Cursor::new(a).read_to_end(&mut v).unwrap(), a.len());
        assert_eq!(Cursor::new(b).read_to_end(&mut v).unwrap(), b.len());
        assert_eq!(v, data);
    }

    #[test]
    fn read_to_string() {
        let mut c = Cursor::new(&b""[..]);
        let mut v = String::new();
        assert_eq!(c.read_to_string(&mut v).unwrap(), 0);
        assert_eq!(v, "");

        let mut c = Cursor::new(&b"1"[..]);
        let mut v = String::new();
        assert_eq!(c.read_to_string(&mut v).unwrap(), 1);
        assert_eq!(v, "1");

        let mut c = Cursor::new(&b"\xff"[..]);
        let mut v = String::new();
        assert!(c.read_to_string(&mut v).is_err());
    }

    #[test]
    fn read_exact() {
        let mut buf = [0; 4];

        let mut c = Cursor::new(&b""[..]);
        assert_eq!(c.read_exact(&mut buf).unwrap_err().kind(),
                   io::ErrorKind::UnexpectedEof);

        let mut c = Cursor::new(&b"123"[..]).chain(Cursor::new(&b"456789"[..]));
        c.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"1234");
        c.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"5678");
        assert_eq!(c.read_exact(&mut buf).unwrap_err().kind(),
                   io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn read_exact_slice() {
        let mut buf = [0; 4];

        let mut c = &b""[..];
        assert_eq!(c.read_exact(&mut buf).unwrap_err().kind(),
                   io::ErrorKind::UnexpectedEof);

        let mut c = &b"123"[..];
        assert_eq!(c.read_exact(&mut buf).unwrap_err().kind(),
                   io::ErrorKind::UnexpectedEof);
        // make sure the optimized (early returning) method is being used
        assert_eq!(&buf, &[0; 4]);

        let mut c = &b"1234"[..];
        c.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"1234");

        let mut c = &b"56789"[..];
        c.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"5678");
        assert_eq!(c, b"9");
    }

    #[test]
    fn take_eof() {
        struct R;

        impl Read for R {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::Other, ""))
            }
        }
        impl BufRead for R {
            fn fill_buf(&mut self) -> io::Result<&[u8]> {
                Err(io::Error::new(io::ErrorKind::Other, ""))
            }
            fn consume(&mut self, _amt: usize) { }
        }

        let mut buf = [0; 1];
        assert_eq!(0, R.take(0).read(&mut buf).unwrap());
        assert_eq!(b"", R.take(0).fill_buf().unwrap());
    }

    fn cmp_bufread<Br1: BufRead, Br2: BufRead>(mut br1: Br1, mut br2: Br2, exp: &[u8]) {
        let mut cat = Vec::new();
        loop {
            let consume = {
                let buf1 = br1.fill_buf().unwrap();
                let buf2 = br2.fill_buf().unwrap();
                let minlen = if buf1.len() < buf2.len() { buf1.len() } else { buf2.len() };
                assert_eq!(buf1[..minlen], buf2[..minlen]);
                cat.extend_from_slice(&buf1[..minlen]);
                minlen
            };
            if consume == 0 {
                break;
            }
            br1.consume(consume);
            br2.consume(consume);
        }
        assert_eq!(br1.fill_buf().unwrap().len(), 0);
        assert_eq!(br2.fill_buf().unwrap().len(), 0);
        assert_eq!(&cat[..], &exp[..])
    }

    #[test]
    fn chain_bufread() {
        let testdata = b"ABCDEFGHIJKL";
        let chain1 = (&testdata[..3]).chain(&testdata[3..6])
                                     .chain(&testdata[6..9])
                                     .chain(&testdata[9..]);
        let chain2 = (&testdata[..4]).chain(&testdata[4..8])
                                     .chain(&testdata[8..]);
        cmp_bufread(chain1, chain2, &testdata[..]);
    }

    #[test]
    fn chain_zero_length_read_is_not_eof() {
        let a = b"A";
        let b = b"B";
        let mut s = String::new();
        let mut chain = (&a[..]).chain(&b[..]);
        chain.read(&mut []).unwrap();
        chain.read_to_string(&mut s).unwrap();
        assert_eq!("AB", s);
    }

    #[bench]
    #[cfg_attr(target_os = "emscripten", ignore)]
    fn bench_read_to_end(b: &mut test::Bencher) {
        b.iter(|| {
            let mut lr = repeat(1).take(10000000);
            let mut vec = Vec::with_capacity(1024);
            Read::read_to_end(&mut lr, &mut vec)
        });
    }
}
