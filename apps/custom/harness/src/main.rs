#![no_std]
#![no_main]

use axstd::println;
use core::{clone::Clone, format_args, panic};
use core::{ffi::c_void, mem::size_of};
use km_command::{fs::LibcDirent, FromBytes};
use km_harness::{executor, harness_command, Command, Harness, MemPort};
use ruxos_posix_api::{
    ctypes::dirent, sys_chdir, sys_close, sys_dup, sys_fstat, sys_getcwd, sys_getdents64,
    sys_mkdirat, sys_openat, sys_unlinkat,
};

/// Size of harness buffer.
const HARNESS_BUF_SIZE: usize = 4096;

/// Buffer for checker to write commands to.
#[link_section = ".data"]
static mut CMD_BUF: [u8; 4096] = [0; 4096];

/// Buffer for checker to read extra data from.
#[link_section = ".data"]
static mut OUTPUT_BUF: [u8; 4096] = [0; 4096];

/// Buffer for checker to read return value from.
#[link_section = ".data"]
static mut RETV_BUF: [u8; size_of::<isize>()] = [0; size_of::<isize>()];

harness_command!(km_command::fs, Openat, {
    // Transfer the path to CStr.
    let mut path = get!(path).clone();
    path.push('\0').unwrap();
    sys_openat(
        get!(dirfd) as i32,
        path.as_ptr() as *const i8,
        get!(flags).bits() as i32,
        get!(mode).bits(),
    ) as isize
});

harness_command!(km_command::fs, Close, {
    sys_close(get!(fd) as i32) as isize
});

harness_command!(km_command::fs, Fstat, {
    sys_fstat(get!(fd) as i32, output!().as_mut_ptr() as *mut c_void) as isize
});

harness_command!(km_command::fs, Getdents1, {
    // only want to get one directory entry per read.
    // So we need to try buffer size from a very small value
    // until it's just enough for one directory entry.
    let mut buf_size = LibcDirent::ONE_DIRENT_BUF_SIZE;
    loop {
        let ret = unsafe {
            sys_getdents64(
                get!(fd) as i32,
                output!().as_mut_ptr() as *mut dirent,
                buf_size,
            )
        };
        if ret >= 0 {
            break ret as isize;
        }
        buf_size += 1;
    }
});

// Not supported by fatfs
harness_command!(km_command::fs, Linkat, { 0 });

harness_command!(km_command::fs, Unlinkat, {
    // Transfer the path to CStr.
    let mut path = get!(path).clone();
    path.push('\0').unwrap();
    sys_unlinkat(
        get!(dirfd) as i32,
        path.as_ptr() as *const i8,
        get!(flags).bits() as i32,
    ) as isize
});

harness_command!(km_command::fs, Mkdirat, {
    // Transfer the path to CStr.
    let mut path = get!(path).clone();
    path.push('\0').unwrap();
    sys_mkdirat(
        get!(dirfd) as i32,
        path.as_ptr() as *const i8,
        get!(mode).bits(),
    ) as isize
});

harness_command!(km_command::fs, Getcwd, {
    sys_getcwd(output!().as_mut_ptr() as *mut i8, output!().len()) as isize
});

harness_command!(km_command::fs, Chdir, {
    // Transfer the path to CStr.
    let mut path = get!(path).clone();
    path.push('\0').unwrap();
    sys_chdir(path.as_ptr() as *const i8) as isize
});

harness_command!(km_command::fs, Dup, {
    sys_dup(get!(oldfd) as i32) as isize
});

harness_command!(km_command, Nop, { 0 });

// Define an executor
executor!(
    FsSyscallExecutor,
    Openat,
    Close,
    Fstat,
    Getdents1,
    Linkat,
    Unlinkat,
    Mkdirat,
    Getcwd,
    Chdir,
    Dup,
    Nop
);

#[no_mangle]
fn main() {
    println!("CMD_BUF at {:p}", unsafe { CMD_BUF.as_ptr() });
    println!("RETV_BUF at {:p}", unsafe { RETV_BUF.as_ptr() });
    println!("OUTPUT_BUF at {:p}", unsafe { OUTPUT_BUF.as_ptr() });
    let mut harness = Harness::<MemPort, FsSyscallExecutor, HARNESS_BUF_SIZE>::new(
        unsafe { MemPort::new(&CMD_BUF, &mut RETV_BUF, &mut OUTPUT_BUF) },
        FsSyscallExecutor,
    );
    loop {
        harness.step();
    }
}
