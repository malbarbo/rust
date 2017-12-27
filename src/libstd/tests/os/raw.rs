// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use libc;
use std::any::TypeId;

macro_rules! ok {
    ($($t:ident)*) => {$(
        assert!(TypeId::of::<libc::$t>() == TypeId::of::<raw::$t>(),
                "{} is wrong", stringify!($t));
    )*}
}

#[test]
fn same() {
    use std::os::raw;
    ok!(c_char c_schar c_uchar c_short c_ushort c_int c_uint c_long c_ulong
        c_longlong c_ulonglong c_float c_double);
}
