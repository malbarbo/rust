// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io::prelude::*;
use test;

#[bench]
fn bench_read_slice(b: &mut test::Bencher) {
    let buf = [5; 1024];
    let mut dst = [0; 128];

    b.iter(|| {
        let mut rd = &buf[..];
        for _ in 0..8 {
            let _ = rd.read(&mut dst);
            test::black_box(&dst);
        }
    })
}

#[bench]
fn bench_write_slice(b: &mut test::Bencher) {
    let mut buf = [0; 1024];
    let src = [5; 128];

    b.iter(|| {
        let mut wr = &mut buf[..];
        for _ in 0..8 {
            let _ = wr.write_all(&src);
            test::black_box(&wr);
        }
    })
}

#[bench]
fn bench_read_vec(b: &mut test::Bencher) {
    let buf = vec![5; 1024];
    let mut dst = [0; 128];

    b.iter(|| {
        let mut rd = &buf[..];
        for _ in 0..8 {
            let _ = rd.read(&mut dst);
            test::black_box(&dst);
        }
    })
}

#[bench]
fn bench_write_vec(b: &mut test::Bencher) {
    let mut buf = Vec::with_capacity(1024);
    let src = [5; 128];

    b.iter(|| {
        let mut wr = &mut buf[..];
        for _ in 0..8 {
            let _ = wr.write_all(&src);
            test::black_box(&wr);
        }
    })
}
