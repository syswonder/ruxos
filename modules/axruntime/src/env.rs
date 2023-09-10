extern crate alloc;
use alloc::vec::Vec;
use core::ffi::c_char;
use core::{ptr, usize};

/// argv for C main function
#[allow(non_upper_case_globals)]
pub static mut argv: *mut *mut c_char = ptr::null_mut();

/// Save cmdline argments
static mut RX_ARGV: Vec<*mut c_char> = Vec::new();

/// A pointer pointing to RX_ENVIRON
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut environ: *mut *mut c_char = ptr::null_mut();

/// Save environment variables
pub static mut RX_ENVIRON: Vec<*mut c_char> = Vec::new();

pub(crate) unsafe fn init_argv(args: Vec<&str>) {
    for arg in args {
        let len = arg.len();
        let arg = arg.as_ptr();
        let buf = buf_alloc(len + 1);
        for i in 0..len {
            *buf.add(i) = *arg.add(i) as i8;
        }
        *buf.add(len) = 0;
        RX_ARGV.push(buf);
    }
    RX_ARGV.push(ptr::null_mut());
    argv = RX_ARGV.as_mut_ptr();
}

/// Generate an iterator for environment variables
pub fn environ_iter() -> impl Iterator<Item = *mut c_char> + 'static {
    unsafe {
        let mut ptrs = environ;
        core::iter::from_fn(move || {
            let ptr = ptrs.read();
            if ptr.is_null() {
                None
            } else {
                ptrs = ptrs.add(1);
                Some(ptr)
            }
        })
    }
}

#[allow(dead_code)]
struct MemoryControlBlock {
    size: usize,
}
const CTRL_BLK_SIZE: usize = core::mem::size_of::<MemoryControlBlock>();

unsafe fn buf_alloc(size: usize) -> *mut c_char {
    let layout = core::alloc::Layout::from_size_align(size + CTRL_BLK_SIZE, 8).unwrap();
    // allocate for buf to meet free function
    let alloc_ptr = alloc::alloc::alloc(layout).cast::<MemoryControlBlock>();
    assert!(!alloc_ptr.is_null(), "alloc failed");
    alloc_ptr.write(MemoryControlBlock { size });
    alloc_ptr.add(1).cast()
}

pub(crate) fn boot_add_environ(env: &str) {
    let ptr = env.as_ptr() as *const i8;
    let size = env.len() + 1;
    if size == 1 {
        return;
    }
    unsafe {
        let buf = buf_alloc(size);
        for i in 0..size - 1 {
            core::ptr::write(buf.add(i), *ptr.add(i));
        }
        core::ptr::write(buf.add(size - 1), 0);
        RX_ENVIRON.push(buf);
    }
}
