// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
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
    use std::ffi::*;
    use std::os::raw::c_char;
    use std::borrow::Cow::{Borrowed, Owned};
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    use std::rc::Rc;
    use std::sync::Arc;

    #[test]
    fn c_to_rust() {
        let data = b"123\0";
        let ptr = data.as_ptr() as *const c_char;
        unsafe {
            assert_eq!(CStr::from_ptr(ptr).to_bytes(), b"123");
            assert_eq!(CStr::from_ptr(ptr).to_bytes_with_nul(), b"123\0");
        }
    }

    #[test]
    fn simple() {
        let s = CString::new("1234").unwrap();
        assert_eq!(s.as_bytes(), b"1234");
        assert_eq!(s.as_bytes_with_nul(), b"1234\0");
    }

    #[test]
    fn build_with_zero1() {
        assert!(CString::new(&b"\0"[..]).is_err());
    }
    #[test]
    fn build_with_zero2() {
        assert!(CString::new(vec![0]).is_err());
    }

    #[test]
    fn build_with_zero3() {
        unsafe {
            let s = CString::from_vec_unchecked(vec![0]);
            assert_eq!(s.as_bytes(), b"\0");
        }
    }

    #[test]
    fn formatted() {
        let s = CString::new(&b"abc\x01\x02\n\xE2\x80\xA6\xFF"[..]).unwrap();
        assert_eq!(format!("{:?}", s), r#""abc\x01\x02\n\xe2\x80\xa6\xff""#);
    }

    #[test]
    fn borrowed() {
        unsafe {
            let s = CStr::from_ptr(b"12\0".as_ptr() as *const _);
            assert_eq!(s.to_bytes(), b"12");
            assert_eq!(s.to_bytes_with_nul(), b"12\0");
        }
    }

    #[test]
    fn to_str() {
        let data = b"123\xE2\x80\xA6\0";
        let ptr = data.as_ptr() as *const c_char;
        unsafe {
            assert_eq!(CStr::from_ptr(ptr).to_str(), Ok("123…"));
            assert_eq!(CStr::from_ptr(ptr).to_string_lossy(), Borrowed("123…"));
        }
        let data = b"123\xE2\0";
        let ptr = data.as_ptr() as *const c_char;
        unsafe {
            assert!(CStr::from_ptr(ptr).to_str().is_err());
            assert_eq!(CStr::from_ptr(ptr).to_string_lossy(), Owned::<str>(format!("123\u{FFFD}")));
        }
    }

    #[test]
    fn to_owned() {
        let data = b"123\0";
        let ptr = data.as_ptr() as *const c_char;

        let owned = unsafe { CStr::from_ptr(ptr).to_owned() };
        assert_eq!(owned.as_bytes_with_nul(), data);
    }

    #[test]
    fn equal_hash() {
        let data = b"123\xE2\xFA\xA6\0";
        let ptr = data.as_ptr() as *const c_char;
        let cstr: &'static CStr = unsafe { CStr::from_ptr(ptr) };

        let mut s = DefaultHasher::new();
        cstr.hash(&mut s);
        let cstr_hash = s.finish();
        let mut s = DefaultHasher::new();
        CString::new(&data[..data.len() - 1]).unwrap().hash(&mut s);
        let cstring_hash = s.finish();

        assert_eq!(cstr_hash, cstring_hash);
    }

    #[test]
    fn from_bytes_with_nul() {
        let data = b"123\0";
        let cstr = CStr::from_bytes_with_nul(data);
        assert_eq!(cstr.map(CStr::to_bytes), Ok(&b"123"[..]));
        let cstr = CStr::from_bytes_with_nul(data);
        assert_eq!(cstr.map(CStr::to_bytes_with_nul), Ok(&b"123\0"[..]));

        unsafe {
            let cstr = CStr::from_bytes_with_nul(data);
            let cstr_unchecked = CStr::from_bytes_with_nul_unchecked(data);
            assert_eq!(cstr, Ok(cstr_unchecked));
        }
    }

    #[test]
    fn from_bytes_with_nul_unterminated() {
        let data = b"123";
        let cstr = CStr::from_bytes_with_nul(data);
        assert!(cstr.is_err());
    }

    #[test]
    fn from_bytes_with_nul_interior() {
        let data = b"1\023\0";
        let cstr = CStr::from_bytes_with_nul(data);
        assert!(cstr.is_err());
    }

    #[test]
    fn into_boxed() {
        let orig: &[u8] = b"Hello, world!\0";
        let cstr = CStr::from_bytes_with_nul(orig).unwrap();
        let boxed: Box<CStr> = Box::from(cstr);
        let cstring = cstr.to_owned().into_boxed_c_str().into_c_string();
        assert_eq!(cstr, &*boxed);
        assert_eq!(&*boxed, &*cstring);
        assert_eq!(&*cstring, cstr);
    }

    #[test]
    fn boxed_default() {
        let boxed = <Box<CStr>>::default();
        assert_eq!(boxed.to_bytes_with_nul(), &[0]);
    }

    #[test]
    fn into_rc() {
        let orig: &[u8] = b"Hello, world!\0";
        let cstr = CStr::from_bytes_with_nul(orig).unwrap();
        let rc: Rc<CStr> = Rc::from(cstr);
        let arc: Arc<CStr> = Arc::from(cstr);

        assert_eq!(&*rc, cstr);
        assert_eq!(&*arc, cstr);

        let rc2: Rc<CStr> = Rc::from(cstr.to_owned());
        let arc2: Arc<CStr> = Arc::from(cstr.to_owned());

        assert_eq!(&*rc2, cstr);
        assert_eq!(&*arc2, cstr);
    }
}
