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
    use std::thread;
    use std::io::*;

    #[test]
    #[cfg_attr(target_os = "emscripten", ignore)]
    fn panic_doesnt_poison() {
        thread::spawn(|| {
            let _a = stdin();
            let _a = _a.lock();
            let _a = stdout();
            let _a = _a.lock();
            let _a = stderr();
            let _a = _a.lock();
            panic!();
        }).join().unwrap_err();

        let _a = stdin();
        let _a = _a.lock();
        let _a = stdout();
        let _a = _a.lock();
        let _a = stderr();
        let _a = _a.lock();
    }
}
