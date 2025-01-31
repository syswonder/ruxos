use alloc::vec::Vec;
use ruxtask::task::TaskStack;

const STACK_SIZE: usize = ruxconfig::TASK_STACK_SIZE;

#[derive(Debug)]
pub struct Stack {
    /// task stack
    task_stack: TaskStack,
    /// stack
    data: Vec<u8>,
    /// index of top byte of stack
    top: usize,
}

impl Stack {
    /// alloc a stack
    pub fn new() -> Self {
        let task_stack = TaskStack::alloc(STACK_SIZE);
        unsafe {
            let start = task_stack.top().as_mut_ptr().sub(STACK_SIZE);

            Self {
                task_stack,
                data: Vec::from_raw_parts(start, STACK_SIZE, STACK_SIZE),
                top: STACK_SIZE,
            }
        }
    }

    /// addr of top of stack
    pub fn sp(&self) -> usize {
        self.data.as_ptr() as usize + self.top
    }

    pub fn stack_size(&self) -> usize {
        self.data.len()
    }

    pub fn stack_top(&self) -> usize {
        self.task_stack.top().into()
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

impl Drop for Stack {
    fn drop(&mut self) {
        error!("execve's stack dropped. {:#?}", self);
    }
}
