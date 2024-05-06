#![allow(unused)]

pub const AT_NULL: usize = 0;
pub const AT_IGNORE: usize = 1;

pub const AT_EXECFD: usize = 2;

/// The address of the program headers of the executable.
pub const AT_PHDR: usize = 3;

pub const AT_PHENT: usize = 4;
pub const AT_PHNUM: usize = 5;
pub const AT_PAGESZ: usize = 6;

/// The base address of the program interpreter (usually, the dynamic linker).
pub const AT_BASE: usize = 7;

pub const AT_FLAGS: usize = 8;
pub const AT_ENTRY: usize = 9;
pub const AT_NOTELF: usize = 10;
pub const AT_UID: usize = 11;
pub const AT_EUID: usize = 12;
pub const AT_GID: usize = 13;
pub const AT_EGID: usize = 14;
pub const AT_PLATFORM: usize = 15;
pub const AT_HWCAP: usize = 16;
pub const AT_CLKTCK: usize = 17;
pub const AT_DCACHEBSIZE: usize = 19;
pub const AT_ICACHEBSIZE: usize = 20;
pub const AT_UCACHEBSIZE: usize = 21;
pub const AT_SECURE: usize = 23;
pub const AT_RANDOM: usize = 25;

/// A pointer to a string containing the pathname used to execute the program.
pub const AT_EXECFN: usize = 31;

/// The address of a page containing the vDSO that the kernel creates
pub const AT_SYSINFO_EHDR: usize = 33;

/// The entry point to the system call function in the vDSO. Not present/needed on all architectures (e.g., absent on x86-64).
pub const AT_SYSINFO: usize = 32;
