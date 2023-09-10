extern crate alloc;
use alloc::vec::Vec;
use core::{ptr, usize};
use core::ffi::c_char;

// environ implementation
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut environ: *mut *mut c_char = ptr::null_mut(); // 不可为空
pub static mut OUR_ENVIRON: Vec<*mut c_char> = Vec::new();

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

pub(crate) unsafe fn boot_add_environ(
    env: &str,
) {
    let ptr = env.as_ptr() as *const i8;
    let size = env.len() + 1; // 算上/0
	if size == 1 {
		return;
	}
    let layout = core::alloc::Layout::from_size_align(size + CTRL_BLK_SIZE, 8).unwrap();
    
    // allocate for buf to meet free function
    let alloc_ptr = alloc::alloc::alloc(layout).cast::<MemoryControlBlock>();
    assert!(!alloc_ptr.is_null(), "alloc failed");
    alloc_ptr.write(MemoryControlBlock { size });
    let buf = alloc_ptr.add(1) as *mut c_char;

    for i in 0..size-1 {
        core::ptr::write(buf.add(i), *ptr.add(i));
    }
    core::ptr::write(buf.add(size - 1), 0);
    OUR_ENVIRON.push(buf);
}