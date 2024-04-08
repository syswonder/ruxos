use core::{mem::size_of, ptr::null_mut};

use crate::*;

#[derive(Debug)]
pub struct Stack {
    sp: usize,
    start: usize,
    end: usize,
}

impl Stack {
    // alloc a stack
    pub fn new() -> Self {
        let size = 0xa00000; // 10M
        let p = sys_mmap(null_mut(), size, 0, 0, 0, 0);

        let start = p as usize;
        let sp = start + size;
        let end = sp;

        Self { sp, start, end }
    }

    pub fn align(&mut self, align: usize) -> usize {
        self.sp -= self.sp % align;
        self.sp
    }

    pub fn push<T: Copy>(&mut self, thing: alloc::vec::Vec<T>, align: usize) -> usize {
        let size = thing.len() * size_of::<T>();
        self.sp -= size;
        self.sp = self.align(align); // align 16B

        if self.sp < self.start {
            panic!("stack overflow");
        }

        let mut pt = self.sp as *mut T;
        unsafe {
            for t in thing {
                *pt = t;
                pt = pt.add(1);
            }
        }

        self.sp
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        sys_munmap(self.start as *mut _, self.end - self.start);
    }
}
