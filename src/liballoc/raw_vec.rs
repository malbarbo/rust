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
    use alloc::raw_vec::RawVec;
    use std::heap::{Heap, Layout};

    #[test]
    fn allocator_param() {
        use alloc::allocator::{Alloc, AllocErr};

        // Writing a test of integration between third-party
        // allocators and RawVec is a little tricky because the RawVec
        // API does not expose fallible allocation methods, so we
        // cannot check what happens when allocator is exhausted
        // (beyond detecting a panic).
        //
        // Instead, this just checks that the RawVec methods do at
        // least go through the Allocator API when it reserves
        // storage.

        // A dumb allocator that consumes a fixed amount of fuel
        // before allocation attempts start failing.
        struct BoundedAlloc { fuel: usize }
        unsafe impl Alloc for BoundedAlloc {
            unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
                let size = layout.size();
                if size > self.fuel {
                    return Err(AllocErr::Unsupported { details: "fuel exhausted" });
                }
                match Heap.alloc(layout) {
                    ok @ Ok(_) => { self.fuel -= size; ok }
                    err @ Err(_) => err,
                }
            }
            unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
                Heap.dealloc(ptr, layout)
            }
        }

        let a = BoundedAlloc { fuel: 500 };
        let mut v: RawVec<u8, _> = RawVec::with_capacity_in(50, a);
        // assert_eq!(v.a.fuel, 450);
        v.reserve(50, 150); // (causes a realloc, thus using 50 + 150 = 200 units of fuel)
        // assert_eq!(v.a.fuel, 250);
    }

    #[test]
    fn reserve_does_not_overallocate() {
        {
            let mut v: RawVec<u32> = RawVec::new();
            // First `reserve` allocates like `reserve_exact`
            v.reserve(0, 9);
            assert_eq!(9, v.cap());
        }

        {
            let mut v: RawVec<u32> = RawVec::new();
            v.reserve(0, 7);
            assert_eq!(7, v.cap());
            // 97 if more than double of 7, so `reserve` should work
            // like `reserve_exact`.
            v.reserve(7, 90);
            assert_eq!(97, v.cap());
        }

        {
            let mut v: RawVec<u32> = RawVec::new();
            v.reserve(0, 12);
            assert_eq!(12, v.cap());
            v.reserve(12, 3);
            // 3 is less than half of 12, so `reserve` must grow
            // exponentially. At the time of writing this test grow
            // factor is 2, so new capacity is 24, however, grow factor
            // of 1.5 is OK too. Hence `>= 18` in assert.
            assert!(v.cap() >= 12 + 12 / 2);
        }
    }
}
