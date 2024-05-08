use alloc::{vec, vec::Vec};

const STACK_SIZE: usize = ruxconfig::TASK_STACK_SIZE;

#[derive(Debug)]
pub struct Stack {
    /// stack
    data: Vec<u8>,
    /// index of top byte of stack
    top: usize,
}

impl Stack {
    /// alloc a stack
    pub fn new() -> Self {
        Self {
            data: vec![0u8; STACK_SIZE],
            top: STACK_SIZE,
        }
    }

    /// addr of top of stack
    pub fn sp(&self) -> usize {
        self.data.as_ptr() as usize + self.top
    }

    /// push data to stack and return the addr of sp
    pub fn push<T>(&mut self, data: &[T], align: usize) -> usize {
        // move sp to right place
        self.top -= core::mem::size_of_val(data);
        self.top = memory_addr::align_down(self.top, align);

        assert!(self.top <= self.data.len(), "sys_execve: stack overflow.");

        // write data into stack
        let sp = self.sp() as *mut T;
        unsafe {
            sp.copy_from_nonoverlapping(data.as_ptr(), data.len());
        }

        sp as usize
    }
}
