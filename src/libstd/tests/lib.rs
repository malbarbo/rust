// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(warnings)]

#![feature(ascii_ctype)]
#![feature(fs_read_write)]
#![feature(io)]
#![feature(rustc_private)]
#![feature(test)]
#![feature(toowned_clone_into)]

extern crate rand;
extern crate test;

use std::fmt;
use std::ops::{Add, Sub, Mul, Div, Rem};

macro_rules! assert_approx_eq {
    ($a:expr, $b:expr) => ({
        let (a, b) = (&$a, &$b);
        assert!((*a - *b).abs() < 1.0e-6,
                "{} is not approximately equal to {}", *a, *b);
    })
}

pub fn test_num<T>(ten: T, two: T) where
    T: PartialEq
     + Add<Output=T> + Sub<Output=T>
     + Mul<Output=T> + Div<Output=T>
     + Rem<Output=T> + fmt::Debug
     + Copy
{
    assert_eq!(ten.add(two),  ten + two);
    assert_eq!(ten.sub(two),  ten - two);
    assert_eq!(ten.mul(two),  ten * two);
    assert_eq!(ten.div(two),  ten / two);
    assert_eq!(ten.rem(two),  ten % two);
}

mod ascii;
mod env;
mod error;
mod f32;
mod f64;
mod ffi;
mod fs;
mod io;
